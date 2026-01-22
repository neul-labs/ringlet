//! Terminal session HTTP handlers.

use crate::http::error::{ApiResponse, HttpError};
use crate::server::ServerState;
use crate::terminal::{SessionId, TerminalSessionInfo};
use axum::{
    extract::{Path, State},
    Json,
};
use clown_core::rpc::error_codes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Request to create a new terminal session.
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// Profile alias to run.
    pub profile_alias: String,
    /// Additional arguments to pass to the agent.
    #[serde(default)]
    pub args: Vec<String>,
    /// Initial terminal columns.
    #[serde(default = "default_cols")]
    pub cols: u16,
    /// Initial terminal rows.
    #[serde(default = "default_rows")]
    pub rows: u16,
    /// Working directory for the session (defaults to profile's home).
    pub working_dir: Option<String>,
}

fn default_cols() -> u16 {
    80
}

fn default_rows() -> u16 {
    24
}

/// Response for session creation.
#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    /// Created session ID.
    pub session_id: SessionId,
    /// WebSocket URL to connect to.
    pub ws_url: String,
}

/// GET /api/terminal/sessions - List all terminal sessions.
pub async fn list_sessions(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<TerminalSessionInfo>>>, HttpError> {
    let sessions = state.terminal_sessions.list_sessions().await;
    Ok(Json(ApiResponse::success(sessions)))
}

/// GET /api/terminal/sessions/:id - Get session info.
pub async fn get_session(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<TerminalSessionInfo>>, HttpError> {
    let session = state
        .terminal_sessions
        .get_session(&session_id)
        .await
        .ok_or_else(|| HttpError::new(error_codes::PROFILE_NOT_FOUND, "Session not found"))?;

    let info = session.info().await;
    Ok(Json(ApiResponse::success(info)))
}

/// POST /api/terminal/sessions - Create a new terminal session.
pub async fn create_session(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<ApiResponse<CreateSessionResponse>>, HttpError> {
    // Get the profile
    let profile = state
        .profile_manager
        .get(&request.profile_alias)
        .map_err(|e| HttpError::new(error_codes::INTERNAL_ERROR, e.to_string()))?
        .ok_or_else(|| HttpError::new(error_codes::PROFILE_NOT_FOUND, "Profile not found"))?;

    // Get agent info
    let agent_registry = state.agent_registry.lock().await;
    let agent = agent_registry
        .get(&profile.agent_id)
        .ok_or_else(|| HttpError::new(error_codes::AGENT_NOT_FOUND, "Agent not found"))?;

    // Build command and args
    let command = agent.binary.clone();
    let args = request.args;

    // Build environment from profile
    let mut env: HashMap<String, String> = profile.env.clone();

    // Add HOME override to point to profile's home directory
    env.insert(
        "HOME".to_string(),
        profile.metadata.home.to_string_lossy().to_string(),
    );

    // Use provided working directory or fall back to profile's home
    let working_dir = request
        .working_dir
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| profile.metadata.home.clone());

    // Create the session
    let initial_size = portable_pty::PtySize {
        rows: request.rows,
        cols: request.cols,
        pixel_width: 0,
        pixel_height: 0,
    };

    let session = state
        .terminal_sessions
        .create_session(
            &request.profile_alias,
            &command,
            &args,
            env,
            &working_dir,
            Some(initial_size),
        )
        .await
        .map_err(|e| HttpError::new(error_codes::EXECUTION_ERROR, e.to_string()))?;

    let session_id = session.id.clone();

    // Build WebSocket URL (relative)
    let ws_url = format!("/ws/terminal/{}", session_id);

    Ok(Json(ApiResponse::success(CreateSessionResponse {
        session_id,
        ws_url,
    })))
}

/// DELETE /api/terminal/sessions/:id - Terminate a session.
pub async fn terminate_session(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    state
        .terminal_sessions
        .terminate_session(&session_id)
        .await
        .map_err(|e| HttpError::new(error_codes::PROFILE_NOT_FOUND, e.to_string()))?;

    Ok(Json(ApiResponse::ok()))
}

/// POST /api/terminal/cleanup - Clean up terminated sessions.
pub async fn cleanup_sessions(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    state.terminal_sessions.cleanup_terminated().await;
    Ok(Json(ApiResponse::ok()))
}
