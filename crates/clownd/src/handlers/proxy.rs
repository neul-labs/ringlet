//! Proxy management handlers.

use crate::server::ServerState;
use clown_core::{
    proxy::{ModelTarget, ProfileProxyConfig, RoutingRule},
    rpc::error_codes,
    Response,
};
use std::collections::HashMap;
use tracing::info;

/// Enable proxy for a profile.
pub async fn enable(alias: &str, state: &ServerState) -> Response {
    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    // Create or enable proxy_config
    let mut updated = profile.clone();
    let mut proxy_config = updated
        .metadata
        .proxy_config
        .unwrap_or_else(ProfileProxyConfig::default);
    proxy_config.enabled = true;
    updated.metadata.proxy_config = Some(proxy_config);

    // Save
    if let Err(e) = state.profile_manager.update(&updated) {
        return Response::error(error_codes::INTERNAL_ERROR, e.to_string());
    }

    info!("Enabled proxy for profile '{}'", alias);
    Response::success(format!("Proxy enabled for profile '{}'", alias))
}

/// Disable proxy for a profile.
pub async fn disable(alias: &str, state: &ServerState) -> Response {
    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    // Disable proxy_config
    let mut updated = profile.clone();
    if let Some(ref mut proxy_config) = updated.metadata.proxy_config {
        proxy_config.enabled = false;
    }

    // Save
    if let Err(e) = state.profile_manager.update(&updated) {
        return Response::error(error_codes::INTERNAL_ERROR, e.to_string());
    }

    info!("Disabled proxy for profile '{}'", alias);
    Response::success(format!("Proxy disabled for profile '{}'", alias))
}

/// Start proxy for a profile.
pub async fn start(alias: &str, state: &ServerState) -> Response {
    // Check if proxy manager is available
    if !state.proxy_manager.is_available() {
        return Response::error(
            error_codes::PROXY_NOT_SUPPORTED,
            "ultrallm binary not found. Install ultrallm to use proxy features.",
        );
    }

    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    // Check if proxy is enabled
    let proxy_config = match &profile.metadata.proxy_config {
        Some(c) if c.enabled => c.clone(),
        Some(_) => {
            return Response::error(
                error_codes::PROXY_NOT_ENABLED,
                format!("Proxy not enabled for profile '{}'. Run 'clown proxy enable {}' first.", alias, alias),
            )
        }
        None => {
            return Response::error(
                error_codes::PROXY_NOT_ENABLED,
                format!("Proxy not configured for profile '{}'. Run 'clown proxy enable {}' first.", alias, alias),
            )
        }
    };

    // Get profile home
    let profile_home = match state.profile_manager.get_home(alias) {
        Ok(home) => home,
        Err(e) => {
            return Response::error(
                error_codes::INTERNAL_ERROR,
                format!("Failed to get profile home: {}", e),
            )
        }
    };

    // Start proxy
    match state.proxy_manager.start(alias, &profile_home, &proxy_config).await {
        Ok(port) => {
            info!("Started proxy for profile '{}' on port {}", alias, port);
            Response::success(format!("Proxy started for profile '{}' on port {}", alias, port))
        }
        Err(e) => Response::error(error_codes::PROXY_START_FAILED, e.to_string()),
    }
}

/// Stop proxy for a profile.
pub async fn stop(alias: &str, state: &ServerState) -> Response {
    match state.proxy_manager.stop(alias).await {
        Ok(()) => {
            info!("Stopped proxy for profile '{}'", alias);
            Response::success(format!("Proxy stopped for profile '{}'", alias))
        }
        Err(e) => Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    }
}

/// Stop all proxies.
pub async fn stop_all(state: &ServerState) -> Response {
    match state.proxy_manager.stop_all().await {
        Ok(()) => {
            info!("Stopped all proxies");
            Response::success("All proxies stopped")
        }
        Err(e) => Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    }
}

/// Get proxy status.
pub async fn status(alias: Option<&str>, state: &ServerState) -> Response {
    let instances = state.proxy_manager.status().await;

    if let Some(a) = alias {
        let filtered: Vec<_> = instances.into_iter().filter(|i| i.alias == a).collect();
        Response::ProxyStatus(filtered)
    } else {
        Response::ProxyStatus(instances)
    }
}

/// Get proxy configuration for a profile.
pub async fn config(alias: &str, state: &ServerState) -> Response {
    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    let proxy_config = profile
        .metadata
        .proxy_config
        .unwrap_or_else(ProfileProxyConfig::default);

    Response::ProxyConfig(proxy_config)
}

/// Get proxy logs for a profile.
pub async fn logs(alias: &str, lines: Option<usize>, state: &ServerState) -> Response {
    match state.proxy_manager.read_logs(alias, lines).await {
        Ok(content) => Response::ProxyLogs(content),
        Err(e) => Response::error(error_codes::PROXY_NOT_RUNNING, e.to_string()),
    }
}

