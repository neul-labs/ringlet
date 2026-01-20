//! Usage HTTP handlers.

use crate::handlers;
use crate::http::error::{ApiResponse, HttpError};
use crate::server::ServerState;
use axum::{
    extract::{Query, State},
    Json,
};
use clown_core::{Response, UsagePeriod, UsageStatsResponse};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    /// Time period filter
    pub period: Option<String>,
    /// Filter by profile
    pub profile: Option<String>,
    /// Filter by model
    pub model: Option<String>,
}

/// Parse period string into UsagePeriod enum.
fn parse_period(s: &str) -> UsagePeriod {
    match s.to_lowercase().as_str() {
        "today" => UsagePeriod::Today,
        "yesterday" => UsagePeriod::Yesterday,
        "week" | "this_week" | "thisweek" => UsagePeriod::ThisWeek,
        "month" | "this_month" | "thismonth" => UsagePeriod::ThisMonth,
        "7d" | "last7days" | "last_7_days" => UsagePeriod::Last7Days,
        "30d" | "last30days" | "last_30_days" => UsagePeriod::Last30Days,
        "all" => UsagePeriod::All,
        _ => UsagePeriod::Today,
    }
}

/// GET /api/usage - Get usage statistics.
pub async fn get_usage(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<UsageQuery>,
) -> Result<Json<ApiResponse<UsageStatsResponse>>, HttpError> {
    let period = query.period.as_deref().map(parse_period);
    let response = handlers::usage::get_usage(
        period.as_ref(),
        query.profile.as_deref(),
        query.model.as_deref(),
        &state,
    )
    .await;

    match response {
        Response::Usage(usage) => Ok(Json(ApiResponse::success(usage))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}

#[derive(Debug, Deserialize)]
pub struct ImportClaudeQuery {
    /// Path to Claude home directory
    pub claude_dir: Option<PathBuf>,
}

/// POST /api/usage/import-claude - Import usage from Claude's native files.
pub async fn import_claude(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ImportClaudeQuery>,
) -> Result<Json<ApiResponse<String>>, HttpError> {
    let response = handlers::usage::import_claude(query.claude_dir.as_ref(), &state).await;

    match response {
        Response::Success { message } => Ok(Json(ApiResponse::success(message))),
        Response::Error { code, message } => Err(HttpError::new(code, message)),
        _ => Err(HttpError::internal("Unexpected response type")),
    }
}
