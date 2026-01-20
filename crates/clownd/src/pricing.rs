//! LiteLLM pricing loader for cost calculation.
//!
//! This module handles:
//! - Loading model pricing from cached LiteLLM JSON
//! - Calculating costs from token usage
//! - Only applies to "self" provider profiles

use anyhow::{Context, Result};
use clown_core::{ClownPaths, CostBreakdown, LiteLLMModelPricing, TokenUsage};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{debug, warn};

/// URL for LiteLLM pricing data.
pub const LITELLM_PRICING_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

/// Pricing loader for LiteLLM model pricing data.
pub struct PricingLoader {
    paths: ClownPaths,
    /// Cached pricing data (loaded lazily).
    cache: RwLock<Option<HashMap<String, LiteLLMModelPricing>>>,
}

/// Raw LiteLLM pricing entry (more fields than we need).
#[derive(Debug, Deserialize)]
struct RawLiteLLMPricing {
    input_cost_per_token: Option<f64>,
    output_cost_per_token: Option<f64>,
    cache_creation_input_token_cost: Option<f64>,
    cache_read_input_token_cost: Option<f64>,
    max_input_tokens: Option<u64>,
    max_output_tokens: Option<u64>,
    litellm_provider: Option<String>,
    supports_prompt_caching: Option<bool>,
}

impl From<RawLiteLLMPricing> for LiteLLMModelPricing {
    fn from(raw: RawLiteLLMPricing) -> Self {
        Self {
            input_cost_per_token: raw.input_cost_per_token,
            output_cost_per_token: raw.output_cost_per_token,
            cache_creation_input_token_cost: raw.cache_creation_input_token_cost,
            cache_read_input_token_cost: raw.cache_read_input_token_cost,
            max_input_tokens: raw.max_input_tokens,
            max_output_tokens: raw.max_output_tokens,
            litellm_provider: raw.litellm_provider,
            supports_prompt_caching: raw.supports_prompt_caching,
        }
    }
}

impl PricingLoader {
    /// Create a new pricing loader.
    pub fn new(paths: ClownPaths) -> Self {
        Self {
            paths,
            cache: RwLock::new(None),
        }
    }

    /// Sync pricing data from LiteLLM GitHub.
    pub fn sync(&self) -> Result<()> {
        debug!("Syncing LiteLLM pricing data from {}", LITELLM_PRICING_URL);

        let response = ureq::get(LITELLM_PRICING_URL)
            .call()
            .context("Failed to fetch LiteLLM pricing data")?;

        let content = response
            .into_string()
            .context("Failed to read pricing data")?;

        // Validate it's valid JSON before saving
        let _: HashMap<String, RawLiteLLMPricing> =
            serde_json::from_str(&content).context("Failed to parse LiteLLM pricing JSON")?;

        // Save to cache file
        let cache_path = self.paths.litellm_pricing_cache();
        std::fs::write(&cache_path, &content)
            .context("Failed to write pricing cache")?;

        debug!("LiteLLM pricing data saved to {:?}", cache_path);

        // Clear in-memory cache to force reload
        if let Ok(mut cache) = self.cache.write() {
            *cache = None;
        }

        Ok(())
    }

    /// Load pricing data from cache file.
    fn load_from_cache(&self) -> Result<HashMap<String, LiteLLMModelPricing>> {
        let cache_path = self.paths.litellm_pricing_cache();

        if !cache_path.exists() {
            return Err(anyhow::anyhow!(
                "LiteLLM pricing cache not found. Run 'clown registry sync' first."
            ));
        }

        let content = std::fs::read_to_string(&cache_path)
            .context("Failed to read pricing cache")?;

        let raw: HashMap<String, RawLiteLLMPricing> =
            serde_json::from_str(&content).context("Failed to parse pricing cache")?;

        Ok(raw.into_iter().map(|(k, v)| (k, v.into())).collect())
    }

    /// Ensure pricing data is loaded into memory.
    fn ensure_loaded(&self) -> Result<()> {
        // Check if already loaded
        if let Ok(cache) = self.cache.read() {
            if cache.is_some() {
                return Ok(());
            }
        }

        // Load from file
        let data = self.load_from_cache()?;

        // Store in cache
        if let Ok(mut cache) = self.cache.write() {
            *cache = Some(data);
        }

        Ok(())
    }

