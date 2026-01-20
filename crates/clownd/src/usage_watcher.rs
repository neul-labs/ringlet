//! Usage file watcher - detects new usage entries from agent native files.
//!
//! Watches data directories for all supported coding agents:
//! - Claude Code: `~/.claude/projects/**/*.jsonl`
//! - Codex CLI: `~/.codex/sessions/**/*.jsonl`
//! - OpenCode: `~/.local/share/opencode/storage/message/**/*.json`
//!
//! When new entries are detected, broadcasts `UsageUpdated` events via WebSocket.

use crate::agent_usage::{claude, codex, opencode, UsageEntry};
use crate::events::EventBroadcaster;
use anyhow::Result;
use clown_core::{AgentType, Event};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Tracks file positions for incremental reading.
#[derive(Debug, Default)]
struct FilePositions {
    /// Map from file path to last read position.
    positions: HashMap<PathBuf, u64>,
    /// Set of message IDs we've already seen (for deduplication).
    seen_ids: HashSet<String>,
}

/// Usage file watcher that monitors agent data directories.
pub struct UsageWatcher {
    /// Event broadcaster for WebSocket notifications.
    broadcaster: Arc<EventBroadcaster>,
}

impl UsageWatcher {
    /// Create a new usage watcher.
    pub fn new(broadcaster: Arc<EventBroadcaster>) -> Self {
        Self { broadcaster }
    }

    /// Start watching all agent directories.
    ///
    /// This spawns a background thread that monitors directories and broadcasts events.
    /// Returns immediately after starting the watcher.
    pub fn start(self) -> Result<()> {
        let broadcaster = self.broadcaster;

        std::thread::spawn(move || {
            if let Err(e) = run_watcher(broadcaster) {
                warn!("Usage watcher error: {}", e);
            }
        });

        Ok(())
    }
}

/// Run the file watcher loop.
fn run_watcher(broadcaster: Arc<EventBroadcaster>) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )?;

    // Directories to watch
    let watch_dirs = [
        (claude::get_data_dir().join("projects"), AgentType::Claude, true),   // JSONL
        (codex::get_data_dir().join("sessions"), AgentType::Codex, true),     // JSONL
        (opencode::get_data_dir().join("storage").join("message"), AgentType::OpenCode, false), // JSON
    ];

    // Start watching directories that exist
    for (dir, agent, _) in &watch_dirs {
        if dir.exists() {
            if let Err(e) = watcher.watch(dir, RecursiveMode::Recursive) {
                warn!("Failed to watch {:?} for {}: {}", dir, agent, e);
            } else {
                info!("Watching {} usage at {:?}", agent, dir);
            }
        } else {
            debug!("{} directory not found: {:?}", agent, dir);
        }
    }

    // Track file positions for incremental reading
    let mut file_state = FilePositions::default();

    info!("Usage watcher started");

    // Process file events
    for event in rx {
        for path in event.paths {
            // Determine which agent this file belongs to
            let agent = determine_agent(&path, &watch_dirs);

            if let Some(agent) = agent {
                // Check if it's a relevant file type
                let is_jsonl = path.extension().map_or(false, |ext| ext == "jsonl");
                let is_json = path.extension().map_or(false, |ext| ext == "json");

                if is_jsonl && matches!(agent, AgentType::Claude | AgentType::Codex) {
                    // Read new entries from JSONL file
                    if let Ok(entries) = read_new_jsonl_entries(&path, &mut file_state, agent) {
                        broadcast_entries(&broadcaster, entries);
                    }
                } else if is_json && matches!(agent, AgentType::OpenCode) {
                    // Parse JSON file
                    if let Ok(Some(entry)) = parse_new_json_entry(&path, &mut file_state) {
                        broadcast_entries(&broadcaster, vec![entry]);
                    }
                }
            }
        }
    }

    info!("Usage watcher stopped");
    Ok(())
}

/// Determine which agent a file path belongs to.
fn determine_agent(
    path: &PathBuf,
    watch_dirs: &[(PathBuf, AgentType, bool)],
) -> Option<AgentType> {
    for (dir, agent, _) in watch_dirs {
        if path.starts_with(dir) {
            return Some(*agent);
        }
    }
    None
}

