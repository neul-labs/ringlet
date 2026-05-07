//! Usage-related request handlers.
//!
//! Handles token/cost usage queries and Claude data import.
//! Integrates with agent_usage module to scan native files from
//! Claude, Codex, and OpenCode agents.

use crate::daemon::agent_usage;
use crate::daemon::server::ServerState;
use chrono::{Datelike, Duration, NaiveDate, Utc};
use ringlet_core::rpc::error_codes;
use ringlet_core::{
    AgentUsage, CostBreakdown, DailyUsage, ModelUsage, Response, TokenUsage, UsageAggregates,
    UsagePeriod, UsageStatsResponse,
};
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
    let period_range = match period_range(&period) {
        Ok(range) => range,
        Err(message) => {
            return Response::error(error_codes::INTERNAL_ERROR, message);
        }
    };

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

    match state.telemetry.load_all_sessions() {
        Ok(all_sessions) => {
            let filtered_sessions: Vec<_> = all_sessions
                .into_iter()
                .filter(|session| {
                    matches_period(
                        session.ended_at.unwrap_or(session.started_at).date_naive(),
                        period_range,
                    ) && profile.is_none_or(|alias| session.profile == alias)
                        && model.is_none_or(|session_model| {
                            session.model.as_deref() == Some(session_model)
                        })
                })
                .collect();

            let telemetry_aggregates =
                crate::daemon::telemetry::TelemetryCollector::aggregate_sessions(
                    &filtered_sessions,
                );
            let mut aggregates = convert_to_usage_aggregates(&telemetry_aggregates);

            if let Some(scan) = agent_scan {
                let filtered_entries = scan
                    .entries
                    .into_iter()
                    .filter(|entry| {
                        // Native agent files currently expose agent-local project/session IDs,
                        // not Ringlet profile aliases, so profile-filtered usage must remain
                        // telemetry-only until Ringlet owns a stable cross-system join key.
                        profile.is_none()
                            && matches_period(entry.timestamp.date_naive(), period_range)
                            && model.is_none_or(|model_filter| entry.model == model_filter)
                    })
                    .collect::<Vec<_>>();
                merge_agent_scan_entries(&mut aggregates, &filtered_entries);
            }

            Response::Usage(Box::new(UsageStatsResponse {
                period: period_desc,
                total_tokens: aggregates.total_tokens.clone(),
                total_cost: aggregates.total_cost.clone(),
                total_sessions: telemetry_aggregates.total_sessions,
                total_runtime_secs: telemetry_aggregates.total_runtime_secs,
                aggregates,
            }))
        }
        Err(e) => Response::error(
            error_codes::INTERNAL_ERROR,
            format!("Failed to get usage: {}", e),
        ),
    }
}

/// Merge filtered agent-native usage data into usage aggregates.
fn merge_agent_scan_entries(aggregates: &mut UsageAggregates, entries: &[agent_usage::UsageEntry]) {
    for entry in entries {
        let model_usage = aggregates
            .by_model
            .entry(entry.model.clone())
            .or_insert_with(|| ModelUsage {
                model: entry.model.clone(),
                tokens: TokenUsage::default(),
                cost: None,
                sessions: 0,
            });
        model_usage.tokens += entry.tokens.clone();
        model_usage.sessions += 1;

        if let Some(cost_usd) = entry.cost_usd {
            add_cost(&mut model_usage.cost, cost_usd);
        }

        let date_key = entry.timestamp.date_naive().to_string();
        let daily_usage = aggregates
            .by_date
            .entry(date_key.clone())
            .or_insert_with(|| DailyUsage {
                date: date_key,
                ..Default::default()
            });
        daily_usage.tokens += entry.tokens.clone();
        daily_usage.sessions += 1;
        if let Some(cost_usd) = entry.cost_usd {
            add_cost(&mut daily_usage.cost, cost_usd);
        }

        let agent_id = entry.agent.to_string();
        let agent_usage = aggregates
            .by_agent
            .entry(agent_id.clone())
            .or_insert_with(|| AgentUsage {
                agent: agent_id,
                ..Default::default()
            });
        agent_usage.tokens += entry.tokens.clone();
        agent_usage.sessions += 1;
        if let Some(cost_usd) = entry.cost_usd {
            add_cost(&mut agent_usage.cost, cost_usd);
        }

        aggregates.total_tokens += entry.tokens.clone();
        if let Some(cost_usd) = entry.cost_usd {
            add_cost(&mut aggregates.total_cost, cost_usd);
        }
    }
}

