//! Output formatting for CLI.

use clown_core::agent::AgentInfo;
use clown_core::profile::ProfileInfo;
use clown_core::provider::ProviderInfo;
use clown_core::proxy::{ProfileProxyConfig, ProxyInstanceInfo, ProxyStatus, RoutingCondition, RoutingRule};
use comfy_table::{Cell, Color, Table};
use std::collections::HashMap;

/// Format agents as a table.
pub fn agents_table(agents: &[AgentInfo]) -> Table {
    let mut table = Table::new();
    table.set_header(vec!["Agent", "Version", "Profiles", "Default Model"]);

    for agent in agents {
        let version = agent.version.clone().unwrap_or_else(|| {
            if agent.installed {
                "unknown".to_string()
            } else {
                "not installed".to_string()
            }
        });

        let version_cell = if agent.installed {
            Cell::new(&version)
        } else {
            Cell::new(&version).fg(Color::DarkGrey)
        };

        table.add_row(vec![
            Cell::new(&agent.id),
            version_cell,
            Cell::new(agent.profile_count),
            Cell::new(agent.default_model.as_deref().unwrap_or("-")),
        ]);
    }

    table
}

/// Format a single agent.
pub fn agent_detail(agent: &AgentInfo) -> String {
    let mut lines = vec![
        format!("ID: {}", agent.id),
        format!("Name: {}", agent.name),
        format!("Installed: {}", agent.installed),
    ];

    if let Some(ref version) = agent.version {
        lines.push(format!("Version: {}", version));
    }

    if let Some(ref path) = agent.binary_path {
        lines.push(format!("Binary: {}", path));
    }

    lines.push(format!("Profiles: {}", agent.profile_count));

    if let Some(ref model) = agent.default_model {
        lines.push(format!("Default Model: {}", model));
    }

    if let Some(ref last_used) = agent.last_used {
        lines.push(format!("Last Used: {}", last_used));
    }

    lines.join("\n")
}

/// Format providers as a table.
pub fn providers_table(providers: &[ProviderInfo]) -> Table {
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Default Model"]);

    for provider in providers {
        table.add_row(vec![
            Cell::new(&provider.id),
            Cell::new(&provider.name),
            Cell::new(provider.provider_type.to_string()),
            Cell::new(provider.default_model.as_deref().unwrap_or("-")),
        ]);
    }

    table
}

/// Format a single provider.
pub fn provider_detail(provider: &ProviderInfo) -> String {
    let mut lines = vec![
        format!("ID: {}", provider.id),
        format!("Name: {}", provider.name),
        format!("Type: {}", provider.provider_type),
    ];

    lines.push("Endpoints:".to_string());
    for endpoint in &provider.endpoints {
        let default_marker = if endpoint.is_default { " (default)" } else { "" };
        lines.push(format!("  {}: {}{}", endpoint.id, endpoint.url, default_marker));
    }

    if let Some(ref model) = provider.default_model {
        lines.push(format!("Default Model: {}", model));
    }

    lines.join("\n")
}

/// Format profiles as a table.
pub fn profiles_table(profiles: &[ProfileInfo]) -> Table {
    let mut table = Table::new();
    table.set_header(vec!["Alias", "Provider", "Endpoint", "Model", "Last Used"]);

    for profile in profiles {
        let last_used = profile
            .last_used
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "never".to_string());

        table.add_row(vec![
            Cell::new(&profile.alias),
            Cell::new(&profile.provider_id),
            Cell::new(&profile.endpoint_id),
            Cell::new(&profile.model),
            Cell::new(&last_used),
        ]);
    }

    table
}

/// Format a single profile.
pub fn profile_detail(profile: &ProfileInfo) -> String {
    let mut lines = vec![
        format!("Alias: {}", profile.alias),
        format!("Agent: {}", profile.agent_id),
        format!("Provider: {}", profile.provider_id),
        format!("Endpoint: {}", profile.endpoint_id),
        format!("Model: {}", profile.model),
        format!("Total Runs: {}", profile.total_runs),
    ];

    if let Some(ref last_used) = profile.last_used {
        lines.push(format!("Last Used: {}", last_used));
    }

    lines.join("\n")
}

