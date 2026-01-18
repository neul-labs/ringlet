//! Command implementations.

use crate::client::DaemonClient;
use crate::output;
use crate::{
    AgentsCommands, AliasesCommands, Commands, DaemonCommands, EnvCommands, HooksCommands,
    ProfilesCommands, ProvidersCommands, RegistryCommands,
};
use anyhow::{anyhow, Result};
use clown_core::{HooksConfig, ProfileCreateRequest, Request, Response};

/// Execute a command.
pub async fn execute(command: &Commands, json: bool) -> Result<()> {
    match command {
        Commands::Agents { command } => execute_agents(command, json).await,
        Commands::Providers { command } => execute_providers(command, json).await,
        Commands::Profiles { command } => execute_profiles(command, json).await,
        Commands::Aliases { command } => execute_aliases(command, json).await,
        Commands::Registry { command } => execute_registry(command, json).await,
        Commands::Stats { agent, provider } => execute_stats(agent, provider, json).await,
        Commands::Daemon { command } => execute_daemon(command, json).await,
        Commands::Env { command } => execute_env(command, json).await,
        Commands::Hooks { command } => execute_hooks(command, json).await,
    }
}

async fn execute_agents(command: &AgentsCommands, json: bool) -> Result<()> {
    let client = DaemonClient::connect()?;

    match command {
        AgentsCommands::List => {
            let response = client.request(&Request::AgentsList)?;
            match response {
                Response::Agents(agents) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&agents)?);
                    } else {
                        println!("{}", output::agents_table(&agents));
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        AgentsCommands::Inspect { id } => {
            let response = client.request(&Request::AgentsInspect { id: id.clone() })?;
            match response {
                Response::Agent(agent) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&agent)?);
                    } else {
                        println!("{}", output::agent_detail(&agent));
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
    }

    Ok(())
}

async fn execute_providers(command: &ProvidersCommands, json: bool) -> Result<()> {
    let client = DaemonClient::connect()?;

    match command {
        ProvidersCommands::List => {
            let response = client.request(&Request::ProvidersList)?;
            match response {
                Response::Providers(providers) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&providers)?);
                    } else {
                        println!("{}", output::providers_table(&providers));
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProvidersCommands::Inspect { id } => {
            let response = client.request(&Request::ProvidersInspect { id: id.clone() })?;
            match response {
                Response::Provider(provider) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&provider)?);
                    } else {
                        println!("{}", output::provider_detail(&provider));
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
    }

    Ok(())
}

