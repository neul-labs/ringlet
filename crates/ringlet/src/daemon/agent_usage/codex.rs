//! Codex CLI usage parser.
//!
//! Parses JSONL files from Codex CLI's native data directory:
//! - Location: `~/.codex/sessions/**/*.jsonl`
//! - Override: `CODEX_HOME` environment variable
//!
//! Codex stores entries with `type: "token_count"` containing usage data.
//! Note: Codex embeds "reasoning tokens" in output_tokens.

use super::UsageEntry;
use ringlet_core::AgentType;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use ringlet_core::TokenUsage;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};
use walkdir::WalkDir;

/// Get the Codex data directory.
///
/// Checks `CODEX_HOME` env var first, falls back to `~/.codex`.
pub fn get_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CODEX_HOME") {
        PathBuf::from(dir)
    } else {
        ringlet_core::home_dir()
            .map(|h| h.join(".codex"))
            .unwrap_or_else(|| PathBuf::from(".codex"))
    }
}

/// Scan Codex's sessions directory for usage data.
pub async fn scan_usage(codex_dir: &Path) -> Result<Vec<UsageEntry>> {
    let sessions_dir = codex_dir.join("sessions");
    if !sessions_dir.exists() {
        debug!("Codex sessions directory not found: {:?}", sessions_dir);
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    // Walk through all subdirectories looking for .jsonl files
    for entry in WalkDir::new(&sessions_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "jsonl") {
            trace!("Parsing Codex JSONL file: {:?}", path);
            match parse_jsonl_file(path) {
                Ok(file_entries) => {
                    debug!(
                        "Parsed {} entries from {:?}",
                        file_entries.len(),
                        path.file_name()
                    );
                    entries.extend(file_entries);
                }
                Err(e) => {
                    warn!("Failed to parse {:?}: {}", path, e);
                }
            }
        }
    }

    Ok(entries)
}

/// Parse a single Codex JSONL file.
fn parse_jsonl_file(path: &Path) -> Result<Vec<UsageEntry>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    // Extract session path from file path for attribution
    let session_path = extract_session_path(path);
    let mut entry_counter = 0u64;

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                trace!("Failed to read line {} in {:?}: {}", line_num + 1, path, e);
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        // Try to parse as a Codex usage entry
        match serde_json::from_str::<CodexEntry>(&line) {
            Ok(codex_entry) => {
                // Only process token_count entries
                if codex_entry.entry_type.as_deref() == Some("token_count") {
                    if let Some(entry) = codex_entry.to_usage_entry(&session_path, &mut entry_counter)
                    {
                        entries.push(entry);
                    }
                }
            }
            Err(e) => {
                trace!(
                    "Skipping non-usage line {} in {:?}: {}",
                    line_num + 1,
                    path,
                    e
                );
            }
        }
    }

    Ok(entries)
}

/// Extract session ID from file path.
///
/// Given `~/.codex/sessions/abc123/session.jsonl`, returns `abc123`.
fn extract_session_path(path: &Path) -> String {
    // Walk up the path to find the session directory
    let mut current = path.parent();
    while let Some(parent) = current {
        if parent.file_name().map_or(false, |n| n == "sessions") {
            // The next component after "sessions" is the session ID
            if let Some(session) = path
                .strip_prefix(parent)
                .ok()
                .and_then(|p| p.components().next())
                .and_then(|c| c.as_os_str().to_str())
            {
                return session.to_string();
            }
        }
        current = parent.parent();
    }
    path.display().to_string()
}

/// A Codex JSONL entry.
///
/// Structure from Codex CLI's native files:
/// ```json
/// {
///   "type": "token_count",
///   "timestamp": "2025-01-20T10:30:00.000Z",
///   "payload": {
///     "model_name": "gpt-4o",
///     "info": {
///       "usage": {
///         "input_tokens": 1000,
///         "output_tokens": 500,
///         "cached_input_tokens": 100,
///         "total_tokens": 1600
///       },
///       "metadata": {
///         "model": "gpt-4o"
///       }
///     }
///   }
/// }
/// ```
#[derive(Debug, Deserialize)]
struct CodexEntry {
    #[serde(rename = "type", default)]
    entry_type: Option<String>,

    #[serde(default)]
    timestamp: Option<String>,

    #[serde(default)]
    payload: Option<CodexPayload>,
}

#[derive(Debug, Deserialize)]
struct CodexPayload {
    #[serde(default)]
    model_name: Option<String>,

    #[serde(default)]
    info: Option<CodexInfo>,
}

#[derive(Debug, Deserialize)]
struct CodexInfo {
    #[serde(default)]
    usage: Option<CodexUsage>,

