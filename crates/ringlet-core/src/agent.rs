//! Agent manifest types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent manifest defining how to detect, configure, and run a CLI coding agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    /// Stable identifier (e.g., "claude", "grok").
    pub id: String,

    /// Human-friendly name (e.g., "Claude Code").
    pub name: String,

    /// Executable name or path.
    pub binary: String,

    /// Flag to detect version (e.g., "--version").
    #[serde(default)]
    pub version_flag: Option<String>,

    /// Detection configuration.
    pub detect: DetectConfig,

    /// Profile isolation configuration.
    pub profile: ProfileConfig,

    /// Model configuration.
    pub models: ModelsConfig,

    /// Whether this agent supports Claude Code-style hooks.
    #[serde(default)]
    pub supports_hooks: bool,

    /// Lifecycle hooks (ringlet-managed, not agent hooks).
    #[serde(default, rename = "hooks")]
    pub lifecycle_hooks: LifecycleHooks,

    /// Optional manual setup tasks.
    #[serde(default)]
    pub setup_tasks: HashMap<String, SetupTask>,
}

/// Configuration for detecting if an agent is installed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectConfig {
    /// Commands to run; success means agent is installed.
    #[serde(default)]
    pub commands: Vec<String>,

    /// Files whose existence indicates installation.
    #[serde(default)]
    pub files: Vec<String>,
}

/// Profile isolation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Isolation strategy (currently only "home-wrapper" supported).
    pub strategy: ProfileStrategy,

    /// Template for profile home directory (e.g., "~/.claude-profiles/{alias}").
    pub source_home: String,

    /// Rhai script for config generation.
    pub script: String,

    /// Required environment variables.
    #[serde(default)]
    pub required_env: Vec<String>,

    /// Optional environment variables.
    #[serde(default)]
    pub optional_env: Vec<String>,

    /// Default provider for this agent (e.g., "self" for self-authenticating agents).
    #[serde(default)]
    pub default_provider: Option<String>,
}

/// Profile isolation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileStrategy {
    /// Full HOME directory isolation.
    HomeWrapper,
}

impl Default for ProfileStrategy {
    fn default() -> Self {
        Self::HomeWrapper
    }
}

/// Model configuration for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    /// Default model identifier.
    #[serde(default)]
    pub default: Option<String>,

    /// Supported model identifiers.
    #[serde(default)]
    pub supported: Vec<String>,
}

/// Lifecycle hooks configuration (ringlet-managed hooks, not agent hooks).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LifecycleHooks {
    /// Commands run when a profile is created.
    #[serde(default)]
    pub create: Vec<String>,

    /// Commands run when a profile is deleted.
    #[serde(default)]
    pub delete: Vec<String>,

    /// Commands run before launching the agent.
    #[serde(default)]
    pub pre_run: Vec<String>,

    /// Commands run after the agent exits.
    #[serde(default)]
    pub post_run: Vec<String>,
}

/// Manual environment setup task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupTask {
    /// Human-readable description.
    pub description: String,

    /// Command to execute.
    pub command: String,
}

/// Runtime information about a detected agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent ID.
    pub id: String,

    /// Agent name.
    pub name: String,

    /// Whether the agent is installed.
    pub installed: bool,

    /// Detected version (if available).
    pub version: Option<String>,

    /// Path to binary (if found).
    pub binary_path: Option<String>,

    /// Number of profiles for this agent.
    pub profile_count: usize,

    /// Default model.
    pub default_model: Option<String>,

    /// Default provider (e.g., "self" for self-authenticating agents).
    pub default_provider: Option<String>,

    /// Whether this agent supports Claude Code-style hooks.
    pub supports_hooks: bool,

    /// Last used timestamp.
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Compatibility types for provider matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderCompatibility {
    Anthropic,
    AnthropicCompatible,
    OpenAi,
    OpenAiCompatible,
}

impl AgentManifest {
    /// Parse from TOML string.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Get supported provider compatibility types for this agent.
    /// This is based on agent ID conventions.
    pub fn supported_provider_types(&self) -> Vec<ProviderCompatibility> {
        match self.id.as_str() {
            "claude" | "droid" | "opencode" => vec![
                ProviderCompatibility::Anthropic,
                ProviderCompatibility::AnthropicCompatible,
            ],
            "codex" | "grok" => vec![
                ProviderCompatibility::OpenAi,
                ProviderCompatibility::OpenAiCompatible,
            ],
            _ => vec![
                ProviderCompatibility::Anthropic,
                ProviderCompatibility::AnthropicCompatible,
                ProviderCompatibility::OpenAi,
                ProviderCompatibility::OpenAiCompatible,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agent_manifest() {
        let toml = r#"
            id = "claude"
            name = "Claude Code"
            binary = "claude"
            version_flag = "--version"

            [detect]
            commands = ["claude --version"]
            files = ["~/.claude/settings.json"]

            [profile]
            strategy = "home-wrapper"
            source_home = "~/.claude-profiles/{alias}"
            script = "claude.rhai"

            [models]
            default = "claude-sonnet-4"
            supported = ["claude-sonnet-4", "claude-opus-4", "MiniMax-M2.1"]

            [hooks]
            create = []
            delete = []
        "#;

        let manifest: AgentManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.id, "claude");
        assert_eq!(manifest.name, "Claude Code");
        assert_eq!(manifest.profile.strategy, ProfileStrategy::HomeWrapper);
    }
}