/// Import usage data from Claude's native files.
pub async fn import_claude(claude_dir: Option<&PathBuf>, _state: &ServerState) -> Response {
    let claude_home = claude_dir
        .cloned()
        .or_else(crate::daemon::claude_import::default_claude_dir);

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

    match crate::daemon::claude_import::import_all(&claude_path) {
        Ok(result) => {
            let mut message = format!(
                "Imported {} input tokens, {} output tokens from stats-cache.json",
                result.stats_cache_tokens.input_tokens, result.stats_cache_tokens.output_tokens
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
        Err(e) => Response::error(error_codes::INTERNAL_ERROR, format!("Import failed: {}", e)),
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
    aggregates: &crate::daemon::telemetry::Aggregates,
) -> UsageAggregates {
    UsageAggregates {
        total_tokens: aggregates.total_tokens.clone(),
        total_cost: aggregates.total_cost.clone(),
        by_date: aggregates.by_date.clone(),
        by_model: aggregates.by_model.clone(),
        by_profile: aggregates.by_profile.clone(),
        by_agent: aggregates
            .by_agent
            .iter()
            .map(|(agent, stats)| {
                (
                    agent.clone(),
                    AgentUsage {
                        agent: agent.clone(),
                        tokens: stats.tokens.clone(),
                        cost: stats.cost.clone(),
                        sessions: stats.sessions,
                        runtime_secs: stats.runtime_secs,
                    },
                )
            })
            .collect(),
    }
}

fn period_range(period: &UsagePeriod) -> Result<Option<(NaiveDate, NaiveDate)>, String> {
    let today = Utc::now().date_naive();

    match period {
        UsagePeriod::Today => Ok(Some((today, today))),
        UsagePeriod::Yesterday => {
            let yesterday = today - Duration::days(1);
            Ok(Some((yesterday, yesterday)))
        }
        UsagePeriod::ThisWeek => {
            let start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
            Ok(Some((start, today)))
        }
        UsagePeriod::ThisMonth => {
            let start = today
                .with_day(1)
                .ok_or_else(|| "Failed to determine start of current month".to_string())?;
            Ok(Some((start, today)))
        }
        UsagePeriod::Last7Days => Ok(Some((today - Duration::days(6), today))),
        UsagePeriod::Last30Days => Ok(Some((today - Duration::days(29), today))),
        UsagePeriod::DateRange { start, end } => {
            let start = NaiveDate::parse_from_str(start, "%Y-%m-%d")
                .map_err(|err| format!("Invalid usage period start date '{}': {}", start, err))?;
            let end = NaiveDate::parse_from_str(end, "%Y-%m-%d")
                .map_err(|err| format!("Invalid usage period end date '{}': {}", end, err))?;
            if start > end {
                return Err(format!(
                    "Invalid usage period range: start date {} is after end date {}",
                    start, end
                ));
            }
            Ok(Some((start, end)))
        }
        UsagePeriod::All => Ok(None),
    }
}

fn matches_period(date: NaiveDate, range: Option<(NaiveDate, NaiveDate)>) -> bool {
    match range {
        Some((start, end)) => date >= start && date <= end,
        None => true,
    }
}

fn add_cost(cost: &mut Option<CostBreakdown>, total_cost: f64) {
    if let Some(existing) = cost {
        existing.total_cost += total_cost;
    } else {
        *cost = Some(CostBreakdown {
            total_cost,
            ..Default::default()
        });
    }
}
