//! Usage-related request handlers.
//!
//! Handles token/cost usage queries and Claude data import.
//! Integrates with agent_usage module to scan native files from
//! Claude, Codex, and OpenCode agents.

use crate::agent_usage;
use crate::server::ServerState;
use ringlet_core::rpc::error_codes;
use ringlet_core::{
    Response, TokenUsage, UsageAggregates, UsagePeriod, UsageStatsResponse,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Get token/cost usage statistics.
///
/// Merges data from multiple sources:
/// 1. Agent native files (Claude, Codex, OpenCode)
/// 2. Telemetry (session counts, runtime)
pub async fn get_usage(
    period: Option<&UsagePeriod>,
    profile: Option<&str>,
    model: Option<&str>,
    state: &ServerState,
) -> Response {
    let period = period.cloned().unwrap_or_default();
    let period_desc = format_period(&period);

    debug!(
        "Getting usage for period={:?}, profile={:?}, model={:?}",
        period, profile, model
    );

    // Scan agent native files for usage data
    let agent_scan = match agent_usage::scan_all_agents().await {
        Ok(result) => {
            if !result.warnings.is_empty() {
                for warning in &result.warnings {
                    warn!("Agent scan warning: {}", warning);
                }
            }
            debug!(
                "Scanned {} entries from agent native files",
                result.total_entries()
            );
            Some(result)
        }
        Err(e) => {
            warn!("Failed to scan agent native files: {}", e);
            None
        }
    };

    // Load aggregates from telemetry
    match state.telemetry.load_aggregates() {
        Ok(telemetry_aggregates) => {
            // Convert telemetry aggregates and merge with agent scan data
            let mut aggregates = convert_to_usage_aggregates(&telemetry_aggregates);

            // Merge agent scan data
            if let Some(scan) = agent_scan {
                merge_agent_scan_data(&mut aggregates, &scan);
            }

            // Filter by profile if specified
            let filtered_aggregates = if let Some(profile_filter) = profile {
                filter_aggregates_by_profile(&aggregates, profile_filter)
            } else {
                aggregates.clone()
            };

            // Calculate totals
            let (total_tokens, total_cost) = calculate_totals(&filtered_aggregates);

            Response::Usage(UsageStatsResponse {
                period: period_desc,
                total_tokens,
                total_cost,
                total_sessions: telemetry_aggregates.total_sessions,
                total_runtime_secs: telemetry_aggregates.total_runtime_secs,
                aggregates: filtered_aggregates,
            })
        }
        Err(e) => Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Failed to get usage: {}", e),
        ),
    }
}

/// Merge agent scan data into usage aggregates.
fn merge_agent_scan_data(
    aggregates: &mut UsageAggregates,
    scan: &agent_usage::ScanResult,
) {
    use ringlet_core::usage::ModelUsage;

    // Aggregate by model
    for entry in &scan.entries {
        let model_usage = aggregates.by_model.entry(entry.model.clone()).or_insert_with(|| {
            ModelUsage {
                model: entry.model.clone(),
                tokens: TokenUsage::default(),
                cost: None,
                sessions: 0,
            }
        });
        model_usage.tokens += entry.tokens.clone();
        model_usage.sessions += 1;

        // Add cost if available
        if let Some(cost_usd) = entry.cost_usd {
            if let Some(ref mut cost) = model_usage.cost {
                cost.total_cost += cost_usd;
            } else {
                model_usage.cost = Some(ringlet_core::CostBreakdown {
                    total_cost: cost_usd,
                    ..Default::default()
                });
            }
        }
    }

    // Update totals from agent data
    for entry in &scan.entries {
        aggregates.total_tokens += entry.tokens.clone();
        if let Some(cost_usd) = entry.cost_usd {
            if let Some(ref mut total_cost) = aggregates.total_cost {
                total_cost.total_cost += cost_usd;
            } else {
                aggregates.total_cost = Some(ringlet_core::CostBreakdown {
                    total_cost: cost_usd,
                    ..Default::default()
                });
            }
        }
    }
}

/// Calculate totals from filtered aggregates.
fn calculate_totals(
    aggregates: &UsageAggregates,
) -> (TokenUsage, Option<ringlet_core::CostBreakdown>) {
    let mut total_tokens = TokenUsage::default();
    let mut total_cost = None;

    for profile_usage in aggregates.by_profile.values() {
        total_tokens += profile_usage.tokens.clone();
        if let Some(ref cost) = profile_usage.cost {
            if let Some(ref mut tc) = total_cost {
                *tc += cost.clone();
            } else {
                total_cost = Some(cost.clone());
            }
        }
    }

    // Also include model-level data that might not be attributed to profiles
    for model_usage in aggregates.by_model.values() {
        // Note: We're already counting tokens in profile aggregates,
        // so we don't double-count here. This is for models not attributed to profiles.
    }

    (total_tokens, total_cost)
}

