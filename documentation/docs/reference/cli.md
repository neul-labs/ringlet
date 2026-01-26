# CLI Reference

Complete reference for all `ringlet` commands.

---

## Global Options

```bash
ringlet [OPTIONS] <COMMAND>
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
ringlet agents list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

**Example:**

```bash
$ ringlet agents list
ID          Name         Installed   Version    Profiles
claude      Claude Code  Yes         1.0.0      3
codex       Codex CLI    Yes         0.5.0      1
grok        Grok CLI     No          -          0
```

### agents inspect

Show detailed information about an agent.

```bash
ringlet agents inspect <AGENT_ID>
```

**Example:**

```bash
$ ringlet agents inspect claude
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
ringlet providers list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

**Example:**

```bash
$ ringlet providers list
ID          Name        Type                 Default Model
anthropic   Anthropic   anthropic            claude-sonnet-4
minimax     MiniMax     anthropic-compatible MiniMax-M2.1
openai      OpenAI      openai               gpt-4o
openrouter  OpenRouter  openai-compatible    auto
```

### providers inspect

Show detailed information about a provider.

```bash
ringlet providers inspect <PROVIDER_ID>
```

**Example:**

```bash
$ ringlet providers inspect minimax
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
ringlet profiles create <AGENT_ID> <ALIAS> [OPTIONS]
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
ringlet profiles create claude my-project --provider anthropic

# With specific endpoint and model
ringlet profiles create claude china-work --provider minimax --endpoint china --model MiniMax-M2.1

# With hooks and MCP servers
ringlet profiles create claude dev --provider anthropic --hooks auto_format --mcp filesystem,github

# With proxy enabled
ringlet profiles create claude smart --provider anthropic --proxy
```

### profiles list

List all profiles.

```bash
ringlet profiles list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--agent <ID>` | Filter by agent |
| `--json` | Output as JSON |

**Example:**

```bash
$ ringlet profiles list
Alias              Provider    Endpoint       Model            Last Used
work-anthropic     anthropic   default        claude-sonnet-4  2026-01-08T11:23:51Z
work-minimax       minimax     international  MiniMax-M2.1     2026-01-08T09:18:12Z
```

### profiles inspect

Show profile details.

```bash
ringlet profiles inspect <ALIAS>
```

**Example:**

```bash
$ ringlet profiles inspect work-minimax
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
ringlet profiles run <ALIAS> [OPTIONS] [-- <AGENT_ARGS>...]
```

| Option | Description |
|--------|-------------|
| `--remote` | Run in daemon with PTY (enables web terminal access) |
| `--cols <N>` | Terminal columns (default: 80, only with --remote) |
| `--rows <N>` | Terminal rows (default: 24, only with --remote) |

**Examples:**

```bash
# Basic run (foreground)
ringlet profiles run my-project

# With additional agent arguments
ringlet profiles run my-project -- /path/to/code --verbose

# Run as remote terminal session (accessible via web UI)
ringlet profiles run my-project --remote
```

### profiles delete

Delete a profile.

```bash
ringlet profiles delete <ALIAS>
```

### profiles env

Export profile environment variables.

```bash
ringlet profiles env <ALIAS>
```

**Example:**

```bash
eval "$(ringlet profiles env my-project)"
claude  # Now uses the profile's configuration
```

---

## terminal

Manage remote terminal sessions. Terminal sessions allow you to run agents in the daemon and access them through the web UI or CLI.

### terminal list

List all terminal sessions.

```bash
ringlet terminal list
```

**Example:**

```bash
$ ringlet terminal list
SESSION ID                            PROFILE          STATE       CLIENTS
--------------------------------------------------------------------------------
46e15057-abbb-42cd-ad0e-52471a76ef9f  my-project       running     1
```

### terminal info

Show detailed information about a session.

```bash
ringlet terminal info <SESSION_ID>
```

**Example:**

```bash
$ ringlet terminal info 46e15057-abbb-42cd-ad0e-52471a76ef9f
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
ringlet terminal kill <SESSION_ID>
```

### terminal attach

Attach to a terminal session from the CLI (not yet implemented - use web UI).

```bash
ringlet terminal attach <SESSION_ID>
```

---

## aliases

Manage shell aliases for quick profile access.

### aliases install

Install a shell alias for a profile.

```bash
ringlet aliases install <ALIAS>
```

**Example:**

```bash
ringlet aliases install my-project
# Now you can run: my-project
```

### aliases uninstall

Remove a shell alias.

```bash
ringlet aliases uninstall <ALIAS>
```

### aliases list

List installed aliases.

```bash
ringlet aliases list
```

---

## proxy

Manage proxy instances for request routing.

### proxy enable

Enable proxy for a profile.

```bash
ringlet proxy enable <ALIAS>
```

### proxy disable

