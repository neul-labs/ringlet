# Architecture Overview

ringlet is a Rust-native workspace built around a central background daemon (`ringletd`) that orchestrates CLI coding agents. The daemon is the core of the system—it owns profile persistence, agent discovery, telemetry collection, and real-time event distribution. The CLI acts as a thin client that automatically starts the daemon on first use and forwards commands over `async-nng` sockets. This daemon-first design ensures consistent state across CLI invocations, enables future UI integrations, and centralizes observability without duplicating logic.

## Goals

- Provide a single entry point that can detect any supported CLI coding agent installed on a system.
- Track how many profiles exist per agent and surface that information instantly to the CLI and service API.
- Keep configuration portable across macOS, Linux, and Windows by using platform-aware paths and avoiding shell-specific behavior.
- Support multiple model providers (MiniMax, Anthropic, OpenAI-compatible APIs, internal gateways) without baking in vendor-specific assumptions.
- Centralize orchestration in a daemon (`ringletd`) that the CLI auto-starts, ensuring a single source of truth for profiles, agent state, and usage telemetry.
- Collect and expose usage statistics (profile invocations, session durations, resource consumption) so users and teams can understand their agent usage patterns.
- Track token usage and costs per session, with pricing from LiteLLM's model database.
- Make it trivial to add new agents through readable manifests rather than bespoke code.

## Components

### Core library (`ringlet-core`)
Holds shared structs (agents, profiles, manifests), serialization helpers (`serde`), and filesystem abstractions. Both the CLI and the service daemon consume this crate.

### CLI (`ringlet`)
A thin client that parses commands such as `agents list`, `profiles create`, and `profiles run`, then forwards them to the daemon over `async-nng`. On first invocation (or when the daemon is not running), the CLI **automatically spawns `ringletd`** in the background before sending the request—users never need to start the daemon manually. The daemon exits after an idle timeout unless pinned with `ringlet daemon --stay-alive`. The CLI renders responses as structured tables or `--json` for scripting. Direct library calls (bypassing the daemon) are reserved for offline/emergency scenarios only.

### Agent registry
A loader that scans declarative manifest files (TOML/YAML) describing how to detect and run each agent. Built-in manifests for Claude Code, Grok CLI, Codex CLI, Droid, and OpenCode ship with the binary, while additional manifests can be dropped into `~/.config/ringlet/agents.d/`. Detection collects binary paths, versions, and last-used timestamps (persisted under `cache/agent-detections.json`) so `ringlet agents list` can surface both availability and freshness.

### Registry client
Synchronizes GitHub-hosted metadata (manifests, profile templates, model catalog). Responsible for pulling `registry.json`, verifying checksums/signatures, caching artifacts under `~/.config/ringlet/registry/`, and exposing commands such as `registry sync`, `registry pin`, and `registry inspect`. Additionally downloads LiteLLM's `model_prices_and_context_window.json` for usage cost calculations. Enterprises can redirect it to private registries via config/env vars.

### Provider registry
Manages API backend definitions (Anthropic, MiniMax, OpenAI, OpenRouter, etc.) that profiles bind to. Each provider specifies endpoints, authentication requirements, and available models. Built-in providers ship with the binary while custom providers can be added to `~/.config/ringlet/providers.d/`. This separation lets users run the same agent (e.g., Claude Code) against different backends (Anthropic API vs MiniMax API) without modifying agent manifests.

