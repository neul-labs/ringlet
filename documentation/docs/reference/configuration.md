# Configuration Reference

Reference for all configuration files used by Clown.

---

## File Locations

| File | Description |
|------|-------------|
| `~/.config/clown/config.toml` | User preferences |
| `~/.config/clown/profiles/<alias>.json` | Profile definitions |
| `~/.config/clown/agents.d/*.toml` | Custom agent manifests |
| `~/.config/clown/providers.d/*.toml` | Custom provider manifests |
| `~/.config/clown/scripts/*.rhai` | Custom Rhai scripts |
| `~/.config/clown/registry/` | Cached registry data |
| `~/.config/clown/telemetry/` | Usage tracking data |

!!! note "Platform-Specific Paths"
    - **macOS/Linux**: `~/.config/clown/`
    - **Windows**: `%APPDATA%\clown\`

---

## User Configuration

### config.toml

User preferences and defaults.

```toml
# Default provider for new profiles
[defaults]
provider = "anthropic"

# Hook preferences
[hooks]
auto_format = true
auto_lint = false

# Custom hooks
[hooks.custom.PostToolUse]
[[hooks.custom.PostToolUse]]
matcher = "Write"
type = "command"
command = "echo 'File written'"

# MCP server preferences
[mcp_servers]
filesystem = true
github = false
github_token = ""

# Custom MCP servers
[mcp_servers.custom.my-server]
command = "node"
args = ["./my-mcp.js"]

# Custom key-value pairs for scripts
[custom]
my_setting = "value"
```

---

## Profile Schema

### profiles/<alias>.json

Profile definition stored as JSON.

```json
{
  "alias": "my-project",
  "agent_id": "claude",
  "provider_id": "anthropic",
  "endpoint_id": "default",
  "model": "claude-sonnet-4",
  "env": {
    "ANTHROPIC_MODEL": "claude-sonnet-4"
  },
  "args": [],
  "working_dir": null,
  "metadata": {
    "created_at": "2026-01-05T10:00:00Z",
    "last_used": "2026-01-08T09:18:12Z",
    "profile_home": "~/.claude-profiles/my-project",
    "hooks_config": {},
    "proxy_config": null
  }
}
```

### Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `alias` | string | Unique profile identifier |
| `agent_id` | string | Agent identifier (e.g., "claude") |
| `provider_id` | string | Provider identifier (e.g., "anthropic") |
| `endpoint_id` | string | Endpoint for multi-region providers |
| `model` | string | Model identifier |
| `env` | object | Environment variables to inject |
| `args` | array | Additional CLI arguments |
| `working_dir` | string? | Optional working directory |
| `metadata` | object | Profile metadata |

### Metadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `created_at` | string | ISO 8601 creation timestamp |
| `last_used` | string | ISO 8601 last used timestamp |
| `profile_home` | string | Path to profile's isolated home |
| `hooks_config` | object | Hook configuration |
| `proxy_config` | object? | Proxy configuration (if enabled) |

### Proxy Configuration

```json
{
  "proxy_config": {
    "enabled": true,
    "port": null,
    "routing": {
      "strategy": "Conditional",
      "rules": [
        {
          "name": "long-context",
          "condition": { "type": "token_count", "min": 50000 },
          "target": "minimax/claude-3-sonnet",
          "priority": 10
        },
        {
          "name": "default",
          "condition": { "type": "always" },
          "target": "anthropic/claude-sonnet-4",
          "priority": 0
        }
      ]
    },
    "model_aliases": {}
  }
}
```

### Hooks Configuration

```json
{
  "hooks_config": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "echo $EVENT",
            "timeout": 5000
          }
        ]
      }
    ],
    "PostToolUse": [],
    "Notification": [],
    "Stop": []
  }
}
```

---

## Agent Manifest

### agents.d/<agent-id>.toml

Define how Clown interacts with an agent.

```toml
id = "claude"
name = "Claude Code"
binary = "claude"
version_flag = "--version"

[detect]
commands = ["claude --version"]
files = ["~/.claude/settings.json"]

[profile]
strategy = "home-wrapper"
source_home = "~/.claude-profiles/{alias}"
script = "claude.rhai"

[models]
default = "claude-sonnet-4"
supported = ["claude-sonnet-4", "claude-opus-4"]

[hooks]
create = []
delete = []
pre_run = []
post_run = []

