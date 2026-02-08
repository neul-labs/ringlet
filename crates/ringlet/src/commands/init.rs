//! Interactive onboarding wizard for ringlet.

use crate::client::DaemonClient;
use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use ringlet_core::{AgentInfo, ProfileCreateRequest, ProviderInfo, Request, Response};

/// Run the interactive init wizard.
pub async fn run_init(skip_daemon: bool, no_profile: bool, auto_yes: bool, json: bool) -> Result<()> {
    let theme = ColorfulTheme::default();

    if !json {
        println!();
        println!("Welcome to Ringlet!");
        println!("This wizard will help you get started with managing coding agents.");
        println!();
    }

    // Step 1: Check/ensure daemon connectivity (unless skipped)
    let client = if skip_daemon {
        if !json {
            println!("Skipping daemon check (--skip-daemon)");
            println!();
        }
        None
    } else {
        if !json {
            print!("Checking daemon connection... ");
        }
        match DaemonClient::connect() {
            Ok(c) => {
                if !json {
                    println!("connected.");
                    println!();
                }
                Some(c)
            }
            Err(e) => {
                if !json {
                    println!("failed.");
                    println!("  Error: {}", e);
                    println!("  Try running 'ringlet daemon start' first.");
                }
                return Err(anyhow!("Could not connect to daemon: {}", e));
            }
        }
    };

    // Step 2: Detect installed agents
    let agents = if let Some(ref c) = client {
        if !json {
            println!("Detecting installed agents...");
        }
        fetch_agents(c)?
    } else {
        if !json {
            println!("Skipping agent detection (no daemon connection)");
        }
        vec![]
    };

    let installed: Vec<_> = agents.iter().filter(|a| a.installed).collect();
    let not_installed: Vec<_> = agents.iter().filter(|a| !a.installed).collect();

    if !json && !agents.is_empty() {
        println!();
        if !installed.is_empty() {
            println!("Installed agents:");
            for agent in &installed {
                let version = agent.version.as_deref().unwrap_or("unknown version");
                println!("  [x] {} ({})", agent.name, version);
            }
        }

        if !not_installed.is_empty() {
            println!();
            println!("Not installed:");
            for agent in &not_installed {
                println!("  [ ] {}", agent.name);
            }
        }
        println!();
    }

    if installed.is_empty() && !skip_daemon {
        if !json {
            println!("No coding agents detected. Please install at least one agent first.");
            println!();
            println!("Supported agents:");
            println!("  - Claude Code: npm install -g @anthropic-ai/claude-code");
            println!("  - Grok: pip install grok-cli");
            println!();
        }
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "message": "No coding agents detected",
                    "agents": agents
                })
            );
        }
        return Ok(());
    }

    // Step 3: Show available providers
    let providers = if let Some(ref c) = client {
        fetch_providers(c)?
    } else {
        vec![]
    };

    if !json && !providers.is_empty() {
        println!("Available providers:");
        for provider in &providers {
            let auth_note = if provider.auth_required {
                " (requires API key)"
            } else {
                ""
            };
            println!("  - {}: {}{}", provider.id, provider.name, auth_note);
        }
        println!();
    }

    // Step 4: Optionally create first profile
    if !no_profile && !installed.is_empty() && client.is_some() {
        let create_profile = if auto_yes {
            true
        } else if json {
            false
        } else {
            Confirm::with_theme(&theme)
                .with_prompt("Would you like to create your first profile?")
                .default(true)
                .interact()?
        };

        if create_profile {
            create_first_profile(client.as_ref().unwrap(), &installed, &providers, &theme, json)
                .await?;
        }
    }

    // Step 5: Show next steps
    if !json {
        println!();
        println!("{}", "=".repeat(50));
        println!("Setup complete!");
        println!();
        println!("Next steps:");
        println!("  ringlet profiles list        View your profiles");
        println!("  ringlet profiles run <alias> Run an agent session");
        println!("  ringlet --help               See all available commands");
        println!();
    } else {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "agents_installed": installed.len(),
                "agents_available": agents.len(),
                "providers_available": providers.len()
            })
        );
    }

    Ok(())
}

