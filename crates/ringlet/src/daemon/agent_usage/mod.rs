//! Multi-agent usage tracking.
//!
//! Scans native data files from supported CLI coding agents to extract
//! token usage and cost information. Currently supports:
//!
//! - **Claude Code**: `~/.claude/projects/**/*.jsonl`
//! - **Codex CLI**: `~/.codex/sessions/**/*.jsonl`
//! - **OpenCode**: `~/.local/share/opencode/storage/**/*.json`

pub mod claude;
pub mod codex;
pub mod opencode;

use crate::daemon::pricing::PricingLoader;
use anyhow::Result;
use chrono::{DateTime, Utc};
use ringlet_core::{AgentType, CostBreakdown, RingletPaths, TokenUsage};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// A single usage entry from an agent's native files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    /// When this usage was recorded.
    pub timestamp: DateTime<Utc>,
    /// Which agent generated this entry.
    pub agent: AgentType,
    /// Unique message identifier (for deduplication).
    pub message_id: String,
    /// Request ID (Claude-specific, used with message_id for deduplication).
    pub request_id: Option<String>,
    /// Model used for this request.
    pub model: String,
    /// Token usage breakdown.
    pub tokens: TokenUsage,
    /// Pre-calculated cost in USD (if available in source data).
    pub cost_usd: Option<f64>,
    /// Project or session path (for profile attribution).
    pub project_path: String,
}

impl UsageEntry {
    /// Generate a unique deduplication key for this entry.
    pub fn dedup_key(&self) -> String {
        match &self.request_id {
            Some(req_id) => format!("{}:{}:{}", self.agent, self.message_id, req_id),
            None => format!("{}:{}", self.agent, self.message_id),
        }
    }
}

/// Result of scanning all agents.
#[derive(Debug, Default)]
pub struct ScanResult {
    /// All usage entries found.
    pub entries: Vec<UsageEntry>,
    /// Entries per agent.
    pub by_agent: std::collections::HashMap<AgentType, Vec<UsageEntry>>,
    /// Warnings encountered during scanning (non-fatal).
    pub warnings: Vec<String>,
}

/// Snapshot of known native usage entry keys for a profile home.
#[derive(Debug, Clone, Default)]
pub struct UsageSnapshot {
    known_keys: HashSet<String>,
}

/// Usage observed for a single completed Ringlet session.
#[derive(Debug, Clone)]
pub struct UsageDelta {
    pub tokens: TokenUsage,
    pub cost: Option<CostBreakdown>,
}

impl ScanResult {
    /// Create a new empty result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add entries from a single agent scan.
    pub fn add_agent_entries(&mut self, agent: AgentType, entries: Vec<UsageEntry>) {
        self.entries.extend(entries.clone());
        self.by_agent.insert(agent, entries);
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Total number of entries.
    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    /// Deduplicate entries by their unique keys.
    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.entries.retain(|entry| seen.insert(entry.dedup_key()));

        // Also deduplicate by_agent
        for entries in self.by_agent.values_mut() {
            let mut agent_seen = HashSet::new();
            entries.retain(|entry| agent_seen.insert(entry.dedup_key()));
        }
    }
}

/// Scan all supported agents for usage data.
///
/// This is the main entry point for usage tracking. It scans data directories
/// for all supported agents and returns aggregated usage entries.
pub async fn scan_all_agents() -> Result<ScanResult> {
    let mut result = ScanResult::new();

    // Scan Claude
    let claude_dir = claude::get_data_dir();
    if claude_dir.exists() {
        debug!("Scanning Claude usage from {:?}", claude_dir);
        match claude::scan_usage(&claude_dir).await {
            Ok(entries) => {
                debug!("Found {} Claude entries", entries.len());
                result.add_agent_entries(AgentType::Claude, entries);
            }
            Err(e) => {
                let warning = format!("Failed to scan Claude usage: {}", e);
                warn!("{}", warning);
                result.add_warning(warning);
            }
        }
    } else {
        debug!("Claude data directory not found: {:?}", claude_dir);
    }

    // Scan Codex
    let codex_dir = codex::get_data_dir();
    if codex_dir.exists() {
        debug!("Scanning Codex usage from {:?}", codex_dir);
        match codex::scan_usage(&codex_dir).await {
            Ok(entries) => {
                debug!("Found {} Codex entries", entries.len());
                result.add_agent_entries(AgentType::Codex, entries);
            }
            Err(e) => {
                let warning = format!("Failed to scan Codex usage: {}", e);
                warn!("{}", warning);
                result.add_warning(warning);
            }
        }
    } else {
        debug!("Codex data directory not found: {:?}", codex_dir);
    }

    // Scan OpenCode
    let opencode_dir = opencode::get_data_dir();
    if opencode_dir.exists() {
        debug!("Scanning OpenCode usage from {:?}", opencode_dir);
        match opencode::scan_usage(&opencode_dir).await {
            Ok(entries) => {
                debug!("Found {} OpenCode entries", entries.len());
                result.add_agent_entries(AgentType::OpenCode, entries);
            }
            Err(e) => {
                let warning = format!("Failed to scan OpenCode usage: {}", e);
                warn!("{}", warning);
                result.add_warning(warning);
            }
        }
    } else {
        debug!("OpenCode data directory not found: {:?}", opencode_dir);
    }

    // Deduplicate all entries
    result.deduplicate();

    Ok(result)
}

