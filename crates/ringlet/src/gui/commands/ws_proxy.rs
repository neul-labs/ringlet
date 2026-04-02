use crate::gui::error::AppError;
use crate::gui::state::AppState;
use futures_util::{SinkExt, StreamExt};
use tauri::{AppHandle, Emitter, State};
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest,
    Message,
};

/// Connect to a WebSocket endpoint on the daemon.
///
/// Returns a connection ID that the frontend uses to identify this connection.
/// Server messages are forwarded as Tauri events:
/// - `ws-message-{id}` for text messages
/// - `ws-binary-{id}` for binary messages (terminal I/O)
/// - `ws-close-{id}` when the connection closes
#[tauri::command]
pub async fn ws_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<String, AppError> {
    let connection = state.connection.read().await;
    let token = state.auth_token.read().await;

    let ws_url = format!("{}{}", connection.ws_url(), path);

    let mut request = ws_url.into_client_request().map_err(|e| {
        AppError::WebSocket(format!("Invalid WebSocket URL: {}", e))
    })?;

    // Pass auth token via Sec-WebSocket-Protocol header
    // Format: "bearer, {token}" — matches daemon's extract_token in auth.rs
    if !token.is_empty() {
        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            format!("bearer, {}", *token).parse().unwrap(),
        );
    }

    // Drop locks before await
    drop(connection);
    drop(token);

    let (ws_stream, _) = tokio_tungstenite::connect_async(request).await?;

    let connection_id = uuid::Uuid::new_v4().to_string();
    let id = connection_id.clone();

    let (write, mut read) = ws_stream.split();

    // Store the write half for sending messages later
    state
        .ws_connections
        .write()
        .await
        .insert(connection_id.clone(), write);

    // Spawn a task to forward server messages to frontend via Tauri events
    let event_id = connection_id.clone();
    let ws_connections = state.ws_connections.clone();
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let _ = app.emit(&format!("ws-message-{}", event_id), text.to_string());
                }
                Ok(Message::Binary(data)) => {
                    let _ = app.emit(&format!("ws-binary-{}", event_id), data.to_vec());
                }
                Ok(Message::Close(_)) => {
                    let _ = app.emit(&format!("ws-close-{}", event_id), ());
                    break;
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                    // Handled by tungstenite automatically
                }
                Ok(Message::Frame(_)) => {}
                Err(_) => {
                    let _ = app.emit(&format!("ws-close-{}", event_id), ());
                    break;
                }
            }
        }
        // Clean up the connection
        ws_connections.write().await.remove(&event_id);
    });

    Ok(id)
}

/// Send a text message over an active WebSocket connection.
#[tauri::command]
pub async fn ws_send(
    state: State<'_, AppState>,
    id: String,
    message: String,
) -> Result<(), AppError> {
    let mut connections = state.ws_connections.write().await;
    if let Some(sender) = connections.get_mut(&id) {
        sender
            .send(Message::Text(message.into()))
            .await
            .map_err(|e| AppError::WebSocket(e.to_string()))?;
        Ok(())
    } else {
        Err(AppError::WebSocket(format!(
            "No WebSocket connection with id '{}'",
            id
        )))
    }
}

/// Send binary data over an active WebSocket connection.
/// Used for terminal PTY I/O.
#[tauri::command]
pub async fn ws_send_binary(
    state: State<'_, AppState>,
    id: String,
    data: Vec<u8>,
) -> Result<(), AppError> {
    let mut connections = state.ws_connections.write().await;
    if let Some(sender) = connections.get_mut(&id) {
        sender
            .send(Message::Binary(data.into()))
            .await
            .map_err(|e| AppError::WebSocket(e.to_string()))?;
        Ok(())
    } else {
        Err(AppError::WebSocket(format!(
            "No WebSocket connection with id '{}'",
            id
        )))
    }
}

/// Close an active WebSocket connection.
#[tauri::command]
pub async fn ws_close(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let mut connections = state.ws_connections.write().await;
    if let Some(mut sender) = connections.remove(&id) {
        let _ = sender.send(Message::Close(None)).await;
    }
    Ok(())
}
