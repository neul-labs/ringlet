//! Sandbox configuration and platform-specific wrappers.
//!
//! Provides sandboxing for terminal sessions using:
//! - Linux: bwrap (bubblewrap)
//! - macOS: sandbox-exec
//! - Windows: No sandboxing (not supported)

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::warn;

/// Sandbox configuration for a terminal session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Whether sandboxing is enabled (default: true on supported platforms).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Custom bwrap flags (Linux only).
    pub bwrap_flags: Option<Vec<String>>,
    /// Custom sandbox-exec profile (macOS only).
    pub sandbox_exec_profile: Option<String>,
}

fn default_enabled() -> bool {
    true
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bwrap_flags: None,
            sandbox_exec_profile: None,
        }
    }
}

impl SandboxConfig {
    /// Create a config with sandboxing disabled.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            bwrap_flags: None,
            sandbox_exec_profile: None,
        }
    }
}

/// Platform detection for sandbox support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxPlatform {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

impl SandboxPlatform {
    /// Detect the current platform.
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            return SandboxPlatform::Linux;
        }

        #[cfg(target_os = "macos")]
        {
            return SandboxPlatform::MacOS;
        }

        #[cfg(target_os = "windows")]
        {
            return SandboxPlatform::Windows;
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            return SandboxPlatform::Unknown;
        }
    }

    /// Check if this platform supports sandboxing.
    pub fn supports_sandboxing(&self) -> bool {
        matches!(self, SandboxPlatform::Linux | SandboxPlatform::MacOS)
    }
}

/// Result of wrapping a command with sandbox.
pub struct SandboxedCommand {
    pub command: String,
    pub args: Vec<String>,
}

