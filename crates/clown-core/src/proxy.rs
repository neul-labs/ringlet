//! Proxy configuration types for profile-level ultrallm proxy support.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Proxy configuration for a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileProxyConfig {
    /// Enable proxy for this profile.
    pub enabled: bool,

    /// Port to run proxy on (auto-assigned if None).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Routing configuration.
    #[serde(default)]
    pub routing: RoutingConfig,

    /// Model aliases (map request model to provider/model target).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub model_aliases: HashMap<String, ModelTarget>,
}

impl Default for ProfileProxyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: None,
            routing: RoutingConfig::default(),
            model_aliases: HashMap::new(),
        }
    }
}

/// Target model for routing/aliasing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTarget {
    /// Provider ID (e.g., "anthropic", "minimax", "zai").
    pub provider: String,

    /// Model name at the provider.
    pub model: String,

    /// Optional API base URL override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,
}

impl ModelTarget {
    /// Create a new model target.
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            api_base: None,
        }
    }

    /// Parse from "provider/model" format.
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(2, '/').collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0], parts[1]))
        } else {
            None
        }
    }

    /// Convert to "provider/model" format.
    pub fn to_string_format(&self) -> String {
        format!("{}/{}", self.provider, self.model)
    }
}

/// Routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Routing strategy.
    #[serde(default)]
    pub strategy: RoutingStrategy,

    /// Routing rules (evaluated in priority order).
    #[serde(default)]
    pub rules: Vec<RoutingRule>,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            strategy: RoutingStrategy::Conditional,
            rules: Vec::new(),
        }
    }
}

/// Routing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    /// Use first matching rule.
    Simple,
    /// Weighted random among matches.
    Weighted,
    /// Pick cheapest option.
    LowestCost,
    /// Learn from latency/errors.
    Adaptive,
    /// Rule-based conditional routing.
    #[default]
    Conditional,
}

/// A routing rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule name (for display/management).
    pub name: String,

    /// Condition that triggers this rule.
    pub condition: RoutingCondition,

    /// Target model (provider/model format or alias).
    pub target: String,

    /// Priority (higher = evaluated first).
    #[serde(default)]
    pub priority: i32,

    /// Optional weight for weighted routing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,
}

impl RoutingRule {
    /// Create a new routing rule.
    pub fn new(name: impl Into<String>, condition: RoutingCondition, target: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            condition,
            target: target.into(),
            priority: 0,
            weight: None,
        }
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Routing condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RoutingCondition {
    /// Route based on token count.
    TokenCount {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<u32>,
    },

    /// Route if request has tools.
    HasTools {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min_count: Option<u32>,
    },

    /// Route if thinking/extended mode is enabled.
    ThinkingMode,

    /// Route based on model name pattern.
    ModelPattern {
        pattern: String,
    },

    /// Always match (default fallback).
    Always,

    /// Combine conditions with AND.
    All {
        conditions: Vec<RoutingCondition>,
    },

    /// Combine conditions with OR.
    Any {
        conditions: Vec<RoutingCondition>,
    },
}

impl RoutingCondition {
    /// Create a token count condition.
    pub fn token_count(min: Option<u32>, max: Option<u32>) -> Self {
        Self::TokenCount { min, max }
    }

    /// Create a "has tools" condition.
    pub fn has_tools(min_count: Option<u32>) -> Self {
        Self::HasTools { min_count }
    }

    /// Create an "always" condition.
    pub fn always() -> Self {
        Self::Always
    }

    /// Parse from a simple string format.
    /// Supports: "always", "tokens > N", "tokens < N", "tools >= N", "thinking"
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();

        if s == "always" {
            return Some(Self::Always);
        }

        if s == "thinking" {
            return Some(Self::ThinkingMode);
        }

        // Parse "tokens > N" or "tokens < N"
        if s.starts_with("tokens") {
            let rest = s.trim_start_matches("tokens").trim();
            if rest.starts_with('>') {
                let n: u32 = rest.trim_start_matches('>').trim().parse().ok()?;
                return Some(Self::TokenCount { min: Some(n), max: None });
            }
            if rest.starts_with('<') {
                let n: u32 = rest.trim_start_matches('<').trim().parse().ok()?;
                return Some(Self::TokenCount { min: None, max: Some(n) });
            }
        }

        // Parse "tools >= N" or "tools > N"
        if s.starts_with("tools") {
            let rest = s.trim_start_matches("tools").trim();
            if rest.starts_with(">=") {
                let n: u32 = rest.trim_start_matches(">=").trim().parse().ok()?;
                return Some(Self::HasTools { min_count: Some(n) });
            }
            if rest.starts_with('>') {
                let n: u32 = rest.trim_start_matches('>').trim().parse().ok()?;
                return Some(Self::HasTools { min_count: Some(n + 1) });
            }
        }

        None
    }
}

/// Proxy instance status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ProxyStatus {
    /// Proxy is starting up.
    Starting,
    /// Proxy is running and healthy.
    Running,
    /// Proxy is unhealthy.
    Unhealthy {
        since: DateTime<Utc>,
        reason: String,
    },
    /// Proxy is stopping.
    Stopping,
    /// Proxy is stopped.
    Stopped,
    /// Proxy failed to start or crashed.
    Failed {
        reason: String,
    },
}

impl Default for ProxyStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Information about a running proxy instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInstanceInfo {
    /// Profile alias.
    pub alias: String,

    /// Port the proxy is listening on.
    pub port: u16,

    /// Process ID.
    pub pid: u32,

    /// Current status.
    pub status: ProxyStatus,

    /// When the proxy was started.
    pub started_at: DateTime<Utc>,

    /// Number of restarts.
    pub restart_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_target_parse() {
        let target = ModelTarget::parse("minimax/claude-3-sonnet").unwrap();
        assert_eq!(target.provider, "minimax");
        assert_eq!(target.model, "claude-3-sonnet");
    }

    #[test]
    fn test_routing_condition_parse() {
        assert!(matches!(
            RoutingCondition::parse("always"),
            Some(RoutingCondition::Always)
        ));

        assert!(matches!(
            RoutingCondition::parse("thinking"),
            Some(RoutingCondition::ThinkingMode)
        ));

        if let Some(RoutingCondition::TokenCount { min, max }) = RoutingCondition::parse("tokens > 100000") {
            assert_eq!(min, Some(100000));
            assert_eq!(max, None);
        } else {
            panic!("Failed to parse token count condition");
        }

        if let Some(RoutingCondition::HasTools { min_count }) = RoutingCondition::parse("tools >= 5") {
            assert_eq!(min_count, Some(5));
        } else {
            panic!("Failed to parse has tools condition");
        }
    }

    #[test]
    fn test_proxy_config_serialization() {
        let config = ProfileProxyConfig {
            enabled: true,
            port: Some(8081),
            routing: RoutingConfig {
                strategy: RoutingStrategy::Conditional,
                rules: vec![
                    RoutingRule::new("default", RoutingCondition::Always, "zai/claude-3-5-sonnet"),
                ],
            },
            model_aliases: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: ProfileProxyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.port, Some(8081));
        assert!(parsed.enabled);
    }
}
