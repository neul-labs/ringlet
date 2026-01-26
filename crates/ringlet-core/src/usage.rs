//! Usage tracking types for token and cost tracking.
//!
//! This module provides types for tracking API usage:
//! - Token usage (always tracked for all profiles)
//! - Cost breakdown (only calculated for "self" provider profiles)
//! - Aggregated usage statistics
//! - Multi-agent support (Claude, Codex, OpenCode)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::AddAssign;

/// Supported agent types for usage tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    /// Claude Code CLI.
    Claude,
    /// OpenAI Codex CLI.
    Codex,
    /// OpenCode editor.
    OpenCode,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Claude => write!(f, "claude"),
            AgentType::Codex => write!(f, "codex"),
            AgentType::OpenCode => write!(f, "opencode"),
        }
    }
}

/// Token usage for a session or aggregated period.
///
/// Always tracked regardless of provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TokenUsage {
    /// Input tokens (prompt).
    pub input_tokens: u64,
    /// Output tokens (completion).
    pub output_tokens: u64,
    /// Cache creation input tokens (Anthropic-specific).
    pub cache_creation_input_tokens: u64,
    /// Cache read input tokens (Anthropic-specific).
    pub cache_read_input_tokens: u64,
}

impl TokenUsage {
    /// Create a new TokenUsage with all zeros.
    pub fn new() -> Self {
        Self::default()
    }

    /// Total tokens (input + output).
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    /// Total input tokens including cache operations.
    pub fn total_input(&self) -> u64 {
        self.input_tokens + self.cache_creation_input_tokens + self.cache_read_input_tokens
    }
}

impl AddAssign for TokenUsage {
    fn add_assign(&mut self, other: Self) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_creation_input_tokens += other.cache_creation_input_tokens;
        self.cache_read_input_tokens += other.cache_read_input_tokens;
    }
}

/// Cost breakdown for usage.
///
/// Only calculated for profiles using the "self" provider (direct API keys).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CostBreakdown {
    /// Input token cost in USD.
    pub input_cost: f64,
    /// Output token cost in USD.
    pub output_cost: f64,
    /// Cache creation cost in USD.
    pub cache_creation_cost: f64,
    /// Cache read cost in USD.
    pub cache_read_cost: f64,
    /// Total cost in USD.
    pub total_cost: f64,
}

impl CostBreakdown {
    /// Create a new CostBreakdown with all zeros.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate from token usage and per-token prices.
    pub fn from_tokens(
        tokens: &TokenUsage,
        input_cost_per_token: f64,
        output_cost_per_token: f64,
        cache_creation_cost_per_token: f64,
        cache_read_cost_per_token: f64,
    ) -> Self {
        let input_cost = tokens.input_tokens as f64 * input_cost_per_token;
        let output_cost = tokens.output_tokens as f64 * output_cost_per_token;
        let cache_creation_cost =
            tokens.cache_creation_input_tokens as f64 * cache_creation_cost_per_token;
        let cache_read_cost = tokens.cache_read_input_tokens as f64 * cache_read_cost_per_token;

        Self {
            input_cost,
            output_cost,
            cache_creation_cost,
            cache_read_cost,
            total_cost: input_cost + output_cost + cache_creation_cost + cache_read_cost,
        }
    }
}

impl AddAssign for CostBreakdown {
    fn add_assign(&mut self, other: Self) {
        self.input_cost += other.input_cost;
        self.output_cost += other.output_cost;
        self.cache_creation_cost += other.cache_creation_cost;
        self.cache_read_cost += other.cache_read_cost;
        self.total_cost += other.total_cost;
    }
}

/// LiteLLM model pricing entry.
///
/// Parsed from LiteLLM's model_prices_and_context_window.json.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LiteLLMModelPricing {
    /// Cost per input token.
    pub input_cost_per_token: Option<f64>,
    /// Cost per output token.
    pub output_cost_per_token: Option<f64>,
    /// Cost per cache creation input token.
    pub cache_creation_input_token_cost: Option<f64>,
    /// Cost per cache read input token.
    pub cache_read_input_token_cost: Option<f64>,
    /// Maximum input tokens supported.
    pub max_input_tokens: Option<u64>,
    /// Maximum output tokens supported.
    pub max_output_tokens: Option<u64>,
    /// LiteLLM provider name.
    pub litellm_provider: Option<String>,
    /// Whether the model supports prompt caching.
    pub supports_prompt_caching: Option<bool>,
}

impl LiteLLMModelPricing {
    /// Calculate cost breakdown from token usage.
    pub fn calculate_cost(&self, tokens: &TokenUsage) -> CostBreakdown {
        CostBreakdown::from_tokens(
            tokens,
            self.input_cost_per_token.unwrap_or(0.0),
            self.output_cost_per_token.unwrap_or(0.0),
            self.cache_creation_input_token_cost.unwrap_or(0.0),
            self.cache_read_input_token_cost.unwrap_or(0.0),
        )
    }
}

