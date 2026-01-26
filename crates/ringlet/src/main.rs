//! ringlet - CLI orchestrator for coding agents.
//!
//! A thin client that auto-starts the daemon and forwards commands over IPC.

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod client;
mod commands;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

/// ringlet - CLI orchestrator for coding agents
#[derive(Parser, Debug)]
#[command(name = "ringlet", version, about, long_about = None)]
struct Cli {
    /// Output as JSON instead of tables
    #[arg(long, global = true)]
    json: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value = "warn")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage agents
    Agents {
        #[command(subcommand)]
        command: AgentsCommands,
    },

    /// Manage providers
    Providers {
        #[command(subcommand)]
        command: ProvidersCommands,
    },

    /// Manage profiles
    Profiles {
        #[command(subcommand)]
        command: ProfilesCommands,
    },

    /// Manage aliases
    Aliases {
        #[command(subcommand)]
        command: AliasesCommands,
    },

    /// Manage registry
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },

    /// View usage statistics (legacy)
    Stats {
        /// Filter by agent ID
        #[arg(long)]
        agent: Option<String>,

        /// Filter by provider ID
        #[arg(long)]
        provider: Option<String>,
    },

    /// View token/cost usage
    Usage {
        #[command(subcommand)]
        command: Option<UsageCommands>,

        /// Time period (today, yesterday, week, month, 7d, 30d, all)
        #[arg(long, short, default_value = "today")]
        period: String,

        /// Filter by profile
        #[arg(long)]
        profile: Option<String>,

        /// Filter by model
        #[arg(long)]
        model: Option<String>,
    },

    /// Daemon management
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },

    /// Run environment setup tasks
    Env {
        #[command(subcommand)]
        command: EnvCommands,
    },

    /// Manage profile hooks
    Hooks {
        #[command(subcommand)]
        command: HooksCommands,
    },

    /// Manage proxy routing
    Proxy {
        #[command(subcommand)]
        command: ProxyCommands,
    },

    /// Manage remote terminal sessions
    Terminal {
        #[command(subcommand)]
        command: TerminalCommands,
    },
}

