# CLI Reference

Complete reference for all `clown` commands.

---

## Global Options

```bash
clown [OPTIONS] <COMMAND>
```

| Option | Description |
|--------|-------------|
| `--log-level <LEVEL>` | Set log level (error, warn, info, debug, trace) |
| `--json` | Output in JSON format |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

---

## agents

Discover and manage AI coding agents.

### agents list

List all detected agents.

```bash
clown agents list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

**Example:**

```bash
$ clown agents list
ID          Name         Installed   Version    Profiles
claude      Claude Code  Yes         1.0.0      3
codex       Codex CLI    Yes         0.5.0      1
grok        Grok CLI     No          -          0
```

### agents inspect

Show detailed information about an agent.

```bash
clown agents inspect <AGENT_ID>
```

**Example:**

```bash
$ clown agents inspect claude
ID: claude
Name: Claude Code
Binary: claude
Version: 1.0.0
Binary Path: /usr/local/bin/claude
Profile Strategy: home-wrapper
Profile Home: ~/.claude-profiles/{alias}
Supports Hooks: Yes
Default Model: claude-sonnet-4
```

---

## providers

Manage API providers.

### providers list

List all available providers.

```bash
clown providers list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

**Example:**

```bash
$ clown providers list
ID          Name        Type                 Default Model
anthropic   Anthropic   anthropic            claude-sonnet-4
minimax     MiniMax     anthropic-compatible MiniMax-M2.1
openai      OpenAI      openai               gpt-4o
openrouter  OpenRouter  openai-compatible    auto
```

### providers inspect

Show detailed information about a provider.

```bash
clown providers inspect <PROVIDER_ID>
```

**Example:**

```bash
$ clown providers inspect minimax
ID: minimax
Name: MiniMax
Type: anthropic-compatible
Endpoints:
  international: https://api.minimax.io/anthropic (default)
  china: https://api.minimaxi.com/anthropic
Auth: MINIMAX_API_KEY
Models: MiniMax-M2.1
```

---

## profiles

Create, manage, and run profiles.

### profiles create

Create a new profile.

```bash
clown profiles create <AGENT_ID> <ALIAS> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-p, --provider <ID>` | Provider to use (required) |
| `--endpoint <ID>` | Specific endpoint for multi-region providers |
| `--model <MODEL>` | Override default model |
| `--hooks <LIST>` | Enable hooks (comma-separated) |
| `--mcp <LIST>` | Enable MCP servers (comma-separated) |
| `--bare` | Create minimal profile without defaults |
| `--proxy` | Enable request routing proxy |
| `--template <NAME>` | Use a registry template |
| `--dry-run` | Show what would be created without creating |

**Examples:**

```bash
# Basic profile
clown profiles create claude my-project --provider anthropic

# With specific endpoint and model
clown profiles create claude china-work --provider minimax --endpoint china --model MiniMax-M2.1

# With hooks and MCP servers
clown profiles create claude dev --provider anthropic --hooks auto_format --mcp filesystem,github

# With proxy enabled
clown profiles create claude smart --provider anthropic --proxy
```

### profiles list

List all profiles.

```bash
clown profiles list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--agent <ID>` | Filter by agent |
| `--json` | Output as JSON |

**Example:**

```bash
$ clown profiles list
Alias              Provider    Endpoint       Model            Last Used
work-anthropic     anthropic   default        claude-sonnet-4  2026-01-08T11:23:51Z
work-minimax       minimax     international  MiniMax-M2.1     2026-01-08T09:18:12Z
```

### profiles inspect

Show profile details.

```bash
clown profiles inspect <ALIAS>
```

**Example:**

```bash
$ clown profiles inspect work-minimax
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

### profiles run

Run a profile.

```bash
clown profiles run <ALIAS> [OPTIONS] [-- <AGENT_ARGS>...]
```

| Option | Description |
|--------|-------------|
| `--remote` | Run in daemon with PTY (enables web terminal access) |
| `--cols <N>` | Terminal columns (default: 80, only with --remote) |
| `--rows <N>` | Terminal rows (default: 24, only with --remote) |

**Examples:**

```bash
# Basic run (foreground)
clown profiles run my-project

# With additional agent arguments
clown profiles run my-project -- /path/to/code --verbose