/// Format environment variables for shell export.
pub fn env_export(env: &std::collections::HashMap<String, String>) -> String {
    env.iter()
        .map(|(k, v)| format!("export {}=\"{}\"", k, v.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Print success message.
pub fn success(message: &str) {
    println!("{}", message);
}

/// Print error message.
pub fn error(message: &str) {
    eprintln!("Error: {}", message);
}

/// Format proxy status as a table.
pub fn proxy_status(instances: &[ProxyInstanceInfo]) {
    if instances.is_empty() {
        println!("No proxy instances running");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec!["Profile", "Port", "PID", "Status", "Restarts", "Started"]);

    for instance in instances {
        let status_str = match &instance.status {
            ProxyStatus::Starting => "starting".to_string(),
            ProxyStatus::Running => "running".to_string(),
            ProxyStatus::Unhealthy { reason, .. } => format!("unhealthy: {}", reason),
            ProxyStatus::Stopping => "stopping".to_string(),
            ProxyStatus::Stopped => "stopped".to_string(),
            ProxyStatus::Failed { reason } => format!("failed: {}", reason),
        };

        let status_cell = match &instance.status {
            ProxyStatus::Running => Cell::new(&status_str).fg(Color::Green),
            ProxyStatus::Unhealthy { .. } | ProxyStatus::Failed { .. } => {
                Cell::new(&status_str).fg(Color::Red)
            }
            _ => Cell::new(&status_str).fg(Color::Yellow),
        };

        let started = instance.started_at.format("%Y-%m-%d %H:%M").to_string();

        table.add_row(vec![
            Cell::new(&instance.alias),
            Cell::new(instance.port),
            Cell::new(instance.pid),
            status_cell,
            Cell::new(instance.restart_count),
            Cell::new(&started),
        ]);
    }

    println!("{}", table);
}

/// Format proxy configuration.
pub fn proxy_config(config: &ProfileProxyConfig) {
    println!("Enabled: {}", config.enabled);
    println!(
        "Port: {}",
        config
            .port
            .map_or("auto".to_string(), |p| p.to_string())
    );
    println!("Strategy: {:?}", config.routing.strategy);

    if config.routing.rules.is_empty() {
        println!("Rules: (none)");
    } else {
        println!("Rules:");
        for rule in &config.routing.rules {
            let condition_str = format_condition(&rule.condition);
            println!(
                "  [{}] {} -> {} (priority: {})",
                rule.name, condition_str, rule.target, rule.priority
            );
        }
    }

    if config.model_aliases.is_empty() {
        println!("Aliases: (none)");
    } else {
        println!("Aliases:");
        for (from, target) in &config.model_aliases {
            println!("  {} -> {}/{}", from, target.provider, target.model);
        }
    }
}

/// Format routing rules as a table.
pub fn proxy_routes(rules: &[RoutingRule]) {
    if rules.is_empty() {
        println!("No routing rules configured");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec!["Name", "Condition", "Target", "Priority"]);

    for rule in rules {
        let condition_str = format_condition(&rule.condition);
        table.add_row(vec![
            Cell::new(&rule.name),
            Cell::new(&condition_str),
            Cell::new(&rule.target),
            Cell::new(rule.priority),
        ]);
    }

    println!("{}", table);
}

/// Format model aliases as a table.
pub fn proxy_aliases(aliases: &HashMap<String, String>) {
    if aliases.is_empty() {
        println!("No model aliases configured");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec!["From", "To"]);

    for (from, to) in aliases {
        table.add_row(vec![Cell::new(from), Cell::new(to)]);
    }

    println!("{}", table);
}

/// Format a routing condition as a human-readable string.
fn format_condition(condition: &RoutingCondition) -> String {
    match condition {
        RoutingCondition::Always => "always".to_string(),
        RoutingCondition::ThinkingMode => "thinking".to_string(),
        RoutingCondition::TokenCount { min, max } => {
            match (min, max) {
                (Some(min), None) => format!("tokens > {}", min),
                (None, Some(max)) => format!("tokens < {}", max),
                (Some(min), Some(max)) => format!("tokens {} - {}", min, max),
                (None, None) => "tokens: any".to_string(),
            }
        }
        RoutingCondition::HasTools { min_count } => {
            match min_count {
                Some(n) => format!("tools >= {}", n),
                None => "has tools".to_string(),
            }
        }
        RoutingCondition::ModelPattern { pattern } => format!("model ~ {}", pattern),
        RoutingCondition::All { conditions } => {
            let parts: Vec<_> = conditions.iter().map(format_condition).collect();
            format!("all({})", parts.join(", "))
        }
        RoutingCondition::Any { conditions } => {
            let parts: Vec<_> = conditions.iter().map(format_condition).collect();
            format!("any({})", parts.join(", "))
        }
    }
}
