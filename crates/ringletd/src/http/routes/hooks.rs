//! Hooks HTTP handlers.

use crate::handlers;
use crate::http::error::{ApiResponse, HttpError};
use crate::server::ServerState;
use axum::{
    extract::{Path, State},
    Json,
};
use ringlet_core::{HooksConfig, Response};
use serde::Deserialize;
use std::sync::Arc;

/// GET /api/profiles/:alias/hooks - List hooks.
pub async fn list(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<HooksConfig>>, HttpError> {
    let response = handlers::hooks::list(&alias, &state).await;

    match response {
        Response::Hooks(hooks) => Ok(Json(ApiResponse::success(hooks))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

#[derive(Debug, Deserialize)]
pub struct AddHookRequest {
    pub event: String,
    pub matcher: String,
    pub command: String,
}

/// POST /api/profiles/:alias/hooks - Add a hook.
pub async fn add(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
    Json(request): Json<AddHookRequest>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::hooks::add(
        &alias,
        &request.event,
        &request.matcher,
        &request.command,
        &state,
    )
    .await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

#[derive(Debug, Deserialize)]
pub struct RemoveHookPath {
    pub alias: String,
    pub event: String,
    pub index: usize,
}

/// DELETE /api/profiles/:alias/hooks/:event/:index - Remove a hook.
pub async fn remove(
    State(state): State<Arc<ServerState>>,
    Path((alias, event, index)): Path<(String, String, usize)>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::hooks::remove(&alias, &event, index, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// POST /api/profiles/:alias/hooks/import - Import hooks config.
pub async fn import(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
    Json(config): Json<HooksConfig>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::hooks::import(&alias, &config, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/profiles/:alias/hooks/export - Export hooks config.
pub async fn export(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<HooksConfig>>, HttpError> {
    let response = handlers::hooks::export(&alias, &state).await;

    match response {
        Response::Hooks(hooks) => Ok(Json(ApiResponse::success(hooks))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}
