//! Execution services for profile runs.
//!
//! `ExecutionAdapter` remains the handler-facing entrypoint, but the work is
//! split between a planner that renders config/env and a launcher that spawns
//! the final process from a prepared execution context.

use anyhow::{Context, Result, anyhow};
use ringlet_core::rpc::ExecutionContext;
use ringlet_core::{AgentManifest, Profile, ProviderManifest, RingletPaths};
use ringlet_scripting::{
    AgentContext, PrefsContext, ProfileContext, ProviderContext, ScriptContext, ScriptEngine,
    ScriptOutput, scripts,
};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use tracing::{debug, info};

use crate::daemon::registry_client::RegistryLock;

/// Execution adapter for running agent profiles.
pub struct ExecutionAdapter {
    planner: ExecutionPlanner,
    launcher: ProcessLauncher,
}

/// Result of running a profile.
pub struct RunResult {
    /// Process ID of the spawned agent.
    pub pid: u32,
    /// Child process handle.
    pub child: Child,
}

/// Builds an execution context from profile, agent, and provider inputs.
struct ExecutionPlanner {
    renderer: ConfigRenderer,
}

/// Renders script-driven config files and environment variables.
struct ConfigRenderer {
    paths: RingletPaths,
}

/// Launches processes from prepared execution contexts.
struct ProcessLauncher;

struct RenderedExecution {
    env: HashMap<String, String>,
    script_output: ScriptOutput,
}

impl ExecutionAdapter {
    /// Create a new execution adapter.
    pub fn new(paths: RingletPaths) -> Self {
        Self {
            planner: ExecutionPlanner::new(paths),
            launcher: ProcessLauncher,
        }
    }

    /// Prepare execution context for CLI-side spawning.
    /// Does everything run() does except actually spawning the process.
    pub fn prepare(
        &self,
        profile: &Profile,
        agent: &AgentManifest,
        provider: &ProviderManifest,
        api_key: &str,
        args: &[String],
        proxy_url: Option<&str>,
    ) -> Result<ExecutionContext> {
        self.planner
            .prepare(profile, agent, provider, api_key, args, proxy_url)
    }

    /// Spawn a process from a prepared execution context.
    pub fn spawn_prepared(&self, context: &ExecutionContext) -> Result<RunResult> {
        self.launcher.spawn_prepared(context)
    }
}

impl ExecutionPlanner {
    fn new(paths: RingletPaths) -> Self {
        Self {
            renderer: ConfigRenderer::new(paths),
        }
    }

    fn prepare(
        &self,
        profile: &Profile,
        agent: &AgentManifest,
        provider: &ProviderManifest,
        api_key: &str,
        args: &[String],
        proxy_url: Option<&str>,
    ) -> Result<ExecutionContext> {
        let rendered = self
            .renderer
            .render(profile, agent, provider, api_key, proxy_url)?;

        let mut env = rendered.env;
        for key in &["PATH", "TERM", "LANG", "LC_ALL", "USER", "SHELL"] {
            if let Ok(val) = std::env::var(key) {
                env.insert(key.to_string(), val);
            }
        }

        let mut combined_args = Vec::new();
        combined_args.extend(profile.args.clone());
        combined_args.extend(rendered.script_output.args);
        combined_args.extend(args.to_vec());

        let working_dir = profile
            .working_dir
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        Ok(ExecutionContext {
            binary: agent.binary.clone(),
            working_dir,
            env,
            args: combined_args,
            alias: profile.alias.clone(),
            run_id: None,
        })
    }
}

impl ConfigRenderer {
    fn new(paths: RingletPaths) -> Self {
        Self { paths }
    }

    fn render(
        &self,
        profile: &Profile,
        agent: &AgentManifest,
        provider: &ProviderManifest,
        api_key: &str,
        proxy_url: Option<&str>,
    ) -> Result<RenderedExecution> {
        let context = build_script_context(profile, agent, provider, proxy_url)?;
        let script_output = self.run_script(&agent.profile.script, &context)?;
        self.write_config_files(profile, &script_output, api_key)?;
        let env = self.build_environment(profile, api_key, &script_output);

        Ok(RenderedExecution { env, script_output })
    }

    /// Run the configuration script.
    fn run_script(&self, script_name: &str, context: &ScriptContext) -> Result<ScriptOutput> {
        let user_script_path = self.paths.scripts_dir().join(script_name);
        let script = if user_script_path.exists() {
            debug!("Using user override script: {:?}", user_script_path);
            std::fs::read_to_string(&user_script_path).context("Failed to read user script")?
        } else if let Some(registry_script) = self.load_registry_script(script_name)? {
            debug!("Using registry script: {}", script_name);
            registry_script
        } else if let Some(builtin) = scripts::get(script_name) {
            debug!("Using built-in script: {}", script_name);
            builtin.to_string()
        } else {
            return Err(anyhow!("Script not found: {}", script_name));
        };

        let engine = ScriptEngine::new();
        engine.run(&script, context)
    }

