use crate::gui::connection::ConnectionConfig;
use crate::gui::error::AppError;
use crate::gui::state::AppState;
use tauri::State;

/// Get the current connection configuration.
#[tauri::command]
pub async fn get_connection(state: State<'_, AppState>) -> Result<ConnectionConfig, AppError> {
    let connection = state.connection.read().await;
    Ok(connection.clone())
}

/// Set the connection configuration and auth token.
#[tauri::command]
pub async fn set_connection(
    state: State<'_, AppState>,
    config: ConnectionConfig,
    token: String,
) -> Result<(), AppError> {
    let mut connection = state.connection.write().await;
    *connection = config;

    let mut auth_token = state.auth_token.write().await;
    *auth_token = token;

    Ok(())
}

/// Test connectivity to a daemon endpoint by probing /api/ping.
#[tauri::command]
pub async fn test_connection(
    state: State<'_, AppState>,
    config: ConnectionConfig,
    token: String,
) -> Result<serde_json::Value, AppError> {
    let url = format!("{}/api/ping", config.base_url());

    let mut request = state.http_client.get(&url);

    if !token.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| AppError::Connection(format!("Cannot reach daemon: {}", e)))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Connection(format!("Invalid response: {}", e)))?;

    Ok(json)
}

/// Load the authentication token from the local config file.
/// Used for local and standalone modes.
#[tauri::command]
pub async fn load_local_token() -> Result<String, AppError> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ringlet");
    let token_path = config_dir.join("http_token");

    let token = std::fs::read_to_string(&token_path)
        .map(|s| s.trim().to_string())
        .map_err(|e| {
            AppError::Connection(format!(
                "Cannot read token from {}: {}",
                token_path.display(),
                e
            ))
        })?;

    Ok(token)
}
