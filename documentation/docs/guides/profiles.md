# Profile Management

Profiles are the heart of Clown. Each profile binds an agent to a specific provider, model, and credentials with complete isolation.

---

## What is a Profile?

A profile contains:

- **Agent binding** - Which CLI tool to use (Claude Code, Codex, etc.)
- **Provider binding** - Which API backend (Anthropic, MiniMax, etc.)
- **Credentials** - API key stored securely in your system keychain
- **Configuration** - Model selection, arguments, hooks, and more

When you run a profile, Clown creates an isolated environment where all configuration is separate from other profiles.

---

## Profile Lifecycle

### Create

```bash
clown profiles create <agent-id> <alias> --provider <provider-id> [options]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--provider <id>` | API provider to use (required) |
| `--endpoint <id>` | Specific endpoint for multi-region providers |
| `--model <model>` | Override default model |
| `--hooks <list>` | Enable agent hooks (comma-separated) |
| `--mcp <list>` | Enable MCP servers (comma-separated) |
| `--bare` | Create minimal profile without defaults |
| `--proxy` | Enable request routing proxy |
| `--template <name>` | Use a registry template |

**Example:**

```bash
# Basic profile
clown profiles create claude my-project --provider anthropic

# With specific model and endpoint
clown profiles create claude china-work --provider minimax --endpoint china --model MiniMax-M2.1

# With hooks and MCP servers
clown profiles create claude dev-claude --provider anthropic --hooks auto_format --mcp filesystem,github

# Using a template
clown profiles create claude quick --provider anthropic --template anthropic-sonnet
```

### List

```bash
clown profiles list [--agent <agent-id>]
```

**Example output:**

```
Alias              Provider    Endpoint       Model            Last Used
work-anthropic     anthropic   default        claude-sonnet-4  2026-01-08T11:23:51Z
work-minimax       minimax     international  MiniMax-M2.1     2026-01-08T09:18:12Z
home-minimax       minimax     international  MiniMax-M2.1     2026-01-07T22:45:00Z
```

### Inspect

```bash
clown profiles inspect <alias>
```

Shows profile configuration (secrets are redacted):

```yaml
Alias: work-minimax
Agent: claude
Provider: minimax
Endpoint: international
Model: MiniMax-M2.1
Created: 2026-01-05T10:00:00Z
Last Used: 2026-01-08T09:18:12Z
Profile Home: ~/.claude-profiles/work-minimax
API Key: ****...****
```

### Run

```bash
clown profiles run <alias> [-- <agent-args>]
```

Launches the agent with the profile's isolated environment:

```bash
# Basic run
clown profiles run my-project

# With additional arguments
clown profiles run my-project -- /path/to/code --verbose
```

### Delete

```bash
clown profiles delete <alias>
```

Removes the profile and runs any cleanup hooks defined in the agent manifest.

---

## Shell Integration

### Export Environment

For manual usage, export a profile's environment to your shell:

```bash
eval "$(clown profiles env <alias>)"
claude  # Now uses the profile's configuration
```

### Shell Aliases

Install quick-access aliases:

```bash
# Install alias
clown aliases install my-project

# Now you can run:
my-project  # Equivalent to: clown profiles run my-project

# Uninstall when done
clown aliases uninstall my-project
```

---

## Model Selection

When creating a profile, the model is determined by (highest to lowest priority):

1. **`--model` flag** - Explicitly specified
2. **Provider default** - From provider manifest
3. **Agent default** - From agent manifest

```bash
# Uses provider default (MiniMax-M2.1 for minimax)
clown profiles create claude work --provider minimax

# Override with explicit model
clown profiles create claude work --provider minimax --model claude-opus-4
```

!!! tip "Changing Models Later"
    To change a profile's model, delete and recreate it, or edit the JSON directly at `~/.config/clown/profiles/<alias>.json`.

---

## Profile Isolation

### How It Works

Clown uses the **home-wrapper** strategy for isolation:

```
~/.claude-profiles/my-project/
├── .claude/
│   ├── settings.json    # Profile-specific settings
│   ├── hooks.json       # Hook configuration
│   └── history/         # Conversation history
└── .config/
    └── ...              # Other agent configs
```

When running a profile:

1. Clown sets `HOME` to the profile directory
2. Agent reads/writes config relative to this new HOME
3. Each profile has completely separate state

### What Gets Isolated

| Isolated | Not Isolated |
|----------|--------------|
| Configuration files | System binaries |
| API credentials | Shell configuration |
| Conversation history | Network access |
| Agent settings | File system access |

---

## Advanced Features

### Profile Hooks

Profiles support event-driven hooks for tool usage and agent events:

```bash
# Add a hook
clown hooks add my-project PreToolUse "Bash|Write" "echo Tool: $EVENT"

# List hooks
clown hooks list my-project

# Remove a hook
clown hooks remove my-project PreToolUse 0
```

See [Hooks Guide](hooks.md) for details.

### Proxy Routing

Enable intelligent request routing:

```bash
# Create profile with proxy
clown profiles create claude smart-routing --provider anthropic --proxy

# Configure routing
clown proxy route add smart-routing cheap-route "tokens < 1000" "minimax/MiniMax-M2.1"
```

See [Proxy Guide](proxy.md) for details.

### MCP Servers

Enable Model Context Protocol servers:

```bash
clown profiles create claude dev --provider anthropic --mcp filesystem,github
```

---

## Profile Schema

Profiles are stored as JSON in `~/.config/clown/profiles/`:

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

!!! warning "Secret Storage"
    API keys are stored in your system's keychain (macOS Keychain, GNOME Keyring, etc.), not in the JSON file.

---

## Best Practices

1. **Use descriptive aliases** - `project-provider-purpose` format is readable
   ```bash
   clown profiles create claude acme-minimax-dev --provider minimax
   ```

2. **One profile per project** - Keep settings isolated
   ```bash
   clown profiles create claude project-a --provider anthropic
   clown profiles create claude project-b --provider anthropic
   ```

3. **Use aliases for quick access** - Install shims for frequent profiles
   ```bash
   clown aliases install project-a
   ```

4. **Export for scripts** - Use JSON output for automation
   ```bash
   clown profiles list --json | jq '.[] | .alias'
   ```

---

## Troubleshooting

### Profile won't start

1. Check agent is detected: `clown agents list`
2. Verify credentials: `clown profiles inspect <alias>`
3. Check daemon status: `clown daemon status`

### Wrong model being used

1. Inspect the profile: `clown profiles inspect <alias>`
2. Verify model field matches expected
3. Recreate with explicit `--model` flag if needed

### Credentials not working

1. Try recreating the profile to re-enter credentials
2. Check if the provider endpoint is reachable
3. Verify API key is valid with the provider directly
