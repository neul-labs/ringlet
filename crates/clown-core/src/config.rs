//! User configuration types.

use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// User configuration from ~/.config/clown/config.toml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserConfig {
    /// Default settings.
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// Hook preferences.
    #[serde(default)]
    pub hooks: HooksPrefs,

    /// MCP server preferences.
    #[serde(default)]
    pub mcp_servers: McpServersPrefs,

    /// Daemon settings.
    #[serde(default)]
    pub daemon: DaemonConfig,

    /// Telemetry settings.
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

/// Default settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default provider ID.
    pub provider: Option<String>,

    /// Default bin directory for aliases.
    pub bin_dir: Option<String>,
}

/// Hook preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksPrefs {
    /// Auto-format after file writes.
    #[serde(default)]
    pub auto_format: bool,

    /// Auto-lint after file writes.
    #[serde(default)]
    pub auto_lint: bool,

    /// Custom hooks.
    #[serde(default)]
    pub custom: HashMap<String, Vec<CustomHook>>,
}

/// Custom hook definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHook {
    /// Matcher pattern (e.g., "Edit|Write").
    pub matcher: String,

    /// Hook type.
    #[serde(rename = "type")]
    pub hook_type: String,

    /// Command to run.
    pub command: String,
}

/// MCP server preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpServersPrefs {
    /// Enable filesystem MCP server.
    #[serde(default)]
    pub filesystem: bool,

    /// Enable GitHub MCP server.
    #[serde(default)]
    pub github: bool,

    /// GitHub token for MCP server.
    pub github_token: Option<String>,

    /// Custom MCP servers.
    #[serde(default)]
    pub custom: HashMap<String, McpServerConfig>,
}

/// Custom MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Command to run.
    pub command: String,

    /// Arguments.
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables.
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Daemon configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Idle timeout in seconds before daemon exits.
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,

    /// TCP port for optional HTTP API (0 = disabled).
    #[serde(default)]
    pub http_port: u16,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            idle_timeout_secs: default_idle_timeout(),
            http_port: 0,
        }
    }
}

fn default_idle_timeout() -> u64 {
    300 // 5 minutes
}

/// Telemetry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable telemetry collection.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Enable resource monitoring (CPU, memory).
    #[serde(default)]
    pub resource_monitoring: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            resource_monitoring: false,
        }
    }
}

fn default_true() -> bool {
    true
}

impl UserConfig {
    /// Load from a TOML file, returning default if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self, toml::de::Error> {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| toml::de::Error::custom(e.to_string()))?;
            toml::from_str(&content)
        } else {
            Ok(Self::default())
        }
    }

    /// Save to a TOML file.
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UserConfig::default();
        assert!(!config.hooks.auto_format);
        assert!(config.telemetry.enabled);
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
            [defaults]
            provider = "anthropic"

            [hooks]
            auto_format = true
            auto_lint = false

            [mcp_servers]
            filesystem = true
            github = false

            [daemon]
            idle_timeout_secs = 600

            [telemetry]
            enabled = true
        "#;

        let config: UserConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.defaults.provider, Some("anthropic".to_string()));
        assert!(config.hooks.auto_format);
        assert!(config.mcp_servers.filesystem);
    }
}
