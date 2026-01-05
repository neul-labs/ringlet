# Architecture Overview

ccswitch is a Rust-native workspace that balances an ergonomic CLI with a future background service and UI. The design keeps discovery logic, profile persistence, and agent execution loosely coupled so new agent integrations and transports stay lightweight.

## Goals

- Provide a single entry point that can detect any supported CLI coding agent installed on a system.
- Track how many profiles exist per agent and surface that information instantly to the CLI and service API.
- Keep configuration portable across macOS, Linux, and Windows by using platform-aware paths and avoiding shell-specific behavior.
- Support multiple model providers (MiniMax, Anthropic, OpenAI-compatible APIs, internal gateways) without baking in vendor-specific assumptions.
- Grow from a simple CLI to a persistent daemon (`ccswitchd`) without duplicating business logic.
- Make it trivial to add new agents through readable manifests rather than bespoke code.

## Components

### Core library (`ccswitch-core`)
Holds shared structs (agents, profiles, manifests), serialization helpers (`serde`), and filesystem abstractions. Both the CLI and the service daemon consume this crate.

### CLI (`ccswitch`)
Parses commands such as `agents list`, `profiles create`, and `profiles switch`. Command handlers call into the core library, emit structured table output, and offer `--json` for scripting.

### Agent registry
A loader that scans declarative manifest files (TOML/YAML) describing how to detect and run each agent. Built-in manifests for Claude Code, Grok CLI, Codex CLI, Droid, and OpenCode ship with the binary, while additional manifests can be dropped into `~/.config/ccswitch/agents.d/`. Detection collects binary paths, versions, and last-used timestamps (persisted under `cache/agent-detections.json`) so `ccswitch agents list` can surface both availability and freshness.

### Registry client
Synchronizes GitHub-hosted metadata (manifests, profile templates, model catalog). Responsible for pulling `registry.json`, verifying checksums/signatures, caching artifacts under `~/.config/ccswitch/registry/`, and exposing commands such as `registry sync`, `registry pin`, and `registry inspect`. Enterprises can redirect it to private registries via config/env vars.

### Profile manager
Stores aliases and environment overrides for each agent+model pairing. When profiles are created, the manager reads manifest metadata to prompt for model names, API keys, and other required values every time so secrets are never reused implicitly. Profiles live under `~/.config/ccswitch/profiles/` (or `%APPDATA%/ccswitch/profiles/` on Windows) and track metadata such as `last_used` to populate CLI dashboards. Import/export commands serialize the entire setup (agents cache pointer, profiles, registry pin) so users can migrate machines.

### Execution adapter
Transforms a profile definition into a runnable command by injecting env vars, rewriting home directories when required (e.g., Claude Code isolation), passing through CLI arguments, and generating shim executables (e.g., `claude-minimax`) that call into the adapter so users get direct aliases per profile. It also exposes a supervised runner for optional environment setup tasks invoked via `ccswitch env setup <alias> <task>` so remapping scripts do not run implicitly.

### Background service (`ccswitchd`)
Runs as a long-lived process, exposes local HTTP/WebSocket APIs, watches for filesystem changes, and notifies any UI clients when agents or profiles change. The CLI can either run standalone or proxy commands through the daemon when it is active.

### UI layer (planned)
A small desktop/web frontend that connects to `ccswitchd` to visualize profiles, switch contexts, and trigger commands. The same APIs keep terminal workflows and graphical workflows consistent.

## Data model

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
env = ["ANTHROPIC_BASE_URL", "ANTHROPIC_AUTH_TOKEN"]

[models]
default = "MiniMax-M2.1"
extras = ["MiniMax-M2.1", "claude-3", "gpt-4"]
```
The example lists MiniMax and other Anthropic/OpenAI-compatible identifiers to illustrate that manifests can enumerate any model supported by the underlying agent.

### Profile schema sketch (JSON)

```json
{
  "alias": "claude-work",
  "agent_id": "claude",
  "model": "MiniMax-M2.1",
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.minimax.io/anthropic",
    "ANTHROPIC_AUTH_TOKEN": "$MINIMAX_API_KEY"
  },
  "launcher": {
    "args": ["--settings", "./strict.json"],
    "working_dir": "/Users/example/projects/workspace"
  }
}
```

## Workflow summaries

- **List agents**: registry loads manifests → detection probes run in parallel, capturing version numbers and last-used timestamps → CLI prints a table with installation path, version, last-used, and profile count (queried from the profile manager).
- **Create profile**: CLI validates the agent+model pairing (optionally pulling defaults from the GitHub registry/template) → profile manager writes schema to disk → optional post-create hook runs (e.g., `claude-profile` scaffold) → CLI displays next steps.
- **Switch profile/run alias**: CLI/service fetch the profile → execution adapter spawns agent process with injected env vars and rewired HOME when necessary → profile manager updates the `last_used` field.
- **Sync registry**: CLI hits GitHub (or configured source) → downloads/validates `registry.json` and referenced manifests/templates/models → caches them locally for offline use.
- **Export/import setup**: `ccswitch export > backup.json` packages profiles, registry pin, and optional cached manifests → `ccswitch import backup.json` rehydrates the environment on a new machine.

## Persistence layout

```
~/.config/ccswitch/
├── config.toml               # user preferences and defaults
├── agents.d/                 # user-supplied agent manifests
│   └── custom-agent.toml
├── registry/                 # cached GitHub metadata
│   ├── current -> commits/f4a12c3
│   └── commits/
│       └── f4a12c3/
│           ├── registry.json
│           ├── agents/
│           ├── profiles/
│           └── models/
├── profiles/
│   └── claude-work.json
├── cache/
│   └── agent-detections.json
└── logs/
    └── ccswitchd.log
```

Windows uses `%APPDATA%\\ccswitch\\` and Linux adheres to the XDG Base Directory spec when variables are defined.

## Background service plan

1. Embed `tokio` runtime to host schedulers, watchers, and HTTP handlers.
2. Provide a local Unix socket (macOS/Linux) or named pipe (Windows) plus optional loopback HTTP endpoint for the UI.
3. Auto-start on demand when the CLI issues a command that benefits from the daemon; exit once an idle timeout is reached unless `ccswitch daemon --stay-alive` keeps it resident.
4. Mirror CLI commands as API routes (e.g., `GET /agents`, `POST /profiles`).
5. Publish change notifications via Server-Sent Events or WebSockets so UI clients stay in sync without polling.
6. Schedule background registry syncs (respecting offline mode) so metadata stays fresh.
7. Expose structured logs/metrics for future observability integrations.

## Cross-platform considerations

- Use `dirs`/`directories` crates to resolve configuration paths.
- Keep launcher scripts optional—prefer Rust-based process control over Bash wrappers when possible.
- Validate that agent detection commands exist before executing to avoid hanging shells on Windows.
- Always allow overriding paths/environment variables so enterprise deployments can point at mirrored binaries.