### Scripting engine
Embeds [Rhai](https://rhai.rs/) to dynamically generate agent-specific configuration files. Each agent has a `.rhai` script that receives provider, profile, and user preference context, then outputs the required config files (JSON, TOML, etc.) and environment variables. Scripts are resolved in order: user override (`~/.config/ringlet/scripts/`) → registry → built-in. This approach allows:
- Adding new agents without recompiling (just add manifest + script)
- User customization of configuration logic
- Support for agent-specific features like Claude Code hooks and MCP servers
- Future-proofing as agents add new configuration options

### Profile manager
Stores aliases and environment overrides for each agent+model pairing. When profiles are created, the manager reads manifest metadata to prompt for model names, API keys, and other required values every time so secrets are never reused implicitly. Profiles live under `~/.config/ringlet/profiles/` (or `%APPDATA%/ringlet/profiles/` on Windows) and track metadata such as `last_used` to populate CLI dashboards. Import/export commands serialize the entire setup (agents cache pointer, profiles, registry pin) so users can migrate machines.

### Execution adapter
Transforms a profile definition into a runnable command by injecting env vars, rewriting home directories when required (e.g., Claude Code isolation), passing through CLI arguments, and generating shim executables (e.g., `claude-minimax`) that call into the adapter so users get direct aliases per profile. It also exposes a supervised runner for optional environment setup tasks invoked via `ringlet env setup <alias> <task>` so remapping scripts do not run implicitly.

### Proxy manager
Manages ultrallm proxy instances for profiles that have proxy enabled. Each profile can have its own proxy instance running on a dedicated port (8080-8180 range). The proxy manager handles:

- **Process lifecycle** – spawns ultrallm processes, monitors health, graceful shutdown
- **Port allocation** – automatically assigns unique ports to each profile
- **Config generation** – creates ultrallm config.yaml from profile routing rules
- **Auto-start** – starts proxy automatically when running a profile with proxy enabled
- **Cleanup** – stops all managed proxies when the daemon shuts down

See [Proxy](proxy.md) for user-facing documentation.

### Hooks processor
Handles event-driven hooks configured at the profile level. When agents that support hooks (e.g., Claude Code) execute tools, the hooks processor:

- **Event matching** – matches tool names against configured matchers
- **Action execution** – runs command or URL actions with event data
- **Timeout handling** – respects configured timeouts for command actions

See [Hooks](hooks.md) for user-facing documentation.

### Terminal session manager
Manages remote terminal sessions that allow agents to run in PTY (pseudo-terminal) mode within the daemon. This enables:

- **PTY emulation** – uses `portable-pty` for cross-platform pseudo-terminal support
- **Multi-client access** – multiple WebSocket clients can view and interact with the same session
- **Scrollback buffer** – maintains up to 1MB of terminal history for reconnecting clients
- **Session lifecycle** – tracks session state (starting, running, terminated) and exit codes
- **Working directory** – supports custom working directories per session

Terminal sessions are accessed via WebSocket (`/ws/terminal/{session_id}`) for real-time I/O (binary data) and control messages (JSON). REST endpoints manage session creation, listing, and termination.

See the HTTP API documentation for endpoint details.

### Usage tracking
Collects token usage (input, output, cache creation, cache read) for every agent session. The system tracks:

- **Token metrics** – input tokens, output tokens, cache creation tokens, cache read tokens
- **Cost calculation** – uses LiteLLM's `model_prices_and_context_window.json` pricing database
- **Aggregations** – rolled-up stats by profile, model, and date

Cost calculation only runs for profiles using the "self" provider (direct API keys), since other providers handle billing differently. Usage data is queryable via CLI (`ringlet usage`), HTTP API (`GET /api/usage`), and the embedded Web UI.

See [Usage](usage.md) for user-facing documentation.

### Background service (`ringletd`) – the core
The daemon is the heart of ringlet. It runs as a long-lived process (auto-started by the CLI) and owns all stateful operations:

- **RPC over `async-nng`** – exposes request/reply sockets that the CLI connects to for every command; this is the canonical communication path.
- **Telemetry collection** – tracks profile usage (invocation counts, timestamps), session durations, and resource consumption (memory, CPU where available). Stats are persisted under `~/.config/ringlet/telemetry/` and surfaced via `ringlet stats` commands.
- **Event distribution** – publishes profile, agent, and registry change notifications over `async-nng` pub/sub sockets so CLI watch modes and UI clients stay synchronized.
- **HTTP/WebSocket APIs** – optional loopback endpoints for UI integrations that cannot speak NNG directly.
- **Filesystem watching** – monitors config directories for external changes and reconciles state.

The CLI is intentionally thin; it auto-spawns the daemon on first use and delegates all real work to it.

### Web UI
A Vue 3 single-page application embedded in the daemon binary (via `rust-embed`) and served at `http://127.0.0.1:8765`. The UI provides:

- **Profile management** – view, create, and manage profiles
- **Usage dashboard** – visualize token usage and costs
- **Remote terminal** – full interactive terminal sessions using xterm.js
- **Real-time updates** – WebSocket connections for live session state

The UI connects to the same HTTP/WebSocket APIs available to external tools, keeping terminal workflows and graphical workflows consistent.

## CLI ↔ daemon transport

The CLI and daemon communicate through `async-nng`, the asynchronous bindings to the NNG (Nanomsg Next Generation) messaging library. This choice keeps the transport cross-platform, embeds cleanly inside the `tokio` runtime, and gives ringlet a uniform abstraction for both request/response RPC traffic and event fan-out.

- **Request/Response:** Each CLI command is serialized (via `serde_json`) into a `req` message that the daemon receives over an `ipc://` endpoint (macOS/Linux under `/tmp/ringletd.sock`, Windows under `%LOCALAPPDATA%/ringlet/ringletd.ipc`). Responses include status codes, stdout/stderr payloads, and optional streaming hints so the CLI can render tables or pass through JSON.
- **Event stream:** The daemon also owns a `pub` socket that emits profile, agent, and registry change notifications. Clients that need real-time updates (CLI watch mode, future UI bridges) bind `sub` sockets or bridge the feed into SSE/WebSocket endpoints.
- **Discovery:** A small bootstrap file (`~/.config/ringlet/daemon-endpoint`) records the active endpoint so the CLI can reconnect after restarts; the endpoint can also be overridden with `CLOWN_DAEMON_ENDPOINT` for tests.

By standardizing on `async-nng`, the daemon avoids ad-hoc socket handling while still presenting optional HTTP/WebSocket surfaces for external tools that cannot speak NNG directly.

## Registry sync pipeline

`ringlet registry sync` is implemented as its own RPC routed over the `async-nng` request socket:

1. The CLI serializes a `RegistrySyncRequest` (requested channel, pin, flags such as `--offline` or `--force`) and sends it to the daemon.
2. The daemon’s registry client acquires a per-channel lock, reads overrides such as `CLOWN_REGISTRY_URL`/`CLOWN_REGISTRY_CHANNEL`, checks `~/.config/ringlet/registry/registry.lock`, and skips network work when the cache already satisfies the request unless `--force` demands a refresh.
3. When online, the daemon downloads `registry.json`, verifies its checksum/signature, fetches any referenced manifests/templates/models that are missing locally, and writes the results into `~/.config/ringlet/registry/commits/<sha>/`.
4. Fresh downloads update `registry.lock` with the resolved commit hash, channel, timestamp, and cache metadata, then trigger a `RegistryUpdated` event on the `async-nng` pub socket so CLIs/UI clients can refresh immediately.
5. The RPC response summarizes the resolved commit, channel, counts of downloaded artifacts, and whether the data came from cache or a network sync; offline mode returns the currently pinned commit with `offline=true`.

The same daemon hooks drive scheduled refreshes (when the cached commit ages past a configurable threshold) and power `registry pin`/`registry inspect` by reusing existing artifacts instead of re-downloading them.

## Data model

### Provider manifest sketch (TOML)

```toml
id = "minimax"
name = "MiniMax"
type = "anthropic-compatible"   # "anthropic" | "anthropic-compatible" | "openai" | "openai-compatible"

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

Note: Configuration generation is handled by Rhai scripts. See `docs/scripting.md`.

### Agent manifest sketch (TOML)

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
script = "claude.rhai"  # Rhai script that generates config

[models]
default = "claude-sonnet-4"
supported = ["claude-sonnet-4", "claude-opus-4", "MiniMax-M2.1"]

[hooks]
create = []
delete = []
pre_run = []
post_run = []
```
The `script` field references a Rhai script that generates agent-specific configuration files. See `docs/scripting.md` for the scripting interface.

### Profile schema sketch (JSON)

```json
{
  "alias": "claude-work-minimax",
  "agent_id": "claude",
  "provider_id": "minimax",
  "endpoint_id": "international",
  "model": "MiniMax-M2.1",
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.minimax.io/anthropic",
    "ANTHROPIC_AUTH_TOKEN": "$KEYCHAIN:minimax-claude-work-minimax"
  },
  "args": ["--settings", "./strict.json"],
  "working_dir": "/Users/example/projects/workspace",
  "metadata": {
    "created_at": "2026-01-08T10:00:00Z",
    "last_used": "2026-01-08T12:00:00Z"
  }
}
```

## Workflow summaries

- **List agents**: registry loads manifests → detection probes run in parallel, capturing version numbers and last-used timestamps → CLI prints a table with installation path, version, last-used, and profile count (queried from the profile manager).
- **Create profile**: CLI validates the agent+provider pairing (optionally pulling defaults from the GitHub registry/template) → Rhai script generates agent-specific config files → profile manager writes schema to disk → optional post-create hook runs → CLI displays next steps.
- **Switch profile/run alias**: CLI/service fetch the profile → execution adapter spawns agent process with injected env vars and rewired HOME when necessary → profile manager updates the `last_used` field.
- **Sync registry**: CLI issues a `RegistrySyncRequest` → daemon downloads/validates `registry.json` (or reuses the cached commit when offline) → artifacts land under `~/.config/ringlet/registry/commits/<sha>/` and a change event is published so other clients update automatically.
- **Export/import setup**: `ringlet export > backup.json` packages profiles, registry pin, and optional cached manifests → `ringlet import backup.json` rehydrates the environment on a new machine.

## Persistence layout

```
~/.config/ringlet/
├── config.toml               # user preferences, hooks, MCP defaults
├── agents.d/                 # user-supplied agent manifests
│   └── custom-agent.toml
├── providers.d/              # user-supplied provider definitions
│   └── custom-provider.toml
├── scripts/                  # user-override Rhai scripts
│   └── claude.rhai           # overrides built-in claude.rhai
├── registry/                 # cached GitHub metadata
│   ├── current -> commits/f4a12c3
│   ├── registry.lock
│   ├── litellm-pricing.json  # LiteLLM model pricing for cost calculation
│   └── commits/
│       └── f4a12c3/
│           ├── registry.json
│           ├── agents/
│           ├── providers/
│           ├── scripts/
│           ├── profiles/
│           └── models/
├── daemon-endpoint           # records active async-nng endpoint
├── profiles/
│   └── claude-work.json
├── cache/
│   └── agent-detections.json
├── telemetry/                # daemon-collected usage stats
│   ├── sessions.jsonl        # per-session records with token/cost data
│   └── aggregates.json       # rolled-up stats per profile/model
└── logs/
    └── ringletd.log
```

Windows uses `%APPDATA%\\ringlet\\` and Linux adheres to the XDG Base Directory spec when variables are defined.

## Background service plan

1. Embed `tokio` runtime to host schedulers, watchers, telemetry collectors, and HTTP handlers.
2. Provide `async-nng` request/reply sockets as the canonical CLI transport (`ipc://` under `/tmp/ringletd.sock` on macOS/Linux, `ipc://` under `%LOCALAPPDATA%/ringlet/ringletd.ipc` on Windows, with an opt-in `tcp://127.0.0.1:<port>` fallback) while still exposing a loopback HTTP endpoint for UI integrations that cannot speak NNG.
3. Auto-start on demand when the CLI issues any command; exit once an idle timeout is reached unless `ringlet daemon --stay-alive` keeps it resident. The CLI always attempts to connect to (or spawn) the daemon first.
4. Mirror CLI commands as structured RPC payloads consumed over the `async-nng` channel and, when needed, expose equivalent HTTP routes (`GET /agents`, `POST /profiles`) for graphical clients.
5. Publish change notifications via `async-nng` pub/sub sockets and mirror them to Server-Sent Events or WebSockets so UI clients stay in sync without polling.
6. Schedule background registry syncs (respecting offline mode) so metadata stays fresh.
7. **Host remote terminal sessions** via PTY emulation, allowing agents to run interactively and be accessed through the web UI or multiple CLI/WebSocket clients simultaneously.
8. **Collect usage telemetry:**
   - Profile invocation counts and timestamps (`last_used`, `total_runs`)
   - Session durations (time from `profiles run` start to exit)
   - Resource consumption snapshots (peak memory, CPU time) for supervised agent processes
   - Aggregate stats per agent, provider, and model for dashboard views
   - Persist telemetry under `~/.config/ringlet/telemetry/` with rotation/compaction
9. Expose structured logs and telemetry via `ringlet stats` commands and optional Prometheus/OpenTelemetry endpoints for enterprise observability.

## Cross-platform considerations

- Use `dirs`/`directories` crates to resolve configuration paths.
- Keep launcher scripts optional—prefer Rust-based process control over Bash wrappers when possible.
- Validate that agent detection commands exist before executing to avoid hanging shells on Windows.
- Always allow overriding paths/environment variables so enterprise deployments can point at mirrored binaries.
