//! Profile manager - handles profile CRUD operations.

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use ringlet_core::{
    expand_template, home_dir, Profile, ProfileCreateRequest, ProfileInfo, ProfileMetadata,
    ProfileProxyConfig, RingletPaths,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};
use zeroize::Zeroizing;

/// Validate profile alias to prevent path traversal attacks.
fn validate_alias(alias: &str) -> Result<()> {
    // Check for path traversal sequences
    if alias.contains("..") || alias.contains('/') || alias.contains('\\') || alias.contains('\0') {
        return Err(anyhow!("Invalid alias: path traversal characters not allowed"));
    }

    // Alias must be non-empty
    if alias.is_empty() {
        return Err(anyhow!("Invalid alias: cannot be empty"));
    }

    // Only allow safe characters: alphanumeric, underscore, hyphen
    if !alias.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(anyhow!("Invalid alias: only alphanumeric characters, underscores, and hyphens allowed"));
    }

    Ok(())
}

/// Profile manager.
pub struct ProfileManager {
    paths: RingletPaths,
}

impl ProfileManager {
    /// Create a new profile manager.
    pub fn new(paths: RingletPaths) -> Self {
        Self { paths }
    }

    /// Create a new profile.
    pub fn create(
        &self,
        request: &ProfileCreateRequest,
        agent_source_home: &str,
        provider_endpoint: &str,
        resolved_model: &str,
    ) -> Result<Profile> {
        // Validate alias to prevent path traversal
        validate_alias(&request.alias)?;

        let profiles_dir = self.paths.profiles_dir();
        let profile_file = profiles_dir.join(format!("{}.json", request.alias));

        // Check if profile already exists
        if profile_file.exists() {
            return Err(anyhow!("Profile already exists: {}", request.alias));
        }

        // Create profile home directory
        let home = expand_template(agent_source_home, &request.alias, &request.agent_id);
        std::fs::create_dir_all(&home)
            .context(format!("Failed to create profile home: {:?}", home))?;

        info!("Created profile home: {:?}", home);

        // Build environment variables
        let mut env = HashMap::new();

        // Store API key in keychain only if provided (self-auth providers don't need it)
        if !request.api_key.is_empty() {
            let keychain_key = format!("ringlet-{}", request.alias);
            store_credential(&keychain_key, &request.api_key)?;
            // Store a reference to the keychain for later retrieval
            env.insert(
                "_RINGLET_KEYCHAIN_KEY".to_string(),
                keychain_key.clone(),
            );
        }

        // Create profile
        let profile = Profile {
            alias: request.alias.clone(),
            agent_id: request.agent_id.clone(),
            provider_id: request.provider_id.clone(),
            endpoint_id: request.endpoint_id.clone().unwrap_or_else(|| "default".to_string()),
            model: resolved_model.to_string(),
            env,
            args: request.args.clone(),
            working_dir: request.working_dir.clone(),
            metadata: ProfileMetadata {
                home,
                created_at: Utc::now(),
                last_used: None,
                total_runs: 0,
                enabled_hooks: request.hooks.clone(),
                enabled_mcp_servers: request.mcp_servers.clone(),
                hooks_config: None,
                proxy_config: if request.proxy {
                    Some(ProfileProxyConfig::default())
                } else {
                    None
                },
                alias_path: None,
            },
        };

        // Save profile
        let json = serde_json::to_string_pretty(&profile)?;
        std::fs::write(&profile_file, &json)
            .context(format!("Failed to write profile: {:?}", profile_file))?;

        info!("Created profile: {}", request.alias);

        Ok(profile)
    }

