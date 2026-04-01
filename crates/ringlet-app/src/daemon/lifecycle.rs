/// Daemon lifecycle utilities.
///
/// Most lifecycle management is handled directly in `commands/daemon.rs`.
/// This module provides helper functions for daemon discovery and status checks.

use std::path::PathBuf;

/// Check if a daemon is currently running by reading the PID file.
pub fn is_daemon_running() -> bool {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ringlet");
    let pid_path = config_dir.join("daemon.pid");

    if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            // Check if process is alive
            #[cfg(unix)]
            {
                unsafe { libc::kill(pid as i32, 0) == 0 }
            }
            #[cfg(not(unix))]
            {
                let _ = pid;
                // On non-Unix, we can't easily check — assume it's running
                true
            }
        } else {
            false
        }
    } else {
        false
    }
}
