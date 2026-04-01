use crate::connection::ConnectionConfig;
use std::collections::HashMap;
use std::process::Child;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Write half of a WebSocket connection for sending messages.
pub type WsSender =
    futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::Message,
    >;

/// Shared application state across all Tauri commands.
pub struct AppState {
    /// Current connection configuration.
    pub connection: Arc<RwLock<ConnectionConfig>>,
    /// Reusable HTTP client for API proxying.
    pub http_client: reqwest::Client,
    /// Authentication token injected into proxied requests.
    pub auth_token: Arc<RwLock<String>>,
    /// Daemon child process handle (standalone mode only).
    pub daemon_process: Arc<RwLock<Option<Child>>>,
    /// Active WebSocket connections keyed by connection ID.
    pub ws_connections: Arc<RwLock<HashMap<String, WsSender>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(RwLock::new(ConnectionConfig::default())),
            http_client: reqwest::Client::new(),
            auth_token: Arc::new(RwLock::new(String::new())),
            daemon_process: Arc::new(RwLock::new(None)),
            ws_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
