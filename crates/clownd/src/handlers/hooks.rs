//! Hooks management handlers.

use crate::server::ServerState;
use clown_core::{rpc::error_codes, HookAction, HookRule, HooksConfig, Response};
use tracing::info;

/// Add a hook rule to a profile.
pub async fn add(
    alias: &str,
    event: &str,
    matcher: &str,
    command: &str,
    state: &ServerState,
) -> Response {
    // Validate event type
    if HooksConfig::event_types().iter().all(|&e| e != event) {
        return Response::error(
            error_codes::INVALID_HOOK_EVENT,
            format!(
                "Invalid event type '{}'. Valid types: {:?}",
                event,
                HooksConfig::event_types()
            ),
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

    // Load agent to check supports_hooks
    let agent_registry = state.agent_registry.lock().await;
    let agent = match agent_registry.get(&profile.agent_id) {
        Some(a) => a,
        None => {
            return Response::error(
                error_codes::AGENT_NOT_FOUND,
                format!("Agent not found: {}", profile.agent_id),
            )
        }
    };

    if !agent.supports_hooks {
        return Response::error(
            error_codes::HOOKS_NOT_SUPPORTED,
            format!("Agent '{}' does not support hooks", agent.id),
        );
    }
    drop(agent_registry);

    // Get or create hooks config
    let mut hooks_config = profile.metadata.hooks_config.clone().unwrap_or_default();

    // Create the hook rule
    let new_rule = HookRule {
        matcher: matcher.to_string(),
        hooks: vec![HookAction::Command {
            command: command.to_string(),
            timeout: None,
        }],
    };

    // Add to the appropriate event
    if let Some(rules) = hooks_config.get_rules_mut(event) {
        rules.push(new_rule);
    }

    // Update profile
    let mut updated_profile = profile.clone();
    updated_profile.metadata.hooks_config = Some(hooks_config);

    if let Err(e) = state.profile_manager.update(&updated_profile) {
        return Response::error(error_codes::INTERNAL_ERROR, e.to_string());
    }

    info!(
        "Added hook rule to profile '{}' for event '{}' with matcher '{}'",
        alias, event, matcher
    );

    Response::success(format!(
        "Hook added to profile '{}' for event '{}'",
        alias, event
    ))
}

/// List hooks for a profile.
pub async fn list(alias: &str, state: &ServerState) -> Response {
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

    let hooks_config = profile.metadata.hooks_config.unwrap_or_default();

    Response::Hooks(hooks_config)
}

/// Remove a hook rule from a profile.
pub async fn remove(alias: &str, event: &str, index: usize, state: &ServerState) -> Response {
    // Validate event type
    if HooksConfig::event_types().iter().all(|&e| e != event) {
        return Response::error(
            error_codes::INVALID_HOOK_EVENT,
            format!(
                "Invalid event type '{}'. Valid types: {:?}",
                event,
                HooksConfig::event_types()
            ),
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

    // Get hooks config
    let mut hooks_config = match profile.metadata.hooks_config.clone() {
        Some(h) => h,
        None => {
            return Response::error(
                error_codes::INTERNAL_ERROR,
                "No hooks configured for this profile",
            )
        }
    };

    // Remove the rule at index
    if let Some(rules) = hooks_config.get_rules_mut(event) {
        if index >= rules.len() {
            return Response::error(
                error_codes::INTERNAL_ERROR,
                format!(
                    "Index {} out of range. {} has {} rules",
                    index,
                    event,
                    rules.len()
                ),
            );
        }
        rules.remove(index);
    }

    // Update profile
    let mut updated_profile = profile.clone();
    updated_profile.metadata.hooks_config = if hooks_config.is_empty() {
        None
    } else {
        Some(hooks_config)
    };

    if let Err(e) = state.profile_manager.update(&updated_profile) {
        return Response::error(error_codes::INTERNAL_ERROR, e.to_string());
    }

    info!(
        "Removed hook rule {} from profile '{}' event '{}'",
        index, alias, event
    );

    Response::success(format!(
        "Hook rule {} removed from profile '{}' event '{}'",
        index, alias, event
    ))
}

/// Import hooks configuration for a profile.
pub async fn import(alias: &str, config: &HooksConfig, state: &ServerState) -> Response {
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

    // Load agent to check supports_hooks
    let agent_registry = state.agent_registry.lock().await;
    let agent = match agent_registry.get(&profile.agent_id) {
        Some(a) => a,
        None => {
            return Response::error(
                error_codes::AGENT_NOT_FOUND,
                format!("Agent not found: {}", profile.agent_id),
            )
        }
    };

    if !agent.supports_hooks {
        return Response::error(
            error_codes::HOOKS_NOT_SUPPORTED,
            format!("Agent '{}' does not support hooks", agent.id),
        );
    }
    drop(agent_registry);

    // Update profile with new hooks config
    let mut updated_profile = profile.clone();
    updated_profile.metadata.hooks_config = if config.is_empty() {
        None
    } else {
        Some(config.clone())
    };

    if let Err(e) = state.profile_manager.update(&updated_profile) {
        return Response::error(error_codes::INTERNAL_ERROR, e.to_string());
    }

    info!("Imported hooks configuration for profile '{}'", alias);

    Response::success(format!("Hooks imported for profile '{}'", alias))
}

/// Export hooks configuration for a profile.
pub async fn export(alias: &str, state: &ServerState) -> Response {
    // Same as list - returns the hooks config
    list(alias, state).await
}
