//! Provider manifest types.

use crate::agent::ProviderCompatibility;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider manifest defining an API backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderManifest {
    /// Stable identifier (e.g., "minimax", "anthropic").
    pub id: String,

    /// Human-friendly name.
    pub name: String,

    /// API compatibility type.
    #[serde(rename = "type")]
    pub provider_type: ProviderType,

    /// Named endpoints with URLs.
    pub endpoints: HashMap<String, String>,

    /// Authentication configuration.
    pub auth: AuthConfig,

    /// Available models.
    pub models: ProviderModels,
}

/// Provider API type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderType {
    Anthropic,
    AnthropicCompatible,
    Openai,
    OpenaiCompatible,
    /// Agent handles its own authentication.
    #[serde(rename = "self")]
    SelfAuth,
}

impl ProviderType {
    /// Convert to agent compatibility type.
    pub fn to_compatibility(self) -> ProviderCompatibility {
        match self {
            Self::Anthropic => ProviderCompatibility::Anthropic,
            Self::AnthropicCompatible => ProviderCompatibility::AnthropicCompatible,
            Self::Openai => ProviderCompatibility::OpenAi,
            Self::OpenaiCompatible => ProviderCompatibility::OpenAiCompatible,
            Self::SelfAuth => ProviderCompatibility::Anthropic, // Default for self-auth
        }
    }

    /// Check if this provider type is self-authenticating.
    pub fn is_self_auth(self) -> bool {
        matches!(self, Self::SelfAuth)
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic => write!(f, "anthropic"),
            Self::AnthropicCompatible => write!(f, "anthropic-compatible"),
            Self::Openai => write!(f, "openai"),
            Self::OpenaiCompatible => write!(f, "openai-compatible"),
            Self::SelfAuth => write!(f, "self"),
        }
    }
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Environment variable name for the API key.
    pub env_key: String,

    /// Prompt message shown when asking for credentials.
    pub prompt: String,

    /// Whether authentication is required (defaults to true).
    #[serde(default = "default_auth_required")]
    pub required: bool,
}

fn default_auth_required() -> bool {
    true
}

/// Available models configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModels {
    /// List of available model identifiers.
    #[serde(default)]
    pub available: Vec<String>,

    /// Default model for this provider.
    #[serde(default)]
    pub default: Option<String>,
}

/// Endpoints configuration with default selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointsConfig {
    /// Named endpoints.
    #[serde(flatten)]
    pub endpoints: HashMap<String, String>,

    /// Default endpoint name.
    pub default: String,
}

/// Runtime information about a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider ID.
    pub id: String,

    /// Provider name.
    pub name: String,

    /// Provider type.
    pub provider_type: ProviderType,

    /// Default model.
    pub default_model: Option<String>,

    /// Available endpoints.
    pub endpoints: Vec<EndpointInfo>,

    /// Default endpoint ID.
    pub default_endpoint: String,

    /// Whether authentication is required.
    pub auth_required: bool,

    /// Authentication prompt message.
    pub auth_prompt: String,
}

/// Endpoint information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// Endpoint ID.
    pub id: String,

    /// Endpoint URL.
    pub url: String,

    /// Whether this is the default endpoint.
    pub is_default: bool,
}

impl ProviderManifest {
    /// Parse from TOML string.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Get the default endpoint ID.
    pub fn default_endpoint(&self) -> Option<&str> {
        self.endpoints.get("default").map(|s| s.as_str())
    }

    /// Get endpoint URL by ID.
    pub fn get_endpoint(&self, id: &str) -> Option<&str> {
        self.endpoints.get(id).map(|s| s.as_str())
    }

    /// Resolve endpoint URL (uses default if id is None).
    pub fn resolve_endpoint(&self, id: Option<&str>) -> Option<&str> {
        let endpoint_id = id.or_else(|| self.default_endpoint())?;
        self.get_endpoint(endpoint_id)
    }

    /// Convert to runtime info.
    pub fn to_info(&self) -> ProviderInfo {
        let default_endpoint = self
            .default_endpoint()
            .unwrap_or("default")
            .to_string();

        let endpoints = self
            .endpoints
            .iter()
            .filter(|(k, _)| *k != "default")
            .map(|(id, url)| EndpointInfo {
                id: id.clone(),
                url: url.clone(),
                is_default: id == &default_endpoint,
            })
            .collect();

        ProviderInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            provider_type: self.provider_type,
            default_model: self.models.default.clone(),
            endpoints,
            default_endpoint,
            auth_required: self.auth.required,
            auth_prompt: self.auth.prompt.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_provider_manifest() {
        let toml = r#"
            id = "minimax"
            name = "MiniMax"
            type = "anthropic-compatible"

            [endpoints]
            international = "https://api.minimax.io/anthropic"
            china = "https://api.minimaxi.com/anthropic"
            default = "international"

            [auth]
            env_key = "MINIMAX_API_KEY"
            prompt = "Enter your MiniMax API key"

            [models]
            available = ["MiniMax-M2.1"]
            default = "MiniMax-M2.1"
        "#;

        let manifest: ProviderManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.id, "minimax");
        assert_eq!(manifest.provider_type, ProviderType::AnthropicCompatible);
        assert_eq!(manifest.default_endpoint(), Some("international"));
    }
}
