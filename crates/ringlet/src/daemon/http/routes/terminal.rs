//! Terminal session HTTP handlers.

use crate::daemon::handlers;
use crate::daemon::http::auth::AuthenticatedTokenHash;
use crate::daemon::http::error::{ApiResponse, HttpError};
use crate::daemon::http::terminal_policy::{
    build_shell_environment, resolve_working_dir, validate_shell,
};
use crate::daemon::server::ServerState;
use crate::daemon::terminal::{SandboxConfig, TerminalSessionInfo};
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use ringlet_core::http_api::{
    CreateShellRequest, CreateTerminalSessionRequest, CreateTerminalSessionResponse,
};
use ringlet_core::rpc::error_codes;
use std::path::PathBuf;
use std::sync::Arc;

/// GET /api/terminal/sessions - List all terminal sessions.
pub async fn list_sessions(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<TerminalSessionInfo>>>, HttpError> {
    let sessions = handlers::terminal::list(&state).await;
    Ok(Json(ApiResponse::success(sessions)))
}

/// GET /api/terminal/sessions/:id - Get session info.
pub async fn get_session(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<TerminalSessionInfo>>, HttpError> {
    let info = handlers::terminal::get(&session_id, &state)
        .await
        .ok_or_else(|| HttpError::new(error_codes::PROFILE_NOT_FOUND, "Session not found"))?;
    Ok(Json(ApiResponse::success(info)))
}

/// POST /api/terminal/sessions - Create a new terminal session.
pub async fn create_session(
    State(state): State<Arc<ServerState>>,
    Extension(token_hash): Extension<AuthenticatedTokenHash>,
    Json(request): Json<CreateTerminalSessionRequest>,
) -> Result<Json<ApiResponse<CreateTerminalSessionResponse>>, HttpError> {
    let working_dir = request
        .working_dir
        .as_ref()
        .map(|dir| resolve_working_dir(&PathBuf::from(dir)))
        .transpose()?;

    // Create the session
    let initial_size = portable_pty::PtySize {
        rows: request.rows,
        cols: request.cols,
        pixel_width: 0,
        pixel_height: 0,
    };

    // Build sandbox configuration
    let sandbox_config = SandboxConfig {
        enabled: !request.no_sandbox,
        bwrap_flags: request.bwrap_flags,
        sandbox_exec_profile: request.sandbox_exec_profile,
    };

    let created = handlers::terminal::create_profile_session(
        &request.profile_alias,
        &request.args,
        working_dir.as_deref(),
        initial_size,
        sandbox_config,
        token_hash.0,
        &state,
    )
    .await
    .map_err(|message| HttpError::new(error_codes::EXECUTION_ERROR, message))?;

    // Build WebSocket URL (relative)
    let ws_url = format!("/ws/terminal/{}", created.session_id);

    Ok(Json(ApiResponse::success(CreateTerminalSessionResponse {
        session_id: created.session_id,
        ws_url,
    })))
}

/// DELETE /api/terminal/sessions/:id - Terminate a session.
pub async fn terminate_session(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    handlers::terminal::terminate(&session_id, &state)
        .await
        .map_err(|message| HttpError::new(error_codes::PROFILE_NOT_FOUND, message))?;

    Ok(Json(ApiResponse::ok()))
}

/// POST /api/terminal/cleanup - Clean up terminated sessions.
pub async fn cleanup_sessions(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    handlers::terminal::cleanup(&state).await;
    Ok(Json(ApiResponse::ok()))
}

/// POST /api/terminal/shell - Create a shell session without a profile.
pub async fn create_shell_session(
    State(state): State<Arc<ServerState>>,
    Extension(token_hash): Extension<AuthenticatedTokenHash>,
    Json(request): Json<CreateShellRequest>,
) -> Result<Json<ApiResponse<CreateTerminalSessionResponse>>, HttpError> {
    // Determine shell to use and validate against whitelist
    let shell = request
        .shell
        .or_else(|| std::env::var("SHELL").ok())
        .unwrap_or_else(|| "/bin/bash".to_string());

    // Validate shell is in allowed whitelist to prevent command injection
    validate_shell(&shell)?;

    // Determine working directory and validate path
    let working_dir = if let Some(dir) = &request.working_dir {
        resolve_working_dir(&PathBuf::from(dir))?
    } else {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    };

    let env = build_shell_environment(&shell);

    // Create the session
    let initial_size = portable_pty::PtySize {
        rows: request.rows,
        cols: request.cols,
        pixel_width: 0,
        pixel_height: 0,
    };

    // Build sandbox configuration
    let sandbox_config = SandboxConfig {
        enabled: !request.no_sandbox,
        bwrap_flags: request.bwrap_flags,
        sandbox_exec_profile: request.sandbox_exec_profile,
    };

    let created = handlers::terminal::create_shell_session(
        &shell,
        env,
        &working_dir,
        initial_size,
        sandbox_config,
        token_hash.0,
        &state,
    )
    .await
    .map_err(|message| HttpError::new(error_codes::EXECUTION_ERROR, message))?;

    // Build WebSocket URL (relative)
    let ws_url = format!("/ws/terminal/{}", created.session_id);

    Ok(Json(ApiResponse::success(CreateTerminalSessionResponse {
        session_id: created.session_id,
        ws_url,
    })))
}