    /// Get pricing for a specific model.
    pub fn get_model_pricing(&self, model: &str) -> Option<LiteLLMModelPricing> {
        if let Err(e) = self.ensure_loaded() {
            warn!("Failed to load pricing data: {}", e);
            return None;
        }

        if let Ok(cache) = self.cache.read() {
            if let Some(data) = cache.as_ref() {
                // Try exact match first
                if let Some(pricing) = data.get(model) {
                    return Some(pricing.clone());
                }

                // Try with common prefixes/variations
                // e.g., "claude-3-5-sonnet-20241022" might be stored as "claude-3-5-sonnet"
                for (key, pricing) in data.iter() {
                    if model.starts_with(key) || key.starts_with(model) {
                        return Some(pricing.clone());
                    }
                }
            }
        }

        None
    }

    /// Calculate cost for token usage.
    ///
    /// Returns `None` if:
    /// - provider_id is not "self"
    /// - pricing data not available for the model
    pub fn calculate_cost(
        &self,
        tokens: &TokenUsage,
        model: &str,
        provider_id: &str,
    ) -> Option<CostBreakdown> {
        // Only calculate costs for "self" provider
        if provider_id != "self" {
            return None;
        }

        let pricing = self.get_model_pricing(model)?;
        Some(pricing.calculate_cost(tokens))
    }

    /// Check if pricing cache exists.
    pub fn has_cache(&self) -> bool {
        self.paths.litellm_pricing_cache().exists()
    }

    /// Get the number of models in the pricing cache.
    pub fn model_count(&self) -> usize {
        if let Err(_) = self.ensure_loaded() {
            return 0;
        }

        if let Ok(cache) = self.cache.read() {
            cache.as_ref().map(|d| d.len()).unwrap_or(0)
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_pricing_json() -> String {
        r#"{
            "claude-3-5-sonnet-20241022": {
                "input_cost_per_token": 0.000003,
                "output_cost_per_token": 0.000015,
                "cache_creation_input_token_cost": 0.00000375,
                "cache_read_input_token_cost": 0.0000003,
                "max_input_tokens": 200000,
                "max_output_tokens": 8192,
                "litellm_provider": "anthropic",
                "supports_prompt_caching": true
            },
            "gpt-4o": {
                "input_cost_per_token": 0.0000025,
                "output_cost_per_token": 0.00001,
                "max_input_tokens": 128000,
                "max_output_tokens": 16384,
                "litellm_provider": "openai"
            }
        }"#
        .to_string()
    }

    #[test]
    fn test_pricing_loader() {
        let dir = tempdir().unwrap();
        let paths = ClownPaths {
            config_dir: dir.path().to_path_buf(),
            cache_dir: dir.path().join("cache"),
            data_dir: dir.path().to_path_buf(),
        };
        paths.ensure_dirs().unwrap();

        // Write test pricing data
        let cache_path = paths.litellm_pricing_cache();
        std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        std::fs::write(&cache_path, create_test_pricing_json()).unwrap();

        let loader = PricingLoader::new(paths);

        // Test getting pricing
        let pricing = loader.get_model_pricing("claude-3-5-sonnet-20241022");
        assert!(pricing.is_some());
        let p = pricing.unwrap();
        assert!((p.input_cost_per_token.unwrap() - 0.000003).abs() < 0.0000001);

        // Test model count
        assert_eq!(loader.model_count(), 2);
    }

    #[test]
    fn test_cost_calculation_self_provider() {
        let dir = tempdir().unwrap();
        let paths = ClownPaths {
            config_dir: dir.path().to_path_buf(),
            cache_dir: dir.path().join("cache"),
            data_dir: dir.path().to_path_buf(),
        };
        paths.ensure_dirs().unwrap();

        let cache_path = paths.litellm_pricing_cache();
        std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        std::fs::write(&cache_path, create_test_pricing_json()).unwrap();

        let loader = PricingLoader::new(paths);

        let tokens = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };

        // Should calculate for "self" provider
        let cost = loader.calculate_cost(&tokens, "claude-3-5-sonnet-20241022", "self");
        assert!(cost.is_some());
        let c = cost.unwrap();
        assert!((c.input_cost - 0.003).abs() < 0.0001);
        assert!((c.output_cost - 0.0075).abs() < 0.0001);

        // Should NOT calculate for other providers
        let cost = loader.calculate_cost(&tokens, "claude-3-5-sonnet-20241022", "anthropic");
        assert!(cost.is_none());
    }
}
