//! WebSocket handler for terminal sessions.
//!
//! Provides bidirectional terminal I/O over WebSocket:
//! - Binary messages: raw terminal data (input/output)
//! - Text messages: JSON control messages (resize, state changes)

use crate::server::ServerState;
use crate::terminal::{SessionId, SessionState};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Control messages from client (JSON).
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TerminalClientMessage {
    /// Resize the terminal.
    Resize { cols: u16, rows: u16 },
    /// Send a signal (SIGINT=2, SIGQUIT=3, etc.).
    Signal { signal: i32 },
}

/// Control messages to client (JSON).
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TerminalServerMessage {
    /// Session state changed.
    StateChanged {
        state: String,
        exit_code: Option<i32>,
    },
    /// Terminal was resized.
    Resized { cols: u16, rows: u16 },
    /// Error occurred.
    Error { message: String },
    /// Session connected successfully.
    Connected { session_id: String },
}

/// WebSocket upgrade handler for terminal sessions.
pub async fn terminal_ws_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_terminal_socket(socket, session_id, state))
}

/// Handle a terminal WebSocket connection.
async fn handle_terminal_socket(socket: WebSocket, session_id: SessionId, state: Arc<ServerState>) {
    let (mut sender, mut receiver) = socket.split();

    // Get the session
    let session = match state.terminal_sessions.get_session(&session_id).await {
        Some(s) => s,
        None => {
            let msg = TerminalServerMessage::Error {
                message: format!("Session not found: {}", session_id),
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }
            return;
        }
    };

    // Check if session is still running
    if session.is_terminated().await {
        let msg = TerminalServerMessage::Error {
            message: "Session has terminated".to_string(),
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
        return;
    }

    // Register this client
    session.add_client().await;
    info!(
        "Terminal client connected to session {} (clients: {})",
        session_id,
        session.client_count().await
    );

    // Send connected message
    let connected_msg = TerminalServerMessage::Connected {
        session_id: session_id.clone(),
    };
    if let Ok(json) = serde_json::to_string(&connected_msg) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            session.remove_client().await;
            return;
        }
    }

    // Send scrollback buffer (terminal history) to the new client
    let scrollback = session.get_scrollback().await;
    if !scrollback.is_empty() {
        debug!("Sending {} bytes of scrollback to client for session {}", scrollback.len(), session_id);
        if sender.send(Message::Binary(scrollback.into())).await.is_err() {
            session.remove_client().await;
            return;
        }
    }

    // Subscribe to terminal output
    let mut output_rx = session.subscribe();

    loop {
        tokio::select! {
            // Handle incoming messages from client
            Some(msg) = receiver.next() => {
                match msg {
                    Ok(Message::Binary(data)) => {
                        // Raw terminal input data
                        debug!("Received {} bytes of input for session {}", data.len(), session_id);
                        if let Err(e) = session.send_input(crate::terminal::session::TerminalInput::Data(data.to_vec())).await {
                            warn!("Failed to send input to session {}: {}", session_id, e);
                            break;
                        }
                    }
                    Ok(Message::Text(text)) => {
                        // JSON control message
                        match serde_json::from_str::<TerminalClientMessage>(&text) {
                            Ok(TerminalClientMessage::Resize { cols, rows }) => {
                                if let Err(e) = session.send_input(crate::terminal::session::TerminalInput::Resize { cols, rows }).await {
                                    warn!("Failed to send resize to session {}: {}", session_id, e);
                                }
                            }
                            Ok(TerminalClientMessage::Signal { signal }) => {
                                if let Err(e) = session.send_input(crate::terminal::session::TerminalInput::Signal(signal)).await {
                                    warn!("Failed to send signal to session {}: {}", session_id, e);
                                }
                            }
                            Err(e) => {
                                debug!("Invalid control message: {}", e);
                                let error_msg = TerminalServerMessage::Error {
                                    message: format!("Invalid message: {}", e),
                                };
                                if let Ok(json) = serde_json::to_string(&error_msg) {
                                    let _ = sender.send(Message::Text(json.into())).await;
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        debug!("Terminal client sent close for session {}", session_id);
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("WebSocket receive error for session {}: {}", session_id, e);
                        break;
                    }
                    _ => {}
                }
            }

            // Forward terminal output to client
            result = output_rx.recv() => {
                match result {
                    Ok(output) => {
                        use crate::terminal::session::TerminalOutput;
                        match output {
                            TerminalOutput::Data(data) => {
                                // Send raw binary data
                                if sender.send(Message::Binary(data.into())).await.is_err() {
                                    break;
                                }
                            }
                            TerminalOutput::StateChanged(state) => {
                                let (state_str, exit_code) = match state {
                                    SessionState::Starting => ("starting".to_string(), None),
                                    SessionState::Running => ("running".to_string(), None),
                                    SessionState::Terminated { exit_code } => ("terminated".to_string(), exit_code),
                                };
                                let msg = TerminalServerMessage::StateChanged {
                                    state: state_str,
                                    exit_code,
                                };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    if sender.send(Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            TerminalOutput::Resized { cols, rows } => {
                                let msg = TerminalServerMessage::Resized { cols, rows };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    if sender.send(Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Terminal client lagged for session {}, missed {} events", session_id, n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Terminal output broadcaster closed for session {}", session_id);
                        break;
                    }
                }
            }
        }
    }

    // Unregister this client
    session.remove_client().await;
    info!(
        "Terminal client disconnected from session {} (clients: {})",
        session_id,
        session.client_count().await
    );
}