/// Capture a baseline snapshot of native usage entries for a specific profile home.
pub async fn snapshot_for_profile(
    agent_id: &str,
    profile_home: &Path,
) -> Result<Option<UsageSnapshot>> {
    let Some(agent) = agent_type_for_id(agent_id) else {
        return Ok(None);
    };

    let entries = scan_agent_profile_home(agent, profile_home).await?;
    Ok(Some(UsageSnapshot {
        known_keys: entries.into_iter().map(|entry| entry.dedup_key()).collect(),
    }))
}

/// Compute the usage delta since a previously captured baseline snapshot.
pub async fn delta_for_profile(
    agent_id: &str,
    profile_home: &Path,
    before: &UsageSnapshot,
    model: &str,
    provider_id: &str,
    paths: &RingletPaths,
) -> Result<Option<UsageDelta>> {
    let Some(agent) = agent_type_for_id(agent_id) else {
        return Ok(None);
    };

    let entries = scan_agent_profile_home(agent, profile_home).await?;
    let new_entries: Vec<UsageEntry> = entries
        .into_iter()
        .filter(|entry| !before.known_keys.contains(&entry.dedup_key()))
        .collect();

    if new_entries.is_empty() {
        return Ok(None);
    }

    let mut tokens = TokenUsage::default();
    let mut total_cost = 0.0;
    let mut has_native_cost = false;

    for entry in &new_entries {
        tokens += entry.tokens.clone();
        if let Some(cost_usd) = entry.cost_usd {
            total_cost += cost_usd;
            has_native_cost = true;
        }
    }

    let cost = if has_native_cost {
        Some(CostBreakdown {
            total_cost,
            ..Default::default()
        })
    } else {
        PricingLoader::new(paths.clone()).calculate_cost(&tokens, model, provider_id)
    };

    Ok(Some(UsageDelta { tokens, cost }))
}

fn agent_type_for_id(agent_id: &str) -> Option<AgentType> {
    match agent_id {
        "claude" => Some(AgentType::Claude),
        "codex" => Some(AgentType::Codex),
        "opencode" => Some(AgentType::OpenCode),
        _ => None,
    }
}

async fn scan_agent_profile_home(agent: AgentType, profile_home: &Path) -> Result<Vec<UsageEntry>> {
    let mut entries = Vec::new();

    for root in profile_usage_roots(agent, profile_home) {
        if !root.exists() {
            continue;
        }

        let mut root_entries = match agent {
            AgentType::Claude => claude::scan_usage(&root).await?,
            AgentType::Codex => codex::scan_usage(&root).await?,
            AgentType::OpenCode => opencode::scan_usage(&root).await?,
        };
        entries.append(&mut root_entries);
    }

    let mut seen = HashSet::new();
    entries.retain(|entry| seen.insert(entry.dedup_key()));
    Ok(entries)
}

fn profile_usage_roots(agent: AgentType, profile_home: &Path) -> Vec<PathBuf> {
    match agent {
        AgentType::Claude => vec![profile_home.join(".claude")],
        AgentType::Codex => vec![profile_home.join(".codex")],
        AgentType::OpenCode => vec![
            profile_home.join(".local/share/opencode"),
            profile_home.join("Library/Application Support/opencode"),
            profile_home.join("AppData/Local/opencode"),
            profile_home.join(".opencode"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup_key() {
        let entry1 = UsageEntry {
            timestamp: Utc::now(),
            agent: AgentType::Claude,
            message_id: "msg_123".to_string(),
            request_id: Some("req_456".to_string()),
            model: "claude-sonnet-4".to_string(),
            tokens: TokenUsage::default(),
            cost_usd: None,
            project_path: "/project".to_string(),
        };

        let entry2 = UsageEntry {
            timestamp: Utc::now(),
            agent: AgentType::Codex,
            message_id: "msg_789".to_string(),
            request_id: None,
            model: "gpt-4o".to_string(),
            tokens: TokenUsage::default(),
            cost_usd: None,
            project_path: "/project".to_string(),
        };

        assert_eq!(entry1.dedup_key(), "claude:msg_123:req_456");
        assert_eq!(entry2.dedup_key(), "codex:msg_789");
    }

    #[test]
    fn test_scan_result_deduplicate() {
        let mut result = ScanResult::new();

        let entry = UsageEntry {
            timestamp: Utc::now(),
            agent: AgentType::Claude,
            message_id: "msg_123".to_string(),
            request_id: Some("req_456".to_string()),
            model: "claude-sonnet-4".to_string(),
            tokens: TokenUsage::default(),
            cost_usd: None,
            project_path: "/project".to_string(),
        };

        // Add the same entry twice
        result.entries.push(entry.clone());
        result.entries.push(entry);

        assert_eq!(result.entries.len(), 2);
        result.deduplicate();
        assert_eq!(result.entries.len(), 1);
    }

    #[test]
    fn test_agent_type_for_id() {
        assert_eq!(agent_type_for_id("claude"), Some(AgentType::Claude));
        assert_eq!(agent_type_for_id("codex"), Some(AgentType::Codex));
        assert_eq!(agent_type_for_id("opencode"), Some(AgentType::OpenCode));
        assert_eq!(agent_type_for_id("unknown"), None);
    }
}
