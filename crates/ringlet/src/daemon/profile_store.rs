//! Profile persistence service.

use anyhow::{Result, anyhow};
use ringlet_core::{Profile, ProfileInfo, RingletPaths};
use std::path::PathBuf;
use tracing::debug;

/// Validate profile alias to prevent path traversal attacks.
pub(crate) fn validate_alias(alias: &str) -> Result<()> {
    if alias.contains("..") || alias.contains('/') || alias.contains('\\') || alias.contains('\0') {
        return Err(anyhow!(
            "Invalid alias: path traversal characters not allowed"
        ));
    }

    if alias.is_empty() {
        return Err(anyhow!("Invalid alias: cannot be empty"));
    }

    if !alias
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(anyhow!(
            "Invalid alias: only alphanumeric characters, underscores, and hyphens allowed"
        ));
    }

    Ok(())
}

/// JSON-backed profile repository.
pub struct ProfileStore {
    paths: RingletPaths,
}

impl ProfileStore {
    pub fn new(paths: RingletPaths) -> Self {
        Self { paths }
    }

    fn profile_file(&self, alias: &str) -> Result<PathBuf> {
        validate_alias(alias)?;
        Ok(self.paths.profiles_dir().join(format!("{}.json", alias)))
    }

    pub fn list(&self, agent_id: Option<&str>) -> Result<Vec<ProfileInfo>> {
        let profiles_dir = self.paths.profiles_dir();
        let mut profiles = Vec::new();

        if !profiles_dir.exists() {
            return Ok(profiles);
        }

        for entry in std::fs::read_dir(&profiles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "json")
                && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(profile) = serde_json::from_str::<Profile>(&content)
                && (agent_id.is_none() || agent_id == Some(profile.agent_id.as_str()))
            {
                profiles.push(profile.to_info());
            }
        }

        profiles.sort_by(|a, b| a.alias.cmp(&b.alias));
        Ok(profiles)
    }

    pub fn get(&self, alias: &str) -> Result<Option<Profile>> {
        let profile_file = self.profile_file(alias)?;
        if !profile_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&profile_file)?;
        let profile: Profile = serde_json::from_str(&content)?;
        Ok(Some(profile))
    }

    pub fn update(&self, profile: &Profile) -> Result<()> {
        let profile_file = self.profile_file(&profile.alias)?;

        if !profile_file.exists() {
            return Err(anyhow!("Profile not found: {}", profile.alias));
        }

        let content = serde_json::to_string_pretty(profile)?;
        std::fs::write(&profile_file, content)?;

        debug!("Updated profile: {}", profile.alias);
        Ok(())
    }

    pub fn save_new(&self, profile: &Profile) -> Result<()> {
        let profile_file = self.profile_file(&profile.alias)?;
        if profile_file.exists() {
            return Err(anyhow!("Profile already exists: {}", profile.alias));
        }

        let content = serde_json::to_string_pretty(profile)?;
        std::fs::write(&profile_file, content)?;

        debug!("Saved new profile: {}", profile.alias);
        Ok(())
    }

    pub fn delete(&self, alias: &str) -> Result<Profile> {
        let profile_file = self.profile_file(alias)?;

        if !profile_file.exists() {
            return Err(anyhow!("Profile not found: {}", alias));
        }

        let content = std::fs::read_to_string(&profile_file)?;
        let profile: Profile = serde_json::from_str(&content)?;
        std::fs::remove_file(&profile_file)?;

        Ok(profile)
    }

    pub fn mark_used(&self, alias: &str) -> Result<()> {
        let mut profile = self
            .get(alias)?
            .ok_or_else(|| anyhow!("Profile not found: {}", alias))?;
        profile.mark_used();
        self.update(&profile)
    }

    pub fn get_home(&self, alias: &str) -> Result<PathBuf> {
        let profile = self
            .get(alias)?
            .ok_or_else(|| anyhow!("Profile not found: {}", alias))?;
        Ok(profile.metadata.home)
    }
}
