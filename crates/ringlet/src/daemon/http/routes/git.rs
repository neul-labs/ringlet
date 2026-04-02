//! Git information HTTP handlers.

use crate::daemon::http::error::{ApiResponse, HttpError};
use crate::daemon::server::ServerState;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
pub struct GitInfoQuery {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct GitCommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

#[derive(Debug, Serialize)]
pub struct GitInfo {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub dirty: bool,
    pub remote_url: Option<String>,
    pub commits: Vec<GitCommitInfo>,
}

/// Validate that a path is within allowed boundaries.
fn validate_path(path: &PathBuf) -> Result<PathBuf, HttpError> {
    let canonical = path.canonicalize().map_err(|e| {
        HttpError::not_found(format!("Path not found: {} ({})", path.display(), e))
    })?;

    let home = dirs::home_dir();
    let tmp = Some(PathBuf::from("/tmp"));

    let is_allowed = [home.as_ref(), tmp.as_ref()]
        .iter()
        .filter_map(|d| d.as_ref())
        .any(|allowed| canonical.starts_with(allowed));

    if !is_allowed {
        return Err(HttpError::forbidden(format!(
            "Access denied: {} is outside allowed directories",
            path.display()
        )));
    }

    Ok(canonical)
}

/// Run a git command and return stdout as a trimmed string.
async fn git_cmd(path: &str, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path);
    for arg in args {
        cmd.arg(arg);
    }
    let output = cmd.output().await.ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// GET /api/git/info - Get git repository information for a path.
pub async fn git_info(
    State(_state): State<Arc<ServerState>>,
    Query(query): Query<GitInfoQuery>,
) -> Result<Json<ApiResponse<GitInfo>>, HttpError> {
    let requested_path = PathBuf::from(&query.path);
    let path = validate_path(&requested_path)?;
    let path_str = path.to_string_lossy().to_string();

    // Check if it's a git repo
    let is_repo = git_cmd(&path_str, &["rev-parse", "--is-inside-work-tree"])
        .await
        .map(|s| s == "true")
        .unwrap_or(false);

    if !is_repo {
        return Ok(Json(ApiResponse::success(GitInfo {
            is_repo: false,
            branch: None,
            dirty: false,
            remote_url: None,
            commits: Vec::new(),
        })));
    }

    // Fetch git info concurrently
    let (branch, status, log, remote) = tokio::join!(
        git_cmd(&path_str, &["rev-parse", "--abbrev-ref", "HEAD"]),
        git_cmd(&path_str, &["status", "--porcelain"]),
        git_cmd(&path_str, &["log", "--format=%h|%s|%an|%aI", "-n", "10"]),
        git_cmd(&path_str, &["remote", "get-url", "origin"]),
    );

    let dirty = status.map(|s| !s.is_empty()).unwrap_or(false);

    let commits = log
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                Some(GitCommitInfo {
                    hash: parts[0].to_string(),
                    message: parts[1].to_string(),
                    author: parts[2].to_string(),
                    date: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(GitInfo {
        is_repo: true,
        branch,
        dirty,
        remote_url: remote,
        commits,
    })))
}
