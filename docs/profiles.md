# Profile management

Profiles are the heart of clown. Each one binds an agent installation to a specific model, set of credentials, and optional launch arguments. The CLI surfaces how many profiles exist per agent so you can spot missing environments before starting work.

## Lifecycle

1. **Create** – `clown profiles create <agent-id> <alias> --provider <provider-id> [--endpoint <endpoint-id>]` prompts for the model you plan to use, required environment variables (API keys), and any agent-specific options (e.g., HOME path for Claude). The `--provider` flag specifies which API backend to use (e.g., `anthropic`, `minimax`, `openrouter`). The optional `--endpoint` flag selects a specific endpoint when providers offer multiple (e.g., `--endpoint china` for MiniMax's China region); if omitted, the provider's default endpoint is used. When multiple profiles already store credentials for the same provider, clown offers to reuse one of those keys (you can pick from a list or enter a new value). Prompts still fire every time so secrets are never silently reused.
2. **Inspect** – `clown profiles inspect <alias>` prints the stored configuration and redacts secrets by default.
3. **List** – `clown profiles list --agent <agent-id>` summarizes aliases per agent and feeds the aggregate counts shown by `clown agents list`.
4. **Run** – `clown profiles run <alias> -- <agent args>` launches the selected agent with the stored configuration, then streams stdout/stderr directly to the caller.
5. **Run remote** – `clown profiles run <alias> --remote` runs the agent in the daemon's PTY, enabling access through the web UI or multiple clients.
6. **Switch (shell helper)** – `eval "$(clown profiles env <alias>)"` exports environment variables into the current shell when you want to run the agent manually.
7. **Delete** – `clown profiles delete <alias>` removes the JSON file and executes any teardown hooks defined in the manifest.
8. **Optional env setup** – `clown env setup <alias> <task>` runs manual environment adjustments (e.g., remapping CLI shims) defined by the manifest.

## Schema reference

| Field | Description |
| --- | --- |
| `alias` | Unique key used when referencing the profile. |
| `agent_id` | Links to a manifest (`claude`, `codex`, etc.). |
| `provider_id` | Links to a provider (`anthropic`, `minimax`, `openrouter`, etc.). See `docs/providers.md`. |
| `endpoint_id` | Which provider endpoint to use (e.g., `international`, `china`). Defaults to the provider's default endpoint. |
| `model` | Default model (e.g., `MiniMax-M2.1`, `claude-sonnet-4`, `gpt-4o`). |
| `env` | Key/value map of environment variables injected before launch. Secrets live in the OS keychain when available. |
| `args` | Default CLI arguments appended when running the agent. |
| `working_dir` | Optional override for the process working directory. |
| `metadata` | Arbitrary manifest-specific fields (e.g., JSON path, profile home, last-used timestamps, `created_at`, `last_used`). |
| `metadata.hooks_config` | Optional hooks configuration for event-driven actions. See [Hooks](hooks.md). |
| `metadata.proxy_config` | Optional proxy configuration for request routing. See [Proxy](proxy.md). |

Profiles are serialized as JSON for readability, but the manager exposes a typed API so other persistence backends (SQLite, remote sync) can be added later.

### Model selection precedence

When creating a profile, the model is determined in this order (highest to lowest priority):

1. **`--model` flag** – explicitly specified during `clown profiles create`
2. **Provider default** – `models.default` from the provider manifest (e.g., MiniMax defaults to `MiniMax-M2.1`)
3. **Agent default** – `models.default` from the agent manifest (e.g., Claude defaults to `claude-sonnet-4`)

Example:
```bash
# Uses provider default (MiniMax-M2.1 for minimax provider)
$ clown profiles create claude work --provider minimax

# Overrides with explicit model
$ clown profiles create claude work --provider minimax --model claude-opus-4
```

The profile stores the resolved model at creation time. To change it later, delete and recreate the profile or edit the JSON directly.

### Templates

Profile templates stored in the GitHub registry (see `docs/registry.md`) provide opinionated defaults for each agent/model combination. Use `clown profiles create <agent> <alias> --template <name>` to pre-populate prompts with registry data (env vars, args, setup task recommendations) before entering secrets.

### Advanced options

Profiles can be customized with additional flags during creation:

- `--hooks <hook1,hook2>` – Enable agent hooks like `auto_format` or `auto_lint`
- `--mcp <server1,server2>` – Enable MCP servers like `filesystem` or `github`
- `--bare` – Create a minimal profile without default hooks or MCP servers
- `--proxy` – Enable proxy routing for this profile (requires ultrallm binary)

See [Scripting](scripting.md) for hooks, MCP servers, and Rhai configuration scripts.
See [Hooks](hooks.md) for event-driven hook configuration.
See [Proxy](proxy.md) for intelligent request routing.

## Example

```text
# Create multiple profiles for the same agent with different providers
$ clown profiles create claude work-anthropic --provider anthropic
✔ Agent detected at /usr/local/bin/claude
✔ Using provider: Anthropic API
? Enter your Anthropic API key: sk-ant-...
✔ Created profile home ~/.claude-profiles/work-anthropic
✔ Stored credentials in OS keychain

$ clown profiles create claude work-minimax --provider minimax --model MiniMax-M2.1
✔ Agent detected at /usr/local/bin/claude
✔ Using provider: MiniMax (Anthropic-compatible)
? Enter your MiniMax API key: ...
✔ Created profile home ~/.claude-profiles/work-minimax
✔ Stored credentials in OS keychain

$ clown profiles create claude home-minimax --provider minimax --model MiniMax-M2.1
✔ Using provider: MiniMax (Anthropic-compatible)
? Reuse existing MiniMax credentials? [work-minimax] Yes
✔ Created profile home ~/.claude-profiles/home-minimax

# Create profile with geo-specific endpoint (China region)
$ clown profiles create claude china-minimax --provider minimax --endpoint china
✔ Using provider: MiniMax (Anthropic-compatible)
✔ Using endpoint: china (https://api.minimaxi.com/anthropic)
? Enter your MiniMax API key: ...
✔ Created profile home ~/.claude-profiles/china-minimax

# Install shims for direct access
$ clown aliases install work-anthropic
✔ Installed shim ~/bin/claude-work-anthropic

$ clown aliases install work-minimax
✔ Installed shim ~/bin/claude-work-minimax

# List all profiles
$ clown profiles list --agent claude
Alias              Provider    Endpoint       Model           Last Used
work-anthropic     anthropic   default        claude-sonnet-4   2026-01-08T11:23:51Z
work-minimax       minimax     international  MiniMax-M2.1    2026-01-08T09:18:12Z
home-minimax       minimax     international  MiniMax-M2.1    2026-01-07T22:45:00Z
china-minimax      minimax     china          MiniMax-M2.1    2026-01-07T20:30:00Z

# Run with specific profile
$ clown profiles run work-minimax -- /path/to/project

# Or use shell integration
$ eval "$(clown profiles env work-anthropic)"
$ claude  # Now uses Anthropic API
```

## Manifest Hooks

Agent manifests may define hooks that run during profile events. Common cases include:

- `create`: initialize directories, copy template config files, or generate wrapper scripts.
- `delete`: remove temporary homes or revoke credentials.
- `pre_run`: validate connectivity or refresh tokens.
- `post_run`: clean up temporary files or log session metrics.

Hooks are executed with a short timeout and receive environment variables describing the alias, agent path, and model.

## Profile Hooks

Profiles can also have event-driven hooks that trigger during agent execution. These are different from manifest hooks and provide fine-grained control over tool usage events.

**Profile hooks support:**
- `PreToolUse` – Before a tool is executed
- `PostToolUse` – After a tool completes
- `Notification` – On agent notifications
- `Stop` – When agent stops

Manage profile hooks with `clown hooks add|list|remove|import|export`. See [Hooks](hooks.md) for full documentation.

## CLI aliases

- `clown aliases install <alias>` installs a per-profile shim (e.g., `claude-work-minimax`) inside the user's preferred `bin` directory so the agent can be launched directly without prepending `clown profiles run`.
- Each shim rewrites `HOME` (when required), injects the stored environment variables, and forwards all arguments to the underlying agent—so every alias keeps its own safe home/profile automatically.
- `clown aliases uninstall <alias>` removes the shim without touching the stored profile, letting you recreate or rename aliases freely.

## Manual environment setup commands

Some teams need extra shell manipulation (remapping CLI tool paths, creating symlinks, patching config files) beyond what clown performs automatically. Agent manifests may declare named setup tasks, but they never run implicitly.

- `clown env setup <alias> <task>` executes the requested task with the profile's environment (e.g., `clown env setup work-minimax cli-remap`).
- Tasks can run scripts, copy files, or rewrite symlinks. They must be idempotent and will be audited like other hooks.
- Because these scripts may be destructive, users must call them manually each time they are required; the CLI will never execute them automatically during profile creation or alias installation.

## Background service interplay

When `clownd` is active (the CLI bootstraps it automatically and it shuts down after an idle timeout unless pinned), the daemon takes ownership of profile persistence and publishes change events:

- CLI commands proxy through the daemon over the `async-nng` request/reply channel, avoiding concurrent writes.
- UI clients subscribe to `/profiles/stream` (SSE/WebSocket) or tap into the `async-nng` pub/sub feed to update automatically when counts change.
- Portable export/import endpoints (`GET /profiles/<alias>` / `POST /profiles`) allow cross-device sync workflows.

## Remote terminal sessions

Profiles can be run in "remote" mode, where the agent executes in a PTY (pseudo-terminal) managed by the daemon. This enables:

- **Web UI access** – interact with the terminal through `http://127.0.0.1:8765/terminal`
- **Multi-client viewing** – multiple browsers/clients can connect to the same session
- **Session persistence** – sessions continue running even when you disconnect
- **Scrollback history** – reconnecting shows up to 1MB of terminal history

```bash
# Start a remote session
$ clown profiles run work-anthropic --remote
Session ID: 46e15057-abbb-42cd-ad0e-52471a76ef9f
Connect at: http://127.0.0.1:8765/terminal

# List active sessions
$ clown terminal list

# Terminate a session
$ clown terminal kill <session-id>
```

Sessions can also be created through the web UI or HTTP API. See the Terminal Sessions section in the HTTP API documentation.

## Best practices

- Keep API keys in environment variables or OS keychains rather than committing them to profile files for shared machines.
- Use descriptive aliases (`<agent>-<project>-<model>`) so table summaries and exports remain readable.
- Leverage the `--json` output flag when integrating with scripts or CI systems that orchestrate multiple profiles or backup/import cycles.
