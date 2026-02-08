//! Filesystem HTTP handlers.

use crate::http::error::{ApiResponse, HttpError};
use crate::server::ServerState;
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

/// GET /api/fs/list - List directory contents.
pub async fn list_directory(
    State(_state): State<Arc<ServerState>>,
    Query(query): Query<ListDirQuery>,
) -> Result<Json<ApiResponse<ListDirResponse>>, HttpError> {
    // Determine path to list
    let path = query
        .path
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

    // Validate path exists and is a directory
    if !path.exists() {
        return Err(HttpError::not_found(format!(
            "Path not found: {}",
            path.display()
        )));
    }

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
