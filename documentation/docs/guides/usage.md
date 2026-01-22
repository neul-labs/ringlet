# Usage Tracking

The usage tracking feature provides native token and cost monitoring for all agent sessions. It tracks token consumption across all profiles and calculates costs for profiles using direct API keys.

---

## Overview

Usage tracking captures:

- **Token Usage**: Input tokens, output tokens, cache creation tokens, and cache read tokens
- **Cost Calculation**: Estimated costs based on LiteLLM's pricing database
- **Session Metrics**: Session counts, runtime durations, and last-used timestamps
- **Aggregations**: Breakdowns by profile, model, and date

### Key Concepts

| Concept | Description |
|---------|-------------|
| Token Tracking | Always enabled for all profiles |
| Cost Calculation | Only calculated when profile uses "self" provider |
| Pricing Source | LiteLLM's `model_prices_and_context_window.json` |
| Pricing Sync | Downloaded during `clown registry sync` |

!!! info "Why 'self' Provider Only for Costs?"
    The `self` provider indicates you're using your own API key directly with a provider like Anthropic. In this case, you pay per-token and cost tracking is meaningful. Other providers (managed services, enterprise gateways) handle billing differently, so cost calculations would be inaccurate.

---

## CLI Commands

### View Usage Summary

```bash
# Default: show today's usage
clown usage

# Specify time period
clown usage --period today
clown usage --period yesterday
clown usage --period week
clown usage --period month
clown usage --period 7d
clown usage --period 30d
clown usage --period all

# Filter by profile
clown usage --profile my-profile

# Filter by model
clown usage --model claude-sonnet-4

# Combine filters
clown usage --period month --profile work-claude
```

### View Breakdown

```bash
# Daily breakdown
clown usage daily --period week

# Usage by model
clown usage models

# Usage by profile
clown usage profiles
```

### Export Usage Data

```bash
# Export as JSON
clown usage export --format json > usage.json

# Export as CSV
clown usage export --format csv --period month > usage.csv
```

### Import Claude Data

Import existing usage data from Claude Code's native files:

```bash
# Import from default location (~/.claude)
clown usage import-claude

# Import from custom location
clown usage import-claude --claude-dir /path/to/.claude
```

This imports data from:

- `~/.claude/stats-cache.json` - Aggregate token usage by model
- `~/.claude/projects/*/session.jsonl` - Session-level data

---

## Web UI

The embedded Web UI includes a dedicated Usage page accessible at `http://127.0.0.1:8765/usage` when the daemon is running.

### Features

- **Period Selector**: Switch between Today, Yesterday, This Week, This Month, Last 7 Days, Last 30 Days, All Time
- **Overview Cards**: Total tokens, total cost, sessions, and runtime at a glance
- **Token Breakdown**: Detailed view of input, output, cache creation, and cache read tokens
- **Cost Breakdown**: Detailed cost breakdown (when available)
- **Profile Table**: Usage breakdown by profile with tokens, cost, and last used date
- **Import Button**: One-click import of Claude Code's native usage data

### Accessing the UI

```bash
# Start the daemon
clown daemon start --stay-alive

# Open in browser
open http://127.0.0.1:8765
```

Navigate to "Usage" in the sidebar to view the usage tracking page.

---

## Token Types

| Token Type | Description |
|------------|-------------|
| Input Tokens | Tokens in the request (prompt, system message, context) |
| Output Tokens | Tokens in the response (completion) |
| Cache Creation | Tokens written to prompt cache |
| Cache Read | Tokens read from prompt cache (cheaper than input) |

---

## HTTP API

### Get Usage Statistics

```http
GET /api/usage
GET /api/usage?period=week
GET /api/usage?profile=my-profile
GET /api/usage?period=month&profile=work-claude
```

**Response:**

```json
{
  "success": true,
  "data": {
    "period": "This week",
    "total_tokens": {
      "input_tokens": 125000,
      "output_tokens": 45000,
      "cache_creation_input_tokens": 10000,
      "cache_read_input_tokens": 50000
    },
    "total_cost": {
      "input_cost": 0.375,
      "output_cost": 0.675,
      "cache_creation_cost": 0.0375,
      "cache_read_cost": 0.015,
      "total_cost": 1.1025
    },
    "total_sessions": 42,
    "total_runtime_secs": 3600,
    "aggregates": {
      "by_profile": {
        "work-claude": {
          "profile": "work-claude",
          "tokens": { "..." },
          "cost": { "..." },
          "sessions": 30,
          "runtime_secs": 2400,
          "last_used": "2026-01-20T10:30:00Z"
        }
      }
    }
  }
}
```

### Import Claude Data

```http
POST /api/usage/import-claude
POST /api/usage/import-claude?claude_dir=/custom/path
```

---

## Configuration

### Pricing Sync

Model pricing is downloaded from LiteLLM during registry sync:

```bash
clown registry sync
```

This downloads `model_prices_and_context_window.json` to:

```
~/.config/clown/registry/litellm-pricing.json
```

### Pricing Data Format

The pricing file includes per-token costs for 200+ models:

```json
{
  "claude-sonnet-4-20250514": {
    "input_cost_per_token": 0.000003,
    "output_cost_per_token": 0.000015,
    "cache_creation_input_token_cost": 0.00000375,
    "cache_read_input_token_cost": 0.0000003
  }
}
```

---

## Data Storage

Usage data is stored under the telemetry directory:

```
~/.config/clown/telemetry/
├── sessions.jsonl        # Per-session records with token/cost data
└── aggregates.json       # Rolled-up stats per profile/model
```

---

## Use Cases

### Track Token Usage Across Profiles

Monitor how many tokens each profile consumes:

```bash
# View all profiles
clown usage profiles

# Check specific profile
clown usage --profile work-claude --period month
```

### Monitor Costs for Direct API Usage

Track costs when using your own API keys:

```bash
# Create profile with self provider
clown profiles create claude direct-api --provider self

# View costs
clown usage --profile direct-api
```

### Import Existing Claude Data

Migrate historical usage data from Claude Code:

```bash
# Import all available Claude data
clown usage import-claude

# Verify import
clown usage --period all
```

### Export for Analysis

Export usage data for external analysis or reporting:

```bash
# Export last month's data as CSV
clown usage export --format csv --period month > monthly-usage.csv

# Export all data as JSON for processing
clown usage export --format json --period all > all-usage.json
```

---

## Troubleshooting

### No usage data showing

1. Verify the daemon is running: `clown daemon status`
2. Check if any profiles have been used: `clown profiles list`
3. Sync registry to get pricing: `clown registry sync`

### Costs showing as "-" or null

1. Verify the profile uses "self" provider
2. Check if pricing data is synced: `ls ~/.config/clown/registry/litellm-pricing.json`
3. Verify the model has pricing in LiteLLM database

### Import not finding data

1. Verify Claude directory exists: `ls ~/.claude`
2. Check for stats-cache.json: `ls ~/.claude/stats-cache.json`
3. Try specifying path explicitly: `clown usage import-claude --claude-dir ~/.claude`
