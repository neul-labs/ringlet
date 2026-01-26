//! HTTP server setup using Axum.

use crate::http::{assets, routes, terminal_ws, websocket};
use crate::server::ServerState;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

/// Run the HTTP server.
pub async fn run_http_server(
    state: Arc<ServerState>,
    port: u16,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // Build the router
    let app = Router::new()
        // API routes
        .nest("/api", routes::api_routes())
        // WebSocket endpoints
        .route("/ws", get(websocket::ws_handler))
        .route("/ws/terminal/{session_id}", get(terminal_ws::terminal_ws_handler))
        // Static assets (CSS, JS, etc.)
        .route("/assets/{*path}", get(assets::serve_static))
        // Favicon
        .route("/favicon.svg", get(assets::serve_favicon))
        // Serve index.html at root
        .route("/", get(assets::serve_index))
        // SPA fallback - serve index.html for all other routes
        .fallback(get(assets::serve_index))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind to address
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind HTTP server to {}: {}", addr, e);
            return;
        }
    };

    info!("HTTP server listening on http://{}", addr);

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
            info!("HTTP server shutting down");
        })
        .await
        .unwrap_or_else(|e| {
            error!("HTTP server error: {}", e);
        });
}
