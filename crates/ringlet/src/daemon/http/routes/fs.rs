//! Filesystem HTTP handlers.

use crate::daemon::http::error::{ApiResponse, HttpError};
use crate::daemon::server::ServerState;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ListDirQuery {
    /// Path to list. Defaults to home directory.
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DirEntry {
    /// Entry name (filename only).
    pub name: String,
    /// Full path to the entry.
    pub path: String,
    /// Whether this is a directory.
    pub is_dir: bool,
}

#[derive(Debug, Serialize)]
pub struct ListDirResponse {
    /// Current path being listed.
    pub path: String,
    /// Parent directory path (None if at root).
    pub parent: Option<String>,
    /// Directory entries.
    pub entries: Vec<DirEntry>,
}

/// Validate that a path is within allowed boundaries.
/// Returns the canonicalized path if valid.
fn validate_path(path: &PathBuf) -> Result<PathBuf, HttpError> {
    // Canonicalize to resolve symlinks and .. components
    let canonical = path.canonicalize().map_err(|e| {
        HttpError::not_found(format!("Path not found: {} ({})", path.display(), e))
    })?;

    // Get allowed root directories
    let home = dirs::home_dir();
    let tmp = Some(PathBuf::from("/tmp"));

    // Check if path is within allowed directories
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

/// GET /api/fs/list - List directory contents.
pub async fn list_directory(
    State(_state): State<Arc<ServerState>>,
    Query(query): Query<ListDirQuery>,
) -> Result<Json<ApiResponse<ListDirResponse>>, HttpError> {
    // Determine path to list
    let requested_path = query
        .path
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

    // Validate and canonicalize path (prevents path traversal)
    let path = validate_path(&requested_path)?;

    // Validate path is a directory
    if !path.is_dir() {
        return Err(HttpError::new(
            ringlet_core::rpc::error_codes::INTERNAL_ERROR,
            format!("Not a directory: {}", path.display()),
        ));
    }

    // Read directory entries
    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(&path).map_err(|e| {
        HttpError::internal(format!("Failed to read directory: {}", e))
    })?;

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // Skip entries we can't read
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue, // Skip if we can't determine type
        };

        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/directories (starting with .)
        if name.starts_with('.') {
            continue;
        }

        let entry_path = entry.path();

        entries.push(DirEntry {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_dir: file_type.is_dir(),
        });
    }

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    // Get parent directory
    let parent = path.parent().map(|p| p.to_string_lossy().to_string());

    Ok(Json(ApiResponse::success(ListDirResponse {
        path: path.to_string_lossy().to_string(),
        parent,
        entries,
    })))
}

#[derive(Debug, Deserialize)]
pub struct PathCompleteQuery {
    /// Path prefix to complete.
    pub prefix: String,
}

#[derive(Debug, Serialize)]
pub struct PathCompletion {
    /// Full path to the entry.
    pub path: String,
    /// Entry name (filename only).
    pub name: String,
    /// Whether this is a directory.
    pub is_dir: bool,
}

#[derive(Debug, Serialize)]
pub struct PathCompleteResponse {
    pub completions: Vec<PathCompletion>,
}

/// GET /api/fs/complete - Path autocompletion for directories.
pub async fn path_complete(
    State(_state): State<Arc<ServerState>>,
    Query(query): Query<PathCompleteQuery>,
) -> Result<Json<ApiResponse<PathCompleteResponse>>, HttpError> {
    let prefix = &query.prefix;

    // Split into parent dir and partial name
    let prefix_path = PathBuf::from(prefix);
    let (parent_dir, partial) = if prefix.ends_with('/') || prefix.ends_with(std::path::MAIN_SEPARATOR) {
        (prefix_path.clone(), String::new())
    } else {
        let parent = prefix_path.parent().unwrap_or(&PathBuf::from("/")).to_path_buf();
        let partial = prefix_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        (parent, partial)
    };

    // Validate parent directory
    let parent = validate_path(&parent_dir)?;

    if !parent.is_dir() {
        return Ok(Json(ApiResponse::success(PathCompleteResponse {
            completions: Vec::new(),
        })));
    }

    let read_dir = std::fs::read_dir(&parent).map_err(|e| {
        HttpError::internal(format!("Failed to read directory: {}", e))
    })?;

    let partial_lower = partial.to_lowercase();
    let mut completions = Vec::new();

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        // Only return directories
        if !file_type.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden directories
        if name.starts_with('.') {
            continue;
        }

        // Filter by partial match (case-insensitive)
        if !partial_lower.is_empty() && !name.to_lowercase().starts_with(&partial_lower) {
            continue;
        }

        let entry_path = entry.path();
        completions.push(PathCompletion {
            path: entry_path.to_string_lossy().to_string(),
            name,
            is_dir: true,
        });

        if completions.len() >= 20 {
            break;
        }
    }

    // Sort alphabetically
    completions.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(Json(ApiResponse::success(PathCompleteResponse {
        completions,
    })))
}