# Run as remote terminal session (accessible via web UI)
clown profiles run my-project --remote
```

### profiles delete

Delete a profile.

```bash
clown profiles delete <ALIAS>
```

### profiles env

Export profile environment variables.

```bash
clown profiles env <ALIAS>
```

**Example:**

```bash
eval "$(clown profiles env my-project)"
claude  # Now uses the profile's configuration
```

---

## terminal

Manage remote terminal sessions. Terminal sessions allow you to run agents in the daemon and access them through the web UI or CLI.

### terminal list

List all terminal sessions.

```bash
clown terminal list
```

**Example:**

```bash
$ clown terminal list
SESSION ID                            PROFILE          STATE       CLIENTS
--------------------------------------------------------------------------------
46e15057-abbb-42cd-ad0e-52471a76ef9f  my-project       running     1
```

### terminal info

Show detailed information about a session.

```bash
clown terminal info <SESSION_ID>
```

**Example:**

```bash
$ clown terminal info 46e15057-abbb-42cd-ad0e-52471a76ef9f
Session ID: 46e15057-abbb-42cd-ad0e-52471a76ef9f
Profile: my-project
State: running
PID: 12345
Size: 80x24
Clients: 1
Created: 2026-01-22T00:22:45Z
```

### terminal kill

Terminate a terminal session.

```bash
clown terminal kill <SESSION_ID>
```

### terminal attach

Attach to a terminal session from the CLI (not yet implemented - use web UI).

```bash
clown terminal attach <SESSION_ID>
```

---

## aliases

Manage shell aliases for quick profile access.

### aliases install

Install a shell alias for a profile.

```bash
clown aliases install <ALIAS>
```

**Example:**

```bash
clown aliases install my-project
# Now you can run: my-project
```

### aliases uninstall

Remove a shell alias.

```bash
clown aliases uninstall <ALIAS>
```

### aliases list

List installed aliases.

```bash
clown aliases list
```

---

## proxy

Manage proxy instances for request routing.

### proxy enable

Enable proxy for a profile.

```bash
clown proxy enable <ALIAS>
```

### proxy disable

Disable proxy for a profile.

```bash
clown proxy disable <ALIAS>
```

### proxy start

Start a proxy instance.

```bash
clown proxy start <ALIAS>
```

### proxy stop

Stop a proxy instance.

```bash
clown proxy stop <ALIAS>
```

### proxy stop-all

Stop all proxy instances.

```bash
clown proxy stop-all
```

### proxy restart

Restart a proxy instance.

```bash
clown proxy restart <ALIAS>
```

### proxy status

Show proxy status.

```bash
clown proxy status [ALIAS]
```

### proxy config

Show proxy configuration.

```bash
clown proxy config <ALIAS>
```

### proxy logs

View proxy logs.

```bash
clown proxy logs <ALIAS> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--lines <N>` | Number of lines to show (default: 50) |

### proxy route add

Add a routing rule.

```bash
clown proxy route add <ALIAS> <NAME> <CONDITION> <TARGET> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--priority <N>` | Rule priority (higher = checked first) |

**Examples:**

```bash
clown proxy route add work "long-context" "tokens > 100000" "minimax/claude-3-sonnet" --priority 10
clown proxy route add work "default" "always" "anthropic/claude-sonnet-4"
```

### proxy route list

List routing rules.

```bash
clown proxy route list <ALIAS>
```

### proxy route remove

Remove a routing rule.

```bash
clown proxy route remove <ALIAS> <NAME>
```

### proxy alias set

Set a model alias.

```bash
clown proxy alias set <ALIAS> <FROM_MODEL> <TO_TARGET>
```

### proxy alias list

List model aliases.

```bash
clown proxy alias list <ALIAS>
```

### proxy alias remove

Remove a model alias.

```bash
clown proxy alias remove <ALIAS> <FROM_MODEL>
```

---

## hooks

Manage profile hooks.

### hooks add

Add a hook to a profile.

```bash
clown hooks add <ALIAS> <EVENT> <MATCHER> <COMMAND>
```

| Parameter | Description |
|-----------|-------------|
| `ALIAS` | Profile alias |
| `EVENT` | Event type: PreToolUse, PostToolUse, Notification, Stop |
| `MATCHER` | Tool pattern (e.g., "Bash\|Write" or "*") |
| `COMMAND` | Shell command to execute |

**Example:**

```bash
clown hooks add myprofile PreToolUse "Bash" "echo 'Running: $EVENT' >> /tmp/clown.log"
```

### hooks list

List hooks for a profile.

```bash
clown hooks list <ALIAS>
```

### hooks remove

Remove a hook.

```bash
clown hooks remove <ALIAS> <EVENT> <INDEX>
```

### hooks import

Import hooks from a file.

```bash
clown hooks import <ALIAS> <FILE>
```

### hooks export

Export hooks to JSON.

```bash
clown hooks export <ALIAS>
```

---

## usage

Track token usage and costs.

### usage (default)

Show usage summary.

```bash
clown usage [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--period <PERIOD>` | Time period: today, yesterday, week, month, 7d, 30d, all |
| `--profile <ALIAS>` | Filter by profile |
| `--model <MODEL>` | Filter by model |

**Example:**

```bash
$ clown usage --period week
Usage Summary (This Week)
─────────────────────────
Total Tokens: 125,000
  Input:       80,000
  Output:      45,000

Estimated Cost: $1.10
```

### usage daily

Show daily breakdown.

```bash
clown usage daily [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--period <PERIOD>` | Time period |

### usage models

Show usage by model.

```bash
clown usage models
```

### usage profiles

Show usage by profile.

```bash
clown usage profiles
```

### usage export

Export usage data.

```bash
clown usage export [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--format <FORMAT>` | Output format: json, csv |
| `--period <PERIOD>` | Time period |

### usage import-claude

Import usage data from Claude Code.

```bash
clown usage import-claude [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--claude-dir <PATH>` | Path to .claude directory |

---

## registry

Manage the GitHub-based registry.

### registry sync

Synchronize registry metadata.

```bash
clown registry sync [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--force` | Force refresh even if cached |
| `--offline` | Use cached data only |

### registry inspect

Show registry status.

```bash
clown registry inspect
```

### registry pin

Pin to a specific version.

```bash
clown registry pin <REF>
```

---

## daemon

Manage the background daemon.

### daemon status

Show daemon status.

```bash
clown daemon status
```

### daemon start

Start the daemon.

```bash
clown daemon start [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--stay-alive` | Keep running indefinitely |

### daemon stop

Stop the daemon.

```bash
clown daemon stop
```

---

## scripts

Manage Rhai configuration scripts.

### scripts test

Test a configuration script.

```bash
clown scripts test <SCRIPT> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--provider <ID>` | Provider to use |
| `--alias <ALIAS>` | Profile alias for context |

---

## export / import

Backup and restore clown configuration.

### export

Export configuration.

```bash
clown export > backup.json
```

### import

Import configuration.

```bash
clown import backup.json
```
