//! OpenCode usage parser.
//!
//! Parses JSON files from OpenCode's native data directory:
//! - Location: `~/.local/share/opencode/storage/message/**/*.json`
//! - Override: `OPENCODE_DATA_DIR` environment variable
//!
//! Unlike Claude and Codex, OpenCode uses individual JSON files (not JSONL).

use super::UsageEntry;
use clown_core::AgentType;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clown_core::TokenUsage;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};
use walkdir::WalkDir;

/// Get the OpenCode data directory.
///
/// Checks `OPENCODE_DATA_DIR` env var first, falls back to XDG data directory.
pub fn get_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OPENCODE_DATA_DIR") {
        PathBuf::from(dir)
    } else {
        // Use XDG data directory on Linux, Application Support on macOS
        dirs::data_local_dir()
            .map(|d| d.join("opencode"))
            .unwrap_or_else(|| {
                clown_core::home_dir()
                    .map(|h| h.join(".local/share/opencode"))
                    .unwrap_or_else(|| PathBuf::from(".opencode"))
            })
    }
}

/// Scan OpenCode's storage directory for usage data.
pub async fn scan_usage(opencode_dir: &Path) -> Result<Vec<UsageEntry>> {
    let storage_dir = opencode_dir.join("storage");
    let message_dir = storage_dir.join("message");

    if !message_dir.exists() {
        debug!("OpenCode message directory not found: {:?}", message_dir);
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    // Walk through all subdirectories looking for .json files
    for entry in WalkDir::new(&message_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            trace!("Parsing OpenCode JSON file: {:?}", path);
            match parse_json_file(path) {
                Ok(Some(usage_entry)) => {
                    entries.push(usage_entry);
                }
                Ok(None) => {
                    // File didn't contain usage data
                    trace!("No usage data in {:?}", path);
                }
                Err(e) => {
                    warn!("Failed to parse {:?}: {}", path, e);
                }
            }
        }
    }

    debug!("Found {} OpenCode entries", entries.len());
    Ok(entries)
}

/// Parse a single OpenCode JSON file.
fn parse_json_file(path: &Path) -> Result<Option<UsageEntry>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let opencode_entry: OpenCodeEntry = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    Ok(opencode_entry.to_usage_entry())
}

/// An OpenCode JSON entry.
///
/// Structure from OpenCode's native files:
/// ```json
/// {
///   "id": "msg_xxx",
///   "sessionId": "session_xxx",
///   "provider": "anthropic",
///   "model": "claude-sonnet-4",
///   "created_at": "2025-01-20T10:30:00Z",
///   "completed_at": "2025-01-20T10:30:05Z",
///   "tokens": {
///     "input_tokens": 1000,
///     "output_tokens": 500,
///     "cache_read_tokens": 100,
///     "cache_write_tokens": 200
///   },
///   "cost_usd": 0.0045
/// }
/// ```
#[derive(Debug, Deserialize)]
struct OpenCodeEntry {
    #[serde(default)]
    id: Option<String>,

    #[serde(rename = "sessionId", default)]
    session_id: Option<String>,

    #[serde(default)]
    provider: Option<String>,

    #[serde(default)]
    model: Option<String>,

    #[serde(default)]
    created_at: Option<String>,

    #[serde(default)]
    completed_at: Option<String>,

    #[serde(default)]
    tokens: Option<OpenCodeTokens>,

    #[serde(default)]
    cost_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct OpenCodeTokens {
    #[serde(default)]
    input_tokens: Option<u64>,

    #[serde(default)]
    output_tokens: Option<u64>,

    #[serde(default)]
    cache_read_tokens: Option<u64>,

    #[serde(default)]
    cache_write_tokens: Option<u64>,
}

impl OpenCodeEntry {
    /// Convert to a UsageEntry if this entry contains usage data.
    fn to_usage_entry(&self) -> Option<UsageEntry> {
        // Must have token data
        let tokens = self.tokens.as_ref()?;

        // Must have at least some token data
        let has_tokens = tokens.input_tokens.is_some()
            || tokens.output_tokens.is_some()
            || tokens.cache_read_tokens.is_some()
            || tokens.cache_write_tokens.is_some();

        if !has_tokens {
            return None;
        }

        // Must have an ID for deduplication
        let message_id = self.id.clone()?;

        // Parse timestamp from created_at
        let timestamp = self
            .created_at
            .as_ref()
            .and_then(|ts| {
                DateTime::parse_from_rfc3339(ts)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| ts.parse::<DateTime<Utc>>().ok())
            })
            .unwrap_or_else(Utc::now);

        // Use session ID for project attribution
        let project_path = self
            .session_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        // OpenCode uses cache_write_tokens instead of cache_creation_input_tokens
        Some(UsageEntry {
            timestamp,
            agent: AgentType::OpenCode,
            message_id,
            request_id: None,
            model: self.model.clone().unwrap_or_else(|| "unknown".to_string()),
            tokens: TokenUsage {
                input_tokens: tokens.input_tokens.unwrap_or(0),
                output_tokens: tokens.output_tokens.unwrap_or(0),
                cache_creation_input_tokens: tokens.cache_write_tokens.unwrap_or(0),
                cache_read_input_tokens: tokens.cache_read_tokens.unwrap_or(0),
            },
            cost_usd: self.cost_usd,
            project_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_opencode_entry() {
        let json = r#"{
            "id": "msg_123",
            "sessionId": "sess_456",
            "provider": "anthropic",
            "model": "claude-sonnet-4",
            "created_at": "2025-01-20T10:30:00Z",
            "completed_at": "2025-01-20T10:30:05Z",
            "tokens": {
                "input_tokens": 1000,
                "output_tokens": 500,
                "cache_read_tokens": 100,
                "cache_write_tokens": 200
            },
            "cost_usd": 0.0045
        }"#;

        let entry: OpenCodeEntry = serde_json::from_str(json).unwrap();
        let usage_entry = entry.to_usage_entry().unwrap();

        assert_eq!(usage_entry.agent, AgentType::OpenCode);
        assert_eq!(usage_entry.message_id, "msg_123");
        assert_eq!(usage_entry.model, "claude-sonnet-4");
        assert_eq!(usage_entry.tokens.input_tokens, 1000);
        assert_eq!(usage_entry.tokens.output_tokens, 500);
        assert_eq!(usage_entry.tokens.cache_read_input_tokens, 100);
        assert_eq!(usage_entry.tokens.cache_creation_input_tokens, 200);
        assert_eq!(usage_entry.cost_usd, Some(0.0045));
        assert_eq!(usage_entry.project_path, "sess_456");
    }

    #[test]
    fn test_skip_entry_without_tokens() {
        let json = r#"{
            "id": "msg_123",
            "sessionId": "sess_456",
            "provider": "anthropic",
            "model": "claude-sonnet-4"
        }"#;

        let entry: OpenCodeEntry = serde_json::from_str(json).unwrap();
        assert!(entry.to_usage_entry().is_none());
    }

    #[test]
    fn test_skip_entry_without_id() {
        let json = r#"{
            "sessionId": "sess_456",
            "tokens": {
                "input_tokens": 1000,
                "output_tokens": 500
            }
        }"#;

        let entry: OpenCodeEntry = serde_json::from_str(json).unwrap();
        assert!(entry.to_usage_entry().is_none());
    }
}
