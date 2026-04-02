//! HTTP server setup using Axum.

use crate::daemon::http::{assets, auth, routes, terminal_ws, websocket, AuthState};
use crate::daemon::server::ServerState;
use axum::{middleware, routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

/// Run the HTTP server.
pub async fn run_http_server(
    state: Arc<ServerState>,
    port: u16,
    token: String,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let auth_state = AuthState { token: Arc::new(token) };

    // Rate limiting configuration: 10 requests per second with burst of 50
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(50)
            .finish()
            .expect("Failed to build rate limiter config"),
    );

    // Routes that require authentication
    let authenticated_routes = Router::new()
        // API routes
        .nest("/api", routes::api_routes())
        // WebSocket endpoints
        .route("/ws", get(websocket::ws_handler))
        .route("/ws/terminal/{session_id}", get(terminal_ws::terminal_ws_handler))
        .layer(GovernorLayer::new(governor_config))
        .layer(middleware::from_fn_with_state(auth_state, auth::auth_middleware))
        .with_state(state.clone());

    // Public routes (static assets, SPA)
    let public_routes = Router::new()
        // Static assets (CSS, JS, etc.)
        .route("/assets/{*path}", get(assets::serve_static))
        // Favicon
        .route("/favicon.svg", get(assets::serve_favicon))
        // Serve index.html at root
        .route("/", get(assets::serve_index))
        // SPA fallback - serve index.html for all other routes
        .fallback(get(assets::serve_index))
        .with_state(state);

    // CORS configuration - restrict to localhost origins only
    let cors = CorsLayer::new()
        .allow_origin([
            "http://127.0.0.1".parse().unwrap(),
            "http://localhost".parse().unwrap(),
            format!("http://127.0.0.1:{}", port).parse().unwrap(),
            format!("http://localhost:{}", port).parse().unwrap(),
        ])
        .allow_methods(Any)
        .allow_headers(Any);

    // Combine routes
    let app = Router::new()
        .merge(authenticated_routes)
        .merge(public_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

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
