//! Provider HTTP handlers.

use crate::handlers;
use crate::http::error::{ApiResponse, HttpError};
use crate::server::ServerState;
use axum::{
    extract::{Path, State},
    Json,
};
use ringlet_core::{ProviderInfo, Response};
use std::sync::Arc;

/// GET /api/providers - List all providers.
pub async fn list(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<ProviderInfo>>>, HttpError> {
    let response = handlers::providers::list(&state).await;

    match response {
        Response::Providers(providers) => Ok(Json(ApiResponse::success(providers))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/providers/:id - Get provider details.
pub async fn inspect(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ProviderInfo>>, HttpError> {
    let response = handlers::providers::inspect(&id, &state).await;

    match response {
        Response::Provider(provider) => Ok(Json(ApiResponse::success(provider))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}
