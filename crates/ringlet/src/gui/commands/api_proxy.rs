use crate::gui::error::AppError;
use crate::gui::state::AppState;
use reqwest::Method;
use tauri::State;

/// Single Tauri command that proxies all HTTP API calls to the daemon.
///
/// Replaces direct `fetch()` calls from the frontend, adding the auth token
/// and routing to the configured daemon endpoint.
#[tauri::command]
pub async fn api_request(
    state: State<'_, AppState>,
    method: String,
    endpoint: String,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value, AppError> {
    let connection = state.connection.read().await;
    let token = state.auth_token.read().await;

    let url = format!("{}/api{}", connection.base_url(), endpoint);

    let http_method = method.parse::<Method>().map_err(|e| {
        AppError::Http(format!("Invalid HTTP method '{}': {}", method, e))
    })?;

    let mut request = state.http_client.request(http_method, &url);

    if !token.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", *token));
    }

    request = request.header("Content-Type", "application/json");

    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request.send().await?;
    let json: serde_json::Value = response.json().await?;

    Ok(json)
}
