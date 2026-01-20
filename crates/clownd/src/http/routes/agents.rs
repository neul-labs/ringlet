//! Agent HTTP handlers.

use crate::handlers;
use crate::http::error::{ApiResponse, HttpError};
use crate::server::ServerState;
use axum::{
    extract::{Path, State},
    Json,
};
use clown_core::{AgentInfo, Response};
use std::sync::Arc;

/// GET /api/agents - List all agents.
pub async fn list(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<AgentInfo>>>, HttpError> {
    let response = handlers::agents::list(&state).await;

    match response {
        Response::Agents(agents) => Ok(Json(ApiResponse::success(agents))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/agents/:id - Get agent details.
pub async fn inspect(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<AgentInfo>>, HttpError> {
    let response = handlers::agents::inspect(&id, &state).await;

    match response {
        Response::Agent(agent) => Ok(Json(ApiResponse::success(agent))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}