    fn load_registry_lock(&self) -> Result<RegistryLock> {
        let lock_path = self.paths.registry_lock();
        if lock_path.exists() {
            let content = std::fs::read_to_string(&lock_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(RegistryLock::default())
        }
    }

    fn load_registry_script(&self, script_name: &str) -> Result<Option<String>> {
        let lock = self.load_registry_lock()?;
        let commit = lock.commit.as_deref().unwrap_or("latest");
        let script_path = self
            .paths
            .registry_commits_dir()
            .join(commit)
            .join("scripts")
            .join(script_name);

        if script_path.exists() {
            Ok(Some(std::fs::read_to_string(&script_path)?))
        } else {
            Ok(None)
        }
    }

    fn write_config_files(
        &self,
        profile: &Profile,
        output: &ScriptOutput,
        api_key: &str,
    ) -> Result<()> {
        let home = &profile.metadata.home;

        for (relative_path, content) in &output.files {
            let full_path = home.join(relative_path);

            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .context(format!("Failed to create directory: {:?}", parent))?;
            }

            let resolved_content = content.replace("${API_KEY}", api_key);
            let contains_sensitive_data = content.contains("${API_KEY}") && !api_key.is_empty();

            std::fs::write(&full_path, &resolved_content)
                .context(format!("Failed to write file: {:?}", full_path))?;

            #[cfg(unix)]
            if contains_sensitive_data {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&full_path, std::fs::Permissions::from_mode(0o600))
                    .context(format!("Failed to set permissions on: {:?}", full_path))?;
                debug!("Set 0o600 permissions on sensitive file: {:?}", full_path);
            }

            debug!("Wrote config file: {:?}", full_path);
        }

        Ok(())
    }

    fn build_environment(
        &self,
        profile: &Profile,
        api_key: &str,
        script_output: &ScriptOutput,
    ) -> HashMap<String, String> {
        let mut env = HashMap::new();

        for (key, value) in &profile.env {
            if !key.starts_with("_RINGLET_") {
                env.insert(key.clone(), value.clone());
            }
        }

        env.insert(
            "HOME".to_string(),
            profile.metadata.home.to_string_lossy().to_string(),
        );

        for (key, value) in &script_output.env {
            let resolved = value.replace("${API_KEY}", api_key);
            env.insert(key.clone(), resolved);
        }

        env
    }
}

impl ProcessLauncher {
    fn spawn_prepared(&self, context: &ExecutionContext) -> Result<RunResult> {
        info!(
            "Spawning prepared command '{}' for profile '{}' in {:?}",
            context.binary, context.alias, context.working_dir
        );

        let mut cmd = Command::new(&context.binary);
        cmd.current_dir(&context.working_dir);
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        cmd.env_clear();
        cmd.envs(&context.env);
        cmd.args(&context.args);

        debug!("Command: {:?}", cmd);

        let child = cmd
            .spawn()
            .context(format!("Failed to spawn: {}", context.binary))?;

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
    proxy_url: Option<&str>,
) -> Result<ScriptContext> {
    // Resolve endpoint URL - handle indirection (e.g., "default" -> "international" -> URL)
    let endpoint_id = &profile.endpoint_id;
    let mut endpoint = provider
        .endpoints
        .get(endpoint_id)
        .or_else(|| {
            provider
                .default_endpoint()
                .and_then(|e| provider.endpoints.get(e))
        })
        .ok_or_else(|| anyhow!("Endpoint not found: {}", endpoint_id))?
        .clone();

    // If the endpoint value is itself a key in endpoints (indirection), resolve it
    if provider.endpoints.contains_key(&endpoint) {
        endpoint = provider.endpoints.get(&endpoint).unwrap().clone();
    }

    // Convert hooks_config to JSON value for script context
    let hooks_config = profile
        .metadata
        .hooks_config
        .as_ref()
        .and_then(|h| serde_json::to_value(h).ok());

    Ok(ScriptContext {
        profile: ProfileContext {
            alias: profile.alias.clone(),
            home: profile.metadata.home.clone(),
            model: profile.model.clone(),
            endpoint,
            hooks: profile.metadata.enabled_hooks.clone(),
            mcp_servers: profile.metadata.enabled_mcp_servers.clone(),
            hooks_config,
            proxy_url: proxy_url.map(String::from),
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
