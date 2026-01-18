//! Proxy manager - spawns and manages ultrallm proxy processes per profile.

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clown_core::{
    BinaryPaths, ClownPaths, ProfileProxyConfig, ProxyInstanceInfo, ProxyStatus,
    RoutingRule, RoutingStrategy,
};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Default base port for proxy instances.
const BASE_PORT: u16 = 8080;
/// Maximum port number for proxy instances.
const MAX_PORT: u16 = 8180;
/// Health check interval in seconds.
const HEALTH_CHECK_INTERVAL_SECS: u64 = 30;

/// Manages ultrallm proxy instances for profiles.
pub struct ProxyManager {
    /// Path to ultrallm binary.
    binary_path: Option<PathBuf>,
    /// Running proxy instances by profile alias.
    instances: RwLock<HashMap<String, ProxyInstance>>,
    /// Port allocator.
    port_allocator: RwLock<PortAllocator>,
    /// Paths configuration.
    paths: ClownPaths,
}

/// A running proxy instance.
pub struct ProxyInstance {
    /// Profile alias.
    pub alias: String,
    /// Port the proxy is listening on.
    pub port: u16,
    /// Process ID.
    pub pid: u32,
    /// The child process handle.
    pub process: Child,
    /// Path to the config file.
    pub config_path: PathBuf,
    /// Path to the log file.
    pub log_path: PathBuf,
    /// When the proxy was started.
    pub started_at: chrono::DateTime<Utc>,
    /// Current status.
    pub status: ProxyStatus,
    /// Number of restarts.
    pub restart_count: u32,
}

/// Port allocator for proxy instances.
struct PortAllocator {
    /// Base port number.
    base_port: u16,
    /// Maximum port number.
    max_port: u16,
    /// Currently allocated ports.
    allocated: HashSet<u16>,
    /// Port assignments by profile alias.
    assignments: HashMap<String, u16>,
}

impl PortAllocator {
    fn new(base_port: u16, max_port: u16) -> Self {
        Self {
            base_port,
            max_port,
            allocated: HashSet::new(),
            assignments: HashMap::new(),
        }
    }

    /// Allocate a port for a profile.
    fn allocate(&mut self, alias: &str, preferred: Option<u16>) -> Result<u16> {
        // Check if already assigned
        if let Some(&port) = self.assignments.get(alias) {
            return Ok(port);
        }

        // Try preferred port
        if let Some(port) = preferred {
            if port >= self.base_port && port <= self.max_port && !self.allocated.contains(&port) {
                self.allocated.insert(port);
                self.assignments.insert(alias.to_string(), port);
                return Ok(port);
            }
        }

        // Find next available port
        for port in self.base_port..=self.max_port {
            if !self.allocated.contains(&port) {
                self.allocated.insert(port);
                self.assignments.insert(alias.to_string(), port);
                return Ok(port);
            }
        }

        Err(anyhow!("No available ports in range {}-{}", self.base_port, self.max_port))
    }

    /// Release a port.
    fn release(&mut self, alias: &str) {
        if let Some(port) = self.assignments.remove(alias) {
            self.allocated.remove(&port);
        }
    }
}

impl ProxyManager {
    /// Create a new proxy manager.
    pub fn new(paths: ClownPaths) -> Self {
        // Try to find local ultrallm binary
        let binary_path = BinaryPaths::find_local_ultrallm();

        if let Some(ref path) = binary_path {
            info!("Found ultrallm binary: {:?}", path);
        } else {
            warn!("ultrallm binary not found - proxy features will be unavailable");
        }

        Self {
            binary_path,
            instances: RwLock::new(HashMap::new()),
            port_allocator: RwLock::new(PortAllocator::new(BASE_PORT, MAX_PORT)),
            paths,
        }
    }

    /// Check if ultrallm binary is available.
    pub fn is_available(&self) -> bool {
        self.binary_path.is_some()
    }

    /// Get the binary path.
    pub fn binary_path(&self) -> Option<&PathBuf> {
        self.binary_path.as_ref()
    }

