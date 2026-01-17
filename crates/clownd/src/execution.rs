//! Execution adapter - runs agents with profile configuration.
//!
//! This module handles:
//! - Running Rhai scripts to generate config files
//! - Setting up profile HOME directory
//! - Writing generated config files
//! - Spawning agent processes with correct environment
//! - Tracking processes for telemetry

use anyhow::{anyhow, Context, Result};
use clown_core::{AgentManifest, ClownPaths, Profile, ProviderManifest};
use clown_scripting::{scripts, AgentContext, PrefsContext, ProfileContext, ProviderContext, ScriptContext, ScriptEngine, ScriptOutput};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use tracing::{debug, error, info};

/// Execution adapter for running agent profiles.
pub struct ExecutionAdapter {
    paths: ClownPaths,
}

/// Result of running a profile.
pub struct RunResult {
    /// Process ID of the spawned agent.
    pub pid: u32,
    /// Child process handle.
    pub child: Child,
}

impl ExecutionAdapter {
    /// Create a new execution adapter.
    pub fn new(paths: ClownPaths) -> Self {
        Self { paths }
    }

    /// Prepare and run a profile.
    pub fn run(
        &self,
        profile: &Profile,
        agent: &AgentManifest,
        provider: &ProviderManifest,
        api_key: &str,
        args: &[String],
    ) -> Result<RunResult> {
        // 1. Build script context
        let context = build_script_context(profile, agent, provider)?;

        // 2. Find and run the Rhai script
        let script_output = self.run_script(&agent.profile.script, &context)?;

        // 3. Write generated config files
        self.write_config_files(profile, &script_output)?;

        // 4. Build environment variables
        let env = self.build_environment(profile, provider, api_key, &script_output)?;

        // 5. Spawn the agent process
        self.spawn_agent(agent, profile, &env, &script_output.args, args)
    }

    /// Run the configuration script.
    fn run_script(&self, script_name: &str, context: &ScriptContext) -> Result<ScriptOutput> {
        // Try to find user override script first
        let user_script_path = self.paths.scripts_dir().join(script_name);
        let script = if user_script_path.exists() {
            debug!("Using user override script: {:?}", user_script_path);
            std::fs::read_to_string(&user_script_path)
                .context("Failed to read user script")?
        } else if let Some(builtin) = scripts::get(script_name) {
            debug!("Using built-in script: {}", script_name);
            builtin.to_string()
        } else {
            return Err(anyhow!("Script not found: {}", script_name));
        };

        // Create engine on-demand (not Send+Sync safe to store)
        let engine = ScriptEngine::new();
        engine.run(&script, context)
    }

    /// Write generated config files to profile home.
    fn write_config_files(&self, profile: &Profile, output: &ScriptOutput) -> Result<()> {
        let home = &profile.metadata.home;

        for (relative_path, content) in &output.files {
            let full_path = home.join(relative_path);

            // Create parent directories if needed
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .context(format!("Failed to create directory: {:?}", parent))?;
            }

            // Write file
            std::fs::write(&full_path, content)
                .context(format!("Failed to write file: {:?}", full_path))?;

            debug!("Wrote config file: {:?}", full_path);
        }

        Ok(())
    }

    /// Build environment variables for the agent process.
    fn build_environment(
        &self,
        profile: &Profile,
        provider: &ProviderManifest,
        api_key: &str,
        script_output: &ScriptOutput,
    ) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();

        // Start with profile's stored env vars (excluding internal ones)
        for (key, value) in &profile.env {
            if !key.starts_with("_CLOWN_") {
                env.insert(key.clone(), value.clone());
            }
        }

        // Override HOME to profile home
        env.insert(
            "HOME".to_string(),
            profile.metadata.home.to_string_lossy().to_string(),
        );

        // Add provider auth env var with actual API key (skip for self-auth)
        if !provider.auth.env_key.is_empty() {
            env.insert(provider.auth.env_key.clone(), api_key.to_string());
        }

        // Add script-generated env vars (replacing ${API_KEY} placeholder)
        for (key, value) in &script_output.env {
            let resolved = value.replace("${API_KEY}", api_key);
            env.insert(key.clone(), resolved);
        }

        Ok(env)
    }

    /// Spawn the agent process.
    fn spawn_agent(
        &self,
        agent: &AgentManifest,
        profile: &Profile,
        env: &HashMap<String, String>,
        script_args: &[String],
        user_args: &[String],
    ) -> Result<RunResult> {
        let working_dir = profile
            .working_dir
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        info!(
            "Spawning agent '{}' with profile '{}' in {:?}",
            agent.binary, profile.alias, working_dir
        );

        let mut cmd = Command::new(&agent.binary);
        cmd.current_dir(&working_dir);
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        // Clear environment and set only what we need
        cmd.env_clear();

        // Preserve essential system env vars
        for key in &["PATH", "TERM", "LANG", "LC_ALL", "USER", "SHELL"] {
            if let Ok(val) = std::env::var(key) {
                cmd.env(key, val);
            }
        }

        // Add profile env vars
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Add args: profile args, script-generated args, then user args
        cmd.args(&profile.args);
        cmd.args(script_args);
        cmd.args(user_args);

        debug!("Command: {:?}", cmd);

        let child = cmd.spawn().context(format!("Failed to spawn: {}", agent.binary))?;

        let pid = child.id();
        info!("Agent started with PID {}", pid);

        Ok(RunResult { pid, child })
    }
}

/// Build script context from profile, agent, and provider.
fn build_script_context(
    profile: &Profile,
    agent: &AgentManifest,
    provider: &ProviderManifest,
) -> Result<ScriptContext> {
    // Resolve endpoint URL - handle indirection (e.g., "default" -> "international" -> URL)
    let endpoint_id = &profile.endpoint_id;
    let mut endpoint = provider
        .endpoints
        .get(endpoint_id)
        .or_else(|| provider.default_endpoint().and_then(|e| provider.endpoints.get(e)))
        .ok_or_else(|| anyhow!("Endpoint not found: {}", endpoint_id))?
        .clone();

    // If the endpoint value is itself a key in endpoints (indirection), resolve it
    if provider.endpoints.contains_key(&endpoint) {
        endpoint = provider.endpoints.get(&endpoint).unwrap().clone();
    }

    Ok(ScriptContext {
        profile: ProfileContext {
            alias: profile.alias.clone(),
            home: profile.metadata.home.clone(),
            model: profile.model.clone(),
            endpoint,
            hooks: profile.metadata.enabled_hooks.clone(),
            mcp_servers: profile.metadata.enabled_mcp_servers.clone(),
        },
        provider: ProviderContext {
            id: provider.id.clone(),
            name: provider.name.clone(),
            provider_type: provider.provider_type.to_string(),
            auth_env_key: provider.auth.env_key.clone(),
        },
        agent: AgentContext {
            id: agent.id.clone(),
            name: agent.name.clone(),
            binary: agent.binary.clone(),
        },
        prefs: PrefsContext::default(),
    })
}