    /// List all profiles, optionally filtered by agent.
    pub fn list(&self, agent_id: Option<&str>) -> Result<Vec<ProfileInfo>> {
        let profiles_dir = self.paths.profiles_dir();
        let mut profiles = Vec::new();

        if !profiles_dir.exists() {
            return Ok(profiles);
        }

        for entry in std::fs::read_dir(&profiles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(profile) = serde_json::from_str::<Profile>(&content) {
                        if agent_id.is_none() || agent_id == Some(profile.agent_id.as_str()) {
                            profiles.push(profile.to_info());
                        }
                    }
                }
            }
        }

        // Sort by alias
        profiles.sort_by(|a, b| a.alias.cmp(&b.alias));
        Ok(profiles)
    }

    /// Get a profile by alias.
    pub fn get(&self, alias: &str) -> Result<Option<Profile>> {
        // Validate alias to prevent path traversal
        validate_alias(alias)?;

        let profile_file = self.paths.profiles_dir().join(format!("{}.json", alias));

        if !profile_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&profile_file)?;
        let profile: Profile = serde_json::from_str(&content)?;
        Ok(Some(profile))
    }

    /// Update a profile.
    pub fn update(&self, profile: &Profile) -> Result<()> {
        // Validate alias to prevent path traversal
        validate_alias(&profile.alias)?;

        let profile_file = self
            .paths
            .profiles_dir()
            .join(format!("{}.json", profile.alias));

        if !profile_file.exists() {
            return Err(anyhow!("Profile not found: {}", profile.alias));
        }

        let content = serde_json::to_string_pretty(profile)?;
        std::fs::write(&profile_file, content)?;

        debug!("Updated profile: {}", profile.alias);
        Ok(())
    }

    /// Delete a profile.
    pub fn delete(&self, alias: &str) -> Result<()> {
        // Validate alias to prevent path traversal
        validate_alias(alias)?;

        let profile_file = self.paths.profiles_dir().join(format!("{}.json", alias));

        if !profile_file.exists() {
            return Err(anyhow!("Profile not found: {}", alias));
        }

        // Load profile to get home dir
        let content = std::fs::read_to_string(&profile_file)?;
        let profile: Profile = serde_json::from_str(&content)?;

        // Delete profile file
        std::fs::remove_file(&profile_file)?;

        // Try to delete keychain entry
        let keychain_key = format!("ringlet-{}", alias);
        let _ = delete_credential(&keychain_key);

        // Optionally delete profile home (ask user first in real implementation)
        // For now, just log
        debug!("Profile home at {:?} preserved", profile.metadata.home);

        info!("Deleted profile: {}", alias);
        Ok(())
    }

    /// Update profile metadata after a run.
    pub fn mark_used(&self, alias: &str) -> Result<()> {
        // Validate alias to prevent path traversal
        validate_alias(alias)?;

        let profile_file = self.paths.profiles_dir().join(format!("{}.json", alias));

        if !profile_file.exists() {
            return Err(anyhow!("Profile not found: {}", alias));
        }

        let content = std::fs::read_to_string(&profile_file)?;
        let mut profile: Profile = serde_json::from_str(&content)?;

        profile.mark_used();

        let json = serde_json::to_string_pretty(&profile)?;
        std::fs::write(&profile_file, &json)?;

        Ok(())
    }

    /// Get environment variables for a profile (for shell export).
    pub fn get_env(&self, alias: &str) -> Result<HashMap<String, String>> {
        let profile = self.get(alias)?
            .ok_or_else(|| anyhow!("Profile not found: {}", alias))?;

        let mut env = profile.env.clone();

        // Add HOME override
        env.insert(
            "HOME".to_string(),
            profile.metadata.home.to_string_lossy().to_string(),
        );

        // Retrieve API key from keychain and add appropriate env vars
        // Use Zeroizing to ensure the API key is zeroed when it goes out of scope
        let keychain_key = format!("ringlet-{}", alias);
        if let Ok(api_key) = get_credential(&keychain_key) {
            let api_key = Zeroizing::new(api_key);
            // TODO: These should come from Rhai script based on provider type
            // For now, add common vars
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert("OPENAI_API_KEY".to_string(), api_key.to_string());
            // api_key is automatically zeroed when it goes out of scope here
        }

        Ok(env)
    }

    /// Get the profile home path for an alias.
    pub fn get_home(&self, alias: &str) -> Result<PathBuf> {
        let profile = self.get(alias)?
            .ok_or_else(|| anyhow!("Profile not found: {}", alias))?;
        Ok(profile.metadata.home)
    }

    /// Get the API key for a profile (wrapped in Zeroizing for secure memory handling).
    pub fn get_api_key(&self, alias: &str) -> Result<String> {
        // Validate alias to prevent injection
        validate_alias(alias)?;

        let keychain_key = format!("ringlet-{}", alias);
        // Note: The caller should wrap the returned String in Zeroizing if holding it
        get_credential(&keychain_key)
    }
}

/// Store a credential in the system keychain.
fn store_credential(key: &str, value: &str) -> Result<()> {
    let entry = keyring::Entry::new("ringlet", key)
        .context("Failed to access system keychain")?;
    entry.set_password(value)
        .context("Failed to store credential in keychain")?;
    Ok(())
}

/// Get a credential from the system keychain.
fn get_credential(key: &str) -> Result<String> {
    let entry = keyring::Entry::new("ringlet", key)
        .context("Failed to access system keychain")?;
    entry.get_password()
        .context("Failed to retrieve credential from keychain")
}

/// Delete a credential from the system keychain.
fn delete_credential(key: &str) -> Result<()> {
    let entry = keyring::Entry::new("ringlet", key)
        .context("Failed to access system keychain")?;
    // Ignore errors if credential doesn't exist
    let _ = entry.delete_credential();
    Ok(())
}
