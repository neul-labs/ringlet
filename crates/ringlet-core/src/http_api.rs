//! Shared HTTP API contracts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ListProfilesQuery {
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RunRequest {
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RunResponse {
    Started { pid: u32 },
    Completed { exit_code: i32 },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AddHookRequest {
    pub event: String,
    pub matcher: String,
    pub command: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SyncRequest {
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub offline: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PinRequest {
    #[serde(rename = "ref")]
    pub ref_: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SetAliasRequest {
    pub to: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ListDirQuery {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ListDirResponse {
    pub path: String,
    pub parent: Option<String>,
    pub entries: Vec<DirEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PathCompleteQuery {
    pub prefix: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PathCompletion {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PathCompleteResponse {
    pub completions: Vec<PathCompletion>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GitInfoQuery {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GitCommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GitInfo {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub dirty: bool,
    pub remote_url: Option<String>,
    pub commits: Vec<GitCommitInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PingResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateTerminalSessionRequest {
    pub profile_alias: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_cols")]
    pub cols: u16,
    #[serde(default = "default_rows")]
    pub rows: u16,
    pub working_dir: Option<String>,
    #[serde(default)]
    pub no_sandbox: bool,
    pub bwrap_flags: Option<Vec<String>>,
    pub sandbox_exec_profile: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateTerminalSessionResponse {
    pub session_id: String,
    pub ws_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateShellRequest {
    pub shell: Option<String>,
    #[serde(default = "default_cols")]
    pub cols: u16,
    #[serde(default = "default_rows")]
    pub rows: u16,
    pub working_dir: Option<String>,
    #[serde(default)]
    pub no_sandbox: bool,
    pub bwrap_flags: Option<Vec<String>>,
    pub sandbox_exec_profile: Option<String>,
}

const fn default_cols() -> u16 {
    80
}

const fn default_rows() -> u16 {
    24
}
