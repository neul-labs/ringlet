//! Profile-related request handlers.

use crate::server::ServerState;
use clown_core::rpc::error_codes;
use clown_core::{ProfileCreateRequest, Response};
use tracing::info;

/// Create a new profile.
pub async fn create(req: &ProfileCreateRequest, state: &ServerState) -> Response {
    info!("Creating profile: {} for agent {}", req.alias, req.agent_id);

    // Validate agent exists and is installed
    let mut agent_registry = state.agent_registry.lock().await;

    // First, check if agent is installed
    let detection = agent_registry.detect(&req.agent_id);
    if !detection.as_ref().map(|d| d.installed).unwrap_or(false) {
        // Check if agent exists at all
        if agent_registry.get(&req.agent_id).is_none() {
            return Response::error(
                error_codes::AGENT_NOT_FOUND,
                format!("Agent not found: {}", req.agent_id),
            );
        }
        return Response::error(
            error_codes::AGENT_NOT_INSTALLED,
            format!("Agent '{}' is not installed", req.agent_id),
        );
    }

    // Get agent info - we know it exists because detect succeeded
    let agent = agent_registry.get(&req.agent_id).unwrap();
    let agent_default_model = agent.models.default.clone();
    let source_home = agent.profile.source_home.clone();

    // Validate provider exists
    let provider = match state.provider_registry.get(&req.provider_id) {
        Some(p) => p,
        None => {
            return Response::error(
                error_codes::PROVIDER_NOT_FOUND,
                format!("Provider not found: {}", req.provider_id),
            );
        }
    };

    // Resolve endpoint
    let default_endpoint = provider.default_endpoint().unwrap_or("default");
    let endpoint_id = req.endpoint_id.as_deref().unwrap_or(default_endpoint);
    let endpoint = match provider.endpoints.get(endpoint_id) {
        Some(e) => e.clone(),
        None => {
            return Response::error(
                error_codes::INVALID_ENDPOINT,
                format!("Endpoint not found: {}", endpoint_id),
            );
        }
    };

    // Resolve model - use request model, or agent default, or provider default
    let resolved_model = req.model.clone()
        .or(agent_default_model)
        .or_else(|| provider.models.default.clone())
        .unwrap_or_else(|| "default".to_string());

    // Create the profile
    match state.profile_manager.create(req, &source_home, &endpoint, &resolved_model) {
        Ok(profile) => {
            info!("Profile '{}' created successfully", profile.alias);
            Response::success(format!("Profile '{}' created successfully", profile.alias))
        }
        Err(e) => Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Failed to create profile: {}", e),
        ),
    }
}

/// List profiles, optionally filtered by agent.
pub async fn list(agent_id: Option<&str>, state: &ServerState) -> Response {
    match state.profile_manager.list(agent_id) {
        Ok(profiles) => Response::Profiles(profiles),
        Err(e) => Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Failed to list profiles: {}", e),
        ),
    }
}

/// Inspect a specific profile.
pub async fn inspect(alias: &str, state: &ServerState) -> Response {
    match state.profile_manager.get(alias) {
        Ok(Some(profile)) => Response::Profile(profile.to_info()),
        Ok(None) => Response::error(
            error_codes::PROFILE_NOT_FOUND,
            format!("Profile not found: {}", alias),
        ),
        Err(e) => Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Failed to read profile: {}", e),
        ),
    }
}

