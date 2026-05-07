//! Secret storage service.

use crate::daemon::profile_store::validate_alias;
use anyhow::{Context, Result};

/// Keychain-backed credential store for profile secrets.
pub struct SecretStore;

impl SecretStore {
    pub fn new() -> Self {
        Self
    }

    pub fn store_api_key(&self, alias: &str, api_key: &str) -> Result<Option<String>> {
        validate_alias(alias)?;

        if api_key.is_empty() {
            return Ok(None);
        }

        let keychain_key = Self::keychain_key(alias);
        let entry = keyring::Entry::new("ringlet", &keychain_key)
            .context("Failed to access system keychain")?;
        entry
            .set_password(api_key)
            .context("Failed to store credential in keychain")?;
        Ok(Some(keychain_key))
    }

    pub fn get_api_key(&self, alias: &str) -> Result<String> {
        validate_alias(alias)?;

        let entry = keyring::Entry::new("ringlet", &Self::keychain_key(alias))
            .context("Failed to access system keychain")?;
        entry
            .get_password()
            .context("Failed to retrieve credential from keychain")
    }

    pub fn delete_api_key(&self, alias: &str) -> Result<()> {
        validate_alias(alias)?;

        let entry = keyring::Entry::new("ringlet", &Self::keychain_key(alias))
            .context("Failed to access system keychain")?;
        let _ = entry.delete_credential();
        Ok(())
    }

    fn keychain_key(alias: &str) -> String {
        format!("ringlet-{}", alias)
    }
}
