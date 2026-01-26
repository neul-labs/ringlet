//! Event types for real-time notifications via WebSocket.

use crate::proxy::ProxyStatus;
use crate::usage::{AgentType, CostBreakdown, TokenUsage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Events broadcast to WebSocket clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Event {
    // Connection events
    /// Sent when a client connects.
    Connected {
        version: String,
        timestamp: DateTime<Utc>,
    },
    /// Periodic heartbeat.
    Heartbeat { timestamp: i64 },

    // Profile events
    /// A profile was created.
    ProfileCreated { alias: String },
    /// A profile was deleted.
    ProfileDeleted { alias: String },
    /// A profile run was started.
    ProfileRunStarted { alias: String, pid: u32 },
    /// A profile run completed.
    ProfileRunCompleted { alias: String, exit_code: i32 },

    // Proxy events
    /// A proxy instance was started.
    ProxyStarted { alias: String, port: u16 },
    /// A proxy instance was stopped.
    ProxyStopped { alias: String },
    /// A proxy instance status changed.
    ProxyStatusChanged {
        alias: String,
        status: ProxyStatus,
    },

    // Registry events
    /// Registry sync started.
    RegistrySyncStarted,
    /// Registry sync completed.
    RegistrySyncCompleted { commit: Option<String> },

    // Usage events
    /// Usage data was updated (new entries from agent files or proxy).
    UsageUpdated {
        /// Agent type that generated the usage.
        agent: AgentType,
        /// Profile alias if attributable.
        profile: Option<String>,
        /// Token usage.
        tokens: TokenUsage,
        /// Cost breakdown if available.
        cost: Option<CostBreakdown>,
    },
}

impl Event {
    /// Get the topic for this event (for subscription filtering).
    pub fn topic(&self) -> &'static str {
        match self {
            Event::Connected { .. } | Event::Heartbeat { .. } => "system",
            Event::ProfileCreated { .. }
            | Event::ProfileDeleted { .. }
            | Event::ProfileRunStarted { .. }
            | Event::ProfileRunCompleted { .. } => "profiles",
            Event::ProxyStarted { .. }
            | Event::ProxyStopped { .. }
            | Event::ProxyStatusChanged { .. } => "proxy",
            Event::RegistrySyncStarted | Event::RegistrySyncCompleted { .. } => "registry",
            Event::UsageUpdated { .. } => "usage",
        }
    }

    /// Get the specific alias if this event is related to a profile/proxy.
    pub fn alias(&self) -> Option<&str> {
        match self {
            Event::ProfileCreated { alias }
            | Event::ProfileDeleted { alias }
            | Event::ProfileRunStarted { alias, .. }
            | Event::ProfileRunCompleted { alias, .. }
            | Event::ProxyStarted { alias, .. }
            | Event::ProxyStopped { alias }
            | Event::ProxyStatusChanged { alias, .. } => Some(alias),
            _ => None,
        }
    }
}

/// Client-to-server WebSocket messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to event topics.
    Subscribe {
        /// Topics to subscribe to: "profiles", "proxy", "registry", "*" (all)
        topics: Vec<String>,
    },
    /// Unsubscribe from event topics.
    Unsubscribe { topics: Vec<String> },
    /// Ping to keep connection alive.
    Ping,
}

/// Server-to-client WebSocket messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// An event occurred.
    Event { event: Event },
    /// Response to ping.
    Pong,
    /// Error message.
    Error { message: String },
}

impl From<Event> for ServerMessage {
    fn from(event: Event) -> Self {
        ServerMessage::Event { event }
    }
}
