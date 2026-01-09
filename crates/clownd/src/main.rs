//! clownd - Background daemon for the clown CLI orchestrator.
//!
//! The daemon is the core of clown. It owns profile state, agent detection,
//! telemetry collection, and real-time event distribution. The CLI auto-starts
//! this daemon on first use.

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod agent_registry;
mod execution;
mod handlers;
mod profile_manager;
mod provider_registry;
mod registry_client;
mod server;
mod telemetry;
mod watcher;

use anyhow::Result;
use clap::Parser;
use clown_core::ClownPaths;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

/// clownd - Background daemon for clown CLI orchestrator
#[derive(Parser, Debug)]
#[command(name = "clownd", version, about)]
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

    info!("clownd v{} starting", clown_core::VERSION);

    // Get paths
    let paths = ClownPaths::default();
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
    let config = clown_core::UserConfig::load(&paths.config_file())
        .unwrap_or_default();

    // Determine idle timeout
    let idle_timeout = if args.stay_alive {
        None
    } else {
        Some(std::time::Duration::from_secs(config.daemon.idle_timeout_secs))
    };

    // Run the server
    match server::run(&socket_path, idle_timeout, &paths).await {
        Ok(()) => {
            info!("clownd shutting down gracefully");
        }
        Err(e) => {
            error!("clownd error: {}", e);
            return Err(e);
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(&paths.daemon_pid());
    let _ = std::fs::remove_file(&paths.daemon_endpoint());
    let _ = std::fs::remove_file(&socket_path);

    Ok(())
}
