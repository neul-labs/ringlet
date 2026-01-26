//! Registry-related request handlers.

use crate::registry_client::RegistryClient;
use crate::server::ServerState;
use ringlet_core::rpc::{error_codes, RegistryStatus};
use ringlet_core::Response;
use tracing::info;

/// Sync registry from remote.
pub async fn sync(force: bool, offline: bool, state: &ServerState) -> Response {
    info!("Syncing registry (force={}, offline={})", force, offline);

    match state.registry_client.sync(force, offline) {
        Ok(status) => Response::RegistryStatus(RegistryStatus {
            commit: status.commit,
            channel: status.channel,
            last_sync: status.last_sync,
            offline: status.offline,
            cached_agents: status.cached_agents,
            cached_providers: status.cached_providers,
            cached_scripts: status.cached_scripts,
        }),
        Err(e) => Response::error(
            error_codes::REGISTRY_ERROR,
            format!("Failed to sync registry: {}", e),
        ),
    }
}

/// Pin to a specific ref.
pub async fn pin(ref_: &str, state: &ServerState) -> Response {
    info!("Pinning to ref: {}", ref_);

    match state.registry_client.pin(ref_) {
        Ok(()) => Response::success(format!("Pinned to: {}", ref_)),
        Err(e) => Response::error(
            error_codes::REGISTRY_ERROR,
            format!("Failed to pin: {}", e),
        ),
    }
}

/// Inspect registry status.
pub async fn inspect(state: &ServerState) -> Response {
    match state.registry_client.get_status(false) {
        Ok(status) => Response::RegistryStatus(RegistryStatus {
            commit: status.commit,
            channel: status.channel,
            last_sync: status.last_sync,
            offline: status.offline,
            cached_agents: status.cached_agents,
            cached_providers: status.cached_providers,
            cached_scripts: status.cached_scripts,
        }),
        Err(e) => Response::error(
            error_codes::REGISTRY_ERROR,
            format!("Failed to get registry status: {}", e),
        ),
    }
}
