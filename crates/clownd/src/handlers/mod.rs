//! Request handlers for the daemon.

use crate::server::ServerState;
use clown_core::{Request, Response};
use tracing::warn;

pub mod agents;
pub mod aliases;
pub mod hooks;
pub mod profiles;
pub mod providers;
pub mod registry;
pub mod stats;

/// Handle an incoming request.
pub async fn handle_request(request: &Request, state: &ServerState) -> Response {
    match request {
        // Agent commands
        Request::AgentsList => agents::list(state).await,
        Request::AgentsInspect { id } => agents::inspect(id, state).await,

        // Provider commands
        Request::ProvidersList => providers::list(state).await,
        Request::ProvidersInspect { id } => providers::inspect(id, state).await,

        // Profile commands
        Request::ProfilesCreate(req) => profiles::create(req, state).await,
        Request::ProfilesList { agent_id } => profiles::list(agent_id.as_deref(), state).await,
        Request::ProfilesInspect { alias } => profiles::inspect(alias, state).await,
        Request::ProfilesRun { alias, args } => profiles::run(alias, args, state).await,
        Request::ProfilesDelete { alias } => profiles::delete(alias, state).await,
        Request::ProfilesEnv { alias } => profiles::env(alias, state).await,

        // Alias commands
        Request::AliasesInstall { alias, bin_dir } => {
            aliases::install(alias, bin_dir.as_ref(), state).await
        }
        Request::AliasesUninstall { alias } => aliases::uninstall(alias, state).await,

        // Registry commands
        Request::RegistrySync { force, offline } => registry::sync(*force, *offline, state).await,
        Request::RegistryPin { ref_ } => registry::pin(ref_, state).await,
        Request::RegistryInspect => registry::inspect(state).await,

        // Stats commands
        Request::Stats { agent_id, provider_id } => {
            stats::get_stats(agent_id.as_deref(), provider_id.as_deref(), state).await
        }

        // Hooks commands
        Request::HooksAdd {
            alias,
            event,
            matcher,
            command,
        } => hooks::add(alias, event, matcher, command, state).await,
        Request::HooksList { alias } => hooks::list(alias, state).await,
        Request::HooksRemove {
            alias,
            event,
            index,
        } => hooks::remove(alias, event, *index, state).await,
        Request::HooksImport { alias, config } => hooks::import(alias, config, state).await,
        Request::HooksExport { alias } => hooks::export(alias, state).await,

        // Ping
        Request::Ping => Response::Pong,

        // Shutdown is handled in server.rs
        Request::Shutdown => Response::success("Shutdown handled by server"),

        // Not yet implemented
        _ => {
            warn!("Unimplemented request: {:?}", request);
            Response::error(
                clown_core::rpc::error_codes::INTERNAL_ERROR,
                "Not yet implemented",
            )
        }
    }
}
