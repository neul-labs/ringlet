# Request Routing with Proxy

The proxy feature enables intelligent request routing from agents to multiple LLM providers. Each profile can have its own proxy instance with dedicated routing configuration.

---

## Overview

Clown integrates with [ultrallm](https://github.com/starbaser/ultrallm), a high-performance Rust-based LLM proxy that supports 25+ providers and multiple routing strategies.

### Key Benefits

- **Cost Optimization**: Route long-context requests to cheaper providers
- **Provider Flexibility**: Use different providers for different use cases
- **Profile Isolation**: Each profile has independent routing configuration
- **Automatic Management**: Proxy starts/stops automatically with profiles

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          ringlet daemon                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Profile: work   │  │ Profile: test   │  │ Profile: cheap  │  │
│  │ Port: 8081      │  │ Port: 8082      │  │ Port: 8083      │  │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
└───────────┼────────────────────┼────────────────────┼───────────┘
            │                    │                    │
            ▼                    ▼                    ▼
┌───────────────────┐ ┌───────────────────┐ ┌───────────────────┐
│ ultrallm :8081    │ │ ultrallm :8082    │ │ ultrallm :8083    │
└─────────┬─────────┘ └─────────┬─────────┘ └─────────┬─────────┘
          │                     │                     │
          ▼                     ▼                     ▼
    ┌──────────┐          ┌──────────┐          ┌──────────┐
    │ Anthropic│          │  MiniMax │          │   Z.AI   │
    └──────────┘          └──────────┘          └──────────┘
```

Each profile's agent runs with `baseUrl` pointing to its local proxy instance.

---

## Prerequisites

The ultrallm binary must be available. Clown looks for it in:

1. `~/.cache/ringlet/binaries/ultrallm`
2. `~/.local/bin/ultrallm`
3. System PATH

!!! tip "Installing ultrallm"
    You can build ultrallm from source or download a release from the [ultrallm repository](https://github.com/starbaser/ultrallm).

---

## Creating Profiles with Proxy

Use the `--proxy` flag when creating a profile:

```bash
ringlet profiles create claude work --proxy -p anthropic
```

This creates a profile with proxy enabled. The proxy configuration is stored in the profile metadata.

---

## CLI Commands

### Proxy Lifecycle

```bash
# Enable proxy for an existing profile
ringlet proxy enable <alias>

# Disable proxy for a profile
ringlet proxy disable <alias>

# Start proxy instance manually
ringlet proxy start <alias>

# Stop proxy instance
ringlet proxy stop <alias>

# Stop all proxy instances
ringlet proxy stop-all

# Restart proxy instance
ringlet proxy restart <alias>
```

### Status and Monitoring

```bash
# Show status of all proxy instances
ringlet proxy status

# Show status for a specific profile
ringlet proxy status <alias>

# Show proxy configuration for a profile
ringlet proxy config <alias>

# View proxy logs (last 50 lines by default)
ringlet proxy logs <alias>

# View more log lines
ringlet proxy logs <alias> --lines 200
```

---

## Routing Rules

Routing rules determine how requests are distributed to providers based on conditions.

### Adding Rules

```bash
ringlet proxy route add <alias> <name> <condition> <target> [--priority N]
```

**Examples:**

```bash
# Route long context to cheaper provider
ringlet proxy route add work "long-context" "tokens > 100000" "minimax/claude-3-sonnet" --priority 10

# Route thinking mode to premium model
ringlet proxy route add work "thinking" "thinking" "anthropic/claude-opus-4" --priority 5

# Default fallback rule
ringlet proxy route add work "default" "always" "anthropic/claude-sonnet-4"
```

### Condition Syntax

| Condition | Description |
|-----------|-------------|
| `always` | Always match (use for default/fallback rules) |
| `thinking` | Match requests with thinking/extended mode enabled |
| `tokens > N` | Match when token count exceeds N |
| `tokens < N` | Match when token count is below N |
| `tools >= N` | Match when tool count is at least N |

### Managing Rules

```bash
# List routing rules
ringlet proxy route list <alias>

# Remove a routing rule
ringlet proxy route remove <alias> <name>
```

---

## Model Aliases

Model aliases map requested model names to provider-specific targets.

```bash
# Set a model alias
ringlet proxy alias set <alias> <from-model> <to-target>

# Examples
ringlet proxy alias set work "claude-sonnet-4" "minimax/claude-3-sonnet"
ringlet proxy alias set work "fast-model" "anthropic/claude-haiku-3"

# List model aliases
ringlet proxy alias list <alias>

# Remove a model alias
ringlet proxy alias remove <alias> <from-model>
```

---

## How It Works

### Port Allocation

- Base port: 8080
- Range: 8080-8180 (supports up to 100 profiles)
- Ports are automatically allocated and released
- Each profile gets a unique port

### Auto-Start Behavior

When you run a profile with proxy enabled:

1. Clown checks if the proxy is already running
2. If not, it starts a new ultrallm instance
3. The agent's configuration is updated with the proxy URL
4. The agent starts and routes requests through the proxy

```bash
ringlet profiles run work
# Proxy automatically starts on port 8081
# Claude Code runs with baseUrl: http://127.0.0.1:8081
```

### Graceful Shutdown

- Proxies stay running between profile runs (for faster subsequent starts)
- When the daemon shuts down, all proxies are gracefully terminated
- SIGTERM is sent first, then SIGKILL after 5 seconds if needed

---

## Profile Structure

When proxy is enabled, the profile home includes:

```
~/.claude-profiles/work/
├── .claude/
│   └── settings.json     # baseUrl: http://localhost:{port}
├── .ultrallm/
│   ├── config.yaml       # Generated routing configuration
│   └── logs/
│       └── proxy.log     # Proxy logs
└── ...
```

---

## Routing Strategies

| Strategy | Description |
|----------|-------------|
| `Simple` | Use first matching rule |
| `Weighted` | Weighted random among matches |
| `LowestCost` | Pick cheapest option |
| `Adaptive` | Learn from latency/errors |
| `Conditional` | Rule-based routing |

---

## Use Cases

### Cost Optimization

Route long-context requests to cheaper providers:

```bash
# Create profile with proxy
ringlet profiles create claude work --proxy -p anthropic

# Add rule to route long context to cheaper provider
ringlet proxy route add work "long-context" "tokens > 100000" "minimax/claude-3-sonnet" --priority 10

# Add default rule for normal requests
ringlet proxy route add work "default" "always" "anthropic/claude-sonnet-4"

# Start using the profile
ringlet profiles run work
```

### Multi-Provider Fallback

Configure fallback to alternative providers when primary is unavailable:

```bash
# Create profile with proxy
ringlet profiles create claude reliable --proxy -p anthropic

# Primary provider (highest priority)
ringlet proxy route add reliable "primary" "always" "anthropic/claude-sonnet-4" --priority 10

# Fallback provider (lower priority)
ringlet proxy route add reliable "fallback" "always" "minimax/claude-3-sonnet" --priority 0
```

### Development vs Production

```bash
# Development: Route to cheaper/faster providers
ringlet profiles create claude dev --proxy -p minimax
ringlet proxy route add dev "default" "always" "minimax/claude-3-sonnet"

# Production: Route to premium providers
ringlet profiles create claude prod --proxy -p anthropic
ringlet proxy route add prod "thinking" "thinking" "anthropic/claude-opus-4" --priority 10
ringlet proxy route add prod "default" "always" "anthropic/claude-sonnet-4"
```

### Tool-Heavy Workloads

Route requests with many tools to more capable models:

```bash
ringlet proxy route add work "heavy-tools" "tools >= 10" "anthropic/claude-opus-4" --priority 5
ringlet proxy route add work "default" "always" "anthropic/claude-sonnet-4"
```

---

## Monitoring

### View Status

```bash
ringlet proxy status

# Output:
# Profile   Port   PID      Status    Restarts   Started
# work      8081   12345    running   0          2026-01-18 10:30
# dev       8082   12346    running   0          2026-01-18 09:15
```

### View Configuration

```bash
ringlet proxy config work

# Output:
# Enabled: true
# Port: auto
# Strategy: Conditional
# Rules:
#   [long-context] tokens > 100000 -> minimax/claude-3-sonnet (priority: 10)
#   [default] always -> anthropic/claude-sonnet-4 (priority: 0)
```

### View Logs

```bash
# View last 50 lines
ringlet proxy logs work

# View more lines
ringlet proxy logs work --lines 200

# Follow in real-time
tail -f ~/.claude-profiles/work/.ultrallm/logs/proxy.log
```

### Health Check

The proxy health endpoint is available at:

```
http://127.0.0.1:{port}/health
```

---

## Troubleshooting

### Proxy not starting

1. Verify ultrallm binary is installed and executable
2. Check if port is already in use
3. Review daemon logs: `ringlet --log-level debug daemon start`

### Connection refused

1. Verify proxy is running: `ringlet proxy status`
2. Confirm port allocation in profile
3. Check proxy logs for errors

### Routing not working

1. Verify routing rules: `ringlet proxy route list <alias>`
2. Check generated config in `.ultrallm/config.yaml`
3. Ensure API keys are set for target providers
