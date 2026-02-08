//! Command implementations.

mod init;

use crate::client::DaemonClient;
use crate::output;
use crate::{
    AgentsCommands, AliasesCommands, Commands, DaemonCommands, EnvCommands, HooksCommands,
    ProfilesCommands, ProvidersCommands, ProxyAliasCommands, ProxyCommands, ProxyRouteCommands,
    RegistryCommands, TerminalCommands, UsageCommands,
};
use anyhow::{anyhow, Result};
use ringlet_core::{HooksConfig, ModelTarget, ProfileCreateRequest, Request, Response, RoutingCondition, RoutingRule, UsagePeriod};

/// Execute a command.
pub async fn execute(command: &Commands, json: bool) -> Result<()> {
    match command {
        Commands::Init { skip_daemon, no_profile, yes } => {
            init::run_init(*skip_daemon, *no_profile, *yes, json).await
        }
        Commands::Agents { command } => execute_agents(command, json).await,
        Commands::Providers { command } => execute_providers(command, json).await,
        Commands::Profiles { command } => execute_profiles(command, json).await,
        Commands::Aliases { command } => execute_aliases(command, json).await,
        Commands::Registry { command } => execute_registry(command, json).await,
        Commands::Stats { agent, provider } => execute_stats(agent, provider, json).await,
        Commands::Usage { command, period, profile, model } => {
            execute_usage(command.as_ref(), period, profile.as_deref(), model.as_deref(), json).await
        }
        Commands::Daemon { command } => execute_daemon(command, json).await,
        Commands::Env { command } => execute_env(command, json).await,
        Commands::Hooks { command } => execute_hooks(command, json).await,
        Commands::Proxy { command } => execute_proxy(command, json).await,
        Commands::Terminal { command } => execute_terminal(command, json).await,
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
        ProfilesCommands::Run { alias, remote, cols, rows, no_sandbox, bwrap_flags, args } => {
            if *remote {
                // Run in remote mode - create a terminal session via HTTP API
                return execute_remote_run(alias, args, *cols, *rows, *no_sandbox, bwrap_flags.as_deref(), json).await;
            }

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

async fn execute_usage(
    command: Option<&UsageCommands>,
    period: &str,
    profile: Option<&str>,
    model: Option<&str>,
    json: bool,
) -> Result<()> {
    let client = DaemonClient::connect()?;

    // Parse period string to UsagePeriod
    let usage_period = parse_period(period);

    match command {
        Some(UsageCommands::Daily { period }) => {
            let response = client.request(&Request::Usage {
                period: Some(parse_period(period)),
                profile: None,
                model: None,
            })?;
            handle_usage_response(response, json)?;
        }
        Some(UsageCommands::Models) => {
            let response = client.request(&Request::Usage {
                period: Some(UsagePeriod::All),
                profile: None,
                model: None,
            })?;
            handle_usage_response(response, json)?;
        }
        Some(UsageCommands::Profiles) => {
            let response = client.request(&Request::Usage {
                period: Some(UsagePeriod::All),
                profile: None,
                model: None,
            })?;
            handle_usage_response(response, json)?;
        }
        Some(UsageCommands::Export { format, period }) => {
            let response = client.request(&Request::Usage {
                period: Some(parse_period(period)),
                profile: None,
                model: None,
            })?;
            match response {
                Response::Usage(usage) => {
                    // Always output as requested format
                    if format == "csv" {
                        println!("period,total_sessions,total_runtime_secs,input_tokens,output_tokens,cache_creation_tokens,cache_read_tokens,total_cost");
                        println!(
                            "{},{},{},{},{},{},{},{}",
                            usage.period,
                            usage.total_sessions,
                            usage.total_runtime_secs,
                            usage.total_tokens.input_tokens,
                            usage.total_tokens.output_tokens,
                            usage.total_tokens.cache_creation_input_tokens,
                            usage.total_tokens.cache_read_input_tokens,
                            usage.total_cost.as_ref().map(|c| c.total_cost).unwrap_or(0.0)
                        );
                    } else {
                        println!("{}", serde_json::to_string_pretty(&usage)?);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        Some(UsageCommands::ImportClaude { claude_dir }) => {
            let response = client.request(&Request::UsageImportClaude {
                claude_dir: claude_dir.clone(),
            })?;
            handle_success_response(response, json)?;
        }
        None => {
            // Default: show usage summary
            let response = client.request(&Request::Usage {
                period: Some(usage_period),
                profile: profile.map(|s| s.to_string()),
                model: model.map(|s| s.to_string()),
            })?;
            handle_usage_response(response, json)?;
        }
    }

    Ok(())
}

fn parse_period(period: &str) -> UsagePeriod {
    match period.to_lowercase().as_str() {
        "today" => UsagePeriod::Today,
        "yesterday" => UsagePeriod::Yesterday,
        "week" | "thisweek" | "this_week" => UsagePeriod::ThisWeek,
        "month" | "thismonth" | "this_month" => UsagePeriod::ThisMonth,
        "7d" | "7days" | "last7days" => UsagePeriod::Last7Days,
        "30d" | "30days" | "last30days" => UsagePeriod::Last30Days,
        "all" | "alltime" | "all_time" => UsagePeriod::All,
        _ => UsagePeriod::Today,
    }
}

fn handle_usage_response(response: Response, json: bool) -> Result<()> {
    match response {
        Response::Usage(usage) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&usage)?);
            } else {
                output::usage_summary(&usage);
            }
            Ok(())
        }
        Response::Error { message, .. } => Err(anyhow!(message)),
        _ => Err(anyhow!("Unexpected response")),
    }
}

async fn execute_daemon(command: &DaemonCommands, json: bool) -> Result<()> {
    match command {
        DaemonCommands::Start { stay_alive } => {
            // Start daemon in foreground by exec'ing ringletd
            let ringletd = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow!("Cannot find parent directory"))?
                .join("ringletd");

            let mut cmd = std::process::Command::new(&ringletd);
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
                        ringlet_core::HookAction::Command { command, timeout } => {
                            let timeout_str = timeout
                                .map(|t| format!(" (timeout: {}ms)", t))
                                .unwrap_or_default();
                            println!("      hook[{}]: command{}", j, timeout_str);
                            println!("        {}", command);
                        }
                        ringlet_core::HookAction::Url { url } => {
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

async fn execute_proxy(command: &ProxyCommands, json: bool) -> Result<()> {
    let client = DaemonClient::connect()?;

    match command {
        ProxyCommands::Enable { alias } => {
            let response = client.request(&Request::ProxyEnable {
                alias: alias.clone(),
            })?;
            handle_success_response(response, json)?;
        }
        ProxyCommands::Disable { alias } => {
            let response = client.request(&Request::ProxyDisable {
                alias: alias.clone(),
            })?;
            handle_success_response(response, json)?;
        }
        ProxyCommands::Start { alias } => {
            let response = client.request(&Request::ProxyStart {
                alias: alias.clone(),
            })?;
            handle_success_response(response, json)?;
        }
        ProxyCommands::Stop { alias } => {
            let response = client.request(&Request::ProxyStop {
                alias: alias.clone(),
            })?;
            handle_success_response(response, json)?;
        }
        ProxyCommands::StopAll => {
            let response = client.request(&Request::ProxyStopAll)?;
            handle_success_response(response, json)?;
        }
        ProxyCommands::Restart { alias } => {
            // Stop then start
            let _ = client.request(&Request::ProxyStop {
                alias: alias.clone(),
            });
            let response = client.request(&Request::ProxyStart {
                alias: alias.clone(),
            })?;
            handle_success_response(response, json)?;
        }
        ProxyCommands::Status { alias } => {
            let response = client.request(&Request::ProxyStatus {
                alias: alias.clone(),
            })?;
            match response {
                Response::ProxyStatus(instances) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&instances)?);
                    } else {
                        output::proxy_status(&instances);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProxyCommands::Config { alias } => {
            let response = client.request(&Request::ProxyConfig {
                alias: alias.clone(),
            })?;
            match response {
                Response::ProxyConfig(config) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&config)?);
                    } else {
                        output::proxy_config(&config);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProxyCommands::Logs { alias, lines } => {
            let response = client.request(&Request::ProxyLogs {
                alias: alias.clone(),
                lines: Some(*lines),
            })?;
            match response {
                Response::ProxyLogs(logs) => println!("{}", logs),
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProxyCommands::Route { command } => execute_proxy_route(command, &client, json)?,
        ProxyCommands::Alias { command } => execute_proxy_alias(command, &client, json)?,
    }

    Ok(())
}

fn execute_proxy_route(
    command: &ProxyRouteCommands,
    client: &DaemonClient,
    json: bool,
) -> Result<()> {
    match command {
        ProxyRouteCommands::Add {
            alias,
            name,
            condition,
            target,
            priority,
        } => {
            // Parse condition string
            let parsed_condition = RoutingCondition::parse(condition)
                .ok_or_else(|| anyhow!("Invalid condition: {}. Valid formats: always, thinking, tokens > N, tokens < N, tools >= N", condition))?;

            let rule = RoutingRule::new(name.clone(), parsed_condition, target.clone())
                .with_priority(*priority);

            let response = client.request(&Request::ProxyRouteAdd {
                alias: alias.clone(),
                rule,
            })?;
            handle_success_response(response, json)?;
        }
        ProxyRouteCommands::List { alias } => {
            let response = client.request(&Request::ProxyRouteList {
                alias: alias.clone(),
            })?;
            match response {
                Response::ProxyRoutes(rules) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&rules)?);
                    } else {
                        output::proxy_routes(&rules);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProxyRouteCommands::Remove { alias, name } => {
            let response = client.request(&Request::ProxyRouteRemove {
                alias: alias.clone(),
                rule_name: name.clone(),
            })?;
            handle_success_response(response, json)?;
        }
    }

    Ok(())
}

fn execute_proxy_alias(
    command: &ProxyAliasCommands,
    client: &DaemonClient,
    json: bool,
) -> Result<()> {
    match command {
        ProxyAliasCommands::Set { alias, from, to } => {
            let response = client.request(&Request::ProxyAliasSet {
                alias: alias.clone(),
                from_model: from.clone(),
                to_target: to.clone(),
            })?;
            handle_success_response(response, json)?;
        }
        ProxyAliasCommands::List { alias } => {
            let response = client.request(&Request::ProxyAliasList {
                alias: alias.clone(),
            })?;
            match response {
                Response::ProxyAliases(aliases) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&aliases)?);
                    } else {
                        output::proxy_aliases(&aliases);
                    }
                }
                Response::Error { message, .. } => return Err(anyhow!(message)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
        ProxyAliasCommands::Remove { alias, from } => {
            let response = client.request(&Request::ProxyAliasRemove {
                alias: alias.clone(),
                from_model: from.clone(),
            })?;
            handle_success_response(response, json)?;
        }
    }

    Ok(())
}

fn handle_success_response(response: Response, json: bool) -> Result<()> {
    match response {
        Response::Success { message } => {
            if json {
                println!("{}", serde_json::json!({"success": message}));
            } else {
                output::success(&message);
            }
            Ok(())
        }
        Response::Error { message, .. } => Err(anyhow!(message)),
        _ => Err(anyhow!("Unexpected response")),
    }
}

// Terminal API base URL (uses HTTP server port)
const TERMINAL_API_BASE: &str = "http://127.0.0.1:8765";

/// Execute remote run - creates a terminal session via HTTP API.
async fn execute_remote_run(
    alias: &str,
    args: &[String],
    cols: u16,
    rows: u16,
    no_sandbox: bool,
    bwrap_flags: Option<&str>,
    json: bool,
) -> Result<()> {
    let url = format!("{}/api/terminal/sessions", TERMINAL_API_BASE);

    let mut request_body = serde_json::json!({
        "profile_alias": alias,
        "args": args,
        "cols": cols,
        "rows": rows,
        "no_sandbox": no_sandbox,
    });

    // Add bwrap_flags if provided
    if let Some(flags) = bwrap_flags {
        let flags_vec: Vec<String> = flags.split(',').map(|s| s.trim().to_string()).collect();
        request_body["bwrap_flags"] = serde_json::json!(flags_vec);
    }

    let response: serde_json::Value = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_json(&request_body)
        .map_err(|e| anyhow!("Failed to create terminal session: {}", e))?
        .into_json()
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    if response["success"].as_bool() != Some(true) {
        if let Some(error) = response["error"]["message"].as_str() {
            return Err(anyhow!("{}", error));
        }
        return Err(anyhow!("Failed to create terminal session"));
    }

    let session_id = response["data"]["session_id"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing session_id in response"))?;

    if json {
        println!("{}", serde_json::json!({
            "session_id": session_id,
            "ws_url": format!("ws://127.0.0.1:8765/ws/terminal/{}", session_id),
            "web_url": format!("http://127.0.0.1:8765/terminal/{}", session_id),
        }));
    } else {
        println!("Terminal session created:");
        println!("  Session ID: {}", session_id);
        println!("  Web UI: http://127.0.0.1:8765/terminal/{}", session_id);
        println!("\nTo attach from CLI: ringlet terminal attach {}", session_id);
    }

    Ok(())
}

/// Execute terminal commands via HTTP API.
async fn execute_terminal(command: &TerminalCommands, json: bool) -> Result<()> {
    match command {
        TerminalCommands::List => {
            let url = format!("{}/api/terminal/sessions", TERMINAL_API_BASE);
            let response: serde_json::Value = ureq::get(&url)
                .call()
                .map_err(|e| anyhow!("Failed to list sessions: {}", e))?
                .into_json()
                .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

            if response["success"].as_bool() != Some(true) {
                if let Some(error) = response["error"]["message"].as_str() {
                    return Err(anyhow!("{}", error));
                }
                return Err(anyhow!("Failed to list sessions"));
            }

            let sessions = response["data"].as_array()
                .ok_or_else(|| anyhow!("Invalid response format"))?;

            if json {
                println!("{}", serde_json::to_string_pretty(sessions)?);
            } else if sessions.is_empty() {
                println!("No active terminal sessions");
            } else {
                println!("{:<36}  {:<15}  {:<10}  {}", "SESSION ID", "PROFILE", "STATE", "CLIENTS");
                println!("{}", "-".repeat(80));
                for session in sessions {
                    println!(
                        "{:<36}  {:<15}  {:<10}  {}",
                        session["id"].as_str().unwrap_or("-"),
                        session["profile_alias"].as_str().unwrap_or("-"),
                        session["state"].as_str().unwrap_or("-"),
                        session["client_count"].as_u64().unwrap_or(0),
                    );
                }
            }
        }
        TerminalCommands::Info { id } => {
            let url = format!("{}/api/terminal/sessions/{}", TERMINAL_API_BASE, id);
            let response: serde_json::Value = ureq::get(&url)
                .call()
                .map_err(|e| anyhow!("Failed to get session: {}", e))?
                .into_json()
                .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

            if response["success"].as_bool() != Some(true) {
                if let Some(error) = response["error"]["message"].as_str() {
                    return Err(anyhow!("{}", error));
                }
                return Err(anyhow!("Session not found"));
            }

            let session = &response["data"];
            if json {
                println!("{}", serde_json::to_string_pretty(session)?);
            } else {
                println!("Session ID: {}", session["id"].as_str().unwrap_or("-"));
                println!("Profile: {}", session["profile_alias"].as_str().unwrap_or("-"));
                println!("State: {}", session["state"].as_str().unwrap_or("-"));
                println!("PID: {}", session["pid"].as_u64().map(|p| p.to_string()).unwrap_or("-".to_string()));
                println!("Size: {}x{}", session["cols"].as_u64().unwrap_or(0), session["rows"].as_u64().unwrap_or(0));
                println!("Clients: {}", session["client_count"].as_u64().unwrap_or(0));
                println!("Created: {}", session["created_at"].as_str().unwrap_or("-"));
            }
        }
        TerminalCommands::Kill { id } => {
            let url = format!("{}/api/terminal/sessions/{}", TERMINAL_API_BASE, id);
            let response: serde_json::Value = ureq::delete(&url)
                .call()
                .map_err(|e| anyhow!("Failed to kill session: {}", e))?
                .into_json()
                .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

            if response["success"].as_bool() != Some(true) {
                if let Some(error) = response["error"]["message"].as_str() {
                    return Err(anyhow!("{}", error));
                }
                return Err(anyhow!("Failed to kill session"));
            }

            if json {
                println!("{}", serde_json::json!({"success": "Session terminated"}));
            } else {
                output::success(&format!("Session {} terminated", id));
            }
        }
        TerminalCommands::Attach { id } => {
            // For now, just print the URL - full terminal attach would require
            // more complex terminal handling (crossterm, raw mode, etc.)
            if json {
                println!("{}", serde_json::json!({
                    "session_id": id,
                    "ws_url": format!("ws://127.0.0.1:8765/ws/terminal/{}", id),
                    "web_url": format!("http://127.0.0.1:8765/terminal/{}", id),
                }));
            } else {
                println!("To view this terminal session, open the web UI:");
                println!("  http://127.0.0.1:8765/terminal/{}", id);
                println!("\nWebSocket URL (for custom clients):");
                println!("  ws://127.0.0.1:8765/ws/terminal/{}", id);
            }
        }
    }

    Ok(())
}