/// Read new entries from a JSONL file (Claude or Codex).
fn read_new_jsonl_entries(
    path: &PathBuf,
    state: &mut FilePositions,
    agent: AgentType,
) -> Result<Vec<UsageEntry>> {
    let mut file = std::fs::File::open(path)?;
    let file_len = file.metadata()?.len();

    // Get last position, default to 0 for new files
    let last_pos = *state.positions.get(path).unwrap_or(&0);

    // If file was truncated or is new, start from beginning
    let start_pos = if file_len < last_pos { 0 } else { last_pos };

    // Seek to last position
    file.seek(SeekFrom::Start(start_pos))?;

    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut current_pos = start_pos;

    // Extract project/session path for attribution
    let project_path = extract_project_path(path, agent);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        current_pos += line.len() as u64 + 1; // +1 for newline

        if line.trim().is_empty() {
            continue;
        }

        // Parse based on agent type
        let entry = match agent {
            AgentType::Claude => parse_claude_line(&line, &project_path),
            AgentType::Codex => parse_codex_line(&line, &project_path, &mut state.seen_ids),
            _ => None,
        };

        if let Some(entry) = entry {
            // Check for duplicates
            let dedup_key = entry.dedup_key();
            if !state.seen_ids.contains(&dedup_key) {
                state.seen_ids.insert(dedup_key);
                entries.push(entry);
            }
        }
    }

    // Update position
    state.positions.insert(path.clone(), current_pos);

    Ok(entries)
}

