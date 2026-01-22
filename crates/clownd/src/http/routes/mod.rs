//! HTTP route handlers.

pub mod agents;
pub mod hooks;
pub mod profiles;
pub mod providers;
pub mod proxy;
pub mod registry;
pub mod stats;
pub mod system;
pub mod terminal;
pub mod usage;

use crate::server::ServerState;
use axum::{routing::get, routing::post, routing::delete, Router};
use std::sync::Arc;

/// Build all API routes.
pub fn api_routes() -> Router<Arc<ServerState>> {
    Router::new()
        // Agents
        .route("/agents", get(agents::list))
        .route("/agents/{id}", get(agents::inspect))
        // Providers
        .route("/providers", get(providers::list))
        .route("/providers/{id}", get(providers::inspect))
        // Profiles
        .route("/profiles", get(profiles::list).post(profiles::create))
        .route(
            "/profiles/{alias}",
            get(profiles::inspect).delete(profiles::delete),
        )
        .route("/profiles/{alias}/run", post(profiles::run))
        .route("/profiles/{alias}/env", get(profiles::env))
        // Hooks
        .route(
            "/profiles/{alias}/hooks",
            get(hooks::list).post(hooks::add),
        )
        .route("/profiles/{alias}/hooks/{event}/{index}", delete(hooks::remove))
        .route("/profiles/{alias}/hooks/import", post(hooks::import))
        .route("/profiles/{alias}/hooks/export", get(hooks::export))
        // Proxy per-profile
        .route("/profiles/{alias}/proxy/enable", post(proxy::enable))
        .route("/profiles/{alias}/proxy/disable", post(proxy::disable))
        .route("/profiles/{alias}/proxy/start", post(proxy::start))
        .route("/profiles/{alias}/proxy/stop", post(proxy::stop))
        .route("/profiles/{alias}/proxy/restart", post(proxy::restart))
        .route("/profiles/{alias}/proxy/status", get(proxy::status_single))
        .route("/profiles/{alias}/proxy/config", get(proxy::config))
        .route("/profiles/{alias}/proxy/logs", get(proxy::logs))
        .route(
            "/profiles/{alias}/proxy/routes",
            get(proxy::route_list).post(proxy::route_add),
        )
        .route("/profiles/{alias}/proxy/routes/{name}", delete(proxy::route_remove))
        .route(
            "/profiles/{alias}/proxy/aliases",
            get(proxy::alias_list),
        )
        .route(
            "/profiles/{alias}/proxy/aliases/{from}",
            axum::routing::put(proxy::alias_set).delete(proxy::alias_remove),
        )
        // Proxy global
        .route("/proxy/status", get(proxy::status_all))
        .route("/proxy/stop-all", post(proxy::stop_all))
        // Registry
        .route("/registry", get(registry::inspect))
        .route("/registry/sync", post(registry::sync))
        .route("/registry/pin", post(registry::pin))
        // Stats (legacy)
        .route("/stats", get(stats::get_stats))
        // Usage
        .route("/usage", get(usage::get_usage))
        .route("/usage/import-claude", post(usage::import_claude))
        // System
        .route("/ping", get(system::ping))
        .route("/shutdown", post(system::shutdown))
        // Terminal sessions
        .route(
            "/terminal/sessions",
            get(terminal::list_sessions).post(terminal::create_session),
        )
        .route(
            "/terminal/sessions/{id}",
            get(terminal::get_session).delete(terminal::terminate_session),
        )
        .route("/terminal/cleanup", post(terminal::cleanup_sessions))
}
