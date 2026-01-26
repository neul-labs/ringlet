//! Agent registry - loads manifests and detects installed agents.

use anyhow::{Context, Result};
use ringlet_core::{expand_tilde, AgentInfo, AgentManifest, RingletPaths};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, warn};

/// Built-in agent manifests (embedded at compile time).
const BUILTIN_AGENTS: &[(&str, &str)] = &[
    ("claude", include_str!("../../../manifests/agents/claude.toml")),
    ("grok", include_str!("../../../manifests/agents/grok.toml")),
    ("codex", include_str!("../../../manifests/agents/codex.toml")),
    ("droid", include_str!("../../../manifests/agents/droid.toml")),
    ("opencode", include_str!("../../../manifests/agents/opencode.toml")),
];

/// Agent registry.
pub struct AgentRegistry {
    agents: HashMap<String, AgentManifest>,
    detection_cache: HashMap<String, DetectionResult>,
}

/// Result of agent detection.
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub installed: bool,
    pub version: Option<String>,
    pub binary_path: Option<String>,
}

impl AgentRegistry {
    /// Create a new agent registry, loading all manifests.
    pub fn new(paths: &RingletPaths) -> Result<Self> {
        let mut agents = HashMap::new();

        // Load built-in manifests
        for (id, toml) in BUILTIN_AGENTS {
            match AgentManifest::from_toml(toml) {
                Ok(manifest) => {
                    debug!("Loaded built-in agent: {}", id);
                    agents.insert(id.to_string(), manifest);
                }
                Err(e) => {
                    warn!("Failed to parse built-in agent {}: {}", id, e);
                }
            }
        }

        // Load user-defined manifests from agents.d/
        let agents_d = paths.agents_d();
        if agents_d.exists() {
            if let Ok(entries) = std::fs::read_dir(&agents_d) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "toml") {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => match AgentManifest::from_toml(&content) {
                                Ok(manifest) => {
                                    debug!("Loaded user agent from {:?}: {}", path, manifest.id);
                                    agents.insert(manifest.id.clone(), manifest);
                                }
                                Err(e) => {
                                    warn!("Failed to parse {:?}: {}", path, e);
                                }
                            },
                            Err(e) => {
                                warn!("Failed to read {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            agents,
            detection_cache: HashMap::new(),
        })
    }

    /// Get an agent manifest by ID.
    pub fn get(&self, id: &str) -> Option<&AgentManifest> {
        self.agents.get(id)
    }

    /// Get all agent IDs.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.agents.keys().map(|s| s.as_str())
    }

    /// Detect if an agent is installed.
    pub fn detect(&mut self, id: &str) -> Option<DetectionResult> {
        // Check cache first
        if let Some(cached) = self.detection_cache.get(id) {
            return Some(cached.clone());
        }

        let manifest = self.agents.get(id)?;
        let result = detect_agent(manifest);
        self.detection_cache.insert(id.to_string(), result.clone());
        Some(result)
    }

    /// Get agent info for all agents.
    pub fn list_all(&mut self, profile_counts: &HashMap<String, usize>) -> Vec<AgentInfo> {
        let mut infos: Vec<AgentInfo> = self
            .agents
            .values()
            .map(|manifest| {
                let detection = self
                    .detection_cache
                    .get(&manifest.id)
                    .cloned()
                    .unwrap_or_else(|| {
                        let result = detect_agent(manifest);
                        self.detection_cache.insert(manifest.id.clone(), result.clone());
                        result
                    });

                AgentInfo {
                    id: manifest.id.clone(),
                    name: manifest.name.clone(),
                    installed: detection.installed,
                    version: detection.version,
                    binary_path: detection.binary_path,
                    profile_count: *profile_counts.get(&manifest.id).unwrap_or(&0),
                    default_model: manifest.models.default.clone(),
                    default_provider: manifest.profile.default_provider.clone(),
                    supports_hooks: manifest.supports_hooks,
                    last_used: None, // TODO: track from telemetry
                }
            })
            .collect();

        // Sort by ID for consistent ordering
        infos.sort_by(|a, b| a.id.cmp(&b.id));
        infos
    }

    /// Get info for a single agent.
    pub fn get_info(&mut self, id: &str, profile_count: usize) -> Option<AgentInfo> {
        let manifest = self.agents.get(id)?;
        let detection = self
            .detection_cache
            .get(id)
            .cloned()
            .unwrap_or_else(|| {
                let result = detect_agent(manifest);
                self.detection_cache.insert(id.to_string(), result.clone());
                result
            });

        Some(AgentInfo {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            installed: detection.installed,
            version: detection.version,
            binary_path: detection.binary_path,
            profile_count,
            default_model: manifest.models.default.clone(),
            default_provider: manifest.profile.default_provider.clone(),
            supports_hooks: manifest.supports_hooks,
            last_used: None,
        })
    }
}

/// Detect if an agent is installed.
fn detect_agent(manifest: &AgentManifest) -> DetectionResult {
    // Try detection commands
    for cmd in &manifest.detect.commands {
        if let Some(result) = try_command(cmd, manifest.version_flag.as_deref()) {
            return result;
        }
    }

    // Try detection files
    for file in &manifest.detect.files {
        let path = expand_tilde(file);
        if path.exists() {
            // File exists, try to find and run the binary
            if let Some(result) = try_binary(&manifest.binary, manifest.version_flag.as_deref()) {
                return result;
            }
            // File exists but can't run binary
            return DetectionResult {
                installed: true,
                version: None,
                binary_path: None,
            };
        }
    }

    // Try the binary directly
    if let Some(result) = try_binary(&manifest.binary, manifest.version_flag.as_deref()) {
        return result;
    }

    DetectionResult {
        installed: false,
        version: None,
        binary_path: None,
    }
}

/// Try running a detection command.
fn try_command(cmd: &str, _version_flag: Option<&str>) -> Option<DetectionResult> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let version = extract_version(&stdout);
        let binary_path = which_binary(parts[0]);

        Some(DetectionResult {
            installed: true,
            version,
            binary_path,
        })
    } else {
        None
    }
}

/// Try running a binary with version flag.
fn try_binary(binary: &str, version_flag: Option<&str>) -> Option<DetectionResult> {
    let flag = version_flag.unwrap_or("--version");

    let output = Command::new(binary)
        .arg(flag)
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let version = extract_version(&stdout);
        let binary_path = which_binary(binary);

        Some(DetectionResult {
            installed: true,
            version,
            binary_path,
        })
    } else {
        None
    }
}

/// Extract version from output.
fn extract_version(output: &str) -> Option<String> {
    // Try common patterns
    for line in output.lines() {
        let line = line.trim();
        // Look for version numbers like "1.2.3" or "v1.2.3"
        if let Some(version) = extract_semver(line) {
            return Some(version);
        }
    }
    None
}

/// Extract semver-like version.
fn extract_semver(s: &str) -> Option<String> {
    let s = s.trim_start_matches(|c: char| !c.is_ascii_digit() && c != 'v');
    let s = s.trim_start_matches('v');

    let end = s
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());

    let version = &s[..end];
    if version.contains('.') && version.chars().next()?.is_ascii_digit() {
        Some(version.to_string())
    } else {
        None
    }
}

/// Find binary path using which.
fn which_binary(binary: &str) -> Option<String> {
    #[cfg(unix)]
    {
        let output = Command::new("which").arg(binary).output().ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout);
            return Some(path.trim().to_string());
        }
    }

    #[cfg(windows)]
    {
        let output = Command::new("where").arg(binary).output().ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout);
            return path.lines().next().map(|s| s.trim().to_string());
        }
    }

    None
}
