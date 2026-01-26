//! ringletd - Background daemon for the ringlet CLI orchestrator.
//!
//! The daemon is the core of ringlet. It owns profile state, agent detection,
//! telemetry collection, and real-time event distribution. The CLI auto-starts
//! this daemon on first use.

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod agent_registry;
mod agent_usage;
mod claude_import;
mod events;
mod execution;
mod handlers;
mod http;
mod pricing;
mod profile_manager;
mod provider_registry;
mod proxy_manager;
mod registry_client;
mod server;
mod telemetry;
mod terminal;
mod usage_watcher;
mod watcher;

use anyhow::Result;
use clap::Parser;
use ringlet_core::RingletPaths;
use server::ServerState;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

/// ringletd - Background daemon for ringlet CLI orchestrator
#[derive(Parser, Debug)]
#[command(name = "ringletd", version, about)]
struct Args {
    /// Keep daemon running indefinitely (disable idle timeout)
    #[arg(long)]
    stay_alive: bool,

    /// Override IPC socket path
    #[arg(long)]
    socket: Option<PathBuf>,

    /// Run in foreground (don't daemonize)
    #[arg(long, short)]
    foreground: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();

    info!("ringletd v{} starting", ringlet_core::VERSION);

    // Get paths
    let paths = RingletPaths::default();
    paths.ensure_dirs()?;

    // Determine socket path
    let socket_path = args.socket.unwrap_or_else(|| paths.ipc_socket());

    info!("IPC socket: {}", socket_path.display());

    // Write PID file
    let pid = std::process::id();
    std::fs::write(&paths.daemon_pid(), pid.to_string())?;
    info!("PID {} written to {}", pid, paths.daemon_pid().display());

    // Write endpoint file for CLI discovery
    std::fs::write(&paths.daemon_endpoint(), socket_path.to_string_lossy().as_ref())?;

    // Load user config
    let config = ringlet_core::UserConfig::load(&paths.config_file())
        .unwrap_or_default();

    // Determine idle timeout
    let idle_timeout = if args.stay_alive {
        None
    } else {
        Some(std::time::Duration::from_secs(config.daemon.idle_timeout_secs))
    };

    // Create shutdown channels
    // - shutdown_tx: stored in ServerState, used by HTTP API to trigger shutdown
    // - nng_shutdown_rx: received by NNG server to know when to stop
    // - http_shutdown_rx: received by HTTP server to know when to stop
    let (shutdown_tx, nng_shutdown_rx) = tokio::sync::oneshot::channel();
    let (http_shutdown_tx, http_shutdown_rx) = tokio::sync::oneshot::channel();

    // Create shared state
    let state = Arc::new(ServerState::new(paths.clone(), shutdown_tx)?);

    // Get HTTP port from config
    let http_port = config.daemon.http_port;

    // Start HTTP server in background task
    let http_state = state.clone();
    let http_handle = tokio::spawn(async move {
        http::run_http_server(http_state, http_port, http_shutdown_rx).await;
    });

    // Run the IPC server (blocks until shutdown)
    let result = server::run(&socket_path, idle_timeout, &paths, state.clone(), nng_shutdown_rx).await;

    // Signal HTTP server to shut down
    let _ = http_shutdown_tx.send(());

    // Wait for HTTP server to finish
    let _ = http_handle.await;

    match result {
        Ok(()) => {
            info!("ringletd shutting down gracefully");
        }
        Err(e) => {
            error!("ringletd error: {}", e);
            // Continue with cleanup even on error
        }
    }

    // Terminate all terminal sessions gracefully
    info!("Terminating terminal sessions...");
    state.terminal_sessions.terminate_all().await;

    // Stop all proxy instances gracefully
    info!("Stopping proxy instances...");
    if let Err(e) = state.proxy_manager.stop_all().await {
        error!("Error stopping proxies: {}", e);
    }

    // Cleanup
    let _ = std::fs::remove_file(&paths.daemon_pid());
    let _ = std::fs::remove_file(&paths.daemon_endpoint());
    let _ = std::fs::remove_file(&socket_path);

    Ok(())
}