async fn execute_profiles(command: &ProfilesCommands, json: bool) -> Result<()> {
    let client = DaemonClient::connect()?;

    match command {
        ProfilesCommands::Create {
            agent,
            alias,
            provider,
            model,
            endpoint,
            api_key,
            hooks,
            mcp,
            bare,
            proxy,
        } => {
            // Get provider info to check if auth is required
            let provider_response = client.request(&Request::ProvidersInspect { id: provider.clone() })?;
            let (auth_required, auth_prompt) = match provider_response {
                Response::Provider(info) => (info.auth_required, info.auth_prompt),
                Response::Error { message, .. } => return Err(anyhow!("{}", message)),
                _ => return Err(anyhow!("Unexpected response")),
            };

            // Only prompt for API key if auth is required
            let api_key = if auth_required {
                match api_key {
                    Some(key) => key.clone(),
                    None => {
                        let prompt = if auth_prompt.is_empty() {
                            "Enter API key".to_string()
                        } else {
                            auth_prompt
                        };
                        dialoguer::Password::new()
                            .with_prompt(&prompt)
                            .interact()?
                    }
                }
            } else {
                // Self-authenticating provider, no API key needed
                String::new()
            };

            let hooks_vec = hooks
                .as_ref()
                .map(|h| h.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let mcp_vec = mcp
                .as_ref()
                .map(|m| m.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let request = ProfileCreateRequest {
                agent_id: agent.clone(),
                alias: alias.clone(),
                provider_id: provider.clone(),
                endpoint_id: endpoint.clone(),
                model: model.clone(),
                api_key,
                hooks: hooks_vec,
                mcp_servers: mcp_vec,
                args: vec![],
                working_dir: None,
                bare: *bare,
                proxy: *proxy,
            };

            let response = client.request(&Request::ProfilesCreate(request))?;
            match response {
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProfilesCommands::List { agent } => {
            let response = client.request(&Request::ProfilesList {
                agent_id: agent.clone(),
            })?;
            match response {
                Response::Profiles(profiles) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&profiles)?);
                    } else if profiles.is_empty() {
                        println!("No profiles found");
                    } else {
                        println!("{}", output::profiles_table(&profiles));
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProfilesCommands::Inspect { alias } => {
            let response = client.request(&Request::ProfilesInspect {
                alias: alias.clone(),
            })?;
            match response {
                Response::Profile(profile) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&profile)?);
                    } else {
                        println!("{}", output::profile_detail(&profile));
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProfilesCommands::Run { alias, args } => {
            let response = client.request(&Request::ProfilesRun {
                alias: alias.clone(),
                args: args.clone(),
            })?;
            match response {
                Response::Success { message } => {
                    if !json {
                        output::success(&message);
                    }
                }
                Response::RunCompleted { exit_code } => {
                    if json {
                        println!("{}", serde_json::json!({"exit_code": exit_code}));
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProfilesCommands::Delete { alias } => {
            let response = client.request(&Request::ProfilesDelete {
                alias: alias.clone(),
            })?;
            match response {
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProfilesCommands::Env { alias } => {
            let response = client.request(&Request::ProfilesEnv {
                alias: alias.clone(),
            })?;
            match response {
                Response::Env(env) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&env)?);
                    } else {
                        println!("{}", output::env_export(&env));
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
    }

    Ok(())
}

async fn execute_aliases(command: &AliasesCommands, json: bool) -> Result<()> {
    let client = DaemonClient::connect()?;

    match command {
        AliasesCommands::Install { alias, bin_dir } => {
            let response = client.request(&Request::AliasesInstall {
                alias: alias.clone(),
                bin_dir: bin_dir.clone(),
            })?;
            match response {
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        AliasesCommands::Uninstall { alias } => {
            let response = client.request(&Request::AliasesUninstall {
                alias: alias.clone(),
            })?;
            match response {
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
    }

    Ok(())
}

async fn execute_registry(command: &RegistryCommands, json: bool) -> Result<()> {
    let client = DaemonClient::connect()?;

    match command {
        RegistryCommands::Sync { force, offline } => {
            let response = client.request(&Request::RegistrySync {
                force: *force,
                offline: *offline,
            })?;
            match response {
                Response::RegistryStatus(status) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    } else {
                        println!("Channel: {}", status.channel);
                        if let Some(commit) = &status.commit {
                            println!("Commit: {}", commit);
                        }
                        if let Some(last_sync) = &status.last_sync {
                            println!("Last Sync: {}", last_sync);
                        }
                        println!("Offline: {}", status.offline);
                    }
                }
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        RegistryCommands::Pin { ref_ } => {
            let response = client.request(&Request::RegistryPin {
                ref_: ref_.clone(),
            })?;
            match response {
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        RegistryCommands::Inspect => {
            let response = client.request(&Request::RegistryInspect)?;
            match response {
                Response::RegistryStatus(status) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    } else {
                        println!("Channel: {}", status.channel);
                        if let Some(commit) = &status.commit {
                            println!("Commit: {}", commit);
                        }
                        if let Some(last_sync) = &status.last_sync {
                            println!("Last Sync: {}", last_sync);
                        }
                        println!("Cached Agents: {}", status.cached_agents);
                        println!("Cached Providers: {}", status.cached_providers);
                        println!("Cached Scripts: {}", status.cached_scripts);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
    }

    Ok(())
}

async fn execute_stats(
    agent: &Option<String>,
    provider: &Option<String>,
    json: bool,
) -> Result<()> {
    let client = DaemonClient::connect()?;

    let response = client.request(&Request::Stats {
        agent_id: agent.clone(),
        provider_id: provider.clone(),
    })?;

    match response {
        Response::Stats(stats) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                println!("Total Sessions: {}", stats.total_sessions);
                println!("Total Runtime: {}s", stats.total_runtime_secs);

                if !stats.by_agent.is_empty() {
                    println!("\nBy Agent:");
                    for (id, s) in &stats.by_agent {
                        println!("  {}: {} sessions, {}s runtime", id, s.sessions, s.runtime_secs);
                    }
                }

                if !stats.by_provider.is_empty() {
                    println!("\nBy Provider:");
                    for (id, s) in &stats.by_provider {
                        println!("  {}: {} sessions, {}s runtime", id, s.sessions, s.runtime_secs);
                    }
                }
            }
        }
        Response::Error { message, .. } => return Err(anyhow!(message)),
        _ => return Err(anyhow!("Unexpected response")),
    }

    Ok(())
}

async fn execute_daemon(command: &DaemonCommands, json: bool) -> Result<()> {
    match command {
        DaemonCommands::Start { stay_alive } => {
            // Start daemon in foreground by exec'ing clownd
            let clownd = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow!("Cannot find parent directory"))?
                .join("clownd");

            let mut cmd = std::process::Command::new(&clownd);
            cmd.arg("--foreground");
            if *stay_alive {
                cmd.arg("--stay-alive");
            }

            let status = cmd.status()?;
            std::process::exit(status.code().unwrap_or(1));
        }
        DaemonCommands::Stop => {
            match DaemonClient::connect() {
                Ok(client) => {
                    client.shutdown()?;
                    if json {
                        println!("{}", serde_json::json!({"success": "Daemon stopped"}));
                    } else {
                        output::success("Daemon stopped");
                    }
                }
                Err(_) => {
                    if json {
                        println!("{}", serde_json::json!({"success": "Daemon not running"}));
                    } else {
                        output::success("Daemon not running");
                    }
                }
            }
        }
        DaemonCommands::Status => {
            match DaemonClient::connect() {
                Ok(client) => {
                    if client.ping() {
                        if json {
                            println!("{}", serde_json::json!({"status": "running"}));
                        } else {
                            println!("Daemon is running");
                        }
                    } else {
                        if json {
                            println!("{}", serde_json::json!({"status": "not responding"}));
                        } else {
                            println!("Daemon not responding");
                        }
                    }
                }
                Err(_) => {
                    if json {
                        println!("{}", serde_json::json!({"status": "stopped"}));
                    } else {
                        println!("Daemon is not running");
                    }
                }
            }
        }
    }

    Ok(())
}

async fn execute_env(command: &EnvCommands, json: bool) -> Result<()> {
    let client = DaemonClient::connect()?;

    match command {
        EnvCommands::Setup { alias, task } => {
            let response = client.request(&Request::EnvSetup {
                alias: alias.clone(),
                task: task.clone(),
            })?;
            match response {
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
    }

    Ok(())
}

async fn execute_hooks(command: &HooksCommands, json: bool) -> Result<()> {
    let client = DaemonClient::connect()?;

    match command {
        HooksCommands::Add {
            alias,
            event,
            matcher,
            command,
        } => {
            let response = client.request(&Request::HooksAdd {
                alias: alias.clone(),
                event: event.clone(),
                matcher: matcher.clone(),
                command: command.clone(),
            })?;
            match response {
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        HooksCommands::List { alias } => {
            let response = client.request(&Request::HooksList {
                alias: alias.clone(),
            })?;
            match response {
                Response::Hooks(hooks) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&hooks)?);
                    } else {
                        print_hooks(&hooks);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        HooksCommands::Remove {
            alias,
            event,
            index,
        } => {
            let response = client.request(&Request::HooksRemove {
                alias: alias.clone(),
                event: event.clone(),
                index: *index,
            })?;
            match response {
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        HooksCommands::Import { alias, file } => {
            let content = std::fs::read_to_string(file)
                .map_err(|e| anyhow!("Failed to read file: {}", e))?;
            let config: HooksConfig = serde_json::from_str(&content)
                .map_err(|e| anyhow!("Invalid hooks JSON: {}", e))?;

            let response = client.request(&Request::HooksImport {
                alias: alias.clone(),
                config,
            })?;
            match response {
                Response::Success { message } => {
                    if json {
                        println!("{}", serde_json::json!({"success": message}));
                    } else {
                        output::success(&message);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        HooksCommands::Export { alias } => {
            let response = client.request(&Request::HooksExport {
                alias: alias.clone(),
            })?;
            match response {
                Response::Hooks(hooks) => {
                    // Always output JSON for export (pipe-friendly)
                    println!("{}", serde_json::to_string_pretty(&hooks)?);
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
    }

    Ok(())
}

fn print_hooks(hooks: &HooksConfig) {
    let events = [
        ("PreToolUse", &hooks.pre_tool_use),
        ("PostToolUse", &hooks.post_tool_use),
        ("Notification", &hooks.notification),
        ("Stop", &hooks.stop),
    ];

    let mut has_hooks = false;
    for (event_name, rules) in &events {
        if !rules.is_empty() {
            has_hooks = true;
            println!("{}:", event_name);
            for (i, rule) in rules.iter().enumerate() {
                println!("  [{}] matcher: {}", i, rule.matcher);
                for (j, action) in rule.hooks.iter().enumerate() {
                    match action {
                        clown_core::HookAction::Command { command, timeout } => {
                            let timeout_str = timeout
                                .map(|t| format!(" (timeout: {}ms)", t))
                                .unwrap_or_default();
                            println!("      hook[{}]: command{}", j, timeout_str);
                            println!("        {}", command);
                        }
                        clown_core::HookAction::Url { url } => {
                            println!("      hook[{}]: url", j);
                            println!("        {}", url);
                        }
                    }
                }
            }
        }
    }

    if !has_hooks {
        println!("No hooks configured");
    }
}