    #[serde(default)]
    metadata: Option<CodexMetadata>,
}

#[derive(Debug, Deserialize)]
struct CodexUsage {
    #[serde(default)]
    input_tokens: Option<u64>,

    #[serde(default)]
    output_tokens: Option<u64>,

    #[serde(default)]
    cached_input_tokens: Option<u64>,

    #[serde(default)]
    total_tokens: Option<u64>,

    // Reasoning tokens (included in output_tokens, tracked separately if available)
    #[serde(default)]
    reasoning_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CodexMetadata {
    #[serde(default)]
    model: Option<String>,
}

impl CodexEntry {
    /// Convert to a UsageEntry if this entry contains usage data.
    fn to_usage_entry(&self, session_path: &str, counter: &mut u64) -> Option<UsageEntry> {
        let payload = self.payload.as_ref()?;
        let info = payload.info.as_ref()?;
        let usage = info.usage.as_ref()?;

        // Must have at least some token data
        let has_tokens = usage.input_tokens.is_some()
            || usage.output_tokens.is_some()
            || usage.total_tokens.is_some();

        if !has_tokens {
            return None;
        }

        // Generate a unique message ID (Codex doesn't have one)
        *counter += 1;
        let message_id = format!(
            "codex_{}_{}_{}",
            session_path,
            self.timestamp.as_deref().unwrap_or("unknown"),
            counter
        );

        // Parse timestamp
        let timestamp = self
            .timestamp
            .as_ref()
            .and_then(|ts| {
                // Try ISO 8601 format first
                DateTime::parse_from_rfc3339(ts)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    // Also try without timezone
                    .or_else(|| ts.parse::<DateTime<Utc>>().ok())
            })
            .unwrap_or_else(Utc::now);

        // Get model name from payload or metadata
        let model = payload
            .model_name
            .clone()
            .or_else(|| info.metadata.as_ref().and_then(|m| m.model.clone()))
            .unwrap_or_else(|| "gpt-4o".to_string()); // Default model for Codex

        // Map Codex token fields to our standard format
        // Note: Codex uses cached_input_tokens instead of cache_read_input_tokens
        Some(UsageEntry {
            timestamp,
            agent: AgentType::Codex,
            message_id,
            request_id: None,
            model,
            tokens: TokenUsage {
                input_tokens: usage.input_tokens.unwrap_or(0),
                output_tokens: usage.output_tokens.unwrap_or(0),
                cache_creation_input_tokens: 0, // Codex doesn't track this
                cache_read_input_tokens: usage.cached_input_tokens.unwrap_or(0),
            },
            cost_usd: None, // Codex doesn't pre-calculate costs
            project_path: session_path.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_codex_entry() {
        let json = r#"{
            "type": "token_count",
            "timestamp": "2025-01-20T10:30:00.000Z",
            "payload": {
                "model_name": "gpt-4o",
                "info": {
                    "usage": {
                        "input_tokens": 1000,
                        "output_tokens": 500,
                        "cached_input_tokens": 100,
                        "total_tokens": 1600
                    },
                    "metadata": {
                        "model": "gpt-4o"
                    }
                }
            }
        }"#;

        let entry: CodexEntry = serde_json::from_str(json).unwrap();
        let mut counter = 0;
        let usage_entry = entry.to_usage_entry("session123", &mut counter).unwrap();

        assert_eq!(usage_entry.agent, AgentType::Codex);
        assert_eq!(usage_entry.model, "gpt-4o");
        assert_eq!(usage_entry.tokens.input_tokens, 1000);
        assert_eq!(usage_entry.tokens.output_tokens, 500);
        assert_eq!(usage_entry.tokens.cache_read_input_tokens, 100);
        assert!(usage_entry.cost_usd.is_none());
    }

    #[test]
    fn test_skip_non_token_count_entry() {
        let json = r#"{
            "type": "message",
            "timestamp": "2025-01-20T10:30:00.000Z",
            "content": "Hello"
        }"#;

        let entry: CodexEntry = serde_json::from_str(json).unwrap();
        // Should be filtered out because type != "token_count"
        assert_ne!(entry.entry_type.as_deref(), Some("token_count"));
    }

    #[test]
    fn test_extract_session_path() {
        let path = PathBuf::from("/home/user/.codex/sessions/abc123/session.jsonl");
        assert_eq!(extract_session_path(&path), "abc123");

        let path2 = PathBuf::from("/home/user/.codex/sessions/xyz/sub/data.jsonl");
        assert_eq!(extract_session_path(&path2), "xyz");
    }
}
