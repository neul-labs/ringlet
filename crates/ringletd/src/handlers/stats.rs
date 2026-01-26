//! Stats-related request handlers.

use crate::server::ServerState;
use ringlet_core::rpc::{error_codes, AgentStats, ProfileStats, ProviderStats, StatsResponse};
use ringlet_core::Response;
use std::collections::HashMap;

/// Get usage statistics.
pub async fn get_stats(
    agent_id: Option<&str>,
    provider_id: Option<&str>,
    state: &ServerState,
) -> Response {
    match state
        .telemetry
        .get_stats(agent_id, provider_id)
    {
        Ok(aggregates) => {
            // Convert to response types
            let by_agent: HashMap<String, AgentStats> = aggregates
                .by_agent
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        AgentStats {
                            sessions: v.sessions,
                            runtime_secs: v.runtime_secs,
                            profiles: 0, // TODO: count profiles
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
