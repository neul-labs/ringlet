//! Profile HTTP handlers.

use crate::daemon::handlers;
use crate::daemon::http::error::{ApiResponse, HttpError};
use crate::daemon::server::ServerState;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use ringlet_core::http_api::{ListProfilesQuery, RunRequest, RunResponse};
use ringlet_core::{ProfileCreateRequest, ProfileInfo, Response};
use std::collections::HashMap;
use std::sync::Arc;

/// GET /api/profiles - List all profiles.
pub async fn list(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ListProfilesQuery>,
) -> Result<Json<ApiResponse<Vec<ProfileInfo>>>, HttpError> {
    let response = handlers::profiles::list(query.agent.as_deref(), &state).await;

    match response {
        Response::Profiles(profiles) => Ok(Json(ApiResponse::success(profiles))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// POST /api/profiles - Create a profile.
pub async fn create(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<ProfileCreateRequest>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::profiles::create(&request, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/profiles/:alias - Get profile details.
pub async fn inspect(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<ProfileInfo>>, HttpError> {
    let response = handlers::profiles::inspect(&alias, &state).await;

    match response {
        Response::Profile(profile) => Ok(Json(ApiResponse::success(profile))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// DELETE /api/profiles/:alias - Delete a profile.
pub async fn delete(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<()>>, HttpError> {
    let response = handlers::profiles::delete(&alias, &state).await;

    match response {
        Response::Success { .. } => Ok(Json(ApiResponse::ok())),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// POST /api/profiles/:alias/run - Run a profile.
pub async fn run(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
    Json(request): Json<RunRequest>,
) -> Result<Json<ApiResponse<RunResponse>>, HttpError> {
    let response = handlers::profiles::run(&alias, &request.args, &state).await;

    match response {
        Response::RunStarted { pid } => {
            Ok(Json(ApiResponse::success(RunResponse::Started { pid })))
        }
        Response::RunCompleted { exit_code } => {
            Ok(Json(ApiResponse::success(RunResponse::Completed {
                exit_code,
            })))
        }
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

/// GET /api/profiles/:alias/env - Get profile environment variables.
pub async fn env(
    State(state): State<Arc<ServerState>>,
    Path(alias): Path<String>,
) -> Result<Json<ApiResponse<HashMap<String, String>>>, HttpError> {
    let response = handlers::profiles::env(&alias, &state).await;

    match response {
        Response::Env(env) => Ok(Json(ApiResponse::success(env))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}