    /// Start a proxy for a profile.
    pub async fn start(
        &self,
        alias: &str,
        profile_home: &PathBuf,
        config: &ProfileProxyConfig,
    ) -> Result<u16> {
        let binary_path = self.binary_path.as_ref()
            .ok_or_else(|| anyhow!("ultrallm binary not available"))?;

        // Check if already running
        {
            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(alias) {
                if matches!(instance.status, ProxyStatus::Running) {
                    return Ok(instance.port);
                }
            }
        }

        // Allocate port
        let port = {
            let mut allocator = self.port_allocator.write().await;
            allocator.allocate(alias, config.port)?
        };

        // Create .ultrallm directory in profile home
        let ultrallm_dir = profile_home.join(".ultrallm");
        std::fs::create_dir_all(&ultrallm_dir)
            .context("Failed to create .ultrallm directory")?;

        let logs_dir = ultrallm_dir.join("logs");
        std::fs::create_dir_all(&logs_dir)
            .context("Failed to create logs directory")?;

        // Generate config file
        let config_path = ultrallm_dir.join("config.yaml");
        self.generate_config(&config_path, port, config)?;

        // Open log file
        let log_path = logs_dir.join("proxy.log");
        let log_file = File::create(&log_path)
            .context("Failed to create log file")?;

        // Spawn ultrallm process
        info!("Starting proxy for profile '{}' on port {}", alias, port);
        let process = Command::new(binary_path)
            .args(["serve", "--config", &config_path.to_string_lossy()])
            .stdout(Stdio::from(log_file.try_clone()?))
            .stderr(Stdio::from(log_file))
            .spawn()
            .context("Failed to spawn ultrallm process")?;

        let pid = process.id();
        info!("Proxy started for '{}' with PID {}", alias, pid);

        // Store instance
        let instance = ProxyInstance {
            alias: alias.to_string(),
            port,
            pid,
            process,
            config_path,
            log_path,
            started_at: Utc::now(),
            status: ProxyStatus::Starting,
            restart_count: 0,
        };

        self.instances.write().await.insert(alias.to_string(), instance);

        // Wait a moment for the proxy to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Check if it's running
        if self.check_health(port).await {
            let mut instances = self.instances.write().await;
            if let Some(instance) = instances.get_mut(alias) {
                instance.status = ProxyStatus::Running;
            }
        }

        Ok(port)
    }

    /// Stop a proxy for a profile.
    pub async fn stop(&self, alias: &str) -> Result<()> {
        let mut instances = self.instances.write().await;

        if let Some(mut instance) = instances.remove(alias) {
            instance.status = ProxyStatus::Stopping;
            info!("Stopping proxy for profile '{}'", alias);

            // Try graceful shutdown first
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                // Send SIGTERM
                unsafe {
                    libc::kill(instance.pid as i32, libc::SIGTERM);
                }
            }

            // Wait for process to exit (with timeout)
            let timeout = tokio::time::Duration::from_secs(5);
            let start = std::time::Instant::now();

            loop {
                match instance.process.try_wait() {
                    Ok(Some(_)) => break, // Process exited
                    Ok(None) => {
                        if start.elapsed() > timeout {
                            // Force kill
                            warn!("Proxy for '{}' didn't exit gracefully, killing", alias);
                            let _ = instance.process.kill();
                            break;
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        error!("Error waiting for proxy: {}", e);
                        break;
                    }
                }
            }

            // Release port
            self.port_allocator.write().await.release(alias);
            info!("Proxy stopped for profile '{}'", alias);
        }

        Ok(())
    }

    /// Stop all proxies.
    pub async fn stop_all(&self) -> Result<()> {
        let aliases: Vec<String> = {
            let instances = self.instances.read().await;
            instances.keys().cloned().collect()
        };

        for alias in aliases {
            if let Err(e) = self.stop(&alias).await {
                warn!("Failed to stop proxy for '{}': {}", alias, e);
            }
        }

        Ok(())
    }

    /// Get status of all proxies.
    pub async fn status(&self) -> Vec<ProxyInstanceInfo> {
        let instances = self.instances.read().await;
        instances.values()
            .map(|i| ProxyInstanceInfo {
                alias: i.alias.clone(),
                port: i.port,
                pid: i.pid,
                status: i.status.clone(),
                started_at: i.started_at,
                restart_count: i.restart_count,
            })
            .collect()
    }

