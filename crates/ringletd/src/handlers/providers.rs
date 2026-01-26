//! Provider-related request handlers.

use crate::server::ServerState;
use ringlet_core::{rpc::error_codes, Response};

/// List all providers.
pub async fn list(state: &ServerState) -> Response {
    let providers = state.provider_registry.list_all();
    Response::Providers(providers)
}

/// Inspect a specific provider.
pub async fn inspect(id: &str, state: &ServerState) -> Response {
    match state.provider_registry.get_info(id) {
        Some(provider) => Response::Provider(provider),
        None => Response::error(
            error_codes::PROVIDER_NOT_FOUND,
            format!("Provider not found: {}", id),
        ),
    }
}
