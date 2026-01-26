//! Rhai scripting engine for ringlet configuration generation.
//!
//! This crate provides:
//! - A sandboxed Rhai engine for running configuration scripts
//! - Built-in functions for JSON and TOML encoding
//! - Built-in scripts for each supported agent
//!
//! ## Script Context
//!
//! Scripts receive a context object with:
//! - `profile`: Profile information (alias, agent, provider, model, etc.)
//! - `provider`: Provider information (type, endpoints, auth)
//! - `agent`: Agent information (binary, profile strategy)
//! - `prefs`: User preferences (from config.toml)
//!
//! ## Script Output
//!
//! Scripts should return an object with:
//! - `files`: Map of relative paths to file contents
//! - `env`: Map of environment variables to set
//! - `hooks`: Optional hooks configuration
//! - `mcp_servers`: Optional MCP servers configuration

mod engine;
mod functions;

pub use engine::{
    AgentContext, PrefsContext, ProfileContext, ProviderContext,
    ScriptContext, ScriptEngine, ScriptOutput,
};

/// Built-in scripts for each agent.
pub mod scripts {
    pub const CLAUDE: &str = include_str!("scripts/claude.rhai");
    pub const GROK: &str = include_str!("scripts/grok.rhai");
    pub const CODEX: &str = include_str!("scripts/codex.rhai");
    pub const DROID: &str = include_str!("scripts/droid.rhai");
    pub const OPENCODE: &str = include_str!("scripts/opencode.rhai");

    /// Get built-in script by name.
    pub fn get(name: &str) -> Option<&'static str> {
        match name {
            "claude.rhai" => Some(CLAUDE),
            "grok.rhai" => Some(GROK),
            "codex.rhai" => Some(CODEX),
            "droid.rhai" => Some(DROID),
            "opencode.rhai" => Some(OPENCODE),
            _ => None,
        }
    }
}
