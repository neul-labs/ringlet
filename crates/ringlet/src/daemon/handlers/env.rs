//! Environment setup task handlers.

use crate::daemon::handlers::profiles::prepare_execution_context;
use crate::daemon::server::ServerState;
use ringlet_core::Response;
use ringlet_core::rpc::error_codes;
use tokio::process::Command;
use tracing::info;

/// Run a manifest-defined setup task for a profile.
pub async fn setup(alias: &str, task: &str, state: &ServerState) -> Response {
    let prepared = match prepare_execution_context(alias, &[], state, false, false).await {
        Ok(prepared) => prepared,
        Err(response) => return response,
    };

    let agent_registry = state.agent_registry.lock().await;
    let agent = match agent_registry.get(&prepared.profile.agent_id) {
        Some(agent) => agent,
        None => {
            return Response::error(
                error_codes::AGENT_NOT_FOUND,
                format!("Agent not found: {}", prepared.profile.agent_id),
            );
        }
    };

    let setup_task = match agent.setup_tasks.get(task) {
        Some(task) => task,
        None => {
            let available = if agent.setup_tasks.is_empty() {
                "no setup tasks are defined".to_string()
            } else {
                let mut names: Vec<_> = agent.setup_tasks.keys().cloned().collect();
                names.sort();
                format!("available tasks: {}", names.join(", "))
            };

            return Response::error(
                error_codes::INTERNAL_ERROR,
                format!(
                    "Setup task '{}' not found for agent '{}'; {}",
                    task, agent.id, available
                ),
            );
        }
    };

    info!(
        "Running setup task '{}' for profile '{}' (agent '{}')",
        task, alias, prepared.profile.agent_id
    );

    let mut command = shell_command(&setup_task.command);
    command.current_dir(&prepared.context.working_dir);
    command.env_clear();
    command.envs(&prepared.context.env);

    match command.output().await {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let message = if stdout.is_empty() {
                format!("Setup task '{}' completed for profile '{}'", task, alias)
            } else {
                format!(
                    "Setup task '{}' completed for profile '{}': {}",
                    task, alias, stdout
                )
            };
            Response::success(message)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let detail = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                format!("exit status {}", output.status)
            };

            Response::error(
                error_codes::EXECUTION_ERROR,
                format!(
                    "Setup task '{}' failed for profile '{}': {}",
                    task, alias, detail
                ),
            )
        }
        Err(e) => Response::error(
            error_codes::EXECUTION_ERROR,
            format!(
                "Failed to start setup task '{}' for profile '{}': {}",
                task, alias, e
            ),
        ),
    }
}

#[cfg(unix)]
fn shell_command(command: &str) -> Command {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-lc").arg(command);
    cmd
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg(command);
    cmd
}
