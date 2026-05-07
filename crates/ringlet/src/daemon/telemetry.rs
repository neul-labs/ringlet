//! Telemetry collection - tracks profile usage statistics.
//!
//! This module handles:
//! - Tracking per-session data (profile, start time, duration, exit code)
//! - Token usage and cost tracking (costs only for "self" provider)
//! - Persisting sessions to sessions.jsonl
//! - Aggregating statistics

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use ringlet_core::{CostBreakdown, DailyUsage, ModelUsage, ProfileUsage, RingletPaths, TokenUsage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tracing::{debug, warn};

/// A recorded session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Stable Ringlet session identifier.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub session_id: String,
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
    /// Where the session was launched from.
    #[serde(default)]
    pub source: SessionSource,
    /// Model used (if known).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Token usage (always tracked when available).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenUsage>,
    /// Cost breakdown (only for "self" provider).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<CostBreakdown>,
}

/// Where a session was launched from.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionSource {
    #[default]
    ProfileRun,
    TerminalSession,
    ShellSession,
}

/// Context used to record terminal-session telemetry after PTY exit.
#[derive(Debug, Clone)]
pub struct SessionTelemetryContext {
    pub session_id: String,
    pub profile: String,
    pub agent_id: String,
    pub provider_id: String,
    pub model: Option<String>,
    pub source: SessionSource,
    pub profile_home: PathBuf,
    pub usage_baseline: Option<crate::daemon::agent_usage::UsageSnapshot>,
    pub paths: RingletPaths,
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
    pub by_profile: HashMap<String, ProfileUsage>,
    /// Per-date statistics.
    #[serde(default)]
    pub by_date: HashMap<String, DailyUsage>,
    /// Per-model statistics.
    #[serde(default)]
    pub by_model: HashMap<String, ModelUsage>,
    /// Total sessions count.
    #[serde(default)]
    pub total_sessions: u64,
    /// Total runtime in seconds.
    #[serde(default)]
    pub total_runtime_secs: u64,
    /// Total token usage.
    #[serde(default)]
    pub total_tokens: TokenUsage,
    /// Total cost (only from "self" provider profiles).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_cost: Option<CostBreakdown>,
}

/// Per-agent statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentStats {
    pub sessions: u64,
    pub runtime_secs: u64,
    #[serde(default)]
    pub tokens: TokenUsage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<CostBreakdown>,
}

/// Per-provider statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderStats {
    pub sessions: u64,
    pub runtime_secs: u64,
    #[serde(default)]
    pub tokens: TokenUsage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<CostBreakdown>,
}

/// Telemetry collector.
pub struct TelemetryCollector {
    paths: RingletPaths,
}

impl TelemetryCollector {
    /// Create a new telemetry collector.
    pub fn new(paths: RingletPaths) -> Self {
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
        Self::accumulate_session(&mut aggregates, session);

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
        let sessions = self.load_all_sessions()?;
        let filtered: Vec<Session> = sessions
            .into_iter()
            .filter(|session| {
                agent_id.is_none_or(|aid| session.agent_id == aid)
                    && provider_id.is_none_or(|pid| session.provider_id == pid)
            })
            .collect();

        Ok(Self::aggregate_sessions(&filtered))
    }

    /// Load all recorded sessions.
    pub fn load_all_sessions(&self) -> Result<Vec<Session>> {
        let sessions_path = self.paths.sessions_log();
        if !sessions_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&sessions_path)?;
        let reader = BufReader::new(file);

