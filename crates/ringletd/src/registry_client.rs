//! Registry client - syncs manifests from GitHub.
//!
//! This module handles:
//! - Fetching registry.json from GitHub
//! - Downloading agent/provider manifests and scripts
//! - Caching artifacts under ~/.config/ringlet/registry/commits/
//! - Managing registry.lock (current commit/channel)
//! - Syncing LiteLLM pricing data
//! - Offline mode support

use anyhow::{anyhow, Context, Result};
use ringlet_core::RingletPaths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Default registry URL.
const DEFAULT_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/neul-labs/ringlet/main/manifests";

/// Registry client for syncing from GitHub.
pub struct RegistryClient {
    paths: RingletPaths,
    base_url: String,
}

/// Registry index loaded from registry.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIndex {
    /// Version of the registry format.
    pub version: u32,

    /// Current channel (e.g., "stable", "beta").
    #[serde(default = "default_channel")]
    pub channel: String,

    /// Commit SHA.
    #[serde(default)]
    pub commit: Option<String>,

    /// Available agents.
    #[serde(default)]
    pub agents: HashMap<String, ArtifactInfo>,

    /// Available providers.
    #[serde(default)]
    pub providers: HashMap<String, ArtifactInfo>,

    /// Available scripts.
    #[serde(default)]
    pub scripts: HashMap<String, ArtifactInfo>,
}

fn default_channel() -> String {
    "stable".to_string()
}

/// Information about an artifact in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    /// Path relative to registry root.
    pub path: String,

    /// SHA256 checksum.
    #[serde(default)]
    pub checksum: Option<String>,

    /// Version.
    #[serde(default)]
    pub version: Option<String>,
}

/// Registry lock file contents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistryLock {
    /// Current channel.
    pub channel: String,

    /// Current commit SHA.
    pub commit: Option<String>,

    /// Last sync timestamp.
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,

    /// Pinned ref (if any).
    pub pinned_ref: Option<String>,
}

/// Registry sync status.
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub channel: String,
    pub commit: Option<String>,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub offline: bool,
    pub cached_agents: usize,
    pub cached_providers: usize,
    pub cached_scripts: usize,
}

impl RegistryClient {
    /// Create a new registry client.
    pub fn new(paths: RingletPaths) -> Self {
        Self {
            paths,
            base_url: DEFAULT_REGISTRY_URL.to_string(),
        }
    }

    /// Sync registry from remote.
    pub fn sync(&self, force: bool, offline: bool) -> Result<SyncStatus> {
        if offline {
            return self.get_status(true);
        }

        // Check if we need to sync
        let lock = self.load_lock()?;
        if !force && !self.needs_sync(&lock) {
            info!("Registry is up to date");
            return self.get_status(false);
        }

        // Fetch registry index
        let index = self.fetch_index()?;

        // Download artifacts
        self.download_artifacts(&index)?;

        // Sync LiteLLM pricing data
        if let Err(e) = self.sync_litellm_pricing() {
            warn!("Failed to sync LiteLLM pricing: {}. Cost tracking may be unavailable.", e);
        }

        // Update lock file
        let new_lock = RegistryLock {
            channel: index.channel.clone(),
            commit: index.commit.clone(),
            last_sync: Some(chrono::Utc::now()),
            pinned_ref: lock.pinned_ref,
        };
        self.save_lock(&new_lock)?;

        info!("Registry synced: {:?}", index.commit);
        self.get_status(false)
    }

    /// Pin to a specific ref.
    pub fn pin(&self, ref_: &str) -> Result<()> {
        let mut lock = self.load_lock()?;
        lock.pinned_ref = Some(ref_.to_string());
        self.save_lock(&lock)?;
        info!("Pinned to ref: {}", ref_);
        Ok(())
    }

    /// Get current status.
    pub fn get_status(&self, offline: bool) -> Result<SyncStatus> {
        let lock = self.load_lock()?;
        let cache_dir = self.get_cache_dir(&lock)?;

        let cached_agents = count_files(&cache_dir.join("agents"));
        let cached_providers = count_files(&cache_dir.join("providers"));
        let cached_scripts = count_files(&cache_dir.join("scripts"));

        Ok(SyncStatus {
            channel: lock.channel,
            commit: lock.commit,
            last_sync: lock.last_sync,
            offline,
            cached_agents,
            cached_providers,
            cached_scripts,
        })
    }

