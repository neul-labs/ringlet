//! System HTTP handlers.

use crate::daemon::handlers;
use crate::daemon::http::error::{ApiResponse, HttpError};
use crate::daemon::server::ServerState;
use axum::{Json, extract::State};
use ringlet_core::http_api::PingResponse;
use std::sync::Arc;

/// GET /api/ping - Health check.
pub async fn ping(State(_state): State<Arc<ServerState>>) -> Json<ApiResponse<PingResponse>> {
    Json(ApiResponse::success(PingResponse {
        status: "ok".to_string(),
        version: ringlet_core::VERSION.to_string(),
    }))
}

/// POST /api/shutdown - Shutdown the daemon.
pub async fn shutdown(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    handlers::system::shutdown(&state).await;
    Ok(Json(ApiResponse::ok()))
}
