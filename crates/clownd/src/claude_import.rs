//! Import usage data from Claude's native files.
//!
//! Claude Code stores usage data in:
//! - `~/.claude/stats-cache.json` - Aggregate token usage by model
//! - `~/.claude/projects/*/session.jsonl` - Session-level data

use anyhow::{Context, Result};
use clown_core::TokenUsage;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Result of importing Claude data.
#[derive(Debug, Default)]
pub struct ClaudeImportResult {
    /// Total tokens from stats-cache.json
    pub stats_cache_tokens: TokenUsage,
    /// Tokens by model from stats-cache.json
    pub by_model: HashMap<String, TokenUsage>,
    /// Number of sessions imported from JSONL files
    pub sessions_imported: usize,
    /// Any errors encountered (non-fatal)
    pub warnings: Vec<String>,
}

/// Import all available Claude usage data.
pub fn import_all(claude_dir: &Path) -> Result<ClaudeImportResult> {
    let mut result = ClaudeImportResult::default();

    // Import from stats-cache.json
    let stats_cache = claude_dir.join("stats-cache.json");
    if stats_cache.exists() {
        match import_stats_cache(&stats_cache) {
            Ok((total, by_model)) => {
                result.stats_cache_tokens = total;
                result.by_model = by_model;
                info!("Imported stats-cache.json: {} input tokens, {} output tokens",
                    result.stats_cache_tokens.input_tokens,
                    result.stats_cache_tokens.output_tokens);
            }
            Err(e) => {
                let warning = format!("Failed to import stats-cache.json: {}", e);
                warn!("{}", warning);
                result.warnings.push(warning);
            }
        }
    }

    // Import from session JSONL files
    let projects_dir = claude_dir.join("projects");
    if projects_dir.exists() {
        match import_sessions(&projects_dir) {
            Ok(count) => {
                result.sessions_imported = count;
                info!("Imported {} sessions from JSONL files", count);
            }
            Err(e) => {
                let warning = format!("Failed to import session files: {}", e);
                warn!("{}", warning);
                result.warnings.push(warning);
            }
        }
    }

    Ok(result)
}

/// Import from Claude's stats-cache.json.
///
/// Returns (total_tokens, by_model).
fn import_stats_cache(path: &Path) -> Result<(TokenUsage, HashMap<String, TokenUsage>)> {
    #[derive(Deserialize)]
    struct StatsCacheModel {
        #[serde(rename = "inputTokens")]
        input_tokens: Option<u64>,
        #[serde(rename = "outputTokens")]
        output_tokens: Option<u64>,
        #[serde(rename = "cacheReadInputTokens")]
        cache_read_input_tokens: Option<u64>,
        #[serde(rename = "cacheCreationInputTokens")]
        cache_creation_input_tokens: Option<u64>,
    }

    #[derive(Deserialize)]
    struct StatsCache {
        #[serde(rename = "modelUsage")]
        model_usage: Option<HashMap<String, StatsCacheModel>>,
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let cache: StatsCache = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    let mut total = TokenUsage::default();
    let mut by_model = HashMap::new();

    if let Some(model_usage) = cache.model_usage {
        for (model, usage) in model_usage {
            debug!("Importing usage for model: {}", model);
            let tokens = TokenUsage {
                input_tokens: usage.input_tokens.unwrap_or(0),
                output_tokens: usage.output_tokens.unwrap_or(0),
                cache_creation_input_tokens: usage.cache_creation_input_tokens.unwrap_or(0),
                cache_read_input_tokens: usage.cache_read_input_tokens.unwrap_or(0),
            };
            total += tokens.clone();
            by_model.insert(model, tokens);
        }
    }

    Ok((total, by_model))
}

/// Import sessions from JSONL files in projects directory.
///
/// Returns the number of sessions imported.
fn import_sessions(projects_dir: &Path) -> Result<usize> {
    let mut count = 0;

    // Find all session.jsonl files
    for entry in std::fs::read_dir(projects_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // Look for session.jsonl in each project directory
        let session_file = path.join("session.jsonl");
        if session_file.exists() {
            match import_session_file(&session_file) {
                Ok(session_count) => {
                    count += session_count;
                }
                Err(e) => {
                    debug!("Failed to import {}: {}", session_file.display(), e);
                }
            }
        }

        // Also check for .session.jsonl files (alternate naming)
        for file in std::fs::read_dir(&path)? {
            let file = file?;
            let file_path = file.path();
            if file_path.extension().map_or(false, |ext| ext == "jsonl") {
                if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                    if name.contains("session") {
                        match import_session_file(&file_path) {
                            Ok(session_count) => {
                                count += session_count;
                            }
                            Err(e) => {
                                debug!("Failed to import {}: {}", file_path.display(), e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Import a single session JSONL file.
///
/// Returns the number of session entries found.
fn import_session_file(path: &Path) -> Result<usize> {
    use std::io::{BufRead, BufReader};

    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Try to parse as JSON and extract usage data
        if let Ok(entry) = serde_json::from_str::<SessionEntry>(&line) {
            if entry.usage.is_some() {
                count += 1;
            }
        }
    }

    Ok(count)
}

/// A session entry from JSONL file.
#[derive(Deserialize)]
struct SessionEntry {
    #[serde(default)]
    usage: Option<SessionUsage>,
}

/// Usage data from a session entry.
#[derive(Deserialize)]
struct SessionUsage {
    #[serde(rename = "input_tokens")]
    _input_tokens: Option<u64>,
    #[serde(rename = "output_tokens")]
    _output_tokens: Option<u64>,
}

/// Get the default Claude home directory.
pub fn default_claude_dir() -> Option<PathBuf> {
    clown_core::home_dir().map(|h| h.join(".claude"))
}
