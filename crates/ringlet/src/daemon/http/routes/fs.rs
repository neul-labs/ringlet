//! Filesystem HTTP handlers.

use crate::daemon::handlers;
use crate::daemon::http::error::{ApiResponse, HttpError};
use crate::daemon::server::ServerState;
use crate::daemon::workspace_service::{DirectoryListing, PathCompletions, WorkspaceError};
use axum::{
    Json,
    extract::{Query, State},
};
use ringlet_core::http_api::{
    DirEntry, ListDirQuery, ListDirResponse, PathCompleteQuery, PathCompleteResponse,
    PathCompletion,
};
use std::sync::Arc;

/// GET /api/fs/list - List directory contents.
pub async fn list_directory(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ListDirQuery>,
) -> Result<Json<ApiResponse<ListDirResponse>>, HttpError> {
    let listing = handlers::workspace::list_directory(
        query.path.as_deref().map(std::path::Path::new),
        &state,
    )
    .await
    .map_err(workspace_error_to_http)?;

    Ok(Json(ApiResponse::success(directory_listing_to_response(
        listing,
    ))))
}

/// GET /api/fs/complete - Path autocompletion for directories.
pub async fn path_complete(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<PathCompleteQuery>,
) -> Result<Json<ApiResponse<PathCompleteResponse>>, HttpError> {
    let completions = handlers::workspace::complete_paths(&query.prefix, &state)
        .await
        .map_err(workspace_error_to_http)?;

    Ok(Json(ApiResponse::success(path_completions_to_response(
        completions,
    ))))
}

fn directory_listing_to_response(listing: DirectoryListing) -> ListDirResponse {
    ListDirResponse {
        path: listing.path,
        parent: listing.parent,
        entries: listing
            .entries
            .into_iter()
            .map(|entry| DirEntry {
                name: entry.name,
                path: entry.path,
                is_dir: entry.is_dir,
            })
            .collect(),
    }
}

fn path_completions_to_response(completions: PathCompletions) -> PathCompleteResponse {
    PathCompleteResponse {
        completions: completions
            .completions
            .into_iter()
            .map(|entry| PathCompletion {
                path: entry.path,
                name: entry.name,
                is_dir: entry.is_dir,
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
