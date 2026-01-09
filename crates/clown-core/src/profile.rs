//! Profile types and management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A profile binding an agent to a provider with specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Unique alias for this profile.
    pub alias: String,

    /// Agent ID (e.g., "claude", "grok").
    pub agent_id: String,

    /// Provider ID (e.g., "minimax", "anthropic").
    pub provider_id: String,

    /// Endpoint ID within the provider.
    pub endpoint_id: String,

    /// Model to use.
    pub model: String,

    /// Environment variables to inject.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Default CLI arguments.
    #[serde(default)]
    pub args: Vec<String>,

    /// Optional working directory override.
    #[serde(default)]
    pub working_dir: Option<PathBuf>,

    /// Profile metadata.
    pub metadata: ProfileMetadata,
}

/// Profile metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    /// Profile home directory.
    pub home: PathBuf,

    /// When the profile was created.
    pub created_at: DateTime<Utc>,

    /// When the profile was last used.
    #[serde(default)]
    pub last_used: Option<DateTime<Utc>>,

    /// Total number of runs.
    #[serde(default)]
    pub total_runs: u64,

    /// Enabled hooks (for display/info).
    #[serde(default)]
    pub enabled_hooks: Vec<String>,

    /// Enabled MCP servers (for display/info).
    #[serde(default)]
    pub enabled_mcp_servers: Vec<String>,
}

/// Summary information about a profile for listings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    /// Profile alias.
    pub alias: String,

    /// Agent ID.
    pub agent_id: String,

    /// Provider ID.
    pub provider_id: String,

    /// Endpoint ID.
    pub endpoint_id: String,

    /// Model.
    pub model: String,

    /// Last used timestamp.
    pub last_used: Option<DateTime<Utc>>,

    /// Total runs.
    pub total_runs: u64,
}

/// Request to create a new profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCreateRequest {
    /// Agent ID.
    pub agent_id: String,

    /// Profile alias.
    pub alias: String,

    /// Provider ID.
    pub provider_id: String,

    /// Endpoint ID (optional, uses provider default).
    pub endpoint_id: Option<String>,

    /// Model (optional, uses provider/agent default).
    pub model: Option<String>,

    /// API key (will be stored in keychain).
    pub api_key: String,

    /// Enable specific hooks.
    #[serde(default)]
    pub hooks: Vec<String>,

    /// Enable specific MCP servers.
    #[serde(default)]
    pub mcp_servers: Vec<String>,

    /// Extra arguments.
    #[serde(default)]
    pub args: Vec<String>,

    /// Working directory.
    pub working_dir: Option<PathBuf>,

    /// Whether to skip hooks and MCP servers (bare profile).
    #[serde(default)]
    pub bare: bool,
}

impl Profile {
    /// Create a new profile JSON filename.
    pub fn filename(&self) -> String {
        format!("{}.json", self.alias)
    }

    /// Update last_used and increment total_runs.
    pub fn mark_used(&mut self) {
        self.metadata.last_used = Some(Utc::now());
        self.metadata.total_runs += 1;
    }

    /// Convert to summary info.
    pub fn to_info(&self) -> ProfileInfo {
        ProfileInfo {
            alias: self.alias.clone(),
            agent_id: self.agent_id.clone(),
            provider_id: self.provider_id.clone(),
            endpoint_id: self.endpoint_id.clone(),
            model: self.model.clone(),
            last_used: self.metadata.last_used,
            total_runs: self.metadata.total_runs,
        }
    }
}

impl ProfileMetadata {
    /// Create new metadata for a fresh profile.
    pub fn new(home: PathBuf) -> Self {
        Self {
            home,
            created_at: Utc::now(),
            last_used: None,
            total_runs: 0,
            enabled_hooks: Vec::new(),
            enabled_mcp_servers: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_serialization() {
        let profile = Profile {
            alias: "work-minimax".to_string(),
            agent_id: "claude".to_string(),
            provider_id: "minimax".to_string(),
            endpoint_id: "international".to_string(),
            model: "MiniMax-M2.1".to_string(),
            env: HashMap::new(),
            args: vec![],
            working_dir: None,
            metadata: ProfileMetadata::new(PathBuf::from("/home/user/.claude-profiles/work-minimax")),
        };

        let json = serde_json::to_string_pretty(&profile).unwrap();
        let parsed: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.alias, "work-minimax");
    }
}
