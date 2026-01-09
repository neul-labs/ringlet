//! Agent-related request handlers.

use crate::server::ServerState;
use clown_core::{rpc::error_codes, Response};
use std::collections::HashMap;

/// List all agents.
pub async fn list(state: &ServerState) -> Response {
    let mut agent_registry = state.agent_registry.lock().await;

    // Get profile counts per agent
    let profile_counts = get_profile_counts(state).await;

    let agents = agent_registry.list_all(&profile_counts);
    Response::Agents(agents)
}

/// Inspect a specific agent.
pub async fn inspect(id: &str, state: &ServerState) -> Response {
    let mut agent_registry = state.agent_registry.lock().await;

    // Get profile count for this agent
    let profile_counts = get_profile_counts(state).await;
    let profile_count = *profile_counts.get(id).unwrap_or(&0);

    match agent_registry.get_info(id, profile_count) {
        Some(agent) => Response::Agent(agent),
        None => Response::error(
            error_codes::AGENT_NOT_FOUND,
            format!("Agent not found: {}", id),
        ),
    }
}

/// Get profile counts per agent by scanning the profiles directory.
async fn get_profile_counts(state: &ServerState) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    let profiles_dir = state.paths.profiles_dir();

    if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(profile) = serde_json::from_str::<clown_core::Profile>(&content) {
                        *counts.entry(profile.agent_id).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    counts
}