/// Add a routing rule to a profile.
pub async fn route_add(alias: &str, rule: &RoutingRule, state: &ServerState) -> Response {
    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    // Get or create proxy config
    let mut updated = profile.clone();
    let mut proxy_config = updated
        .metadata
        .proxy_config
        .unwrap_or_else(ProfileProxyConfig::default);

    // Check for duplicate rule name
    if proxy_config.routing.rules.iter().any(|r| r.name == rule.name) {
        return Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Rule '{}' already exists. Remove it first or use a different name.", rule.name),
        );
    }

    // Add rule and sort by priority (descending)
    proxy_config.routing.rules.push(rule.clone());
    proxy_config
        .routing
        .rules
        .sort_by(|a, b| b.priority.cmp(&a.priority));

    updated.metadata.proxy_config = Some(proxy_config);

    // Save
    if let Err(e) = state.profile_manager.update(&updated) {
        return Response::error(error_codes::INTERNAL_ERROR, e.to_string());
    }

    info!(
        "Added routing rule '{}' to profile '{}'",
        rule.name, alias
    );
    Response::success(format!(
        "Routing rule '{}' added to profile '{}'",
        rule.name, alias
    ))
}

/// List routing rules for a profile.
pub async fn route_list(alias: &str, state: &ServerState) -> Response {
    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    let rules = profile
        .metadata
        .proxy_config
        .map(|c| c.routing.rules)
        .unwrap_or_default();

    Response::ProxyRoutes(rules)
}

/// Remove a routing rule from a profile.
pub async fn route_remove(alias: &str, rule_name: &str, state: &ServerState) -> Response {
    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    // Get proxy config
    let mut updated = profile.clone();
    let mut proxy_config = match updated.metadata.proxy_config {
        Some(c) => c,
        None => {
            return Response::error(
                error_codes::ROUTE_NOT_FOUND,
                format!("No proxy configuration for profile '{}'", alias),
            )
        }
    };

    // Find and remove the rule
    let original_len = proxy_config.routing.rules.len();
    proxy_config.routing.rules.retain(|r| r.name != rule_name);

    if proxy_config.routing.rules.len() == original_len {
        return Response::error(
            error_codes::ROUTE_NOT_FOUND,
            format!("Rule '{}' not found in profile '{}'", rule_name, alias),
        );
    }

    updated.metadata.proxy_config = Some(proxy_config);

    // Save
    if let Err(e) = state.profile_manager.update(&updated) {
        return Response::error(error_codes::INTERNAL_ERROR, e.to_string());
    }

    info!("Removed routing rule '{}' from profile '{}'", rule_name, alias);
    Response::success(format!(
        "Routing rule '{}' removed from profile '{}'",
        rule_name, alias
    ))
}

/// Set a model alias for a profile.
pub async fn alias_set(
    alias: &str,
    from_model: &str,
    to_target: &str,
    state: &ServerState,
) -> Response {
    // Parse target
    let target = match ModelTarget::parse(to_target) {
        Some(t) => t,
        None => {
            return Response::error(
                error_codes::INTERNAL_ERROR,
                format!("Invalid target format '{}'. Expected 'provider/model'.", to_target),
            )
        }
    };

    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    // Get or create proxy config
    let mut updated = profile.clone();
    let mut proxy_config = updated
        .metadata
        .proxy_config
        .unwrap_or_else(ProfileProxyConfig::default);

    // Add/update alias
    proxy_config
        .model_aliases
        .insert(from_model.to_string(), target);

    updated.metadata.proxy_config = Some(proxy_config);

    // Save
    if let Err(e) = state.profile_manager.update(&updated) {
        return Response::error(error_codes::INTERNAL_ERROR, e.to_string());
    }

    info!(
        "Set model alias '{}' -> '{}' for profile '{}'",
        from_model, to_target, alias
    );
    Response::success(format!(
        "Model alias '{}' -> '{}' set for profile '{}'",
        from_model, to_target, alias
    ))
}

/// List model aliases for a profile.
pub async fn alias_list(alias: &str, state: &ServerState) -> Response {
    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    let aliases: HashMap<String, String> = profile
        .metadata
        .proxy_config
        .map(|c| {
            c.model_aliases
                .into_iter()
                .map(|(k, v)| (k, v.to_string_format()))
                .collect()
        })
        .unwrap_or_default();

    Response::ProxyAliases(aliases)
}

/// Remove a model alias from a profile.
pub async fn alias_remove(alias: &str, from_model: &str, state: &ServerState) -> Response {
    // Load profile
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            )
        }
        Err(e) => return Response::error(error_codes::INTERNAL_ERROR, e.to_string()),
    };

    // Get proxy config
    let mut updated = profile.clone();
    let mut proxy_config = match updated.metadata.proxy_config {
        Some(c) => c,
        None => {
            return Response::error(
                error_codes::ALIAS_NOT_FOUND,
                format!("No proxy configuration for profile '{}'", alias),
            )
        }
    };

    // Remove the alias
    if proxy_config.model_aliases.remove(from_model).is_none() {
        return Response::error(
            error_codes::ALIAS_NOT_FOUND,
            format!("Alias '{}' not found in profile '{}'", from_model, alias),
        );
    }

    updated.metadata.proxy_config = Some(proxy_config);

    // Save
    if let Err(e) = state.profile_manager.update(&updated) {
        return Response::error(error_codes::INTERNAL_ERROR, e.to_string());
    }

    info!(
        "Removed model alias '{}' from profile '{}'",
        from_model, alias
    );
    Response::success(format!(
        "Model alias '{}' removed from profile '{}'",
        from_model, alias
    ))
}
