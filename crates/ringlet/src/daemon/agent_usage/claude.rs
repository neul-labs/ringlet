//! Claude Code usage parser.
//!
//! Parses JSONL files from Claude Code's native data directory:
//! - Location: `~/.claude/projects/**/*.jsonl`
//! - Override: `CLAUDE_CONFIG_DIR` environment variable
//!
//! Each line contains a JSON object with token usage and optional cost data.

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

/// Get the Claude data directory.
///
/// Checks `CLAUDE_CONFIG_DIR` env var first, falls back to `~/.claude`.
pub fn get_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        PathBuf::from(dir)
    } else {
        ringlet_core::home_dir()
            .map(|h| h.join(".claude"))
            .unwrap_or_else(|| PathBuf::from(".claude"))
    }
}

/// Scan Claude's projects directory for usage data.
pub async fn scan_usage(claude_dir: &Path) -> Result<Vec<UsageEntry>> {
    let projects_dir = claude_dir.join("projects");
    if !projects_dir.exists() {
        debug!("Claude projects directory not found: {:?}", projects_dir);
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    // Walk through all subdirectories looking for .jsonl files
    for entry in WalkDir::new(&projects_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "jsonl") {
            trace!("Parsing Claude JSONL file: {:?}", path);
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

/// Parse a single Claude JSONL file.
fn parse_jsonl_file(path: &Path) -> Result<Vec<UsageEntry>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    // Extract project path from file path for attribution
    let project_path = extract_project_path(path);

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

        // Try to parse as a Claude usage entry
        match serde_json::from_str::<ClaudeEntry>(&line) {
            Ok(claude_entry) => {
                if let Some(entry) = claude_entry.to_usage_entry(&project_path) {
                    entries.push(entry);
                }
            }
            Err(e) => {
                // Not all lines contain usage data, this is expected
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

/// Extract project name from file path.
///
/// Given `~/.claude/projects/my-project/session.jsonl`, returns `my-project`.
fn extract_project_path(path: &Path) -> String {
    // Walk up the path to find the project directory
    let mut current = path.parent();
    while let Some(parent) = current {
        if parent.file_name().map_or(false, |n| n == "projects") {
            // The next component after "projects" is the project name
            if let Some(project) = path
                .strip_prefix(parent)
                .ok()
                .and_then(|p| p.components().next())
                .and_then(|c| c.as_os_str().to_str())
            {
                return project.to_string();
            }
        }
        current = parent.parent();
    }
    path.display().to_string()
}

/// A Claude JSONL entry.
///
/// Structure from Claude Code's native files:
/// ```json
/// {
///   "timestamp": "2025-01-20T10:30:00.000Z",
///   "message": {
///     "usage": {
///       "input_tokens": 1000,
///       "output_tokens": 500,
///       "cache_creation_input_tokens": 200,
///       "cache_read_input_tokens": 100
///     }
///   },
///   "model": "claude-sonnet-4-20250514",
///   "costUSD": 0.0045,
///   "messageId": "msg_xxx",
///   "requestId": "req_xxx"
/// }
/// ```
#[derive(Debug, Deserialize)]
struct ClaudeEntry {
    #[serde(default)]
    timestamp: Option<String>,

    #[serde(default)]
    message: Option<ClaudeMessage>,

    #[serde(default)]
    model: Option<String>,

    #[serde(rename = "costUSD", default)]
    cost_usd: Option<f64>,

    #[serde(rename = "messageId", default)]
    message_id: Option<String>,

    #[serde(rename = "requestId", default)]
    request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeMessage {
    #[serde(default)]
    usage: Option<ClaudeUsage>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    #[serde(default)]
    input_tokens: Option<u64>,

    #[serde(default)]
    output_tokens: Option<u64>,

    #[serde(default)]
    cache_creation_input_tokens: Option<u64>,

    #[serde(default)]
    cache_read_input_tokens: Option<u64>,
}

impl ClaudeEntry {
    /// Convert to a UsageEntry if this entry contains usage data.
    fn to_usage_entry(&self, project_path: &str) -> Option<UsageEntry> {
        // Must have message with usage data
        let usage = self.message.as_ref()?.usage.as_ref()?;

        // Must have at least some token data
        let has_tokens = usage.input_tokens.is_some()
            || usage.output_tokens.is_some()
            || usage.cache_creation_input_tokens.is_some()
            || usage.cache_read_input_tokens.is_some();

        if !has_tokens {
            return None;
        }

        // Must have a message ID for deduplication
        let message_id = self.message_id.clone()?;

        // Parse timestamp, default to now if missing
        let timestamp = self
            .timestamp
            .as_ref()
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Some(UsageEntry {
            timestamp,
            agent: AgentType::Claude,
            message_id,
            request_id: self.request_id.clone(),
            model: self.model.clone().unwrap_or_else(|| "unknown".to_string()),
            tokens: TokenUsage {
                input_tokens: usage.input_tokens.unwrap_or(0),
                output_tokens: usage.output_tokens.unwrap_or(0),
                cache_creation_input_tokens: usage.cache_creation_input_tokens.unwrap_or(0),
                cache_read_input_tokens: usage.cache_read_input_tokens.unwrap_or(0),
            },
            cost_usd: self.cost_usd,
            project_path: project_path.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_entry() {
        let json = r#"{
            "timestamp": "2025-01-20T10:30:00.000Z",
            "message": {
                "usage": {
                    "input_tokens": 1000,
                    "output_tokens": 500,
                    "cache_creation_input_tokens": 200,
                    "cache_read_input_tokens": 100
                }
            },
            "model": "claude-sonnet-4-20250514",
            "costUSD": 0.0045,
            "messageId": "msg_123",
            "requestId": "req_456"
        }"#;

        let entry: ClaudeEntry = serde_json::from_str(json).unwrap();
        let usage_entry = entry.to_usage_entry("/project/test").unwrap();

        assert_eq!(usage_entry.agent, AgentType::Claude);
        assert_eq!(usage_entry.message_id, "msg_123");
        assert_eq!(usage_entry.request_id, Some("req_456".to_string()));
        assert_eq!(usage_entry.model, "claude-sonnet-4-20250514");
        assert_eq!(usage_entry.tokens.input_tokens, 1000);
        assert_eq!(usage_entry.tokens.output_tokens, 500);
        assert_eq!(usage_entry.tokens.cache_creation_input_tokens, 200);
        assert_eq!(usage_entry.tokens.cache_read_input_tokens, 100);
        assert_eq!(usage_entry.cost_usd, Some(0.0045));
    }

    #[test]
    fn test_skip_non_usage_entry() {
        // Entry without usage data should return None
        let json = r#"{
            "timestamp": "2025-01-20T10:30:00.000Z",
            "type": "user_message",
            "content": "Hello"
        }"#;

        let entry: ClaudeEntry = serde_json::from_str(json).unwrap();
        assert!(entry.to_usage_entry("/project").is_none());
    }

    #[test]
    fn test_extract_project_path() {
        let path = PathBuf::from("/home/user/.claude/projects/my-project/session.jsonl");
        assert_eq!(extract_project_path(&path), "my-project");

        let path2 = PathBuf::from("/home/user/.claude/projects/work/sub/session.jsonl");
        assert_eq!(extract_project_path(&path2), "work");
    }
}
