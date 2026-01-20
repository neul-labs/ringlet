//! Platform-aware path handling for clown.

use directories::ProjectDirs;
use std::path::PathBuf;

/// Provides platform-appropriate paths for clown data.
#[derive(Debug, Clone)]
pub struct ClownPaths {
    /// Base config directory (~/.config/clown on Linux, etc.)
    pub config_dir: PathBuf,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Data directory (same as config on most platforms)
    pub data_dir: PathBuf,
}

impl ClownPaths {
    /// Create paths using platform conventions.
    pub fn new() -> Option<Self> {
        let proj_dirs = ProjectDirs::from("", "", "clown")?;

        Some(Self {
            config_dir: proj_dirs.config_dir().to_path_buf(),
            cache_dir: proj_dirs.cache_dir().to_path_buf(),
            data_dir: proj_dirs.data_dir().to_path_buf(),
        })
    }

    /// User-supplied agent manifests directory.
    pub fn agents_d(&self) -> PathBuf {
        self.config_dir.join("agents.d")
    }

    /// User-supplied provider manifests directory.
    pub fn providers_d(&self) -> PathBuf {
        self.config_dir.join("providers.d")
    }

    /// User-override scripts directory.
    pub fn scripts_dir(&self) -> PathBuf {
        self.config_dir.join("scripts")
    }

    /// Profiles storage directory.
    pub fn profiles_dir(&self) -> PathBuf {
        self.config_dir.join("profiles")
    }

    /// Registry cache directory.
    pub fn registry_dir(&self) -> PathBuf {
        self.config_dir.join("registry")
    }

    /// Registry commits cache.
    pub fn registry_commits_dir(&self) -> PathBuf {
        self.registry_dir().join("commits")
    }

    /// Registry lock file.
    pub fn registry_lock(&self) -> PathBuf {
        self.registry_dir().join("registry.lock")
    }

    /// Telemetry data directory.
    pub fn telemetry_dir(&self) -> PathBuf {
        self.config_dir.join("telemetry")
    }

    /// Sessions log file (JSONL).
    pub fn sessions_log(&self) -> PathBuf {
        self.telemetry_dir().join("sessions.jsonl")
    }

    /// Aggregated stats file.
    pub fn aggregates_file(&self) -> PathBuf {
        self.telemetry_dir().join("aggregates.json")
    }

    /// Usage aggregates file (token/cost tracking).
    pub fn usage_aggregates_file(&self) -> PathBuf {
        self.telemetry_dir().join("usage-aggregates.json")
    }

    /// LiteLLM pricing cache file.
    pub fn litellm_pricing_cache(&self) -> PathBuf {
        self.registry_dir().join("litellm-pricing.json")
    }

    /// Agent detection cache.
    pub fn agent_detections_cache(&self) -> PathBuf {
        self.cache_dir.join("agent-detections.json")
    }

    /// User config file.
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Daemon endpoint file (stores IPC path).
    pub fn daemon_endpoint(&self) -> PathBuf {
        self.config_dir.join("daemon-endpoint")
    }

    /// Daemon PID file.
    pub fn daemon_pid(&self) -> PathBuf {
        self.config_dir.join("daemon.pid")
    }

    /// Logs directory.
    pub fn logs_dir(&self) -> PathBuf {
        self.config_dir.join("logs")
    }

    /// Daemon log file.
    pub fn daemon_log(&self) -> PathBuf {
        self.logs_dir().join("clownd.log")
    }

    /// IPC socket path (platform-specific).
    pub fn ipc_socket(&self) -> PathBuf {
        #[cfg(unix)]
        {
            PathBuf::from("/tmp/clownd.sock")
        }
        #[cfg(windows)]
        {
            self.config_dir.join("clownd.ipc")
        }
    }

    /// Ensure all required directories exist.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.agents_d())?;
        std::fs::create_dir_all(&self.providers_d())?;
        std::fs::create_dir_all(&self.scripts_dir())?;
        std::fs::create_dir_all(&self.profiles_dir())?;
        std::fs::create_dir_all(&self.registry_dir())?;
        std::fs::create_dir_all(&self.telemetry_dir())?;
        std::fs::create_dir_all(&self.logs_dir())?;
        Ok(())
    }
}

impl Default for ClownPaths {
    fn default() -> Self {
        Self::new().expect("Failed to determine platform directories")
    }
}

/// Expand ~ to home directory in a path string.
pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = home_dir() {
            return home.join(&path[2..]);
        }
    } else if path == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

/// Get home directory.
pub fn home_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf())
}

/// Expand template variables in a path string.
/// Supports: {alias}, {agent-id}
pub fn expand_template(template: &str, alias: &str, agent_id: &str) -> PathBuf {
    let expanded = template
        .replace("{alias}", alias)
        .replace("{agent-id}", agent_id);
    expand_tilde(&expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let path = expand_tilde("~/test");
        assert!(!path.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_expand_template() {
        let result = expand_template("~/.{agent-id}-profiles/{alias}", "work", "claude");
        let s = result.to_string_lossy();
        assert!(s.contains(".claude-profiles"));
        assert!(s.contains("work"));
    }
}
