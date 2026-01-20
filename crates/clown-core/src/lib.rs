//! Core types and utilities for the clown CLI orchestrator.
//!
//! This crate provides shared types used by both the CLI (`clown`) and
//! the daemon (`clownd`), including:
//!
//! - Agent manifest definitions
//! - Provider manifest definitions
//! - Profile types
//! - User configuration
//! - RPC message types
//! - Platform-aware path handling
//! - Error types

pub mod agent;
pub mod binary;
pub mod config;
pub mod error;
pub mod events;
pub mod hooks;
pub mod paths;
pub mod profile;
pub mod provider;
pub mod proxy;
pub mod rpc;
pub mod usage;

pub use agent::{AgentInfo, AgentManifest, ProviderCompatibility};
pub use binary::{BinaryConfig, BinaryPaths};
pub use config::UserConfig;
pub use error::{ClownError, Result};
pub use events::{ClientMessage, Event, ServerMessage};
pub use hooks::{HookAction, HookRule, HooksConfig};
pub use paths::{expand_template, expand_tilde, home_dir, ClownPaths};
pub use profile::{Profile, ProfileCreateRequest, ProfileInfo, ProfileMetadata};
pub use provider::{ProviderInfo, ProviderManifest, ProviderType};
pub use proxy::{
    ModelTarget, ProfileProxyConfig, ProxyInstanceInfo, ProxyStatus, RoutingCondition,
    RoutingConfig, RoutingRule, RoutingStrategy,
};
pub use rpc::{RegistryStatus, Request, Response, StatsResponse, UsageStatsResponse};
pub use usage::{
    AgentType, CostBreakdown, DailyUsage, LiteLLMModelPricing, ModelUsage, ProfileUsage,
    SessionUsage, TokenUsage, UsageAggregates, UsagePeriod, UsageResponse,
};

/// Clown version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name.
pub const APP_NAME: &str = "clown";

/// Daemon name.
pub const DAEMON_NAME: &str = "clownd";
