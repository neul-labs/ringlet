//! Profile-related request handlers.

use crate::daemon::agent_usage;
use crate::daemon::server::{PendingPreparedRun, ServerState};
use ringlet_core::rpc::ExecutionContext;
use ringlet_core::rpc::error_codes;
use ringlet_core::{Event, Profile, ProfileCreateRequest, Response};
use tracing::{info, warn};
use uuid::Uuid;

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
    if !provider.endpoints.contains_key(endpoint_id) {
        return Response::error(
            error_codes::INVALID_ENDPOINT,
            format!("Endpoint not found: {}", endpoint_id),
        );
    }

    // Resolve model - use request model, or agent default, or provider default
    // But validate that the model is compatible with the provider's available models
    let candidate_model = req
        .model
        .clone()
        .or(agent_default_model)
        .or_else(|| provider.models.default.clone())
        .unwrap_or_else(|| "default".to_string());

    // If provider has an explicit list of available models, validate compatibility
    // Fall back to provider's default model if candidate isn't supported
    let resolved_model = if !provider.models.available.is_empty() {
        if provider.models.available.contains(&candidate_model) {
            candidate_model
        } else {
            // Candidate model not supported by provider, use provider's default
            provider.models.default.clone().unwrap_or(candidate_model)
        }
    } else {
        // Provider doesn't restrict models (e.g., self-auth or passthrough)
        candidate_model
    };

    // Create the profile
    match state
        .profile_manager
        .create(req, &source_home, &resolved_model)
    {
        Ok(mut profile) => {
            info!("Profile '{}' created successfully", profile.alias);

            // Auto-install alias unless --no-alias was specified
            let alias_installed = if req.no_alias {
                false
            } else {
                match super::aliases::install_alias_sync(&profile.alias) {
                    Ok(path) => {
                        info!("Installed alias at {:?}", path);
                        profile.metadata.alias_path = Some(path);
                        // Save updated profile with alias path
                        if let Err(e) = state.profile_store.update(&profile) {
                            tracing::warn!("Failed to update profile with alias path: {}", e);
                        }
                        true
                    }
                    Err(e) => {
                        // Warn but don't fail - alias installation is optional
                        tracing::warn!("Failed to install alias for '{}': {}", profile.alias, e);
                        false
                    }
                }
            };

            // Broadcast event
            state.broadcast(Event::ProfileCreated {
                alias: profile.alias.clone(),
            });

            // Build response message
            let message = if alias_installed {
                format!(
                    "Profile '{}' created. Run with: {}",
                    profile.alias, profile.alias
                )
            } else {
                format!(
                    "Profile '{}' created. Run with: ringlet profiles run {}",
                    profile.alias, profile.alias
                )
            };

            Response::success(message)
        }
        Err(e) => Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Failed to create profile: {}", e),
        ),
    }
}

/// List profiles, optionally filtered by agent.
pub async fn list(agent_id: Option<&str>, state: &ServerState) -> Response {
    match state.profile_store.list(agent_id) {
        Ok(profiles) => Response::Profiles(profiles),
        Err(e) => Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Failed to list profiles: {}", e),
        ),
    }
}

