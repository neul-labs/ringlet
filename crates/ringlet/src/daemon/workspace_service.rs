//! Local workspace inspection service for filesystem and git data.

use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Debug)]
pub enum WorkspaceError {
    NotFound(String),
    Invalid(String),
    Internal(String),
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(message) => write!(f, "{}", message),
            Self::Invalid(message) => write!(f, "{}", message),
            Self::Internal(message) => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for WorkspaceError {}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub struct DirectoryListing {
    pub path: String,
    pub parent: Option<String>,
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Clone)]
pub struct PathMatch {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub struct PathCompletions {
    pub completions: Vec<PathMatch>,
}

#[derive(Debug, Clone)]
pub struct GitCommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct GitRepositoryInfo {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub dirty: bool,
    pub remote_url: Option<String>,
    pub commits: Vec<GitCommitInfo>,
}

pub struct WorkspaceService;

impl WorkspaceService {
    pub fn new() -> Self {
        Self
    }

    pub fn list_directory(
        &self,
        requested_path: Option<&Path>,
    ) -> Result<DirectoryListing, WorkspaceError> {
        let requested_path = requested_path
            .map(PathBuf::from)
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));
        let path = validate_existing_path(&requested_path)?;

        if !path.is_dir() {
            return Err(WorkspaceError::Invalid(format!(
                "Not a directory: {}",
                path.display()
            )));
        }

        let read_dir = std::fs::read_dir(&path)
            .map_err(|e| WorkspaceError::Internal(format!("Failed to read directory: {}", e)))?;

        let mut entries = Vec::new();
        for entry in read_dir {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }

            entries.push(DirectoryEntry {
                name,
                path: entry.path().to_string_lossy().to_string(),
                is_dir: file_type.is_dir(),
            });
        }

        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(DirectoryListing {
            path: path.to_string_lossy().to_string(),
            parent: path.parent().map(|p| p.to_string_lossy().to_string()),
            entries,
        })
    }

    pub fn complete_paths(&self, prefix: &str) -> Result<PathCompletions, WorkspaceError> {
        let prefix_path = PathBuf::from(prefix);
        let (parent_dir, partial) =
            if prefix.ends_with('/') || prefix.ends_with(std::path::MAIN_SEPARATOR) {
                (prefix_path.clone(), String::new())
            } else {
                let parent = prefix_path
                    .parent()
                    .unwrap_or_else(|| Path::new("/"))
                    .to_path_buf();
                let partial = prefix_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                (parent, partial)
            };

        let parent = validate_existing_path(&parent_dir)?;
        if !parent.is_dir() {
            return Ok(PathCompletions {
                completions: Vec::new(),
            });
        }

        let read_dir = std::fs::read_dir(&parent)
            .map_err(|e| WorkspaceError::Internal(format!("Failed to read directory: {}", e)))?;

        let partial_lower = partial.to_lowercase();
        let mut completions = Vec::new();

        for entry in read_dir {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };

            if !file_type.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }

            if !partial_lower.is_empty() && !name.to_lowercase().starts_with(&partial_lower) {
                continue;
            }

            completions.push(PathMatch {
                path: entry.path().to_string_lossy().to_string(),
                name,
                is_dir: true,
            });

            if completions.len() >= 20 {
                break;
            }
        }

        completions.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(PathCompletions { completions })
    }

    pub async fn git_info(
        &self,
        requested_path: &Path,
    ) -> Result<GitRepositoryInfo, WorkspaceError> {
        let path = validate_existing_path(requested_path)?;
        let path_str = path.to_string_lossy().to_string();

        let is_repo = git_cmd(&path_str, &["rev-parse", "--is-inside-work-tree"])
            .await
            .map(|s| s == "true")
            .unwrap_or(false);

        if !is_repo {
            return Ok(GitRepositoryInfo {
                is_repo: false,
                branch: None,
                dirty: false,
                remote_url: None,
                commits: Vec::new(),
            });
        }

        let (branch, status, log, remote) = tokio::join!(
            git_cmd(&path_str, &["rev-parse", "--abbrev-ref", "HEAD"]),
            git_cmd(&path_str, &["status", "--porcelain"]),
            git_cmd(&path_str, &["log", "--format=%h|%s|%an|%aI", "-n", "10"]),
            git_cmd(&path_str, &["remote", "get-url", "origin"]),
        );

        let dirty = status.map(|s| !s.is_empty()).unwrap_or(false);

        let commits = log
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                if parts.len() == 4 {
                    Some(GitCommitInfo {
                        hash: parts[0].to_string(),
                        message: parts[1].to_string(),
                        author: parts[2].to_string(),
                        date: parts[3].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(GitRepositoryInfo {
            is_repo: true,
            branch,
            dirty,
            remote_url: remote,
            commits,
        })
    }
}

fn validate_existing_path(path: &Path) -> Result<PathBuf, WorkspaceError> {
    path.canonicalize().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            WorkspaceError::NotFound(format!("Path not found: {}", path.display()))
        } else {
            WorkspaceError::Internal(format!("Failed to access path: {}", e))
        }
    })
}

async fn git_cmd(path: &str, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path);
    for arg in args {
        cmd.arg(arg);
    }
    let output = cmd.output().await.ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_directory_hides_dotfiles_and_sorts_directories_first() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::create_dir(root.join("zeta")).unwrap();
        std::fs::create_dir(root.join("alpha")).unwrap();
        std::fs::write(root.join("beta.txt"), "file").unwrap();
        std::fs::write(root.join(".hidden"), "secret").unwrap();

        let listing = WorkspaceService::new().list_directory(Some(root)).unwrap();
        let names = listing
            .entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["alpha", "zeta", "beta.txt"]);
        assert_eq!(listing.path, root.canonicalize().unwrap().to_string_lossy());
        assert!(listing.parent.is_some());
    }

    #[test]
    fn complete_paths_returns_matching_directories_only() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::create_dir(root.join("ringlet")).unwrap();
        std::fs::create_dir(root.join("rust")).unwrap();
        std::fs::write(root.join("readme.md"), "file").unwrap();

        let prefix = root.join("r").to_string_lossy().to_string();
        let completions = WorkspaceService::new().complete_paths(&prefix).unwrap();
        let names = completions
            .completions
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["ringlet", "rust"]);
        assert!(completions.completions.iter().all(|entry| entry.is_dir));
    }

    #[tokio::test]
    async fn git_info_for_non_repository_returns_empty_result() {
        let temp = tempfile::tempdir().unwrap();

        let info = WorkspaceService::new().git_info(temp.path()).await.unwrap();

        assert!(!info.is_repo);
        assert!(info.branch.is_none());
        assert!(!info.dirty);
        assert!(info.remote_url.is_none());
        assert!(info.commits.is_empty());
    }
}