    /// Get status of a specific proxy.
    pub async fn status_for(&self, alias: &str) -> Option<ProxyInstanceInfo> {
        let instances = self.instances.read().await;
        instances.get(alias).map(|i| ProxyInstanceInfo {
            alias: i.alias.clone(),
            port: i.port,
            pid: i.pid,
            status: i.status.clone(),
            started_at: i.started_at,
            restart_count: i.restart_count,
        })
    }

    /// Get the proxy URL for a profile if running.
    pub async fn proxy_url(&self, alias: &str) -> Option<String> {
        let instances = self.instances.read().await;
        instances.get(alias).and_then(|i| {
            if matches!(i.status, ProxyStatus::Running) {
                Some(format!("http://127.0.0.1:{}", i.port))
            } else {
                None
            }
        })
    }

    /// Check if a proxy is healthy.
    async fn check_health(&self, port: u16) -> bool {
        let url = format!("http://127.0.0.1:{}/health", port);

        // Use a simple TCP connection check since we don't have reqwest
        match tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Generate ultrallm config from ProfileProxyConfig.
    fn generate_config(
        &self,
        path: &PathBuf,
        port: u16,
        config: &ProfileProxyConfig,
    ) -> Result<()> {
        let mut yaml = String::new();

        // Server section
        yaml.push_str(&format!(r#"server:
  host: "127.0.0.1"
  port: {}

"#, port));

        // Model list - generate from routing rules
        yaml.push_str("model_list:\n");

        // Collect unique targets from routing rules
        let mut targets: HashSet<String> = HashSet::new();
        for rule in &config.routing.rules {
            targets.insert(rule.target.clone());
        }

        // Add model aliases
        for (_, target) in &config.model_aliases {
            targets.insert(target.to_string_format());
        }

        // Generate model entries
        for target in &targets {
            if let Some((provider, model)) = target.split_once('/') {
                yaml.push_str(&format!(r#"  - model_name: "{}"
    litellm_params:
      model: "{}/{}"
      api_key: "${{{{ {}_API_KEY }}}}"
"#, target, provider, model, provider.to_uppercase()));
            }
        }

        // Router settings
        yaml.push_str(&format!(r#"
router_settings:
  routing_strategy: "{}"
"#, match config.routing.strategy {
            RoutingStrategy::Simple => "simple",
            RoutingStrategy::Weighted => "weighted",
            RoutingStrategy::LowestCost => "lowest-cost",
            RoutingStrategy::Adaptive => "adaptive",
            RoutingStrategy::Conditional => "conditional",
        }));

        // Add rules if conditional routing
        if !config.routing.rules.is_empty() {
            yaml.push_str("  rules:\n");
            for rule in &config.routing.rules {
                yaml.push_str(&format!(r#"    - name: "{}"
      model: "{}"
      priority: {}
"#, rule.name, rule.target, rule.priority));
            }
        }

        // Write config file
        let mut file = File::create(path)
            .context("Failed to create config file")?;
        file.write_all(yaml.as_bytes())
            .context("Failed to write config file")?;

        debug!("Generated proxy config at {:?}", path);
        Ok(())
    }

    /// Read proxy logs for a profile.
    pub async fn read_logs(&self, alias: &str, lines: Option<usize>) -> Result<String> {
        let instances = self.instances.read().await;
        let instance = instances.get(alias)
            .ok_or_else(|| anyhow!("Proxy not found for profile '{}'", alias))?;

        let content = std::fs::read_to_string(&instance.log_path)
            .context("Failed to read log file")?;

        if let Some(n) = lines {
            let all_lines: Vec<&str> = content.lines().collect();
            let start = if all_lines.len() > n { all_lines.len() - n } else { 0 };
            Ok(all_lines[start..].join("\n"))
        } else {
            Ok(content)
        }
    }
}

impl Drop for ProxyManager {
    fn drop(&mut self) {
        // Synchronous cleanup - try to kill all processes
        if let Ok(mut instances) = self.instances.try_write() {
            for (alias, mut instance) in instances.drain() {
                warn!("Cleaning up proxy for '{}' on drop", alias);
                let _ = instance.process.kill();
            }
        }
    }
}