/// Inspect a specific profile.
pub async fn inspect(alias: &str, state: &ServerState) -> Response {
    match state.profile_store.get(alias) {
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

/// Run a profile (non-blocking for HTTP - returns immediately with PID).
pub async fn run(alias: &str, args: &[String], state: &ServerState) -> Response {
    let prepared = match prepare_execution_context(alias, args, state, true, true).await {
        Ok(prepared) => prepared,
        Err(response) => return response,
    };

    let profile = prepared.profile;
    let session_id = Uuid::new_v4().to_string();
    let usage_baseline =
        match agent_usage::snapshot_for_profile(&profile.agent_id, &profile.metadata.home).await {
            Ok(snapshot) => snapshot,
            Err(e) => {
                warn!(
                    "Failed to capture usage baseline for profile '{}': {}",
                    profile.alias, e
                );
                None
            }
        };

    // Record session start
    let started_at = chrono::Utc::now();

    match state.execution_adapter.spawn_prepared(&prepared.context) {
        Ok(result) => {
            let pid = result.pid;

            // Broadcast run started event
            state.broadcast(Event::ProfileRunStarted {
                alias: alias.to_string(),
                pid,
            });

            // Spawn background task to wait for completion and record telemetry
            let alias_owned = alias.to_string();
            let profile_agent_id = profile.agent_id.clone();
            let profile_provider_id = profile.provider_id.clone();
            let profile_model = profile.model.clone();
            let profile_home = profile.metadata.home.clone();
            let paths = state.paths.clone();
            let events = state.events.clone();
            let mut child = result.child;

            tokio::spawn(async move {
                match tokio::task::spawn_blocking(move || child.wait()).await {
                    Ok(Ok(status)) => {
                        let exit_code = status.code().unwrap_or(-1);
                        let ended_at = chrono::Utc::now();
                        let duration = ended_at.signed_duration_since(started_at);

                        info!(
                            "Profile '{}' completed with exit code {}",
                            alias_owned, exit_code
                        );

                        let usage_delta = match usage_baseline.as_ref() {
                            Some(baseline) => match agent_usage::delta_for_profile(
                                &profile_agent_id,
                                &profile_home,
                                baseline,
                                &profile_model,
                                &profile_provider_id,
                                &paths,
                            )
                            .await
                            {
                                Ok(delta) => delta,
                                Err(e) => {
                                    warn!(
                                        "Failed to compute usage delta for profile '{}': {}",
                                        alias_owned, e
                                    );
                                    None
                                }
                            },
                            None => None,
                        };

                        let telemetry = crate::daemon::telemetry::TelemetryCollector::new(paths);
                        let session = crate::daemon::telemetry::Session {
                            session_id,
                            profile: alias_owned.clone(),
                            agent_id: profile_agent_id,
                            provider_id: profile_provider_id,
                            started_at,
                            ended_at: Some(ended_at),
                            duration_secs: Some(duration.num_seconds() as u64),
                            exit_code: Some(exit_code),
                            source: crate::daemon::telemetry::SessionSource::ProfileRun,
                            model: Some(profile_model),
                            tokens: usage_delta.as_ref().map(|delta| delta.tokens.clone()),
                            cost: usage_delta.and_then(|delta| delta.cost),
                        };
                        if let Err(e) = telemetry.record_session(&session) {
                            warn!("Failed to record session: {}", e);
                        }

                        events.broadcast(Event::ProfileRunCompleted {
                            alias: alias_owned,
                            exit_code,
                        });
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Failed to wait for process: {}", e);
                    }
                    Err(e) => {
                        tracing::error!("Failed to join profile wait task: {}", e);
                    }
                }
            });

            // Return immediately with the PID
            Response::RunStarted { pid }
        }
        Err(e) => Response::error(
            error_codes::EXECUTION_ERROR,
            format!("Failed to run profile: {}", e),
        ),
    }
}

pub(crate) struct PreparedProfileExecution {
    pub profile: Profile,
    pub context: ExecutionContext,
}

/// Build a prepared execution context for a profile.
pub(crate) async fn prepare_execution_context(
    alias: &str,
    args: &[String],
    state: &ServerState,
    mark_used: bool,
    start_proxy: bool,
) -> Result<PreparedProfileExecution, Response> {
    let profile = match state.profile_store.get(alias) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Err(Response::error(
                error_codes::PROFILE_NOT_FOUND,
                format!("Profile not found: {}", alias),
            ));
        }
        Err(e) => {
            return Err(Response::error(
                error_codes::INTERNAL_ERROR,
                format!("Failed to read profile: {}", e),
            ));
        }
    };

    info!("Preparing profile: {} (agent: {})", alias, profile.agent_id);

    let agent_registry = state.agent_registry.lock().await;
    let agent = match agent_registry.get(&profile.agent_id) {
        Some(a) => a.clone(),
        None => {
            return Err(Response::error(
                error_codes::AGENT_NOT_FOUND,
                format!("Agent not found: {}", profile.agent_id),
            ));
        }
    };
    drop(agent_registry);

    let provider = match state.provider_registry.get(&profile.provider_id) {
        Some(p) => p.clone(),
        None => {
            return Err(Response::error(
                error_codes::PROVIDER_NOT_FOUND,
                format!("Provider not found: {}", profile.provider_id),
            ));
        }
    };

    let api_key = if provider.auth.required {
        match state.secret_store.get_api_key(alias) {
            Ok(key) => key,
            Err(e) => {
                return Err(Response::error(
                    error_codes::INTERNAL_ERROR,
                    format!("Failed to retrieve API key: {}", e),
                ));
            }
        }
    } else {
        String::new()
    };

    let proxy_url = if start_proxy {
        if let Some(ref proxy_config) = profile.metadata.proxy_config {
            if proxy_config.enabled {
                match state
                    .proxy_manager
                    .start(alias, &profile.metadata.home, proxy_config)
                    .await
                {
                    Ok(port) => {
                        info!("Proxy started for '{}' on port {}", alias, port);
                        Some(format!("http://127.0.0.1:{}", port))
                    }
                    Err(e) => {
                        return Err(Response::error(
                            error_codes::EXECUTION_ERROR,
                            format!("Failed to start proxy: {}", e),
                        ));
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    match state.execution_adapter.prepare(
        &profile,
        &agent,
        &provider,
        &api_key,
        args,
        proxy_url.as_deref(),
    ) {
        Ok(context) => {
            if mark_used && let Err(e) = state.profile_store.mark_used(alias) {
                tracing::warn!("Failed to mark profile as used: {}", e);
            }

            Ok(PreparedProfileExecution { profile, context })
        }
        Err(e) => Err(Response::error(
            error_codes::EXECUTION_ERROR,
            format!("Failed to prepare profile: {}", e),
        )),
    }
}

/// Prepare execution context for CLI-side spawning.
pub async fn prepare(alias: &str, args: &[String], state: &ServerState) -> Response {
    match prepare_execution_context(alias, args, state, true, true).await {
        Ok(prepared) => {
            let run_id = Uuid::new_v4().to_string();
            let usage_baseline = match agent_usage::snapshot_for_profile(
                &prepared.profile.agent_id,
                &prepared.profile.metadata.home,
            )
            .await
            {
                Ok(snapshot) => snapshot,
                Err(e) => {
                    warn!(
                        "Failed to capture usage baseline for prepared profile '{}': {}",
                        prepared.profile.alias, e
                    );
                    None
                }
            };

            state.pending_prepared_runs.lock().await.insert(
                run_id.clone(),
                PendingPreparedRun {
                    session_id: run_id.clone(),
                    profile: prepared.profile.alias.clone(),
                    agent_id: prepared.profile.agent_id.clone(),
                    provider_id: prepared.profile.provider_id.clone(),
                    model: prepared.profile.model.clone(),
                    profile_home: prepared.profile.metadata.home.clone(),
                    usage_baseline,
                },
            );

            let mut context = prepared.context;
            context.run_id = Some(run_id);
            Response::ExecutionContext(context)
        }
        Err(response) => response,
    }
}

/// Complete a CLI-attached prepared run and record telemetry in the daemon.
pub async fn complete(
    run_id: &str,
    started_at: chrono::DateTime<chrono::Utc>,
    ended_at: chrono::DateTime<chrono::Utc>,
    exit_code: i32,
    state: &ServerState,
) -> Response {
    let pending = state.pending_prepared_runs.lock().await.remove(run_id);
    let Some(pending) = pending else {
        return Response::error(
            error_codes::PROFILE_NOT_FOUND,
            format!("Prepared run not found: {}", run_id),
        );
    };

    let usage_delta = match pending.usage_baseline.as_ref() {
        Some(baseline) => match agent_usage::delta_for_profile(
            &pending.agent_id,
            &pending.profile_home,
            baseline,
            &pending.model,
            &pending.provider_id,
            &state.paths,
        )
        .await
        {
            Ok(delta) => delta,
            Err(e) => {
                warn!(
                    "Failed to compute usage delta for prepared run '{}': {}",
                    run_id, e
                );
                None
            }
        },
        None => None,
    };

    let telemetry = crate::daemon::telemetry::TelemetryCollector::new(state.paths.clone());
    let session = crate::daemon::telemetry::Session {
        session_id: pending.session_id,
        profile: pending.profile,
        agent_id: pending.agent_id,
        provider_id: pending.provider_id,
        started_at,
        ended_at: Some(ended_at),
        duration_secs: Some(
            ended_at
                .signed_duration_since(started_at)
                .num_seconds()
                .max(0) as u64,
        ),
        exit_code: Some(exit_code),
        source: crate::daemon::telemetry::SessionSource::ProfileRun,
        model: Some(pending.model),
        tokens: usage_delta.as_ref().map(|delta| delta.tokens.clone()),
        cost: usage_delta.and_then(|delta| delta.cost),
    };

    match telemetry.record_session(&session) {
        Ok(()) => Response::RunCompleted { exit_code },
        Err(e) => Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Failed to record run telemetry: {}", e),
        ),
    }
}

/// Delete a profile.
pub async fn delete(alias: &str, state: &ServerState) -> Response {
    // First, get the profile to check for alias_path
    let alias_path = match state.profile_store.get(alias) {
        Ok(Some(profile)) => profile.metadata.alias_path.clone(),
        _ => None,
    };

    match state.profile_manager.delete(alias) {
        Ok(()) => {
            // Try to remove alias if it was installed
            if alias_path.is_some()
                && let Some(removed) = super::aliases::uninstall_alias_sync(alias)
            {
                info!("Removed alias at {:?}", removed);
            }

            // Broadcast event
            state.broadcast(Event::ProfileDeleted {
                alias: alias.to_string(),
            });

            Response::success(format!("Profile '{}' deleted", alias))
        }
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

/// Sensitive environment variable keys that should never be exposed via HTTP.
const SENSITIVE_ENV_KEYS: &[&str] = &[
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "CLAUDE_API_KEY",
    "API_KEY",
    "SECRET",
    "TOKEN",
    "PASSWORD",
    "CREDENTIAL",
];

/// Check if an environment variable key is sensitive.
fn is_sensitive_key(key: &str) -> bool {
    let key_upper = key.to_uppercase();
    SENSITIVE_ENV_KEYS
        .iter()
        .any(|&sensitive| key_upper.contains(sensitive))
}

/// Get environment variables for shell export.
/// NOTE: Sensitive keys (API keys, tokens) are filtered out for security.
pub async fn env(alias: &str, state: &ServerState) -> Response {
    match prepare_execution_context(alias, &[], state, false, false).await {
        Ok(prepared) => {
            let mut env = prepared.context.env;
            // Filter out sensitive environment variables to prevent credential leakage
            env.retain(|key, _| !is_sensitive_key(key));
            Response::Env(env)
        }
        Err(response) => response,
    }
}
