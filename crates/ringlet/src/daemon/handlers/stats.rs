//! Stats-related request handlers.

use crate::daemon::server::ServerState;
use ringlet_core::Response;
use ringlet_core::rpc::{AgentStats, ProfileStats, ProviderStats, StatsResponse, error_codes};
use std::collections::{HashMap, HashSet};

/// Get usage statistics.
pub async fn get_stats(
    agent_id: Option<&str>,
    provider_id: Option<&str>,
    state: &ServerState,
) -> Response {
    match state.telemetry.load_all_sessions() {
        Ok(sessions) => {
            let filtered_sessions: Vec<_> = sessions
                .into_iter()
                .filter(|session| {
                    agent_id.is_none_or(|aid| session.agent_id == aid)
                        && provider_id.is_none_or(|pid| session.provider_id == pid)
                })
                .collect();
            let aggregates = crate::daemon::telemetry::TelemetryCollector::aggregate_sessions(
                &filtered_sessions,
            );

            let mut agent_profiles: HashMap<String, HashSet<String>> = HashMap::new();
            for session in &filtered_sessions {
                agent_profiles
                    .entry(session.agent_id.clone())
                    .or_default()
                    .insert(session.profile.clone());
            }

            let by_agent: HashMap<String, AgentStats> = aggregates
                .by_agent
                .into_iter()
                .map(|(k, v)| {
                    let profiles = agent_profiles.get(&k).map_or(0, HashSet::len);
                    (
                        k,
                        AgentStats {
                            sessions: v.sessions,
                            runtime_secs: v.runtime_secs,
                            profiles,
                        },
                    )
                })
                .collect();

            let by_provider: HashMap<String, ProviderStats> = aggregates
                .by_provider
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        ProviderStats {
                            sessions: v.sessions,
                            runtime_secs: v.runtime_secs,
                        },
                    )
                })
                .collect();

            let by_profile: HashMap<String, ProfileStats> = aggregates
                .by_profile
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        ProfileStats {
                            sessions: v.sessions,
                            runtime_secs: v.runtime_secs,
                            last_used: v.last_used,
                        },
                    )
                })
                .collect();

            Response::Stats(StatsResponse {
                by_agent,
                by_provider,
                by_profile,
                total_sessions: aggregates.total_sessions,
                total_runtime_secs: aggregates.total_runtime_secs,
            })
        }
        Err(e) => Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Failed to get stats: {}", e),
        ),
    }
}