supports_hooks = true
```

### Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique identifier |
| `name` | string | Display name |
| `binary` | string | Executable name |
| `version_flag` | string | Flag to get version |
| `detect.commands` | array | Commands to detect installation |
| `detect.files` | array | Files that indicate installation |
| `profile.strategy` | string | Isolation strategy (home-wrapper) |
| `profile.source_home` | string | Template for profile home path |
| `profile.script` | string | Rhai script for config generation |
| `models.default` | string | Default model |
| `models.supported` | array | List of supported models |
| `supports_hooks` | boolean | Whether agent supports hooks |

---

## Provider Manifest

### providers.d/<provider-id>.toml

Define API backends.

```toml
id = "minimax"
name = "MiniMax"
type = "anthropic-compatible"

[endpoints]
international = "https://api.minimax.io/anthropic"
china = "https://api.minimaxi.com/anthropic"
default = "international"

[auth]
env_key = "MINIMAX_API_KEY"
prompt = "Enter your MiniMax API key"

[models]
available = ["MiniMax-M2.1"]
default = "MiniMax-M2.1"
```

### Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique identifier |
| `name` | string | Display name |
| `type` | string | API compatibility type |
| `endpoints` | object | Named endpoints with URLs |
| `endpoints.default` | string | Default endpoint name |
| `auth.env_key` | string | Environment variable for API key |
| `auth.prompt` | string | Prompt message for API key |
| `models.available` | array | Available models |
| `models.default` | string | Default model |

### Provider Types

| Type | Description |
|------|-------------|
| `anthropic` | Native Anthropic API |
| `anthropic-compatible` | Anthropic-compatible APIs |
| `openai` | Native OpenAI API |
| `openai-compatible` | OpenAI-compatible APIs |

---

## Directory Structure

### Full Layout

```
~/.config/clown/
├── config.toml               # User preferences
├── daemon-endpoint           # Active daemon endpoint
├── agents.d/                 # Custom agent manifests
│   └── custom-agent.toml
├── providers.d/              # Custom provider manifests
│   └── custom-provider.toml
├── scripts/                  # Custom Rhai scripts
│   └── claude.rhai
├── profiles/                 # Profile definitions
│   ├── my-project.json
│   └── work-claude.json
├── registry/                 # Cached registry data
│   ├── current -> commits/f4a12c3
│   ├── registry.lock
│   ├── litellm-pricing.json
│   └── commits/
│       └── f4a12c3/
│           ├── registry.json
│           ├── agents/
│           ├── providers/
│           ├── scripts/
│           └── models/
├── cache/                    # Detection cache
│   └── agent-detections.json
├── telemetry/                # Usage data
│   ├── sessions.jsonl
│   └── aggregates.json
└── logs/
    └── clownd.log
```

### Profile Home Structure

When a profile runs, Clown creates an isolated home:

```
~/.claude-profiles/my-project/
├── .claude/
│   ├── settings.json         # Agent settings
│   ├── hooks.json            # Hook configuration
│   └── history/              # Conversation history
├── .claude.json              # MCP server config
└── .ultrallm/                # Proxy config (if enabled)
    ├── config.yaml
    └── logs/
        └── proxy.log
```

---

## Environment Variables

### Override Locations

| Variable | Description |
|----------|-------------|
| `CLOWN_CONFIG_DIR` | Override config directory |
| `CLOWN_DAEMON_ENDPOINT` | Override daemon endpoint |
| `CLOWN_REGISTRY_URL` | Override registry URL |
| `CLOWN_REGISTRY_CHANNEL` | Override registry channel |

### Runtime Variables

These are injected when running a profile:

| Variable | Description |
|----------|-------------|
| `HOME` | Set to profile home for isolation |
| `ANTHROPIC_BASE_URL` | API endpoint (Anthropic agents) |
| `ANTHROPIC_AUTH_TOKEN` | API key (Anthropic agents) |
| `ANTHROPIC_MODEL` | Model identifier |

---

## Telemetry Data

### sessions.jsonl

Per-session records (JSON Lines format):

```json
{"profile":"my-project","model":"claude-sonnet-4","started_at":"2026-01-08T10:00:00Z","ended_at":"2026-01-08T10:30:00Z","tokens":{"input":5000,"output":2000,"cache_creation":1000,"cache_read":500},"cost":{"input":0.015,"output":0.030,"total":0.045}}
```

### aggregates.json

Rolled-up statistics:

```json
{
  "by_profile": {
    "my-project": {
      "total_tokens": 125000,
      "total_cost": 1.10,
      "sessions": 42,
      "last_used": "2026-01-08T10:30:00Z"
    }
  },
  "by_model": {
    "claude-sonnet-4": {
      "total_tokens": 100000,
      "total_cost": 0.90
    }
  }
}
```