/// Run a profile.
pub async fn run(alias: &str, args: &[String], state: &ServerState) -> Response {
    // Get the profile first
    let profile = match state.profile_manager.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            );
        }
        Err(e) => {
            return Response::error(
                error_codes::INTERNAL_ERROR,
                format!("Failed to read profile: {}", e),
            );
        }
    };

    info!("Running profile: {} (agent: {})", alias, profile.agent_id);

    // Get the agent manifest
    let mut agent_registry = state.agent_registry.lock().await;
    let agent = match agent_registry.get(&profile.agent_id) {
        Some(a) => a.clone(),
        None => {
            return Response::error(
                error_codes::AGENT_NOT_FOUND,
                format!("Agent not found: {}", profile.agent_id),
            );
        }
    };
    drop(agent_registry);

    // Get the provider manifest
    let provider = match state.provider_registry.get(&profile.provider_id) {
        Some(p) => p.clone(),
        None => {
            return Response::error(
                error_codes::PROVIDER_NOT_FOUND,
                format!("Provider not found: {}", profile.provider_id),
            );
        }
    };

    // Get API key from credential storage (only if provider requires auth)
    let api_key = if provider.auth.required {
        match state.profile_manager.get_api_key(alias) {
            Ok(key) => key,
            Err(e) => {
                return Response::error(
                    error_codes::INTERNAL_ERROR,
                    format!("Failed to retrieve API key: {}", e),
                );
            }
        }
    } else {
        // Self-authenticating provider, no API key needed
        String::new()
    };

    // Start proxy if enabled for this profile
    let proxy_url = if let Some(ref proxy_config) = profile.metadata.proxy_config {
        if proxy_config.enabled {
            match state.proxy_manager.start(alias, &profile.metadata.home, proxy_config).await {
                Ok(port) => {
                    info!("Proxy started for '{}' on port {}", alias, port);
                    Some(format!("http://127.0.0.1:{}", port))
                }
                Err(e) => {
                    return Response::error(
                        error_codes::EXECUTION_ERROR,
                        format!("Failed to start proxy: {}", e),
                    );
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // Record session start
    let started_at = chrono::Utc::now();

    // Run the profile using execution adapter
    match state.execution_adapter.run(&profile, &agent, &provider, &api_key, args, proxy_url.as_deref()) {
        Ok(result) => {
            // Mark profile as used
            if let Err(e) = state.profile_manager.mark_used(alias) {
                tracing::warn!("Failed to mark profile as used: {}", e);
            }

            // Wait for the process to complete
            let mut child = result.child;
            match child.wait() {
                Ok(status) => {
                    let exit_code = status.code().unwrap_or(-1);
                    let ended_at = chrono::Utc::now();
                    let duration = ended_at.signed_duration_since(started_at);

                    info!("Profile '{}' completed with exit code {}", alias, exit_code);

                    // Record session to telemetry
                    let session = crate::telemetry::Session {
                        profile: alias.to_string(),
                        agent_id: profile.agent_id.clone(),
                        provider_id: profile.provider_id.clone(),
                        started_at,
                        ended_at: Some(ended_at),
                        duration_secs: Some(duration.num_seconds() as u64),
                        exit_code: Some(exit_code),
                    };
                    if let Err(e) = state.telemetry.record_session(&session) {
                        tracing::warn!("Failed to record session: {}", e);
                    }

                    Response::RunCompleted { exit_code }
                }
                Err(e) => Response::error(
                    error_codes::EXECUTION_ERROR,
                    format!("Failed to wait for process: {}", e),
                ),
            }
        }
        Err(e) => Response::error(
            error_codes::EXECUTION_ERROR,
            format!("Failed to run profile: {}", e),
        ),
    }
}

/// Delete a profile.
pub async fn delete(alias: &str, state: &ServerState) -> Response {
    match state.profile_manager.delete(alias) {
        Ok(()) => Response::success(format!("Profile '{}' deleted", alias)),
        Err(e) => {
            // Check if it's a "not found" error
            let msg = e.to_string();
            if msg.contains("not found") {
                Response::error(error_codes::PROFILE_NOT_FOUND, msg)
            } else {
                Response::error(error_codes::INTERNAL_ERROR, msg)
            }
        }
    }
}

/// Get environment variables for shell export.
pub async fn env(alias: &str, state: &ServerState) -> Response {
    match state.profile_manager.get_env(alias) {
        Ok(env) => Response::Env(env),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                Response::error(error_codes::PROFILE_NOT_FOUND, msg)
            } else {
                Response::error(error_codes::INTERNAL_ERROR, msg)
            }
        }
    }
}
