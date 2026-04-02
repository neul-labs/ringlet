//! Client for communicating with the ringlet daemon.

use anyhow::{anyhow, Context, Result};
use ringlet_core::{RingletPaths, Request, Response};
use nng::options::Options;
use nng::{Protocol, Socket};
use std::process::{Command, Stdio};
use std::time::Duration;
use tracing::{debug, info};

/// Client for the ringlet daemon.
pub struct DaemonClient {
    socket: Socket,
}

impl DaemonClient {
    /// Connect to the daemon, starting it if necessary.
    pub fn connect() -> Result<Self> {
        let paths = RingletPaths::default();

        // Check if daemon is running
        let socket_path = if paths.daemon_endpoint().exists() {
            let endpoint = std::fs::read_to_string(paths.daemon_endpoint())?;
            std::path::PathBuf::from(endpoint.trim())
        } else {
            paths.ipc_socket()
        };

        // Try to connect
        match Self::try_connect(&socket_path) {
            Ok(client) => {
                debug!("Connected to existing daemon");
                Ok(client)
            }
            Err(_) => {
                // Start daemon
                info!("Starting daemon...");
                Self::start_daemon(&paths)?;

                // Wait for daemon to be ready
                for i in 0..50 {
                    std::thread::sleep(Duration::from_millis(100));
                    if let Ok(client) = Self::try_connect(&socket_path) {
                        debug!("Connected to daemon after {} attempts", i + 1);
                        return Ok(client);
                    }
                }

                Err(anyhow!("Failed to connect to daemon after starting it"))
            }
        }
    }

    /// Try to connect to existing daemon.
    fn try_connect(socket_path: &std::path::Path) -> Result<Self> {
        let socket = Socket::new(Protocol::Req0)
            .context("Failed to create nng socket")?;

        let url = format!("ipc://{}", socket_path.display());
        socket.dial(&url)
            .context(format!("Failed to connect to {}", url))?;

        // Set timeouts
        socket.set_opt::<nng::options::SendTimeout>(Some(Duration::from_secs(30)))?;
        socket.set_opt::<nng::options::RecvTimeout>(Some(Duration::from_secs(60)))?;

        Ok(Self { socket })
    }

    /// Start the daemon process via `ringlet daemon`.
    fn start_daemon(paths: &RingletPaths) -> Result<()> {
        let ringlet = std::env::current_exe()?;

        debug!("Starting daemon: {} daemon", ringlet.display());

        // Ensure directories exist
        paths.ensure_dirs()?;

        // Start daemon in background
        Command::new(&ringlet)
            .args(["daemon"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start ringlet daemon")?;

        Ok(())
    }

    /// Send a request and receive a response.
    pub fn request(&self, request: &Request) -> Result<Response> {
        let json = serde_json::to_vec(request)?;
        let msg = nng::Message::from(&json[..]);

        self.socket.send(msg)
            .map_err(|(_, e)| anyhow!("Send failed: {}", e))?;

        let response_msg = self.socket.recv()
            .context("Failed to receive response")?;

        let response: Response = serde_json::from_slice(&response_msg)?;
        Ok(response)
    }

    /// Check if daemon is running.
    pub fn ping(&self) -> bool {
        matches!(self.request(&Request::Ping), Ok(Response::Pong))
    }

    /// Shutdown the daemon.
    pub fn shutdown(&self) -> Result<()> {
        self.request(&Request::Shutdown)?;
        Ok(())
    }
}