/// Parse a single Claude JSONL line.
fn parse_claude_line(line: &str, project_path: &str) -> Option<UsageEntry> {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;

    #[derive(Deserialize)]
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

    #[derive(Deserialize)]
    struct ClaudeMessage {
        #[serde(default)]
        usage: Option<ClaudeUsage>,
    }

    #[derive(Deserialize)]
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

    let entry: ClaudeEntry = serde_json::from_str(line).ok()?;
    let usage = entry.message?.usage?;
    let message_id = entry.message_id?;

    let timestamp = entry.timestamp
        .and_then(|ts| DateTime::parse_from_rfc3339(&ts).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    Some(UsageEntry {
        timestamp,
        agent: AgentType::Claude,
        message_id,
        request_id: entry.request_id,
        model: entry.model.unwrap_or_else(|| "unknown".to_string()),
        tokens: clown_core::TokenUsage {
            input_tokens: usage.input_tokens.unwrap_or(0),
            output_tokens: usage.output_tokens.unwrap_or(0),
            cache_creation_input_tokens: usage.cache_creation_input_tokens.unwrap_or(0),
            cache_read_input_tokens: usage.cache_read_input_tokens.unwrap_or(0),
        },
        cost_usd: entry.cost_usd,
        project_path: project_path.to_string(),
    })
}

/// Parse a single Codex JSONL line.
fn parse_codex_line(line: &str, session_path: &str, seen_ids: &mut HashSet<String>) -> Option<UsageEntry> {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct CodexEntry {
        #[serde(rename = "type", default)]
        entry_type: Option<String>,
        #[serde(default)]
        timestamp: Option<String>,
        #[serde(default)]
        payload: Option<CodexPayload>,
    }

    #[derive(Deserialize)]
    struct CodexPayload {
        #[serde(default)]
        model_name: Option<String>,
        #[serde(default)]
        info: Option<CodexInfo>,
    }

    #[derive(Deserialize)]
    struct CodexInfo {
        #[serde(default)]
        usage: Option<CodexUsage>,
        #[serde(default)]
        metadata: Option<CodexMetadata>,
    }

    #[derive(Deserialize)]
    struct CodexUsage {
        #[serde(default)]
        input_tokens: Option<u64>,
        #[serde(default)]
        output_tokens: Option<u64>,
        #[serde(default)]
        cached_input_tokens: Option<u64>,
    }

    #[derive(Deserialize)]
    struct CodexMetadata {
        #[serde(default)]
        model: Option<String>,
    }

    let entry: CodexEntry = serde_json::from_str(line).ok()?;

    // Only process token_count entries
    if entry.entry_type.as_deref() != Some("token_count") {
        return None;
    }

    let payload = entry.payload?;
    let info = payload.info?;
    let usage = info.usage?;

    // Generate unique ID (Codex doesn't have message IDs)
    let timestamp_str = entry.timestamp.as_deref().unwrap_or("unknown");
    let counter = seen_ids.len(); // Use seen count as counter
    let message_id = format!("codex_{}_{}", timestamp_str, counter);

    let timestamp = entry.timestamp
        .and_then(|ts| DateTime::parse_from_rfc3339(&ts).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let model = payload.model_name
        .or_else(|| info.metadata.and_then(|m| m.model))
        .unwrap_or_else(|| "gpt-4o".to_string());

    Some(UsageEntry {
        timestamp,
        agent: AgentType::Codex,
        message_id,
        request_id: None,
        model,
        tokens: clown_core::TokenUsage {
            input_tokens: usage.input_tokens.unwrap_or(0),
            output_tokens: usage.output_tokens.unwrap_or(0),
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: usage.cached_input_tokens.unwrap_or(0),
        },
        cost_usd: None,
        project_path: session_path.to_string(),
    })
}

/// Parse a new OpenCode JSON file.
fn parse_new_json_entry(path: &PathBuf, state: &mut FilePositions) -> Result<Option<UsageEntry>> {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct OpenCodeEntry {
        #[serde(default)]
        id: Option<String>,
        #[serde(rename = "sessionId", default)]
        session_id: Option<String>,
        #[serde(default)]
        model: Option<String>,
        #[serde(default)]
        created_at: Option<String>,
        #[serde(default)]
        tokens: Option<OpenCodeTokens>,
        #[serde(default)]
        cost_usd: Option<f64>,
    }

    #[derive(Deserialize)]
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

    let content = std::fs::read_to_string(path)?;

    // Check if we've seen this file content before (by hash or ID)
    let entry: OpenCodeEntry = serde_json::from_str(&content)?;

    let message_id = match entry.id {
        Some(id) => id,
        None => return Ok(None),
    };

    // Check for duplicates
    let dedup_key = format!("opencode:{}", message_id);
    if state.seen_ids.contains(&dedup_key) {
        return Ok(None);
    }
    state.seen_ids.insert(dedup_key);

    let tokens = match entry.tokens {
        Some(t) => t,
        None => return Ok(None),
    };

    let timestamp = entry.created_at
        .and_then(|ts| DateTime::parse_from_rfc3339(&ts).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    Ok(Some(UsageEntry {
        timestamp,
        agent: AgentType::OpenCode,
        message_id,
        request_id: None,
        model: entry.model.unwrap_or_else(|| "unknown".to_string()),
        tokens: clown_core::TokenUsage {
            input_tokens: tokens.input_tokens.unwrap_or(0),
            output_tokens: tokens.output_tokens.unwrap_or(0),
            cache_creation_input_tokens: tokens.cache_write_tokens.unwrap_or(0),
            cache_read_input_tokens: tokens.cache_read_tokens.unwrap_or(0),
        },
        cost_usd: entry.cost_usd,
        project_path: entry.session_id.unwrap_or_else(|| "unknown".to_string()),
    }))
}

/// Extract project/session path from file path.
fn extract_project_path(path: &PathBuf, agent: AgentType) -> String {
    match agent {
        AgentType::Claude => {
            // Find "projects" in path and get next component
            for (i, component) in path.components().enumerate() {
                if component.as_os_str() == "projects" {
                    if let Some(next) = path.components().nth(i + 1) {
                        return next.as_os_str().to_string_lossy().to_string();
                    }
                }
            }
            path.display().to_string()
        }
        AgentType::Codex => {
            // Find "sessions" in path and get next component
            for (i, component) in path.components().enumerate() {
                if component.as_os_str() == "sessions" {
                    if let Some(next) = path.components().nth(i + 1) {
                        return next.as_os_str().to_string_lossy().to_string();
                    }
                }
            }
            path.display().to_string()
        }
        AgentType::OpenCode => {
            // Use filename without extension
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        }
    }
}

/// Broadcast usage entries as events.
fn broadcast_entries(broadcaster: &EventBroadcaster, entries: Vec<UsageEntry>) {
    for entry in entries {
        debug!("Broadcasting usage update: {} {:?}", entry.agent, entry.tokens);

        let event = Event::UsageUpdated {
            agent: entry.agent,
            profile: Some(entry.project_path.clone()),
            tokens: entry.tokens.clone(),
            cost: entry.cost_usd.map(|c| clown_core::CostBreakdown {
                input_cost: 0.0,
                output_cost: 0.0,
                cache_creation_cost: 0.0,
                cache_read_cost: 0.0,
                total_cost: c,
            }),
        };

        broadcaster.broadcast(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_project_path_claude() {
        let path = PathBuf::from("/home/user/.claude/projects/my-project/session.jsonl");
        assert_eq!(extract_project_path(&path, AgentType::Claude), "my-project");
    }

    #[test]
    fn test_extract_project_path_codex() {
        let path = PathBuf::from("/home/user/.codex/sessions/abc123/data.jsonl");
        assert_eq!(extract_project_path(&path, AgentType::Codex), "abc123");
    }

    #[test]
    fn test_parse_claude_line() {
        let line = r#"{"timestamp":"2025-01-20T10:00:00Z","message":{"usage":{"input_tokens":100,"output_tokens":50}},"model":"claude-sonnet-4","messageId":"msg_123"}"#;
        let entry = parse_claude_line(line, "test-project").unwrap();

        assert_eq!(entry.agent, AgentType::Claude);
        assert_eq!(entry.message_id, "msg_123");
        assert_eq!(entry.tokens.input_tokens, 100);
        assert_eq!(entry.tokens.output_tokens, 50);
    }
}