/// Filter aggregates by profile.
fn filter_aggregates_by_profile(
    aggregates: &UsageAggregates,
    profile: &str,
) -> UsageAggregates {
    UsageAggregates {
        total_tokens: TokenUsage::default(), // Will be recalculated
        total_cost: None,
        by_date: HashMap::new(),
        by_model: HashMap::new(),
        by_profile: aggregates
            .by_profile
            .iter()
            .filter(|(k, _)| k.as_str() == profile)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    }
}

/// Import usage data from Claude's native files.
pub async fn import_claude(
    claude_dir: Option<&PathBuf>,
    _state: &ServerState,
) -> Response {
    let claude_home = claude_dir
        .cloned()
        .or_else(crate::claude_import::default_claude_dir);

    let Some(claude_path) = claude_home else {
        return Response::error(
            error_codes::INTERNAL_ERROR,
            "Could not determine Claude home directory",
        );
    };

    if !claude_path.exists() {
        return Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Claude directory not found: {:?}", claude_path),
        );
    }

    info!("Importing Claude usage data from {:?}", claude_path);

    match crate::claude_import::import_all(&claude_path) {
        Ok(result) => {
            let mut message = format!(
                "Imported {} input tokens, {} output tokens from stats-cache.json",
                result.stats_cache_tokens.input_tokens,
                result.stats_cache_tokens.output_tokens
            );

            if result.sessions_imported > 0 {
                message.push_str(&format!(
                    ". Found {} session entries from JSONL files",
                    result.sessions_imported
                ));
            }

            if !result.warnings.is_empty() {
                message.push_str(&format!(". Warnings: {}", result.warnings.join("; ")));
            }

            Response::success(message)
        }
        Err(e) => {
            Response::error(error_codes::INTERNAL_ERROR, format!("Import failed: {}", e))
        }
    }
}

/// Format period for display.
fn format_period(period: &UsagePeriod) -> String {
    match period {
        UsagePeriod::Today => "Today".to_string(),
        UsagePeriod::Yesterday => "Yesterday".to_string(),
        UsagePeriod::ThisWeek => "This week".to_string(),
        UsagePeriod::ThisMonth => "This month".to_string(),
        UsagePeriod::Last7Days => "Last 7 days".to_string(),
        UsagePeriod::Last30Days => "Last 30 days".to_string(),
        UsagePeriod::DateRange { start, end } => format!("{} to {}", start, end),
        UsagePeriod::All => "All time".to_string(),
    }
}

/// Convert telemetry Aggregates to UsageAggregates.
fn convert_to_usage_aggregates(
    aggregates: &crate::telemetry::Aggregates,
) -> UsageAggregates {
    use ringlet_core::usage::ProfileUsage;
    use std::collections::HashMap;

    let by_profile: HashMap<String, ProfileUsage> = aggregates
        .by_profile
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                ProfileUsage {
                    profile: k.clone(),
                    provider_id: String::new(), // TODO: Get from profile
                    tokens: v.tokens.clone(),
                    cost: v.cost.clone(),
                    sessions: v.sessions,
                    runtime_secs: v.runtime_secs,
                    last_used: v.last_used,
                },
            )
        })
        .collect();

    UsageAggregates {
        total_tokens: aggregates.total_tokens.clone(),
        total_cost: aggregates.total_cost.clone(),
        by_date: HashMap::new(), // TODO: Implement date-based tracking
        by_model: HashMap::new(), // TODO: Implement model-based tracking
        by_profile,
    }
}

/// Filter aggregates by profile.
fn filter_by_profile(
    aggregates: &crate::telemetry::Aggregates,
    profile: &str,
) -> UsageAggregates {
    use ringlet_core::usage::ProfileUsage;
    use std::collections::HashMap;

    let by_profile: HashMap<String, ProfileUsage> = aggregates
        .by_profile
        .iter()
        .filter(|(k, _)| k.as_str() == profile)
        .map(|(k, v)| {
            (
                k.clone(),
                ProfileUsage {
                    profile: k.clone(),
                    provider_id: String::new(),
                    tokens: v.tokens.clone(),
                    cost: v.cost.clone(),
                    sessions: v.sessions,
                    runtime_secs: v.runtime_secs,
                    last_used: v.last_used,
                },
            )
        })
        .collect();

    // Calculate totals from filtered profiles
    let mut total_tokens = TokenUsage::default();
    let mut total_cost = None;
    let mut total_sessions = 0u64;
    let mut total_runtime = 0u64;

    for profile_usage in by_profile.values() {
        total_tokens += profile_usage.tokens.clone();
        total_sessions += profile_usage.sessions;
        total_runtime += profile_usage.runtime_secs;
        if let Some(ref cost) = profile_usage.cost {
            if let Some(ref mut tc) = total_cost {
                *tc += cost.clone();
            } else {
                total_cost = Some(cost.clone());
            }
        }
    }

    UsageAggregates {
        total_tokens,
        total_cost,
        by_date: HashMap::new(),
        by_model: HashMap::new(),
        by_profile,
    }
}

