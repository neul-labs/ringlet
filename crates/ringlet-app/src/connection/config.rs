use serde::{Deserialize, Serialize};

/// Connection mode for the Tauri app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionMode {
    /// Connect to a daemon running on localhost.
    Local,
    /// Connect to a daemon on a remote machine.
    Remote,
    /// Auto-start and manage the daemon lifecycle.
    Standalone,
}

impl Default for ConnectionMode {
    fn default() -> Self {
        Self::Local
    }
}

/// Connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub mode: ConnectionMode,
    pub host: String,
    pub port: u16,
    pub tls: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            mode: ConnectionMode::Local,
            host: "127.0.0.1".to_string(),
            port: 8765,
            tls: false,
        }
    }
}

impl ConnectionConfig {
    /// Build the HTTP base URL for API requests.
    pub fn base_url(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }

    /// Build the WebSocket base URL.
    pub fn ws_url(&self) -> String {
        let scheme = if self.tls { "wss" } else { "ws" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }
}
