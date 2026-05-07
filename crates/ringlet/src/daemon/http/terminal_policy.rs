//! Shared terminal-route validation and environment helpers.

use crate::daemon::http::error::HttpError;
use crate::daemon::http::path_access::validate_existing_path;
use ringlet_core::rpc::error_codes;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Whitelist of allowed shells to prevent command injection.
const ALLOWED_SHELLS: &[&str] = &[
    "/bin/bash",
    "/bin/sh",
    "/bin/zsh",
    "/bin/fish",
    "/usr/bin/bash",
    "/usr/bin/sh",
    "/usr/bin/zsh",
    "/usr/bin/fish",
];

pub fn validate_shell(shell: &str) -> Result<(), HttpError> {
    if ALLOWED_SHELLS.contains(&shell) {
        Ok(())
    } else {
        Err(HttpError::new(
            error_codes::INTERNAL_ERROR,
            format!("Shell '{}' not in allowed whitelist", shell),
        ))
    }
}

pub fn resolve_working_dir(path: &Path) -> Result<PathBuf, HttpError> {
    validate_existing_path(path).map_err(|e| match e.status {
        axum::http::StatusCode::NOT_FOUND => HttpError::new(
            error_codes::INTERNAL_ERROR,
            format!("Invalid working directory: {}", path.display()),
        ),
        _ => e,
    })
}

pub fn build_shell_environment(shell: &str) -> HashMap<String, String> {
    let mut env = HashMap::new();

    for key in &["PATH", "LANG", "LC_ALL", "USER", "LOGNAME"] {
        if let Ok(val) = std::env::var(key) {
            env.insert(key.to_string(), val);
        }
    }

    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    env.insert("HOME".to_string(), home_dir.to_string_lossy().to_string());
    env.insert("SHELL".to_string(), shell.to_string());
    env.insert("TERM".to_string(), "xterm-256color".to_string());

    env
}