    /// Fetch the registry index.
    fn fetch_index(&self) -> Result<RegistryIndex> {
        let url = format!("{}/registry.json", self.base_url);
        debug!("Fetching registry index from: {}", url);

        // Use a simple HTTP client (blocking for simplicity)
        let response = ureq::get(&url)
            .call()
            .context("Failed to fetch registry.json")?;

        let index: RegistryIndex = response
            .into_json()
            .context("Failed to parse registry.json")?;

        Ok(index)
    }

    /// Download all artifacts from the registry.
    fn download_artifacts(&self, index: &RegistryIndex) -> Result<()> {
        let cache_dir = self.paths.registry_commits_dir().join(
            index.commit.as_deref().unwrap_or("latest"),
        );
        std::fs::create_dir_all(&cache_dir)?;

        // Download agents
        for (id, info) in &index.agents {
            self.download_artifact(&cache_dir.join("agents"), id, info)?;
        }

        // Download providers
        for (id, info) in &index.providers {
            self.download_artifact(&cache_dir.join("providers"), id, info)?;
        }

        // Download scripts
        for (id, info) in &index.scripts {
            self.download_artifact(&cache_dir.join("scripts"), id, info)?;
        }

        Ok(())
    }

    /// Download a single artifact.
    fn download_artifact(
        &self,
        target_dir: &PathBuf,
        id: &str,
        info: &ArtifactInfo,
    ) -> Result<()> {
        std::fs::create_dir_all(target_dir)?;

        let url = format!("{}/{}", self.base_url, info.path);
        debug!("Downloading artifact: {} from {}", id, url);

        let response = ureq::get(&url)
            .call()
            .context(format!("Failed to fetch artifact: {}", id))?;

        let content = response
            .into_string()
            .context("Failed to read artifact content")?;

        // TODO: Verify checksum if provided

        let filename = std::path::Path::new(&info.path)
            .file_name()
            .ok_or_else(|| anyhow!("Invalid artifact path"))?;

        let target_path = target_dir.join(filename);
        std::fs::write(&target_path, &content)?;

        debug!("Downloaded: {:?}", target_path);
        Ok(())
    }

    /// Check if we need to sync.
    fn needs_sync(&self, lock: &RegistryLock) -> bool {
        // Sync if no last_sync or older than 24 hours
        match lock.last_sync {
            Some(last) => {
                let age = chrono::Utc::now().signed_duration_since(last);
                age.num_hours() >= 24
            }
            None => true,
        }
    }

    /// Get the cache directory for current lock.
    fn get_cache_dir(&self, lock: &RegistryLock) -> Result<PathBuf> {
        let commit = lock.commit.as_deref().unwrap_or("latest");
        Ok(self.paths.registry_commits_dir().join(commit))
    }

    /// Load the registry lock file.
    fn load_lock(&self) -> Result<RegistryLock> {
        let lock_path = self.paths.registry_lock();
        if lock_path.exists() {
            let content = std::fs::read_to_string(&lock_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(RegistryLock::default())
        }
    }

    /// Save the registry lock file.
    fn save_lock(&self, lock: &RegistryLock) -> Result<()> {
        let lock_path = self.paths.registry_lock();
        let content = serde_json::to_string_pretty(lock)?;
        std::fs::write(lock_path, content)?;
        Ok(())
    }

    /// Sync LiteLLM pricing data.
    fn sync_litellm_pricing(&self) -> Result<()> {
        use crate::pricing::{PricingLoader, LITELLM_PRICING_URL};

        debug!("Syncing LiteLLM pricing data");

        let response = ureq::get(LITELLM_PRICING_URL)
            .call()
            .context("Failed to fetch LiteLLM pricing data")?;

        let content = response
            .into_string()
            .context("Failed to read pricing data")?;

        // Validate it's valid JSON before saving
        let _: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse LiteLLM pricing JSON")?;

        // Save to cache file
        let cache_path = self.paths.litellm_pricing_cache();
        std::fs::write(&cache_path, &content)
            .context("Failed to write pricing cache")?;

        info!("LiteLLM pricing data synced ({} bytes)", content.len());
        Ok(())
    }
}

/// Count files in a directory.
fn count_files(dir: &PathBuf) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}