/// Fetch agents from daemon.
fn fetch_agents(client: &DaemonClient) -> Result<Vec<AgentInfo>> {
    let response = client.request(&Request::AgentsList)?;
    match response {
        Response::Agents(agents) => Ok(agents),
        Response::Error { message, .. } => Err(anyhow!("Failed to list agents: {}", message)),
        _ => Err(anyhow!("Unexpected response from daemon")),
    }
}

/// Fetch providers from daemon.
fn fetch_providers(client: &DaemonClient) -> Result<Vec<ProviderInfo>> {
    let response = client.request(&Request::ProvidersList)?;
    match response {
        Response::Providers(providers) => Ok(providers),
        Response::Error { message, .. } => Err(anyhow!("Failed to list providers: {}", message)),
        _ => Err(anyhow!("Unexpected response from daemon")),
    }
}

/// Create the user's first profile interactively.
async fn create_first_profile(
    client: &DaemonClient,
    agents: &[&AgentInfo],
    providers: &[ProviderInfo],
    theme: &ColorfulTheme,
    json: bool,
) -> Result<()> {
    if !json {
        println!();
        println!("--- Create Your First Profile ---");
        println!();
    }

    // Select agent
    let agent_names: Vec<String> = agents
        .iter()
        .map(|a| {
            let version = a.version.as_deref().unwrap_or("");
            if version.is_empty() {
                a.name.clone()
            } else {
                format!("{} ({})", a.name, version)
            }
        })
        .collect();

    let agent_idx = Select::with_theme(theme)
        .with_prompt("Select an agent")
        .items(&agent_names)
        .default(0)
        .interact()?;
    let selected_agent = agents[agent_idx];

    // Select provider
    let provider_names: Vec<String> = providers
        .iter()
        .map(|p| {
            let auth = if p.auth_required {
                " (API key required)"
            } else {
                ""
            };
            format!("{} - {}{}", p.id, p.name, auth)
        })
        .collect();

    // Default to "self" provider if available, otherwise first
    let default_provider_idx = providers
        .iter()
        .position(|p| p.id == "self")
        .unwrap_or(0);

    let provider_idx = Select::with_theme(theme)
        .with_prompt("Select a provider")
        .items(&provider_names)
        .default(default_provider_idx)
        .interact()?;
    let selected_provider = &providers[provider_idx];

    // Get profile alias
    let default_alias = format!("{}-default", selected_agent.id);
    let alias: String = Input::with_theme(theme)
        .with_prompt("Profile alias")
        .default(default_alias)
        .interact_text()?;

    // Get API key if required
    let api_key = if selected_provider.auth_required {
        let prompt = if selected_provider.auth_prompt.is_empty() {
            format!("Enter {} API key", selected_provider.name)
        } else {
            selected_provider.auth_prompt.clone()
        };

        Password::with_theme(theme)
            .with_prompt(&prompt)
            .interact()?
    } else {
        String::new()
    };

    // Create the profile
    let request = ProfileCreateRequest {
        agent_id: selected_agent.id.clone(),
        alias: alias.clone(),
        provider_id: selected_provider.id.clone(),
        endpoint_id: None,
        model: None,
        api_key,
        hooks: vec![],
        mcp_servers: vec![],
        args: vec![],
        working_dir: None,
        bare: false,
        proxy: false,
    };

    let response = client.request(&Request::ProfilesCreate(request))?;
    match response {
        Response::Success { message } => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "profile_created": true,
                        "alias": alias,
                        "message": message
                    })
                );
            } else {
                println!();
                println!("Profile '{}' created successfully!", alias);
                println!();
                println!("Run it with: ringlet profiles run {}", alias);
            }
        }
        Response::Error { message, .. } => {
            return Err(anyhow!("Failed to create profile: {}", message));
        }
        _ => return Err(anyhow!("Unexpected response")),
    }

    Ok(())
}
