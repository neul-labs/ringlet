//! Telemetry collection - tracks profile usage statistics.
//!
//! This module handles:
//! - Tracking per-session data (profile, start time, duration, exit code)
//! - Persisting sessions to sessions.jsonl
//! - Aggregating statistics

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clown_core::ClownPaths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use tracing::{debug, warn};

/// A recorded session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Profile alias.
    pub profile: String,
    /// Agent ID.
    pub agent_id: String,
    /// Provider ID.
    pub provider_id: String,
    /// Start timestamp.
    pub started_at: DateTime<Utc>,
    /// End timestamp.
    pub ended_at: Option<DateTime<Utc>>,
    /// Duration in seconds.
    pub duration_secs: Option<u64>,
    /// Exit code.
    pub exit_code: Option<i32>,
}

/// Aggregated statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Aggregates {
    /// Per-agent statistics.
    #[serde(default)]
    pub by_agent: HashMap<String, AgentStats>,
    /// Per-provider statistics.
    #[serde(default)]
    pub by_provider: HashMap<String, ProviderStats>,
    /// Per-profile statistics.
    #[serde(default)]
    pub by_profile: HashMap<String, ProfileStats>,
    /// Total sessions count.
    #[serde(default)]
    pub total_sessions: u64,
    /// Total runtime in seconds.
    #[serde(default)]
    pub total_runtime_secs: u64,
}

/// Per-agent statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentStats {
    pub sessions: u64,
    pub runtime_secs: u64,
}

/// Per-provider statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderStats {
    pub sessions: u64,
    pub runtime_secs: u64,
}

/// Per-profile statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileStats {
    pub sessions: u64,
    pub runtime_secs: u64,
    pub last_used: Option<DateTime<Utc>>,
}

/// Telemetry collector.
pub struct TelemetryCollector {
    paths: ClownPaths,
}

impl TelemetryCollector {
    /// Create a new telemetry collector.
    pub fn new(paths: ClownPaths) -> Self {
        Self { paths }
    }

    /// Record a session.
    pub fn record_session(&self, session: &Session) -> Result<()> {
        // Append to sessions.jsonl
        let sessions_path = self.paths.sessions_log();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&sessions_path)
            .context("Failed to open sessions log")?;

        let line = serde_json::to_string(session)?;
        writeln!(file, "{}", line)?;

        debug!("Recorded session for profile: {}", session.profile);

        // Update aggregates
        self.update_aggregates(session)?;

        Ok(())
    }

    /// Update aggregated statistics.
    fn update_aggregates(&self, session: &Session) -> Result<()> {
        let mut aggregates = self.load_aggregates()?;
        let duration = session.duration_secs.unwrap_or(0);

        // Update totals
        aggregates.total_sessions += 1;
        aggregates.total_runtime_secs += duration;

        // Update by-agent
        let agent_stats = aggregates
            .by_agent
            .entry(session.agent_id.clone())
            .or_default();
        agent_stats.sessions += 1;
        agent_stats.runtime_secs += duration;

        // Update by-provider
        let provider_stats = aggregates
            .by_provider
            .entry(session.provider_id.clone())
            .or_default();
        provider_stats.sessions += 1;
        provider_stats.runtime_secs += duration;

        // Update by-profile
        let profile_stats = aggregates
            .by_profile
            .entry(session.profile.clone())
            .or_default();
        profile_stats.sessions += 1;
        profile_stats.runtime_secs += duration;
        profile_stats.last_used = session.ended_at;

        self.save_aggregates(&aggregates)?;
        Ok(())
    }

    /// Load aggregated statistics.
    pub fn load_aggregates(&self) -> Result<Aggregates> {
        let path = self.paths.aggregates_file();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Aggregates::default())
        }
    }

    /// Save aggregated statistics.
    fn save_aggregates(&self, aggregates: &Aggregates) -> Result<()> {
        let path = self.paths.aggregates_file();
        let content = serde_json::to_string_pretty(aggregates)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get statistics, optionally filtered by agent or provider.
    pub fn get_stats(
        &self,
        agent_id: Option<&str>,
        provider_id: Option<&str>,
    ) -> Result<Aggregates> {
        let aggregates = self.load_aggregates()?;

        if agent_id.is_none() && provider_id.is_none() {
            return Ok(aggregates);
        }

        // Filter if needed
        let mut filtered = Aggregates::default();

        if let Some(aid) = agent_id {
            if let Some(stats) = aggregates.by_agent.get(aid) {
                filtered.by_agent.insert(aid.to_string(), stats.clone());
                filtered.total_sessions += stats.sessions;
                filtered.total_runtime_secs += stats.runtime_secs;
            }
        }

        if let Some(pid) = provider_id {
            if let Some(stats) = aggregates.by_provider.get(pid) {
                filtered.by_provider.insert(pid.to_string(), stats.clone());
                // Only add if not already counted
                if agent_id.is_none() {
                    filtered.total_sessions += stats.sessions;
                    filtered.total_runtime_secs += stats.runtime_secs;
                }
            }
        }

        // If no filters matched, return empty
        if agent_id.is_none() && provider_id.is_none() {
            filtered = aggregates;
        }

        Ok(filtered)
    }

    /// Load recent sessions.
    pub fn load_recent_sessions(&self, limit: usize) -> Result<Vec<Session>> {
        let sessions_path = self.paths.sessions_log();
        if !sessions_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&sessions_path)?;
        let reader = BufReader::new(file);

        let mut sessions: Vec<Session> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| serde_json::from_str(&line).ok())
            .collect();

        // Return last N sessions
        let start = sessions.len().saturating_sub(limit);
        Ok(sessions.split_off(start))
    }
}