        Ok(reader
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| match serde_json::from_str(&line) {
                Ok(session) => Some(session),
                Err(err) => {
                    warn!("Skipping invalid telemetry session record: {}", err);
                    None
                }
            })
            .collect())
    }

    /// Load recent sessions.
    pub fn load_recent_sessions(&self, limit: usize) -> Result<Vec<Session>> {
        let mut sessions = self.load_all_sessions()?;

        // Return last N sessions
        let start = sessions.len().saturating_sub(limit);
        Ok(sessions.split_off(start))
    }

    /// Build aggregates from a filtered set of sessions.
    pub fn aggregate_sessions(sessions: &[Session]) -> Aggregates {
        let mut aggregates = Aggregates::default();
        for session in sessions {
            Self::accumulate_session(&mut aggregates, session);
        }
        aggregates
    }

    fn accumulate_session(aggregates: &mut Aggregates, session: &Session) {
        let duration = session.duration_secs.unwrap_or(0);

        aggregates.total_sessions += 1;
        aggregates.total_runtime_secs += duration;

        if let Some(ref tokens) = session.tokens {
            aggregates.total_tokens += tokens.clone();
        }

        if let Some(ref cost) = session.cost {
            if let Some(ref mut total_cost) = aggregates.total_cost {
                *total_cost += cost.clone();
            } else {
                aggregates.total_cost = Some(cost.clone());
            }
        }

        let agent_stats = aggregates
            .by_agent
            .entry(session.agent_id.clone())
            .or_default();
        agent_stats.sessions += 1;
        agent_stats.runtime_secs += duration;
        if let Some(ref tokens) = session.tokens {
            agent_stats.tokens += tokens.clone();
        }
        if let Some(ref cost) = session.cost {
            if let Some(ref mut agent_cost) = agent_stats.cost {
                *agent_cost += cost.clone();
            } else {
                agent_stats.cost = Some(cost.clone());
            }
        }

        let provider_stats = aggregates
            .by_provider
            .entry(session.provider_id.clone())
            .or_default();
        provider_stats.sessions += 1;
        provider_stats.runtime_secs += duration;
        if let Some(ref tokens) = session.tokens {
            provider_stats.tokens += tokens.clone();
        }
        if let Some(ref cost) = session.cost {
            if let Some(ref mut provider_cost) = provider_stats.cost {
                *provider_cost += cost.clone();
            } else {
                provider_stats.cost = Some(cost.clone());
            }
        }

        let profile_stats = aggregates
            .by_profile
            .entry(session.profile.clone())
            .or_insert_with(|| ProfileUsage {
                profile: session.profile.clone(),
                provider_id: session.provider_id.clone(),
                ..Default::default()
            });
        profile_stats.sessions += 1;
        profile_stats.runtime_secs += duration;
        profile_stats.last_used = session.ended_at;
        if let Some(ref tokens) = session.tokens {
            profile_stats.tokens += tokens.clone();
        }
        if let Some(ref cost) = session.cost {
            if let Some(ref mut profile_cost) = profile_stats.cost {
                *profile_cost += cost.clone();
            } else {
                profile_stats.cost = Some(cost.clone());
            }
        }

        let date_key = session
            .ended_at
            .unwrap_or(session.started_at)
            .date_naive()
            .to_string();
        let daily_stats = aggregates
            .by_date
            .entry(date_key.clone())
            .or_insert_with(|| DailyUsage {
                date: date_key.clone(),
                ..Default::default()
            });
        daily_stats.sessions += 1;
        if let Some(ref tokens) = session.tokens {
            daily_stats.tokens += tokens.clone();
        }
        if let Some(ref cost) = session.cost {
            if let Some(ref mut daily_cost) = daily_stats.cost {
                *daily_cost += cost.clone();
            } else {
                daily_stats.cost = Some(cost.clone());
            }
        }

        if let Some(ref model) = session.model {
            let model_stats =
                aggregates
                    .by_model
                    .entry(model.clone())
                    .or_insert_with(|| ModelUsage {
                        model: model.clone(),
                        ..Default::default()
                    });
            model_stats.sessions += 1;
            if let Some(ref tokens) = session.tokens {
                model_stats.tokens += tokens.clone();
            }
            if let Some(ref cost) = session.cost {
                if let Some(ref mut model_cost) = model_stats.cost {
                    *model_cost += cost.clone();
                } else {
                    model_stats.cost = Some(cost.clone());
                }
            }
        }
    }
}
