//! Profile manager - handles profile CRUD operations.

use crate::daemon::profile_store::ProfileStore;
use crate::daemon::secret_store::SecretStore;
use anyhow::{Result, anyhow};
use chrono::Utc;
use ringlet_core::{
    Profile, ProfileCreateRequest, ProfileMetadata, ProfileProxyConfig, RingletPaths,
    expand_template,
};
use std::collections::HashMap;
use tracing::{debug, info};

/// Profile manager.
pub struct ProfileManager {
    profile_store: ProfileStore,
    secret_store: SecretStore,
}

impl ProfileManager {
    /// Create a new profile manager.
    pub fn new(paths: RingletPaths) -> Self {
        Self {
            profile_store: ProfileStore::new(paths),
            secret_store: SecretStore::new(),
        }
    }

    /// Create a new profile.
    pub fn create(
        &self,
        request: &ProfileCreateRequest,
        agent_source_home: &str,
        resolved_model: &str,
    ) -> Result<Profile> {
        if self.profile_store.get(&request.alias)?.is_some() {
            return Err(anyhow!("Profile already exists: {}", request.alias));
        }

        // Create profile home directory
        let home = expand_template(agent_source_home, &request.alias, &request.agent_id);
        std::fs::create_dir_all(&home)
            .map_err(|e| anyhow!("Failed to create profile home {:?}: {}", home, e))?;

        info!("Created profile home: {:?}", home);

        // Build environment variables
        let mut env = HashMap::new();

        if let Some(keychain_key) = self
            .secret_store
            .store_api_key(&request.alias, &request.api_key)?
        {
            env.insert("_RINGLET_KEYCHAIN_KEY".to_string(), keychain_key);
        }

        // Create profile
        let profile = Profile {
            alias: request.alias.clone(),
            agent_id: request.agent_id.clone(),
            provider_id: request.provider_id.clone(),
            endpoint_id: request
                .endpoint_id
                .clone()
                .unwrap_or_else(|| "default".to_string()),
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

        self.profile_store.save_new(&profile)?;

        info!("Created profile: {}", request.alias);

        Ok(profile)
    }

    /// Delete a profile.
    pub fn delete(&self, alias: &str) -> Result<()> {
        let profile = self.profile_store.delete(alias)?;
        self.secret_store.delete_api_key(alias)?;

        // Optionally delete profile home (ask user first in real implementation)
        // For now, just log
        debug!("Profile home at {:?} preserved", profile.metadata.home);

        info!("Deleted profile: {}", alias);
        Ok(())
    }
}
