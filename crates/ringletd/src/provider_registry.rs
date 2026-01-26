//! Provider registry - loads provider manifests.

use anyhow::Result;
use ringlet_core::{RingletPaths, ProviderInfo, ProviderManifest};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Built-in provider manifests (embedded at compile time).
const BUILTIN_PROVIDERS: &[(&str, &str)] = &[
    ("anthropic", include_str!("../manifests/providers/anthropic.toml")),
    ("minimax", include_str!("../manifests/providers/minimax.toml")),
    ("minimax-openai", include_str!("../manifests/providers/minimax-openai.toml")),
    ("openai", include_str!("../manifests/providers/openai.toml")),
    ("openrouter", include_str!("../manifests/providers/openrouter.toml")),
    ("self", include_str!("../manifests/providers/self.toml")),
    ("zai", include_str!("../manifests/providers/zai.toml")),
    ("zai-openai", include_str!("../manifests/providers/zai-openai.toml")),
];

/// Provider registry.
pub struct ProviderRegistry {
    providers: HashMap<String, ProviderManifest>,
}

impl ProviderRegistry {
    /// Create a new provider registry, loading all manifests.
    pub fn new(paths: &RingletPaths) -> Result<Self> {
        let mut providers = HashMap::new();

        // Load built-in manifests
        for (id, toml) in BUILTIN_PROVIDERS {
            match ProviderManifest::from_toml(toml) {
                Ok(manifest) => {
                    debug!("Loaded built-in provider: {}", id);
                    providers.insert(id.to_string(), manifest);
                }
                Err(e) => {
                    warn!("Failed to parse built-in provider {}: {}", id, e);
                }
            }
        }

        // Load user-defined manifests from providers.d/
        let providers_d = paths.providers_d();
        if providers_d.exists() {
            if let Ok(entries) = std::fs::read_dir(&providers_d) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "toml") {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => match ProviderManifest::from_toml(&content) {
                                Ok(manifest) => {
                                    debug!("Loaded user provider from {:?}: {}", path, manifest.id);
                                    providers.insert(manifest.id.clone(), manifest);
                                }
                                Err(e) => {
                                    warn!("Failed to parse {:?}: {}", path, e);
                                }
                            },
                            Err(e) => {
                                warn!("Failed to read {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self { providers })
    }

    /// Get a provider manifest by ID.
    pub fn get(&self, id: &str) -> Option<&ProviderManifest> {
        self.providers.get(id)
    }

    /// Get all provider IDs.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.providers.keys().map(|s| s.as_str())
    }

    /// List all providers.
    pub fn list_all(&self) -> Vec<ProviderInfo> {
        let mut infos: Vec<ProviderInfo> = self
            .providers
            .values()
            .map(|m| m.to_info())
            .collect();

        // Sort by ID for consistent ordering
        infos.sort_by(|a, b| a.id.cmp(&b.id));
        infos
    }

    /// Get info for a single provider.
    pub fn get_info(&self, id: &str) -> Option<ProviderInfo> {
        self.providers.get(id).map(|m| m.to_info())
    }
}