Disable proxy for a profile.

```bash
ringlet proxy disable <ALIAS>
```

### proxy start

Start a proxy instance.

```bash
ringlet proxy start <ALIAS>
```

### proxy stop

Stop a proxy instance.

```bash
ringlet proxy stop <ALIAS>
```

### proxy stop-all

Stop all proxy instances.

```bash
ringlet proxy stop-all
```

### proxy restart

Restart a proxy instance.

```bash
ringlet proxy restart <ALIAS>
```

### proxy status

Show proxy status.

```bash
ringlet proxy status [ALIAS]
```

### proxy config

Show proxy configuration.

```bash
ringlet proxy config <ALIAS>
```

### proxy logs

View proxy logs.

```bash
ringlet proxy logs <ALIAS> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--lines <N>` | Number of lines to show (default: 50) |

### proxy route add

Add a routing rule.

```bash
ringlet proxy route add <ALIAS> <NAME> <CONDITION> <TARGET> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--priority <N>` | Rule priority (higher = checked first) |

**Examples:**

```bash
ringlet proxy route add work "long-context" "tokens > 100000" "minimax/claude-3-sonnet" --priority 10
ringlet proxy route add work "default" "always" "anthropic/claude-sonnet-4"
```

### proxy route list

List routing rules.

```bash
ringlet proxy route list <ALIAS>
```

### proxy route remove

Remove a routing rule.

```bash
ringlet proxy route remove <ALIAS> <NAME>
```

### proxy alias set

Set a model alias.

```bash
ringlet proxy alias set <ALIAS> <FROM_MODEL> <TO_TARGET>
```

### proxy alias list

List model aliases.

```bash
ringlet proxy alias list <ALIAS>
```

### proxy alias remove

Remove a model alias.

```bash
ringlet proxy alias remove <ALIAS> <FROM_MODEL>
```

---

## hooks

Manage profile hooks.

### hooks add

Add a hook to a profile.

```bash
ringlet hooks add <ALIAS> <EVENT> <MATCHER> <COMMAND>
```

| Parameter | Description |
|-----------|-------------|
| `ALIAS` | Profile alias |
| `EVENT` | Event type: PreToolUse, PostToolUse, Notification, Stop |
| `MATCHER` | Tool pattern (e.g., "Bash\|Write" or "*") |
| `COMMAND` | Shell command to execute |

**Example:**

```bash
ringlet hooks add myprofile PreToolUse "Bash" "echo 'Running: $EVENT' >> /tmp/ringlet.log"
```

### hooks list

List hooks for a profile.

```bash
ringlet hooks list <ALIAS>
```

### hooks remove

Remove a hook.

```bash
ringlet hooks remove <ALIAS> <EVENT> <INDEX>
```

### hooks import

Import hooks from a file.

```bash
ringlet hooks import <ALIAS> <FILE>
```

### hooks export

Export hooks to JSON.

```bash
ringlet hooks export <ALIAS>
```

---

## usage

Track token usage and costs.

### usage (default)

Show usage summary.

```bash
ringlet usage [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--period <PERIOD>` | Time period: today, yesterday, week, month, 7d, 30d, all |
| `--profile <ALIAS>` | Filter by profile |
| `--model <MODEL>` | Filter by model |

**Example:**

```bash
$ ringlet usage --period week
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
ringlet usage daily [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--period <PERIOD>` | Time period |

### usage models

Show usage by model.

```bash
ringlet usage models
```

### usage profiles

Show usage by profile.

```bash
ringlet usage profiles
```

### usage export

Export usage data.

```bash
ringlet usage export [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--format <FORMAT>` | Output format: json, csv |
| `--period <PERIOD>` | Time period |

### usage import-claude

Import usage data from Claude Code.

```bash
ringlet usage import-claude [OPTIONS]
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
ringlet registry sync [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--force` | Force refresh even if cached |
| `--offline` | Use cached data only |

### registry inspect

Show registry status.

```bash
ringlet registry inspect
```

### registry pin

Pin to a specific version.

```bash
ringlet registry pin <REF>
```

---

## daemon

Manage the background daemon.

### daemon status

Show daemon status.

```bash
ringlet daemon status
```

### daemon start

Start the daemon.

```bash
ringlet daemon start [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--stay-alive` | Keep running indefinitely |

### daemon stop

Stop the daemon.

```bash
ringlet daemon stop
```

---

## scripts

Manage Rhai configuration scripts.

### scripts test

Test a configuration script.

```bash
ringlet scripts test <SCRIPT> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--provider <ID>` | Provider to use |
| `--alias <ALIAS>` | Profile alias for context |

---

## export / import

Backup and restore ringlet configuration.

### export

Export configuration.

```bash
ringlet export > backup.json
```

### import

Import configuration.

```bash
ringlet import backup.json
```
