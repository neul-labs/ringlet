//! Git information HTTP handlers.

use crate::daemon::handlers;
use crate::daemon::http::error::{ApiResponse, HttpError};
use crate::daemon::server::ServerState;
use crate::daemon::workspace_service::{GitRepositoryInfo, WorkspaceError};
use axum::{
    Json,
    extract::{Query, State},
};
use ringlet_core::http_api::{GitCommitInfo, GitInfo, GitInfoQuery};
use std::path::PathBuf;
use std::sync::Arc;

/// GET /api/git/info - Get git repository information for a path.
pub async fn git_info(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<GitInfoQuery>,
) -> Result<Json<ApiResponse<GitInfo>>, HttpError> {
    let requested_path = PathBuf::from(&query.path);
    let info = handlers::workspace::git_info(&requested_path, &state)
        .await
        .map_err(workspace_error_to_http)?;

    Ok(Json(ApiResponse::success(git_info_to_response(info))))
}

fn git_info_to_response(info: GitRepositoryInfo) -> GitInfo {
    GitInfo {
        is_repo: info.is_repo,
        branch: info.branch,
        dirty: info.dirty,
        remote_url: info.remote_url,
        commits: info
            .commits
            .into_iter()
            .map(|commit| GitCommitInfo {
                hash: commit.hash,
                message: commit.message,
                author: commit.author,
                date: commit.date,
            })
            .collect(),
    }
}

fn workspace_error_to_http(error: WorkspaceError) -> HttpError {
    match error {
        WorkspaceError::NotFound(message) => HttpError::not_found(message),
        WorkspaceError::Invalid(message) => HttpError::internal(message),
        WorkspaceError::Internal(message) => HttpError::internal(message),
    }
}
