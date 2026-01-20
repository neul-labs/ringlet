//! Registry HTTP handlers.

use crate::handlers;
use crate::http::error::{ApiResponse, HttpError};
use crate::server::ServerState;
use axum::{
    extract::State,
    Json,
};
use clown_core::{RegistryStatus, Response};
use serde::Deserialize;
use std::sync::Arc;

/// GET /api/registry - Get registry status.
pub async fn inspect(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<RegistryStatus>>, HttpError> {
    let response = handlers::registry::inspect(&state).await;

    match response {
        Response::RegistryStatus(status) => Ok(Json(ApiResponse::success(status))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub offline: bool,
}

/// POST /api/registry/sync - Sync registry.
pub async fn sync(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<SyncRequest>,
) -> Result<Json<ApiResponse<RegistryStatus>>, HttpError> {
    let response = handlers::registry::sync(request.force, request.offline, &state).await;

    match response {
        Response::RegistryStatus(status) => Ok(Json(ApiResponse::success(status))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

#[derive(Debug, Deserialize)]
pub struct PinRequest {
    #[serde(rename = "ref")]
    pub ref_: String,
}

/// POST /api/registry/pin - Pin registry to a specific ref.
pub async fn pin(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<PinRequest>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::registry::pin(&request.ref_, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}
