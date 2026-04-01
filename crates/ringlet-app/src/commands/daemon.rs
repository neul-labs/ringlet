use crate::error::AppError;
use crate::state::AppState;
use std::process::Command;
use tauri::State;

/// Start the ringletd daemon in standalone mode.
///
/// Finds the `ringletd` binary (bundled or in PATH), spawns it with
/// `--stay-alive --foreground` flags, polls `/api/ping` until ready,
/// then reads the auth token from `~/.config/ringlet/http_token`.
#[tauri::command]
pub async fn start_daemon(state: State<'_, AppState>) -> Result<String, AppError> {
    // Check if already running
    {
        let process = state.daemon_process.read().await;
        if process.is_some() {
            return Err(AppError::Daemon("Daemon is already running".to_string()));
        }
    }

    // Find ringletd binary
    let ringletd_path = which::which("ringletd").map_err(|_| {
        AppError::Daemon(
            "Cannot find 'ringletd' binary. Ensure it is installed and in PATH.".to_string(),
        )
    })?;

    // Spawn the daemon
    let child = Command::new(&ringletd_path)
        .args(["--stay-alive", "--foreground"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| AppError::Daemon(format!("Failed to start daemon: {}", e)))?;

    {
        let mut process = state.daemon_process.write().await;
        *process = Some(child);
    }

    // Poll /api/ping until the daemon is ready (up to 15 seconds)
    let connection = state.connection.read().await;
    let ping_url = format!("{}/api/ping", connection.base_url());
    drop(connection);

    let client = &state.http_client;
    let mut ready = false;

    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        if let Ok(resp) = client
            .get(&ping_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            if resp.status().is_success() || resp.status().as_u16() == 401 {
                // 401 means the server is up but we need a token
                ready = true;
                break;
            }
        }
    }

    if !ready {
        // Clean up if daemon didn't start
        let mut process = state.daemon_process.write().await;
        if let Some(mut child) = process.take() {
            let _ = child.kill();
        }
        return Err(AppError::Daemon(
            "Daemon did not become ready within 15 seconds".to_string(),
        ));
    }

    // Read the auth token
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ringlet");
    let token_path = config_dir.join("http_token");

    let token = std::fs::read_to_string(&token_path)
        .map(|s| s.trim().to_string())
        .map_err(|e| AppError::Daemon(format!("Daemon started but cannot read token: {}", e)))?;

    // Store the token
    {
        let mut auth_token = state.auth_token.write().await;
        *auth_token = token.clone();
    }

    Ok(token)
}

/// Stop the daemon process (standalone mode).
///
/// First tries a graceful shutdown via `POST /api/shutdown`, then waits
/// up to 5 seconds before force-killing the process.
#[tauri::command]
pub async fn stop_daemon(state: State<'_, AppState>) -> Result<(), AppError> {
    // Try graceful shutdown via API
    let connection = state.connection.read().await;
    let token = state.auth_token.read().await;

    let shutdown_url = format!("{}/api/shutdown", connection.base_url());
    let _ = state
        .http_client
        .post(&shutdown_url)
        .header("Authorization", format!("Bearer {}", *token))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await;

    drop(connection);
    drop(token);

    // Wait for process to exit, then force kill if needed
    let mut process = state.daemon_process.write().await;
    if let Some(mut child) = process.take() {
        // Give it 5 seconds to shut down gracefully
        for _ in 0..10 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            match child.try_wait() {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => continue,
                Err(_) => break,
            }
        }
        // Force kill if still running
        let _ = child.kill();
        let _ = child.wait();
    }

    // Clean up PID and endpoint files
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ringlet");
    let _ = std::fs::remove_file(config_dir.join("daemon.pid"));
    let _ = std::fs::remove_file(config_dir.join("daemon-endpoint"));

    Ok(())
}
