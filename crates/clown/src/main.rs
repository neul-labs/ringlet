//! clown - CLI orchestrator for coding agents.
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

/// clown - CLI orchestrator for coding agents
#[derive(Parser, Debug)]
#[command(name = "clown", version, about, long_about = None)]
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

    /// View usage statistics
    Stats {
        /// Filter by agent ID
        #[arg(long)]
        agent: Option<String>,

        /// Filter by provider ID
        #[arg(long)]
        provider: Option<String>,
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
