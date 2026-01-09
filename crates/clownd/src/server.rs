//! IPC server using nng (nanomsg next generation).

use crate::agent_registry::AgentRegistry;
use crate::execution::ExecutionAdapter;
use crate::handlers;
use crate::profile_manager::ProfileManager;
use crate::provider_registry::ProviderRegistry;
use crate::registry_client::RegistryClient;
use crate::telemetry::TelemetryCollector;
use anyhow::{Context, Result};
use clown_core::{ClownPaths, Request, Response};
use nng::options::Options;
use nng::{Protocol, Socket};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Server state shared across request handlers.
pub struct ServerState {
    pub paths: ClownPaths,
    pub last_activity: Mutex<Instant>,
    pub agent_registry: Mutex<AgentRegistry>,
    pub provider_registry: ProviderRegistry,
    pub profile_manager: ProfileManager,
    pub execution_adapter: ExecutionAdapter,
    pub registry_client: RegistryClient,
    pub telemetry: TelemetryCollector,
}

impl ServerState {
    pub fn new(paths: ClownPaths) -> Result<Self> {
        let agent_registry = AgentRegistry::new(&paths)?;
        let provider_registry = ProviderRegistry::new(&paths)?;
        let profile_manager = ProfileManager::new(paths.clone());
        let execution_adapter = ExecutionAdapter::new(paths.clone());
        let registry_client = RegistryClient::new(paths.clone());
        let telemetry = TelemetryCollector::new(paths.clone());

        Ok(Self {
            paths,
            last_activity: Mutex::new(Instant::now()),
            agent_registry: Mutex::new(agent_registry),
            provider_registry,
            profile_manager,
            execution_adapter,
            registry_client,
            telemetry,
        })
    }

    pub async fn touch(&self) {
        *self.last_activity.lock().await = Instant::now();
    }

    pub async fn idle_duration(&self) -> Duration {
        self.last_activity.lock().await.elapsed()
    }
}

/// Run the IPC server.
pub async fn run(
    socket_path: &Path,
    idle_timeout: Option<Duration>,
    paths: &ClownPaths,
) -> Result<()> {
    // Remove stale socket file if it exists
    if socket_path.exists() {
        std::fs::remove_file(socket_path)
            .context("Failed to remove stale socket file")?;
    }

    // Create rep (reply) socket
    let socket = Socket::new(Protocol::Rep0)
        .context("Failed to create nng socket")?;

    // Build IPC URL
    let url = format!("ipc://{}", socket_path.display());
    socket.listen(&url)
        .context(format!("Failed to listen on {}", url))?;

    info!("Listening on {}", url);

    let state = Arc::new(ServerState::new(paths.clone())?);

    // Spawn idle timeout checker if configured
    let state_clone = state.clone();
    let shutdown_flag = Arc::new(Mutex::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();

    if let Some(timeout) = idle_timeout {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                let idle = state_clone.idle_duration().await;
                if idle > timeout {
                    info!("Idle timeout reached ({:?}), initiating shutdown", timeout);
                    *shutdown_flag_clone.lock().await = true;
                    break;
                }
            }
        });
    }

    // Main request loop
    loop {
        // Check shutdown flag
        if *shutdown_flag.lock().await {
            break;
        }

        // Try to receive with a timeout so we can check shutdown flag periodically
        let msg = match recv_with_timeout(&socket, Duration::from_secs(1)) {
            Ok(Some(msg)) => msg,
            Ok(None) => continue, // Timeout, check shutdown flag
            Err(e) => {
                error!("Error receiving message: {}", e);
                continue;
            }
        };

        state.touch().await;

        // Parse request
        let request: Request = match serde_json::from_slice(&msg) {
            Ok(req) => req,
            Err(e) => {
                warn!("Failed to parse request: {}", e);
                let response = Response::error(
                    clown_core::rpc::error_codes::INTERNAL_ERROR,
                    format!("Invalid request: {}", e),
                );
                send_response(&socket, &response)?;
                continue;
            }
        };

        debug!("Received request: {:?}", request);

        // Handle shutdown request specially
        if matches!(request, Request::Shutdown) {
            info!("Shutdown requested");
            let response = Response::success("Shutting down");
            send_response(&socket, &response)?;
            break;
        }

        // Handle request
        let response = handlers::handle_request(&request, &state).await;

        debug!("Sending response: {:?}", response);

        send_response(&socket, &response)?;
    }

    Ok(())
}

/// Receive a message with timeout.
fn recv_with_timeout(socket: &Socket, timeout: Duration) -> Result<Option<nng::Message>> {
    // Set receive timeout
    socket.set_opt::<nng::options::RecvTimeout>(Some(timeout))?;

    match socket.recv() {
        Ok(msg) => Ok(Some(msg)),
        Err(nng::Error::TimedOut) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Send a response.
fn send_response(socket: &Socket, response: &Response) -> Result<()> {
    let json = serde_json::to_vec(response)?;
    let msg = nng::Message::from(&json[..]);
    socket.send(msg).map_err(|(_, e)| anyhow::anyhow!("Send failed: {}", e))?;
    Ok(())
}
