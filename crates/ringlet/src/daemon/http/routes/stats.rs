//! Stats HTTP handlers.

use crate::daemon::handlers;
use crate::daemon::http::error::{ApiResponse, HttpError};
use crate::daemon::server::ServerState;
use axum::{
    extract::{Query, State},
    Json,
};
use ringlet_core::{Response, StatsResponse};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub agent: Option<String>,
    pub provider: Option<String>,
}

/// GET /api/stats - Get usage statistics.
pub async fn get_stats(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<ApiResponse<StatsResponse>>, HttpError> {
    let response =
        handlers::stats::get_stats(query.agent.as_deref(), query.provider.as_deref(), &state).await;

    match response {
        Response::Stats(stats) => Ok(Json(ApiResponse::success(stats))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}