/// Usage period for queries.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UsagePeriod {
    /// Today only.
    #[default]
    Today,
    /// Yesterday only.
    Yesterday,
    /// Current week (Monday to Sunday).
    ThisWeek,
    /// Current month.
    ThisMonth,
    /// Last 7 days.
    Last7Days,
    /// Last 30 days.
    Last30Days,
    /// Custom date range.
    DateRange {
        start: String,
        end: String,
    },
    /// All time.
    All,
}

/// Daily usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyUsage {
    /// Date string (YYYY-MM-DD).
    pub date: String,
    /// Token usage for the day.
    pub tokens: TokenUsage,
    /// Cost breakdown (None if no "self" provider usage).
    pub cost: Option<CostBreakdown>,
    /// Number of sessions.
    pub sessions: u64,
}

/// Per-model usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Model identifier.
    pub model: String,
    /// Token usage.
    pub tokens: TokenUsage,
    /// Cost breakdown (None if no "self" provider usage).
    pub cost: Option<CostBreakdown>,
    /// Number of sessions.
    pub sessions: u64,
}

/// Per-profile usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileUsage {
    /// Profile alias.
    pub profile: String,
    /// Provider ID.
    pub provider_id: String,
    /// Token usage.
    pub tokens: TokenUsage,
    /// Cost breakdown (None if provider != "self").
    pub cost: Option<CostBreakdown>,
    /// Number of sessions.
    pub sessions: u64,
    /// Total runtime in seconds.
    pub runtime_secs: u64,
    /// Last used timestamp.
    pub last_used: Option<DateTime<Utc>>,
}

/// Aggregated usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageAggregates {
    /// Total token usage.
    pub total_tokens: TokenUsage,
    /// Total cost (only from "self" provider profiles).
    pub total_cost: Option<CostBreakdown>,
    /// Usage by date (YYYY-MM-DD).
    pub by_date: HashMap<String, DailyUsage>,
    /// Usage by model.
    pub by_model: HashMap<String, ModelUsage>,
    /// Usage by profile.
    pub by_profile: HashMap<String, ProfileUsage>,
}

/// Usage query response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    /// Period description.
    pub period: String,
    /// Aggregated usage data.
    pub aggregates: UsageAggregates,
    /// Recent sessions (optional, for detailed view).
    pub recent_sessions: Option<Vec<SessionUsage>>,
}

/// Session usage record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUsage {
    /// Session ID.
    pub session_id: String,
    /// Profile alias.
    pub profile: String,
    /// Agent ID.
    pub agent_id: String,
    /// Provider ID.
    pub provider_id: String,
    /// Model used.
    pub model: Option<String>,
    /// Token usage.
    pub tokens: TokenUsage,
    /// Cost breakdown (None if provider != "self").
    pub cost: Option<CostBreakdown>,
    /// Session timestamp.
    pub timestamp: DateTime<Utc>,
    /// Duration in seconds.
    pub duration_secs: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_add() {
        let mut a = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: 10,
            cache_read_input_tokens: 5,
        };
        let b = TokenUsage {
            input_tokens: 200,
            output_tokens: 100,
            cache_creation_input_tokens: 20,
            cache_read_input_tokens: 10,
        };
        a += b;
        assert_eq!(a.input_tokens, 300);
        assert_eq!(a.output_tokens, 150);
        assert_eq!(a.cache_creation_input_tokens, 30);
        assert_eq!(a.cache_read_input_tokens, 15);
    }

    #[test]
    fn test_cost_calculation() {
        let tokens = TokenUsage {
            input_tokens: 1_000_000, // 1M tokens
            output_tokens: 500_000,
            cache_creation_input_tokens: 100_000,
            cache_read_input_tokens: 200_000,
        };

        // Claude Sonnet 4 pricing (per token)
        let cost = CostBreakdown::from_tokens(
            &tokens,
            0.000003,   // $3/MTok input
            0.000015,   // $15/MTok output
            0.00000375, // $3.75/MTok cache creation
            0.0000003,  // $0.30/MTok cache read
        );

        assert!((cost.input_cost - 3.0).abs() < 0.001);
        assert!((cost.output_cost - 7.5).abs() < 0.001);
        assert!((cost.cache_creation_cost - 0.375).abs() < 0.001);
        assert!((cost.cache_read_cost - 0.06).abs() < 0.001);
        assert!((cost.total_cost - 10.935).abs() < 0.001);
    }

    #[test]
    fn test_litellm_pricing_calculate() {
        let pricing = LiteLLMModelPricing {
            input_cost_per_token: Some(0.000003),
            output_cost_per_token: Some(0.000015),
            cache_creation_input_token_cost: Some(0.00000375),
            cache_read_input_token_cost: Some(0.0000003),
            ..Default::default()
        };

        let tokens = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };

        let cost = pricing.calculate_cost(&tokens);
        assert!((cost.input_cost - 0.003).abs() < 0.0001);
        assert!((cost.output_cost - 0.0075).abs() < 0.0001);
    }
}
