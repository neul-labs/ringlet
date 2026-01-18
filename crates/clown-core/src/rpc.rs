//! RPC message types for CLI ↔ daemon communication.

use crate::agent::AgentInfo;
use crate::hooks::HooksConfig;
use crate::profile::{ProfileCreateRequest, ProfileInfo};
use crate::provider::ProviderInfo;
use crate::proxy::{ProfileProxyConfig, ProxyInstanceInfo, RoutingRule};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Request from CLI to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    // Agent commands
    AgentsList,
    AgentsInspect { id: String },

    // Provider commands
    ProvidersList,
    ProvidersInspect { id: String },

    // Profile commands
    ProfilesCreate(ProfileCreateRequest),
    ProfilesList { agent_id: Option<String> },
    ProfilesInspect { alias: String },
    ProfilesRun { alias: String, args: Vec<String> },
    ProfilesDelete { alias: String },
    ProfilesEnv { alias: String },

    // Alias commands
    AliasesInstall { alias: String, bin_dir: Option<PathBuf> },
    AliasesUninstall { alias: String },

    // Registry commands
    RegistrySync { force: bool, offline: bool },
    RegistryPin { ref_: String },
    RegistryInspect,

    // Stats commands
    Stats { agent_id: Option<String>, provider_id: Option<String> },

    // Env setup commands
    EnvSetup { alias: String, task: String },

    // Hooks commands
    HooksAdd {
        alias: String,
        event: String,
        matcher: String,
        command: String,
    },
    HooksList { alias: String },
    HooksRemove {
        alias: String,
        event: String,
        index: usize,
    },
    HooksImport { alias: String, config: HooksConfig },
    HooksExport { alias: String },

    // Proxy commands
    ProxyEnable { alias: String },
    ProxyDisable { alias: String },
    ProxyStart { alias: String },
    ProxyStop { alias: String },
    ProxyStopAll,
    ProxyRestart { alias: String },
    ProxyStatus { alias: Option<String> },
    ProxyRouteAdd { alias: String, rule: RoutingRule },
    ProxyRouteRemove { alias: String, rule_name: String },
    ProxyRouteList { alias: String },
    ProxyAliasSet { alias: String, from_model: String, to_target: String },
    ProxyAliasRemove { alias: String, from_model: String },
    ProxyAliasList { alias: String },
    ProxyConfig { alias: String },
    ProxyLogs { alias: String, lines: Option<usize> },

    // Daemon commands
    Ping,
    Shutdown,
}

/// Response from daemon to CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Response {
    /// List of agents.
    Agents(Vec<AgentInfo>),

    /// Single agent details.
    Agent(AgentInfo),

    /// List of providers.
    Providers(Vec<ProviderInfo>),

    /// Single provider details.
    Provider(ProviderInfo),

    /// List of profiles.
    Profiles(Vec<ProfileInfo>),

    /// Single profile details.
    Profile(ProfileInfo),

    /// Hooks configuration.
    Hooks(HooksConfig),

    /// Proxy status information.
    ProxyStatus(Vec<ProxyInstanceInfo>),

    /// Proxy configuration.
    ProxyConfig(ProfileProxyConfig),

    /// Routing rules list.
    ProxyRoutes(Vec<RoutingRule>),

    /// Model aliases.
    ProxyAliases(HashMap<String, String>),

    /// Proxy logs.
    ProxyLogs(String),

    /// Environment variables for shell export.
    Env(HashMap<String, String>),

    /// Registry status.
    RegistryStatus(RegistryStatus),

    /// Usage statistics.
    Stats(StatsResponse),

    /// Generic success message.
    Success { message: String },

    /// Profile run started (returns process ID for tracking).
    RunStarted { pid: u32 },

    /// Profile run completed.
    RunCompleted { exit_code: i32 },

    /// Pong response.
    Pong,

    /// Error response.
    Error { code: i32, message: String },
}

/// Registry sync status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatus {
    /// Current commit hash.
    pub commit: Option<String>,

    /// Current channel.
    pub channel: String,

    /// Last sync timestamp.
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,

    /// Whether running in offline mode.
    pub offline: bool,

    /// Number of cached agents.
    pub cached_agents: usize,

    /// Number of cached providers.
    pub cached_providers: usize,

    /// Number of cached scripts.
    pub cached_scripts: usize,
}

/// Usage statistics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    /// Per-agent statistics.
    pub by_agent: HashMap<String, AgentStats>,

    /// Per-provider statistics.
    pub by_provider: HashMap<String, ProviderStats>,

    /// Per-profile statistics.
    pub by_profile: HashMap<String, ProfileStats>,

    /// Total sessions.
    pub total_sessions: u64,

    /// Total runtime (seconds).
    pub total_runtime_secs: u64,
}

/// Per-agent statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentStats {
    /// Total sessions.
    pub sessions: u64,

    /// Total runtime (seconds).
    pub runtime_secs: u64,

    /// Number of profiles.
    pub profiles: usize,
}

/// Per-provider statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderStats {
    /// Total sessions.
    pub sessions: u64,

    /// Total runtime (seconds).
    pub runtime_secs: u64,
}

/// Per-profile statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileStats {
    /// Total sessions.
    pub sessions: u64,

    /// Total runtime (seconds).
    pub runtime_secs: u64,

    /// Last used.
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Error codes.
pub mod error_codes {
    pub const AGENT_NOT_FOUND: i32 = 1001;
    pub const PROVIDER_NOT_FOUND: i32 = 1002;
    pub const PROFILE_NOT_FOUND: i32 = 1003;
    pub const PROFILE_EXISTS: i32 = 1004;
    pub const AGENT_NOT_INSTALLED: i32 = 1005;
    pub const INCOMPATIBLE_PROVIDER: i32 = 1006;
    pub const INVALID_ENDPOINT: i32 = 1007;
    pub const HOOKS_NOT_SUPPORTED: i32 = 1008;
    pub const INVALID_HOOK_EVENT: i32 = 1009;
    pub const PROXY_NOT_ENABLED: i32 = 1010;
    pub const PROXY_NOT_RUNNING: i32 = 1011;
    pub const PROXY_ALREADY_RUNNING: i32 = 1012;
    pub const PROXY_START_FAILED: i32 = 1013;
    pub const PROXY_NOT_SUPPORTED: i32 = 1014;
    pub const ROUTE_NOT_FOUND: i32 = 1015;
    pub const ALIAS_NOT_FOUND: i32 = 1016;
    pub const SCRIPT_ERROR: i32 = 2001;
    pub const EXECUTION_ERROR: i32 = 2002;
    pub const REGISTRY_ERROR: i32 = 3001;
    pub const INTERNAL_ERROR: i32 = 9999;
}

impl Response {
    /// Create an error response.
    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self::Error {
            code,
            message: message.into(),
        }
    }

    /// Create a success response.
    pub fn success(message: impl Into<String>) -> Self {
        Self::Success {
            message: message.into(),
        }
    }

    /// Check if this is an error response.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = Request::AgentsList;
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("agents_list"));

        let parsed: Request = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, Request::AgentsList));
    }

    #[test]
    fn test_response_serialization() {
        let resp = Response::success("Profile created");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("success"));
    }
}
