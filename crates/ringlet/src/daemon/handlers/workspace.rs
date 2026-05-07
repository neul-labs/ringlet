//! Workspace inspection handlers used by the HTTP layer.

use crate::daemon::server::ServerState;
use crate::daemon::workspace_service::{
    DirectoryListing, GitRepositoryInfo, PathCompletions, WorkspaceError,
};
use std::path::Path;

pub async fn list_directory(
    path: Option<&Path>,
    state: &ServerState,
) -> Result<DirectoryListing, WorkspaceError> {
    state.workspace_service.list_directory(path)
}

pub async fn complete_paths(
    prefix: &str,
    state: &ServerState,
) -> Result<PathCompletions, WorkspaceError> {
    state.workspace_service.complete_paths(prefix)
}

pub async fn git_info(
    path: &Path,
    state: &ServerState,
) -> Result<GitRepositoryInfo, WorkspaceError> {
    state.workspace_service.git_info(path).await
}
