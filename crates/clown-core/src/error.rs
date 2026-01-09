//! Error types for clown.

use thiserror::Error;

/// Core error type for clown operations.
#[derive(Error, Debug)]
pub enum ClownError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Profile already exists: {0}")]
    ProfileExists(String),

    #[error("Agent not installed: {0}")]
    AgentNotInstalled(String),

    #[error("Incompatible provider: agent '{agent}' does not support provider type '{provider_type}'")]
    IncompatibleProvider {
        agent: String,
        provider_type: String,
    },

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Script error: {0}")]
    ScriptError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Daemon not running")]
    DaemonNotRunning,

    #[error("Daemon connection failed: {0}")]
    DaemonConnection(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Registry sync failed: {0}")]
    RegistrySync(String),

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("Detection failed for agent '{agent}': {message}")]
    DetectionFailed { agent: String, message: String },

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias using ClownError.
pub type Result<T> = std::result::Result<T, ClownError>;
