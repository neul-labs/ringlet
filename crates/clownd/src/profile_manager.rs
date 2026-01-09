//! Profile manager - handles profile CRUD operations.

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clown_core::{
    expand_template, home_dir, Profile, ProfileCreateRequest, ProfileInfo, ProfileMetadata,
    ClownPaths,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

/// Profile manager.
pub struct ProfileManager {
    paths: ClownPaths,
}

impl ProfileManager {
    /// Create a new profile manager.
    pub fn new(paths: ClownPaths) -> Self {
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
            let keychain_key = format!("clown-{}", request.alias);
            store_credential(&keychain_key, &request.api_key)?;
            // Store a reference to the keychain for later retrieval
            env.insert(
                "_CLOWN_KEYCHAIN_KEY".to_string(),
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
        let profile_file = self.paths.profiles_dir().join(format!("{}.json", alias));

        if !profile_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&profile_file)?;
        let profile: Profile = serde_json::from_str(&content)?;
        Ok(Some(profile))
    }

    /// Delete a profile.
    pub fn delete(&self, alias: &str) -> Result<()> {
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
        let keychain_key = format!("clown-{}", alias);
        let _ = delete_credential(&keychain_key);

        // Optionally delete profile home (ask user first in real implementation)
        // For now, just log
        debug!("Profile home at {:?} preserved", profile.metadata.home);

        info!("Deleted profile: {}", alias);
        Ok(())
    }

    /// Update profile metadata after a run.
    pub fn mark_used(&self, alias: &str) -> Result<()> {
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
        let keychain_key = format!("clown-{}", alias);
        if let Ok(api_key) = get_credential(&keychain_key) {
            // TODO: These should come from Rhai script based on provider type
            // For now, add common vars
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.clone());
            env.insert("OPENAI_API_KEY".to_string(), api_key);
        }

        Ok(env)
    }

    /// Get the profile home path for an alias.
    pub fn get_home(&self, alias: &str) -> Result<PathBuf> {
        let profile = self.get(alias)?
            .ok_or_else(|| anyhow!("Profile not found: {}", alias))?;
        Ok(profile.metadata.home)
    }

    /// Get the API key for a profile.
    pub fn get_api_key(&self, alias: &str) -> Result<String> {
        let keychain_key = format!("clown-{}", alias);
        get_credential(&keychain_key)
    }
}

/// Store a credential in the keychain.
fn store_credential(key: &str, value: &str) -> Result<()> {
    #[cfg(feature = "keyring")]
    {
        let entry = keyring::Entry::new("clown", key)?;
        entry.set_password(value)?;
        Ok(())
    }

    #[cfg(not(feature = "keyring"))]
    {
        // Fallback: store in a file (not secure, for development only)
        let creds_dir = get_creds_dir()
            .ok_or_else(|| anyhow!("Cannot find home directory"))?;
        std::fs::create_dir_all(&creds_dir)?;
        let cred_file = creds_dir.join(key);
        std::fs::write(&cred_file, value)?;
        // Set permissions to user-only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&cred_file, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
}

/// Get a credential from the keychain.
fn get_credential(key: &str) -> Result<String> {
    #[cfg(feature = "keyring")]
    {
        let entry = keyring::Entry::new("clown", key)?;
        Ok(entry.get_password()?)
    }

    #[cfg(not(feature = "keyring"))]
    {
        // Fallback: read from file
        let creds_dir = get_creds_dir()
            .ok_or_else(|| anyhow!("Cannot find home directory"))?;
        let cred_file = creds_dir.join(key);
        Ok(std::fs::read_to_string(cred_file)?)
    }
}

/// Delete a credential from the keychain.
fn delete_credential(key: &str) -> Result<()> {
    #[cfg(feature = "keyring")]
    {
        let entry = keyring::Entry::new("clown", key)?;
        entry.delete_credential()?;
        Ok(())
    }

    #[cfg(not(feature = "keyring"))]
    {
        // Fallback: delete file
        let creds_dir = get_creds_dir()
            .ok_or_else(|| anyhow!("Cannot find home directory"))?;
        let cred_file = creds_dir.join(key);
        if cred_file.exists() {
            std::fs::remove_file(cred_file)?;
        }
        Ok(())
    }
}

/// Get credentials directory path.
fn get_creds_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".config/clown/credentials"))
}
