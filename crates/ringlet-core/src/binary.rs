//! Binary management types and paths for external tools like ultrallm.
//!
//! This module provides path management for binaries. Actual download and
//! process management is handled by ringletd.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default ultrallm version to use.
pub const DEFAULT_ULTRALLM_VERSION: &str = "latest";

/// Binary configuration for a specific tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryConfig {
    /// Name of the binary.
    pub name: String,

    /// Version to use ("latest" or specific version).
    pub version: String,

    /// Optional local path override (for development).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<PathBuf>,
}

impl Default for BinaryConfig {
    fn default() -> Self {
        Self {
            name: "ultrallm".to_string(),
            version: DEFAULT_ULTRALLM_VERSION.to_string(),
            local_path: None,
        }
    }
}

/// Binary paths manager.
#[derive(Debug, Clone)]
pub struct BinaryPaths {
    /// Cache directory for binaries.
    cache_dir: PathBuf,
}

impl BinaryPaths {
    /// Create a new binary paths manager.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get the binaries cache directory.
    pub fn binaries_dir(&self) -> PathBuf {
        self.cache_dir.join("binaries")
    }

    /// Get path to cached ultrallm binary.
    pub fn ultrallm_path(&self, version: &str) -> PathBuf {
        let platform = Self::platform_string();
        let filename = format!("ultrallm-{}-{}", version, platform);

        #[cfg(windows)]
        let filename = format!("{}.exe", filename);

        self.binaries_dir().join(filename)
    }

    /// Get platform string for binary downloads.
    pub fn platform_string() -> &'static str {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return "linux-x86_64";

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return "linux-aarch64";

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return "darwin-x86_64";

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return "darwin-aarch64";

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return "windows-x86_64";

        #[cfg(not(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "windows", target_arch = "x86_64"),
        )))]
        return "unknown";
    }

    /// Get download URL for ultrallm binary.
    pub fn ultrallm_download_url(version: &str) -> String {
        let platform = Self::platform_string();
        if version == "latest" {
            format!(
                "https://github.com/dipankar/ultrallm/releases/latest/download/ultrallm-{}",
                platform
            )
        } else {
            format!(
                "https://github.com/dipankar/ultrallm/releases/download/v{}/ultrallm-{}",
                version, platform
            )
        }
    }

    /// Find local ultrallm binary for development.
    /// Checks common development locations.
    pub fn find_local_ultrallm() -> Option<PathBuf> {
        // Check common development locations
        let mut candidates: Vec<PathBuf> = vec![
            // Local build in ultrallm project
            PathBuf::from("/home/dipankar/Code/ultrallm/target/release/ultrallm"),
        ];

        // Add home-relative paths if home directory is available
        if let Some(home) = dirs_home() {
            candidates.push(home.join(".local/bin/ultrallm"));
            candidates.push(home.join(".cargo/bin/ultrallm"));
        }

        for candidate in candidates {
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
        }

        None
    }

    /// List installed ultrallm versions from cache.
    pub fn installed_versions(&self) -> Vec<String> {
        let mut versions = Vec::new();

        if let Ok(entries) = std::fs::read_dir(self.binaries_dir()) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("ultrallm-") {
                    // Extract version from filename
                    if let Some(rest) = name.strip_prefix("ultrallm-") {
                        let version = rest
                            .split('-')
                            .next()
                            .unwrap_or("unknown")
                            .to_string();
                        if !versions.contains(&version) {
                            versions.push(version);
                        }
                    }
                }
            }
        }

        versions
    }
}

/// Get home directory using directories crate.
fn dirs_home() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_binary_paths_new() {
        let dir = tempdir().unwrap();
        let paths = BinaryPaths::new(dir.path().to_path_buf());
        assert!(paths.binaries_dir().ends_with("binaries"));
    }

    #[test]
    fn test_ultrallm_path() {
        let dir = tempdir().unwrap();
        let paths = BinaryPaths::new(dir.path().to_path_buf());
        let path = paths.ultrallm_path("1.0.0");
        assert!(path.to_string_lossy().contains("ultrallm-1.0.0"));
    }

    #[test]
    fn test_platform_string() {
        let platform = BinaryPaths::platform_string();
        assert!(!platform.is_empty());
        assert_ne!(platform, "unknown");
    }

    #[test]
    fn test_download_url() {
        let url = BinaryPaths::ultrallm_download_url("1.0.0");
        assert!(url.contains("v1.0.0"));
        assert!(url.contains("ultrallm"));

        let latest_url = BinaryPaths::ultrallm_download_url("latest");
        assert!(latest_url.contains("latest"));
    }
}
