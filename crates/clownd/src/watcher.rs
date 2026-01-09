//! Filesystem watcher - detects external changes to config files.
//!
//! This module watches:
//! - ~/.config/clown/agents.d/ for new/updated agent manifests
//! - ~/.config/clown/providers.d/ for new/updated provider manifests
//! - ~/.config/clown/profiles/ for profile changes
//! - ~/.config/clown/scripts/ for script overrides

use anyhow::Result;
use clown_core::ClownPaths;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Filesystem change event.
#[derive(Debug, Clone)]
pub enum ChangeEvent {
    /// Agent manifest added or modified.
    AgentChanged(String),
    /// Provider manifest added or modified.
    ProviderChanged(String),
    /// Profile changed.
    ProfileChanged(String),
    /// Script changed.
    ScriptChanged(String),
    /// Something was removed.
    Removed(String),
}

/// Filesystem watcher configuration.
pub struct FileWatcher {
    paths: ClownPaths,
}

impl FileWatcher {
    /// Create a new file watcher.
    pub fn new(paths: ClownPaths) -> Self {
        Self { paths }
    }

    /// Start watching and return a channel for events.
    pub fn start(&self) -> Result<mpsc::Receiver<ChangeEvent>> {
        let (tx, rx) = mpsc::channel();

        let config_dir = self.paths.config_dir.clone();
        let agents_d = self.paths.agents_d();
        let providers_d = self.paths.providers_d();
        let profiles_dir = self.paths.profiles_dir();
        let scripts_dir = self.paths.scripts_dir();

        // Spawn watcher thread
        std::thread::spawn(move || {
            let (event_tx, event_rx) = mpsc::channel();

            let mut watcher = match RecommendedWatcher::new(
                move |res: Result<notify::Event, notify::Error>| {
                    if let Ok(event) = res {
                        let _ = event_tx.send(event);
                    }
                },
                Config::default().with_poll_interval(Duration::from_secs(2)),
            ) {
                Ok(w) => w,
                Err(e) => {
                    warn!("Failed to create watcher: {}", e);
                    return;
                }
            };

            // Watch directories
            for dir in &[&agents_d, &providers_d, &profiles_dir, &scripts_dir] {
                if dir.exists() {
                    if let Err(e) = watcher.watch(dir, RecursiveMode::NonRecursive) {
                        warn!("Failed to watch {:?}: {}", dir, e);
                    } else {
                        debug!("Watching: {:?}", dir);
                    }
                }
            }

            info!("Filesystem watcher started");

            // Process events
            for event in event_rx {
                for path in event.paths {
                    if let Some(change) = classify_change(&path, &agents_d, &providers_d, &profiles_dir, &scripts_dir) {
                        debug!("Detected change: {:?}", change);
                        if tx.send(change).is_err() {
                            // Receiver dropped, stop watching
                            break;
                        }
                    }
                }
            }

            info!("Filesystem watcher stopped");
        });

        Ok(rx)
    }
}

/// Classify a filesystem change into a ChangeEvent.
fn classify_change(
    path: &Path,
    agents_d: &Path,
    providers_d: &Path,
    profiles_dir: &Path,
    scripts_dir: &Path,
) -> Option<ChangeEvent> {
    let filename = path.file_name()?.to_string_lossy().to_string();

    if path.starts_with(agents_d) {
        if filename.ends_with(".toml") {
            let id = filename.trim_end_matches(".toml").to_string();
            if path.exists() {
                return Some(ChangeEvent::AgentChanged(id));
            } else {
                return Some(ChangeEvent::Removed(format!("agent:{}", id)));
            }
        }
    }

    if path.starts_with(providers_d) {
        if filename.ends_with(".toml") {
            let id = filename.trim_end_matches(".toml").to_string();
            if path.exists() {
                return Some(ChangeEvent::ProviderChanged(id));
            } else {
                return Some(ChangeEvent::Removed(format!("provider:{}", id)));
            }
        }
    }

    if path.starts_with(profiles_dir) {
        if filename.ends_with(".json") {
            let alias = filename.trim_end_matches(".json").to_string();
            if path.exists() {
                return Some(ChangeEvent::ProfileChanged(alias));
            } else {
                return Some(ChangeEvent::Removed(format!("profile:{}", alias)));
            }
        }
    }

    if path.starts_with(scripts_dir) {
        if filename.ends_with(".rhai") {
            let name = filename.clone();
            if path.exists() {
                return Some(ChangeEvent::ScriptChanged(name));
            } else {
                return Some(ChangeEvent::Removed(format!("script:{}", name)));
            }
        }
    }

    None
}
