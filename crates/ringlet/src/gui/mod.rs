//! Tauri desktop GUI module (feature-gated behind `gui`).
//!
//! All code previously in `crates/ringlet-app/src/` now lives here.

pub mod commands;
pub mod connection;
pub mod daemon;
pub mod error;
pub mod state;

pub use connection::{ConnectionConfig, ConnectionMode};
pub use error::AppError;
pub use state::AppState;

use tauri::Manager;

/// Launch the Tauri desktop GUI.
pub fn launch_gui(
    standalone: bool,
    remote: Option<String>,
    port: u16,
    token: Option<String>,
) {
    let mode = if standalone {
        ConnectionMode::Standalone
    } else if remote.is_some() {
        ConnectionMode::Remote
    } else {
        ConnectionMode::Local
    };

    let host = remote.unwrap_or_else(|| "127.0.0.1".to_string());
    let token = token.unwrap_or_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // API proxy
            commands::api_request,
            // WebSocket proxy
            commands::ws_connect,
            commands::ws_send,
            commands::ws_send_binary,
            commands::ws_close,
            // Connection management
            commands::get_connection,
            commands::set_connection,
            commands::test_connection,
            commands::load_local_token,
            // Daemon lifecycle
            commands::start_daemon,
            commands::stop_daemon,
            // Native OS features
            commands::pick_directory,
        ])
        .setup(move |app| {
            let state = app.state::<AppState>();
            let config = ConnectionConfig {
                mode,
                host,
                port,
                tls: false,
            };

            // Set initial connection config
            let conn = state.connection.clone();
            let auth = state.auth_token.clone();
            tauri::async_runtime::block_on(async {
                *conn.write().await = config;
                if !token.is_empty() {
                    *auth.write().await = token;
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Stop daemon if running in standalone mode
                let app = window.app_handle().clone();
                let state = app.state::<AppState>();
                let daemon = state.daemon_process.clone();
                let connection = state.connection.clone();
                let auth_token = state.auth_token.clone();
                let http_client = state.http_client.clone();

                tauri::async_runtime::block_on(async {
                    let conn = connection.read().await;
                    if conn.mode == ConnectionMode::Standalone {
                        // Try graceful shutdown
                        let token = auth_token.read().await;
                        let shutdown_url =
                            format!("{}/api/shutdown", conn.base_url());
                        let _ = http_client
                            .post(&shutdown_url)
                            .header(
                                "Authorization",
                                format!("Bearer {}", *token),
                            )
                            .timeout(std::time::Duration::from_secs(2))
                            .send()
                            .await;
                        drop(token);
                        drop(conn);

                        // Force kill if still running
                        let mut process = daemon.write().await;
                        if let Some(mut child) = process.take() {
                            let _ = child.kill();
                            let _ = child.wait();
                        }
                    }
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
