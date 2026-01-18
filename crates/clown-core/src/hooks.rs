//! Claude Code hooks configuration types.
//!
//! This module provides types for managing Claude Code hooks, which allow
//! executing commands or calling URLs at specific points during agent execution.

use serde::{Deserialize, Serialize};

/// Hooks configuration for a profile.
///
/// Contains rules for different event types that Claude Code supports.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct HooksConfig {
    /// Hooks triggered before a tool is used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_tool_use: Vec<HookRule>,

    /// Hooks triggered after a tool is used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_tool_use: Vec<HookRule>,

    /// Hooks triggered on notifications.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notification: Vec<HookRule>,

    /// Hooks triggered when the agent stops.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop: Vec<HookRule>,
}

impl HooksConfig {
    /// Check if the hooks config is empty.
    pub fn is_empty(&self) -> bool {
        self.pre_tool_use.is_empty()
            && self.post_tool_use.is_empty()
            && self.notification.is_empty()
            && self.stop.is_empty()
    }

    /// Get a mutable reference to the rules for a given event type.
    pub fn get_rules_mut(&mut self, event: &str) -> Option<&mut Vec<HookRule>> {
        match event {
            "PreToolUse" => Some(&mut self.pre_tool_use),
            "PostToolUse" => Some(&mut self.post_tool_use),
            "Notification" => Some(&mut self.notification),
            "Stop" => Some(&mut self.stop),
            _ => None,
        }
    }

    /// Get a reference to the rules for a given event type.
    pub fn get_rules(&self, event: &str) -> Option<&Vec<HookRule>> {
        match event {
            "PreToolUse" => Some(&self.pre_tool_use),
            "PostToolUse" => Some(&self.post_tool_use),
            "Notification" => Some(&self.notification),
            "Stop" => Some(&self.stop),
            _ => None,
        }
    }

    /// Get all event types that have rules.
    pub fn event_types() -> &'static [&'static str] {
        &["PreToolUse", "PostToolUse", "Notification", "Stop"]
    }
}

/// A hook rule that matches specific tools/events and executes actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookRule {
    /// Matcher pattern (e.g., "Bash|Write|Edit" or "*" for all).
    pub matcher: String,

    /// Actions to execute when the rule matches.
    pub hooks: Vec<HookAction>,
}

/// An action to execute when a hook rule matches.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HookAction {
    /// Execute a shell command synchronously.
    Command {
        /// The command to execute. Use $EVENT for JSON event data.
        command: String,
        /// Optional timeout in milliseconds.
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u32>,
    },
    /// Call a URL asynchronously (fire and forget).
    Url {
        /// The URL to call.
        url: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_serialization() {
        let config = HooksConfig {
            pre_tool_use: vec![HookRule {
                matcher: "Bash|Write".to_string(),
                hooks: vec![HookAction::Command {
                    command: "echo $EVENT".to_string(),
                    timeout: Some(5000),
                }],
            }],
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("PreToolUse"));
        assert!(json.contains("Bash|Write"));
        assert!(json.contains("command"));

        let parsed: HooksConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_hooks_empty() {
        let config = HooksConfig::default();
        assert!(config.is_empty());

        // Empty hooks should serialize to empty object
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_event_types() {
        let mut config = HooksConfig::default();

        assert!(config.get_rules_mut("PreToolUse").is_some());
        assert!(config.get_rules_mut("PostToolUse").is_some());
        assert!(config.get_rules_mut("Notification").is_some());
        assert!(config.get_rules_mut("Stop").is_some());
        assert!(config.get_rules_mut("InvalidEvent").is_none());
    }
}
