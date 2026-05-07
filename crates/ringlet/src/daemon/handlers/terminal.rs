//! Terminal-session handlers used by the HTTP layer.

use crate::daemon::agent_usage;
use crate::daemon::handlers::profiles::prepare_execution_context;
use crate::daemon::server::ServerState;
use crate::daemon::telemetry::SessionSource;
use crate::daemon::terminal::{
    SandboxConfig, SessionId, SessionTelemetryContext, TerminalSessionInfo,
};
use portable_pty::PtySize;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

pub struct CreatedTerminalSession {
    pub session_id: SessionId,
}

pub async fn list(state: &ServerState) -> Vec<TerminalSessionInfo> {
    state.terminal_sessions.list_sessions().await
}

pub async fn get(session_id: &str, state: &ServerState) -> Option<TerminalSessionInfo> {
    let session_id = session_id.to_string();
    let session = state.terminal_sessions.get_session(&session_id).await?;
    Some(session.info().await)
}

pub async fn create_profile_session(
    profile_alias: &str,
    args: &[String],
    working_dir_override: Option<&Path>,
    initial_size: PtySize,
    sandbox_config: SandboxConfig,
    owner_token_hash: String,
    state: &ServerState,
) -> Result<CreatedTerminalSession, String> {
    let prepared = prepare_execution_context(profile_alias, args, state, true, true)
        .await
        .map_err(|response| match response {
            ringlet_core::Response::Error { message, .. } => message,
            _ => "Unexpected response type".to_string(),
        })?;

    let working_dir = working_dir_override.unwrap_or(prepared.context.working_dir.as_path());

    let telemetry_session_id = Uuid::new_v4().to_string();
    let usage_baseline = match agent_usage::snapshot_for_profile(
        &prepared.profile.agent_id,
        &prepared.profile.metadata.home,
    )
    .await
    {
        Ok(snapshot) => snapshot,
        Err(e) => {
            tracing::warn!(
                "Failed to capture terminal usage baseline for profile '{}': {}",
                prepared.profile.alias,
                e
            );
            None
        }
    };

    let session = state
        .terminal_sessions
        .create_session(
            profile_alias,
            &prepared.context.binary,
            &prepared.context.args,
            prepared.context.env,
            working_dir,
            Some(initial_size),
            sandbox_config,
            owner_token_hash,
            Some(SessionTelemetryContext {
                session_id: telemetry_session_id,
                profile: prepared.profile.alias.clone(),
                agent_id: prepared.profile.agent_id.clone(),
                provider_id: prepared.profile.provider_id.clone(),
                model: Some(prepared.profile.model.clone()),
                source: SessionSource::TerminalSession,
                profile_home: prepared.profile.metadata.home.clone(),
                usage_baseline,
                paths: state.paths.clone(),
            }),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(CreatedTerminalSession {
        session_id: session.id.clone(),
    })
}

pub async fn create_shell_session(
    shell: &str,
    env: HashMap<String, String>,
    working_dir: &Path,
    initial_size: PtySize,
    sandbox_config: SandboxConfig,
    owner_token_hash: String,
    state: &ServerState,
) -> Result<CreatedTerminalSession, String> {
    let session = state
        .terminal_sessions
        .create_session(
            "shell",
            shell,
            &["-l".to_string()],
            env,
            working_dir,
            Some(initial_size),
            sandbox_config,
            owner_token_hash,
            None,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(CreatedTerminalSession {
        session_id: session.id.clone(),
    })
}

pub async fn terminate(session_id: &str, state: &ServerState) -> Result<(), String> {
    let session_id = session_id.to_string();
    state
        .terminal_sessions
        .terminate_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn cleanup(state: &ServerState) {
    state.terminal_sessions.cleanup_terminated().await;
}
