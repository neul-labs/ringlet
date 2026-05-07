//! Terminal session management for remote agent access.
//!
//! This module provides PTY-based terminal sessions that can be accessed
//! remotely via WebSocket connections. Multiple clients can view and interact
//! with the same session simultaneously.

mod manager;
mod pty_bridge;
pub mod sandbox;
pub mod session;

pub use crate::daemon::telemetry::SessionTelemetryContext;
pub use manager::TerminalSessionManager;
pub use sandbox::SandboxConfig;
pub use session::{SessionId, SessionState, TerminalSessionInfo};