#[derive(Subcommand, Debug)]
enum AgentsCommands {
    /// List all agents
    List,
    /// Inspect an agent
    Inspect {
        /// Agent ID
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum ProvidersCommands {
    /// List all providers
    List,
    /// Inspect a provider
    Inspect {
        /// Provider ID
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum ProfilesCommands {
    /// Create a new profile
    Create {
        /// Agent ID
        agent: String,
        /// Profile alias
        alias: String,
        /// Provider ID
        #[arg(long, short)]
        provider: String,
        /// Model (uses provider/agent default if not specified)
        #[arg(long, short)]
        model: Option<String>,
        /// Endpoint ID (uses provider default if not specified)
        #[arg(long, short)]
        endpoint: Option<String>,
        /// API key (will prompt if not provided)
        #[arg(long)]
        api_key: Option<String>,
        /// Enable hooks (comma-separated)
        #[arg(long)]
        hooks: Option<String>,
        /// Enable MCP servers (comma-separated)
        #[arg(long)]
        mcp: Option<String>,
        /// Create minimal profile without hooks/MCP
        #[arg(long)]
        bare: bool,
        /// Enable proxy routing for this profile
        #[arg(long)]
        proxy: bool,
    },
    /// List profiles
    List {
        /// Filter by agent ID
        #[arg(long)]
        agent: Option<String>,
    },
    /// Inspect a profile
    Inspect {
        /// Profile alias
        alias: String,
    },
    /// Run an agent with a profile
    Run {
        /// Profile alias
        alias: String,
        /// Run in remote mode (PTY session viewable via web UI)
        #[arg(long)]
        remote: bool,
        /// Initial terminal columns (for remote mode)
        #[arg(long, default_value = "80")]
        cols: u16,
        /// Initial terminal rows (for remote mode)
        #[arg(long, default_value = "24")]
        rows: u16,
        /// Arguments to pass to the agent
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Delete a profile
    Delete {
        /// Profile alias
        alias: String,
    },
    /// Export environment variables for shell
    Env {
        /// Profile alias
        alias: String,
    },
}

#[derive(Subcommand, Debug)]
enum AliasesCommands {
    /// Install alias shim
    Install {
        /// Profile alias
        alias: String,
        /// Target bin directory
        #[arg(long)]
        bin_dir: Option<std::path::PathBuf>,
    },
    /// Uninstall alias shim
    Uninstall {
        /// Profile alias
        alias: String,
    },
}

#[derive(Subcommand, Debug)]
enum RegistryCommands {
    /// Sync registry from GitHub
    Sync {
        /// Force sync even if cache is fresh
        #[arg(long)]
        force: bool,
        /// Use cached data only
        #[arg(long)]
        offline: bool,
    },
    /// Pin to a specific commit/tag
    Pin {
        /// Git ref to pin
        #[arg(name = "ref")]
        ref_: String,
    },
    /// Inspect registry status
    Inspect,
}

#[derive(Subcommand, Debug)]
enum DaemonCommands {
    /// Start daemon in foreground
    Start {
        /// Keep running indefinitely
        #[arg(long)]
        stay_alive: bool,
    },
    /// Stop the daemon
    Stop,
    /// Check daemon status
    Status,
}

#[derive(Subcommand, Debug)]
enum EnvCommands {
    /// Run a setup task
    Setup {
        /// Profile alias
        alias: String,
        /// Task name
        task: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum UsageCommands {
    /// Show daily usage breakdown
    Daily {
        /// Time period
        #[arg(long, short, default_value = "week")]
        period: String,
    },
    /// Show usage by model
    Models,
    /// Show usage by profile
    Profiles,
    /// Export usage data
    Export {
        /// Output format (json, csv)
        #[arg(long, short, default_value = "json")]
        format: String,
        /// Time period
        #[arg(long, short, default_value = "all")]
        period: String,
    },
    /// Import usage from Claude's native files
    ImportClaude {
        /// Path to Claude home directory (default: ~/.claude)
        #[arg(long)]
        claude_dir: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum HooksCommands {
    /// Add a hook rule to a profile
    Add {
        /// Profile alias
        alias: String,
        /// Event type (PreToolUse, PostToolUse, Notification, Stop)
        event: String,
        /// Matcher pattern (e.g., "Bash|Write" or "*" for all)
        matcher: String,
        /// Command to execute (use $EVENT for JSON event data)
        command: String,
    },
    /// List hooks for a profile
    List {
        /// Profile alias
        alias: String,
    },
    /// Remove a hook rule from a profile
    Remove {
        /// Profile alias
        alias: String,
        /// Event type (PreToolUse, PostToolUse, Notification, Stop)
        event: String,
        /// Rule index (0-based, as shown in list)
        index: usize,
    },
    /// Import hooks from a JSON file
    Import {
        /// Profile alias
        alias: String,
        /// Path to JSON file with hooks configuration
        file: std::path::PathBuf,
    },
    /// Export hooks to JSON
    Export {
        /// Profile alias
        alias: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProxyCommands {
    /// Enable proxy for a profile
    Enable {
        /// Profile alias
        alias: String,
    },
    /// Disable proxy for a profile
    Disable {
        /// Profile alias
        alias: String,
    },
    /// Start proxy instance
    Start {
        /// Profile alias
        alias: String,
    },
    /// Stop proxy instance
    Stop {
        /// Profile alias
        alias: String,
    },
    /// Stop all proxy instances
    StopAll,
    /// Restart proxy instance
    Restart {
        /// Profile alias
        alias: String,
    },
    /// Show proxy status
    Status {
        /// Profile alias (shows all if not specified)
        alias: Option<String>,
    },
    /// Show proxy configuration
    Config {
        /// Profile alias
        alias: String,
    },
    /// View proxy logs
    Logs {
        /// Profile alias
        alias: String,
        /// Number of lines to show
        #[arg(long, short, default_value = "50")]
        lines: usize,
    },
    /// Manage routing rules
    Route {
        #[command(subcommand)]
        command: ProxyRouteCommands,
    },
    /// Manage model aliases
    Alias {
        #[command(subcommand)]
        command: ProxyAliasCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProxyRouteCommands {
    /// Add a routing rule
    Add {
        /// Profile alias
        alias: String,
        /// Rule name
        name: String,
        /// Condition (always, tokens>N, thinking, tools>=N)
        condition: String,
        /// Target model (provider/model)
        target: String,
        /// Priority (higher = evaluated first)
        #[arg(long, default_value = "0")]
        priority: i32,
    },
    /// List routing rules
    List {
        /// Profile alias
        alias: String,
    },
    /// Remove a routing rule
    Remove {
        /// Profile alias
        alias: String,
        /// Rule name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProxyAliasCommands {
    /// Set a model alias
    Set {
        /// Profile alias
        alias: String,
        /// Source model name
        from: String,
        /// Target (provider/model)
        to: String,
    },
    /// List model aliases
    List {
        /// Profile alias
        alias: String,
    },
    /// Remove a model alias
    Remove {
        /// Profile alias
        alias: String,
        /// Source model name to remove
        from: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TerminalCommands {
    /// List active terminal sessions
    List,
    /// Show session info
    Info {
        /// Session ID
        id: String,
    },
    /// Terminate a session
    Kill {
        /// Session ID
        id: String,
    },
    /// Attach to a session (opens web UI)
    Attach {
        /// Session ID
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cli.log_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Execute command
    let result = commands::execute(&cli.command, cli.json).await;

    if let Err(e) = &result {
        if cli.json {
            let error = serde_json::json!({
                "error": e.to_string()
            });
            println!("{}", serde_json::to_string_pretty(&error)?);
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }

    Ok(())
}
