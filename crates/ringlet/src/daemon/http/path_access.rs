//! Filesystem access policy for authenticated local HTTP routes.
//!
//! The HTTP API is loopback-only and bearer-token authenticated, so workspace
//! tooling needs access to normal local repository paths rather than only
//! `HOME` and `/tmp`. Centralize that rule here so routes stay consistent.

use crate::daemon::http::error::HttpError;
use std::path::{Path, PathBuf};

/// Validate and canonicalize an existing local path.
pub fn validate_existing_path(path: &Path) -> Result<PathBuf, HttpError> {
    path.canonicalize()
        .map_err(|e| HttpError::not_found(format!("Path not found: {} ({})", path.display(), e)))
}
