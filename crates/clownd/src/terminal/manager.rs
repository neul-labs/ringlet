//! Terminal session manager.
//!
//! Manages the lifecycle of terminal sessions, including creation,
//! lookup, and cleanup.

use super::pty_bridge::spawn_pty_session;
use super::session::{SessionId, SessionState, TerminalInput, TerminalOutput, TerminalSession, TerminalSessionInfo};
use anyhow::{anyhow, Context, Result};
use portable_pty::PtySize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Manages all active terminal sessions.
pub struct TerminalSessionManager {
    /// Active sessions by ID.
    sessions: RwLock<HashMap<SessionId, Arc<TerminalSession>>>,
    /// Maps profile alias to active session (one active session per profile).
    profile_sessions: RwLock<HashMap<String, SessionId>>,
}

impl Default for TerminalSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalSessionManager {
    /// Create a new session manager.
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            profile_sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Generate a new session ID.
    fn generate_session_id() -> SessionId {
        Uuid::new_v4().to_string()
    }

    /// Create a new terminal session for a profile.
    ///
    /// Returns the session ID and a handle to the session.
    pub async fn create_session(
        &self,
        profile_alias: &str,
        command: &str,
        args: &[String],
        env: HashMap<String, String>,
        working_dir: &Path,
        initial_size: Option<PtySize>,
    ) -> Result<Arc<TerminalSession>> {
        // Check if there's already an active session for this profile
        {
            let profile_sessions = self.profile_sessions.read().await;
            if let Some(existing_id) = profile_sessions.get(profile_alias) {
                let sessions = self.sessions.read().await;
                if let Some(session) = sessions.get(existing_id) {
                    if !session.is_terminated().await {
                        return Err(anyhow!(
                            "Profile '{}' already has an active terminal session: {}",
                            profile_alias,
                            existing_id
                        ));
                    }
                }
            }
        }

        let session_id = Self::generate_session_id();
        let size = initial_size.unwrap_or(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        });

        // Create channels for input/output
        let (input_tx, input_rx) = mpsc::channel::<TerminalInput>(256);
        let (output_tx, _output_rx) = broadcast::channel::<TerminalOutput>(256);

        // Create the session
        let session = Arc::new(TerminalSession::new(
            session_id.clone(),
            profile_alias.to_string(),
            working_dir.to_string_lossy().to_string(),
            input_tx,
            output_tx,
            size,
        ));

        // Store the session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
        }
        {
            let mut profile_sessions = self.profile_sessions.write().await;
            profile_sessions.insert(profile_alias.to_string(), session_id.clone());
        }

        info!(
            "Created terminal session {} for profile '{}'",
            session_id, profile_alias
        );

        // Spawn the PTY process in a background task
        let session_clone = session.clone();
        let command = command.to_string();
        let args = args.to_vec();
        let working_dir = working_dir.to_path_buf();

        tokio::spawn(async move {
            if let Err(e) = spawn_pty_session(
                session_clone.clone(),
                &command,
                &args,
                env,
                &working_dir,
                size,
                input_rx,
            )
            .await
            {
                warn!("PTY session error: {}", e);
                // Mark as terminated with error
                session_clone
                    .set_state(SessionState::Terminated { exit_code: None })
                    .await;
            }
        });

        Ok(session)
    }

    /// Get a session by ID.
    pub async fn get_session(&self, id: &SessionId) -> Option<Arc<TerminalSession>> {
        self.sessions.read().await.get(id).cloned()
    }

    /// Get a session for a profile.
    pub async fn get_session_for_profile(&self, alias: &str) -> Option<Arc<TerminalSession>> {
        let profile_sessions = self.profile_sessions.read().await;
        if let Some(id) = profile_sessions.get(alias) {
            self.sessions.read().await.get(id).cloned()
        } else {
            None
        }
    }

    /// List all sessions.
    pub async fn list_sessions(&self) -> Vec<TerminalSessionInfo> {
        let sessions = self.sessions.read().await;
        let mut infos = Vec::with_capacity(sessions.len());
        for session in sessions.values() {
            infos.push(session.info().await);
        }
        infos
    }

    /// Terminate a session.
    pub async fn terminate_session(&self, id: &SessionId) -> Result<()> {
        let session = self
            .get_session(id)
            .await
            .ok_or_else(|| anyhow!("Session not found: {}", id))?;

        // Send SIGTERM equivalent
        if let Err(e) = session.send_input(TerminalInput::Signal(15)).await {
            debug!("Failed to send SIGTERM to session {}: {}", id, e);
        }

        // Mark as terminated
        session
            .set_state(SessionState::Terminated { exit_code: None })
            .await;

        info!("Terminated session {}", id);
        Ok(())
    }

    /// Remove terminated sessions from tracking.
    pub async fn cleanup_terminated(&self) {
        let mut sessions = self.sessions.write().await;
        let mut profile_sessions = self.profile_sessions.write().await;

        let mut to_remove = Vec::new();
        for (id, session) in sessions.iter() {
            if session.is_terminated().await {
                to_remove.push((id.clone(), session.profile_alias.clone()));
            }
        }

        for (id, alias) in to_remove {
            sessions.remove(&id);
            if profile_sessions.get(&alias) == Some(&id) {
                profile_sessions.remove(&alias);
            }
            debug!("Cleaned up terminated session {}", id);
        }
    }

    /// Get the count of active (non-terminated) sessions.
    pub async fn active_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        let mut count = 0;
        for session in sessions.values() {
            if !session.is_terminated().await {
                count += 1;
            }
        }
        count
    }

    /// Terminate all sessions (for shutdown).
    pub async fn terminate_all(&self) {
        let sessions = self.sessions.read().await;
        for (id, session) in sessions.iter() {
            if !session.is_terminated().await {
                if let Err(e) = session.send_input(TerminalInput::Signal(15)).await {
                    debug!("Failed to send SIGTERM to session {}: {}", id, e);
                }
                session
                    .set_state(SessionState::Terminated { exit_code: None })
                    .await;
            }
        }
        info!("Terminated all terminal sessions");
    }
}
