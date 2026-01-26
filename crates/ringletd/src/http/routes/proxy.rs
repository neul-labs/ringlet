//! Proxy HTTP handlers.

use crate::handlers;
use crate::http::error::{ApiResponse, HttpError};
use crate::server::ServerState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use ringlet_core::{ProfileProxyConfig, ProxyInstanceInfo, Response, RoutingRule};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

/// POST /api/profiles/:alias/proxy/enable - Enable proxy for profile.
pub async fn enable(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::proxy::enable(&alias, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// POST /api/profiles/:alias/proxy/disable - Disable proxy for profile.
pub async fn disable(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::proxy::disable(&alias, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// POST /api/profiles/:alias/proxy/start - Start proxy for profile.
pub async fn start(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::proxy::start(&alias, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// POST /api/profiles/:alias/proxy/stop - Stop proxy for profile.
pub async fn stop(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::proxy::stop(&alias, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// POST /api/profiles/:alias/proxy/restart - Restart proxy for profile.
pub async fn restart(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    // Stop then start
    let _ = handlers::proxy::stop(&alias, &state).await;
    let response = handlers::proxy::start(&alias, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/profiles/:alias/proxy/status - Get proxy status for profile.
pub async fn status_single(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<Vec<ProxyInstanceInfo>>>, HttpError> {
    let response = handlers::proxy::status(Some(&alias), &state).await;

    match response {
        Response::ProxyStatus(status) => Ok(Json(ApiResponse::success(status))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/proxy/status - Get all proxy statuses.
pub async fn status_all(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<ProxyInstanceInfo>>>, HttpError> {
    let response = handlers::proxy::status(None, &state).await;

    match response {
        Response::ProxyStatus(status) => Ok(Json(ApiResponse::success(status))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// POST /api/proxy/stop-all - Stop all proxies.
pub async fn stop_all(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::proxy::stop_all(&state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/profiles/:alias/proxy/config - Get proxy config.
pub async fn config(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<ProfileProxyConfig>>, HttpError> {
    let response = handlers::proxy::config(&alias, &state).await;

    match response {
        Response::ProxyConfig(config) => Ok(Json(ApiResponse::success(config))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub lines: Option<usize>,
}

/// GET /api/profiles/:alias/proxy/logs - Get proxy logs.
pub async fn logs(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<ApiResponse<String>>, HttpError> {
    let response = handlers::proxy::logs(&alias, query.lines, &state).await;

    match response {
        Response::ProxyLogs(logs) => Ok(Json(ApiResponse::success(logs))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/profiles/:alias/proxy/routes - List routing rules.
pub async fn route_list(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<Vec<RoutingRule>>>, HttpError> {
    let response = handlers::proxy::route_list(&alias, &state).await;

    match response {
        Response::ProxyRoutes(routes) => Ok(Json(ApiResponse::success(routes))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// POST /api/profiles/:alias/proxy/routes - Add routing rule.
pub async fn route_add(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
    Json(rule): Json<RoutingRule>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::proxy::route_add(&alias, &rule, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// DELETE /api/profiles/:alias/proxy/routes/:name - Remove routing rule.
pub async fn route_remove(
    State(state): State<Arc<ServerState>>,
    Path((alias, name)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::proxy::route_remove(&alias, &name, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/profiles/:alias/proxy/aliases - List model aliases.
pub async fn alias_list(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<HashMap<String, String>>>, HttpError> {
    let response = handlers::proxy::alias_list(&alias, &state).await;

    match response {
        Response::ProxyAliases(aliases) => Ok(Json(ApiResponse::success(aliases))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

#[derive(Debug, Deserialize)]
pub struct SetAliasRequest {
    pub to: String,
}

/// PUT /api/profiles/:alias/proxy/aliases/:from - Set model alias.
pub async fn alias_set(
    State(state): State<Arc<ServerState>>,
    Path((alias, from)): Path<(String, String)>,
    Json(request): Json<SetAliasRequest>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::proxy::alias_set(&alias, &from, &request.to, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// DELETE /api/profiles/:alias/proxy/aliases/:from - Remove model alias.
pub async fn alias_remove(
    State(state): State<Arc<ServerState>>,
    Path((alias, from)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::proxy::alias_remove(&alias, &from, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}
