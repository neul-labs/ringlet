//! WebSocket handler for real-time event streaming.

use crate::server::ServerState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use chrono::Utc;
use ringlet_core::{ClientMessage, Event, ServerMessage, VERSION};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// WebSocket upgrade handler.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection.
async fn handle_socket(socket: WebSocket, state: Arc<ServerState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to events
    let mut event_rx = state.events.subscribe();

    // Track subscribed topics (default: subscribe to all)
    let mut subscribed_topics: HashSet<String> = HashSet::new();
    subscribed_topics.insert("*".to_string());

    // Send connected event
    let connected_msg = ServerMessage::Event {
        event: Event::Connected {
            version: VERSION.to_string(),
            timestamp: Utc::now(),
        },
    };
    if let Ok(json) = serde_json::to_string(&connected_msg) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    info!("WebSocket client connected");

    // Create heartbeat interval
    let mut heartbeat_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            // Send periodic heartbeat
            _ = heartbeat_interval.tick() => {
                let msg = ServerMessage::Event {
                    event: Event::Heartbeat {
                        timestamp: Utc::now().timestamp(),
                    },
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }

            // Handle incoming messages from client
            Some(msg) = receiver.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            match client_msg {
                                ClientMessage::Subscribe { topics } => {
                                    debug!("Client subscribing to topics: {:?}", topics);
                                    for topic in topics {
                                        subscribed_topics.insert(topic);
                                    }
                                }
                                ClientMessage::Unsubscribe { topics } => {
                                    debug!("Client unsubscribing from topics: {:?}", topics);
                                    for topic in topics {
                                        subscribed_topics.remove(&topic);
                                    }
                                }
                                ClientMessage::Ping => {
                                    let pong = serde_json::to_string(&ServerMessage::Pong).unwrap();
                                    if sender.send(Message::Text(pong.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        } else {
                            warn!("Invalid client message: {}", text);
                            let error = ServerMessage::Error {
                                message: "Invalid message format".to_string(),
                            };
                            if let Ok(json) = serde_json::to_string(&error) {
                                let _ = sender.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        debug!("WebSocket client sent close");
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("WebSocket receive error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            // Forward events from broadcaster to client
            result = event_rx.recv() => {
                match result {
                    Ok(event) => {
                        // Check if client is subscribed to this topic
                        let topic = event.topic();
                        let alias = event.alias();

                        let should_send = subscribed_topics.contains("*")
                            || subscribed_topics.contains(topic)
                            || alias.map(|a| subscribed_topics.contains(&format!("{}:{}", topic, a))).unwrap_or(false);

                        if should_send {
                            let msg = ServerMessage::Event { event };
                            if let Ok(json) = serde_json::to_string(&msg) {
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket client lagged, missed {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Event broadcaster closed");
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket client disconnected");
}
