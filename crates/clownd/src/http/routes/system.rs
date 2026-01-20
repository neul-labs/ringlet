//! System HTTP handlers.

use crate::http::error::{ApiResponse, HttpError};
use crate::server::ServerState;
use axum::{
    extract::State,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub status: String,
    pub version: String,
}

/// GET /api/ping - Health check.
pub async fn ping(
    State(_state): State<Arc<ServerState>>,
) -> Json<ApiResponse<PingResponse>> {
    Json(ApiResponse::success(PingResponse {
        status: "ok".to_string(),
        version: clown_core::VERSION.to_string(),
    }))
}

/// POST /api/shutdown - Shutdown the daemon.
pub async fn shutdown(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    // Signal shutdown via the shutdown channel if available
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        let _ = tx.send(());
    }
    Ok(Json(ApiResponse::ok()))
}