/// Check if bwrap is available on the system.
fn is_bwrap_available() -> bool {
    std::process::Command::new("bwrap")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if sandbox-exec is available on the system.
fn is_sandbox_exec_available() -> bool {
    std::process::Command::new("sandbox-exec")
        .arg("-n")
        .arg("no-network")
        .arg("true")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Default bwrap flags for a sensible sandbox.
///
/// This provides:
/// - Read-only root filesystem
/// - Read-write home directory (for agent configs)
/// - Read-write working directory (for project files)
/// - Read-write /tmp (for temp files)
/// - Network access (agents need API access)
/// - Process isolation (PID/IPC/UTS namespaces)
fn default_bwrap_flags(working_dir: &Path, home: &str) -> Vec<String> {
    let working_dir_str = working_dir.to_string_lossy().to_string();

    vec![
        // Bind root filesystem read-only
        "--ro-bind".to_string(),
        "/".to_string(),
        "/".to_string(),
        // Bind home directory read-write (for agent configs like ~/.claude)
        "--bind".to_string(),
        home.to_string(),
        home.to_string(),
        // Bind working directory read-write
        "--bind".to_string(),
        working_dir_str.clone(),
        working_dir_str.clone(),
        // Bind /tmp for temporary files
        "--bind".to_string(),
        "/tmp".to_string(),
        "/tmp".to_string(),
        // Create /dev nodes
        "--dev".to_string(),
        "/dev".to_string(),
        // Create /proc
        "--proc".to_string(),
        "/proc".to_string(),
        // Unshare namespaces (but NOT network - agents need API access)
        "--unshare-user".to_string(),
        "--unshare-ipc".to_string(),
        "--unshare-pid".to_string(),
        "--unshare-uts".to_string(),
        "--unshare-cgroup".to_string(),
        // Die with parent process (cleanup)
        "--die-with-parent".to_string(),
        // Set working directory
        "--chdir".to_string(),
        working_dir_str,
        // Delimiter before command
        "--".to_string(),
    ]
}

/// Default sandbox-exec profile for macOS.
///
/// This profile:
/// - Denies writes to system directories
/// - Allows writes to home, working dir, and /tmp
/// - Allows network access
/// - Allows process execution
fn default_sandbox_exec_profile(working_dir: &Path, home: &str) -> String {
    let working_dir_str = working_dir.to_string_lossy();

    format!(
        r#"(version 1)
(allow default)
(deny file-write*
    (subpath "/System")
    (subpath "/usr")
    (subpath "/bin")
    (subpath "/sbin")
    (subpath "/Library")
    (subpath "/private/var")
)
(allow file-write*
    (subpath "{home}")
    (subpath "{working_dir}")
    (subpath "/tmp")
    (subpath "/private/tmp")
)
(allow network*)
(allow process-fork)
(allow process-exec)
"#,
        home = home,
        working_dir = working_dir_str
    )
}

/// Wrap a command with bwrap (Linux).
fn wrap_with_bwrap(
    command: &str,
    args: &[String],
    working_dir: &Path,
    config: &SandboxConfig,
) -> Result<SandboxedCommand> {
    if !is_bwrap_available() {
        return Err(anyhow!(
            "bwrap (bubblewrap) not found. Install it or use --no-sandbox"
        ));
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());

    let mut bwrap_args = config
        .bwrap_flags
        .clone()
        .unwrap_or_else(|| default_bwrap_flags(working_dir, &home));

    // Add the actual command and its arguments
    bwrap_args.push(command.to_string());
    bwrap_args.extend(args.iter().cloned());

    Ok(SandboxedCommand {
        command: "bwrap".to_string(),
        args: bwrap_args,
    })
}

/// Wrap a command with sandbox-exec (macOS).
fn wrap_with_sandbox_exec(
    command: &str,
    args: &[String],
    working_dir: &Path,
    config: &SandboxConfig,
) -> Result<SandboxedCommand> {
    if !is_sandbox_exec_available() {
        return Err(anyhow!(
            "sandbox-exec not found (should be available on macOS)"
        ));
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());

    let profile = config
        .sandbox_exec_profile
        .clone()
        .unwrap_or_else(|| default_sandbox_exec_profile(working_dir, &home));

    let mut sandbox_args = vec!["-p".to_string(), profile, command.to_string()];
    sandbox_args.extend(args.iter().cloned());

    Ok(SandboxedCommand {
        command: "sandbox-exec".to_string(),
        args: sandbox_args,
    })
}

/// Prepare a command for execution, optionally with sandboxing.
///
/// If sandboxing is enabled and the platform supports it, the command will be
/// wrapped with the appropriate sandbox tool (bwrap on Linux, sandbox-exec on macOS).
///
/// If the sandbox tool is not available, a warning is logged and the command
/// is returned unwrapped.
pub fn prepare_command(
    command: &str,
    args: &[String],
    working_dir: &Path,
    config: &SandboxConfig,
) -> Result<SandboxedCommand> {
    let platform = SandboxPlatform::detect();

    // If sandboxing disabled or unsupported platform, return command as-is
    if !config.enabled || !platform.supports_sandboxing() {
        return Ok(SandboxedCommand {
            command: command.to_string(),
            args: args.to_vec(),
        });
    }

    // Try to wrap with platform-specific sandbox
    let result = match platform {
        SandboxPlatform::Linux => wrap_with_bwrap(command, args, working_dir, config),
        SandboxPlatform::MacOS => wrap_with_sandbox_exec(command, args, working_dir, config),
        _ => Ok(SandboxedCommand {
            command: command.to_string(),
            args: args.to_vec(),
        }),
    };

    // If sandbox tool not available, warn and continue without sandbox
    match result {
        Ok(cmd) => Ok(cmd),
        Err(e) => {
            warn!(
                "Sandbox tool not available on {:?}, running without sandbox: {}",
                platform, e
            );
            Ok(SandboxedCommand {
                command: command.to_string(),
                args: args.to_vec(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(config.enabled);
        assert!(config.bwrap_flags.is_none());
        assert!(config.sandbox_exec_profile.is_none());
    }

    #[test]
    fn test_sandbox_config_disabled() {
        let config = SandboxConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_prepare_command_disabled() {
        let config = SandboxConfig::disabled();
        let result = prepare_command("echo", &["hello".to_string()], &PathBuf::from("/tmp"), &config);
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.command, "echo");
        assert_eq!(cmd.args, vec!["hello"]);
    }

    #[test]
    fn test_platform_detection() {
        let platform = SandboxPlatform::detect();
        // Just verify it returns something valid
        assert!(matches!(
            platform,
            SandboxPlatform::Linux
                | SandboxPlatform::MacOS
                | SandboxPlatform::Windows
                | SandboxPlatform::Unknown
        ));
    }
}
