//! Terminal session data structures and lifecycle.

use chrono::{DateTime, Utc};
use portable_pty::PtySize;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

/// Maximum scrollback buffer size (bytes).
const MAX_SCROLLBACK_SIZE: usize = 1024 * 1024; // 1MB

/// Unique identifier for a terminal session (UUID).
pub type SessionId = String;

/// Terminal session state.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is starting up.
    Starting,
    /// Session is running.
    Running,
    /// Session has terminated.
    Terminated {
        /// Exit code if available.
        exit_code: Option<i32>,
    },
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Starting => write!(f, "starting"),
            SessionState::Running => write!(f, "running"),
            SessionState::Terminated { exit_code } => {
                if let Some(code) = exit_code {
                    write!(f, "terminated (exit code: {})", code)
                } else {
                    write!(f, "terminated")
                }
            }
        }
    }
}

/// Information about a terminal session (for API responses).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerminalSessionInfo {
    /// Unique session identifier.
    pub id: SessionId,
    /// Profile alias this session is running.
    pub profile_alias: String,
    /// Current session state.
    pub state: SessionState,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// Process ID if available.
    pub pid: Option<u32>,
    /// Terminal columns.
    pub cols: u16,
    /// Terminal rows.
    pub rows: u16,
    /// Number of connected clients.
    pub client_count: usize,
}

/// Input sent to the terminal.
#[derive(Debug, Clone)]
pub enum TerminalInput {
    /// Raw data (keystrokes).
    Data(Vec<u8>),
    /// Resize the terminal.
    Resize { cols: u16, rows: u16 },
    /// Send a signal (SIGINT, SIGTERM, etc.).
    Signal(i32),
}

/// Output from the terminal.
#[derive(Debug, Clone)]
pub enum TerminalOutput {
    /// Raw data from the PTY.
    Data(Vec<u8>),
    /// Session state changed.
    StateChanged(SessionState),
    /// Terminal was resized.
    Resized { cols: u16, rows: u16 },
}

/// A running terminal session.
pub struct TerminalSession {
    /// Unique session identifier.
    pub id: SessionId,
    /// Profile alias this session is running.
    pub profile_alias: String,
    /// Working directory for the session.
    pub working_dir: String,
    /// Current session state.
    state: Arc<RwLock<SessionState>>,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// Channel to send input to the PTY.
    input_tx: mpsc::Sender<TerminalInput>,
    /// Broadcast channel for output (supports multiple clients).
    output_tx: broadcast::Sender<TerminalOutput>,
    /// Current terminal size.
    size: Arc<RwLock<PtySize>>,
    /// Process ID if available.
    pid: Arc<RwLock<Option<u32>>>,
    /// Number of connected clients.
    client_count: Arc<RwLock<usize>>,
    /// Scrollback buffer for terminal output history.
    scrollback: Arc<RwLock<VecDeque<u8>>>,
}

impl TerminalSession {
    /// Create a new terminal session.
    pub fn new(
        id: SessionId,
        profile_alias: String,
        working_dir: String,
        input_tx: mpsc::Sender<TerminalInput>,
        output_tx: broadcast::Sender<TerminalOutput>,
        initial_size: PtySize,
    ) -> Self {
        Self {
            id,
            profile_alias,
            working_dir,
            state: Arc::new(RwLock::new(SessionState::Starting)),
            created_at: Utc::now(),
            input_tx,
            output_tx,
            size: Arc::new(RwLock::new(initial_size)),
            pid: Arc::new(RwLock::new(None)),
            client_count: Arc::new(RwLock::new(0)),
            scrollback: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_SCROLLBACK_SIZE))),
        }
    }

    /// Append data to the scrollback buffer.
    pub async fn append_scrollback(&self, data: &[u8]) {
        let mut scrollback = self.scrollback.write().await;
        // Add new data
        for byte in data {
            scrollback.push_back(*byte);
        }
        // Trim if over limit
        while scrollback.len() > MAX_SCROLLBACK_SIZE {
            scrollback.pop_front();
        }
    }

    /// Get a copy of the scrollback buffer contents.
    pub async fn get_scrollback(&self) -> Vec<u8> {
        let scrollback = self.scrollback.read().await;
        scrollback.iter().copied().collect()
    }

    /// Get the current session state.
    pub async fn state(&self) -> SessionState {
        self.state.read().await.clone()
    }

    /// Set the session state.
    pub async fn set_state(&self, state: SessionState) {
        *self.state.write().await = state.clone();
        // Broadcast state change to all clients
        let _ = self.output_tx.send(TerminalOutput::StateChanged(state));
    }

    /// Set the process ID.
    pub async fn set_pid(&self, pid: u32) {
        *self.pid.write().await = Some(pid);
    }

    /// Get the process ID.
    pub async fn pid(&self) -> Option<u32> {
        *self.pid.read().await
    }

    /// Get the current terminal size.
    pub async fn size(&self) -> PtySize {
        *self.size.read().await
    }

    /// Send input to the terminal.
    pub async fn send_input(&self, input: TerminalInput) -> Result<(), mpsc::error::SendError<TerminalInput>> {
        // Handle resize internally as well
        if let TerminalInput::Resize { cols, rows } = &input {
            *self.size.write().await = PtySize {
                rows: *rows,
                cols: *cols,
                pixel_width: 0,
                pixel_height: 0,
            };
        }
        self.input_tx.send(input).await
    }

    /// Subscribe to terminal output.
    pub fn subscribe(&self) -> broadcast::Receiver<TerminalOutput> {
        self.output_tx.subscribe()
    }

    /// Get the output sender (for PTY bridge to use).
    pub fn output_sender(&self) -> broadcast::Sender<TerminalOutput> {
        self.output_tx.clone()
    }

    /// Increment client count.
    pub async fn add_client(&self) {
        *self.client_count.write().await += 1;
    }

    /// Decrement client count.
    pub async fn remove_client(&self) {
        let mut count = self.client_count.write().await;
        if *count > 0 {
            *count -= 1;
        }
    }

    /// Get the number of connected clients.
    pub async fn client_count(&self) -> usize {
        *self.client_count.read().await
    }

    /// Get session info for API responses.
    pub async fn info(&self) -> TerminalSessionInfo {
        let size = self.size.read().await;
        TerminalSessionInfo {
            id: self.id.clone(),
            profile_alias: self.profile_alias.clone(),
            state: self.state.read().await.clone(),
            created_at: self.created_at,
            pid: *self.pid.read().await,
            cols: size.cols,
            rows: size.rows,
            client_count: *self.client_count.read().await,
        }
    }

    /// Check if the session is terminated.
    pub async fn is_terminated(&self) -> bool {
        matches!(*self.state.read().await, SessionState::Terminated { .. })
    }
}
