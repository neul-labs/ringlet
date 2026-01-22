# System Architecture

Deep dive into how Clown is designed and how its components interact.

---

## Overview

Clown is a Rust-native workspace built around a central background daemon (`clownd`) that orchestrates CLI coding agents. The daemon owns profile persistence, agent discovery, telemetry collection, and real-time event distribution.

```
┌─────────────────────────────────────────────────────────────────┐
│                        User / UI                                 │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                         clown CLI                                │
│    (Thin client - auto-starts daemon, forwards commands)        │
└──────────────────────────────┬──────────────────────────────────┘
                               │ async-nng / HTTP
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                        clownd (Daemon)                           │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │   Profile    │ │    Agent     │ │   Registry   │             │
│  │   Manager    │ │   Registry   │ │    Client    │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │    Proxy     │ │    Usage     │ │   Scripting  │             │
│  │   Manager    │ │   Tracking   │ │    Engine    │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└──────────────────────────────┬──────────────────────────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          ▼                    ▼                    ▼
   ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
   │ Claude Code │      │  Codex CLI  │      │  ultrallm   │
   └─────────────┘      └─────────────┘      │   Proxy     │
                                             └─────────────┘
```

---

## Design Goals

- **Single entry point** - Detect any supported CLI coding agent installed on the system
- **Profile isolation** - Each profile has completely separate configuration and state
- **Multi-provider support** - Run the same agent against different API backends
- **Centralized orchestration** - Daemon ensures single source of truth
- **Usage tracking** - Monitor token consumption and costs across all profiles
- **Extensibility** - Add new agents through manifests and scripts, not code

---

## Core Components

### clown CLI

A thin client that:

- Parses commands like `agents list`, `profiles create`, `profiles run`
- Auto-starts the daemon if not running
- Forwards requests over `async-nng` sockets
- Renders responses as tables or JSON

The CLI never performs stateful operations directly - everything goes through the daemon.

### clownd (Daemon)

The heart of Clown. Runs as a long-lived background process and owns:

| Responsibility | Description |
|----------------|-------------|
| Profile persistence | Stores and retrieves profile configurations |
| Agent discovery | Detects installed agents and their versions |
| Telemetry collection | Tracks usage, sessions, and costs |
| Event distribution | Publishes changes via WebSocket/pub-sub |
| Proxy management | Spawns and monitors ultrallm instances |
| HTTP API | Serves REST endpoints and Web UI |

The daemon auto-starts on first CLI use and exits after idle timeout unless pinned with `--stay-alive`.

### Profile Manager

Handles profile lifecycle:

1. **Creation** - Validates agent+provider pairing, prompts for API key, runs Rhai script
2. **Storage** - Writes profile JSON to `~/.config/clown/profiles/`
3. **Execution** - Creates isolated environment, injects variables, spawns agent
4. **Tracking** - Updates `last_used` timestamps

### Agent Registry

Loads and manages agent manifests:

- Built-in manifests compiled into binary
- User manifests from `~/.config/clown/agents.d/`
- Registry manifests from GitHub

Detection probes run in parallel to capture versions and paths.

### Provider Registry

Manages API backend definitions:

- Built-in providers (Anthropic, MiniMax, OpenAI, OpenRouter)
- User providers from `~/.config/clown/providers.d/`

Providers define endpoints, authentication, and available models.

### Scripting Engine

Embeds [Rhai](https://rhai.rs/) for configuration generation:

- Receives context (provider, profile, preferences)
- Outputs configuration files and environment variables
- Scripts resolved: user override → registry → built-in

### Registry Client

Synchronizes GitHub-hosted metadata:

- Downloads manifests, templates, and model catalog
- Verifies checksums/signatures
- Caches under `~/.config/clown/registry/`
- Downloads LiteLLM pricing for cost calculation

### Proxy Manager

Manages ultrallm proxy instances:

- Spawns processes on dedicated ports (8080-8180 range)
- Generates routing configuration from profile rules
- Monitors health and handles graceful shutdown
- Auto-starts proxies when profiles with proxy enabled run

### Usage Tracking

Collects and aggregates token usage:

- Per-session records in `sessions.jsonl`
- Rolled-up stats in `aggregates.json`
- Cost calculation using LiteLLM pricing
- Queryable via CLI, HTTP API, and Web UI

---

## Communication

### CLI ↔ Daemon Transport

The CLI and daemon communicate through `async-nng`:

```
CLI                                      Daemon
 │                                         │
 │  ──── RegistrySyncRequest ──────►       │
 │                                         │
 │  ◄──── RegistrySyncResponse ────        │
 │                                         │
```

- **Request/Reply**: Commands serialized via `serde_json`, sent over IPC
- **Pub/Sub**: Change notifications for watch modes and UI clients
- **Endpoint**: `/tmp/clownd.sock` (Unix) or `%LOCALAPPDATA%/clown/clownd.ipc` (Windows)

### HTTP API

For UI integrations that can't speak NNG:

- REST endpoints at `http://127.0.0.1:8765`
- WebSocket at `ws://127.0.0.1:8765/ws`
- Serves embedded Web UI

---

## Profile Isolation

### Home Wrapper Strategy

When a profile runs, Clown creates an isolated environment:

```
Real HOME: /home/user
Profile HOME: /home/user/.claude-profiles/my-project

Environment:
  HOME=/home/user/.claude-profiles/my-project
  ANTHROPIC_BASE_URL=https://api.anthropic.com
  ANTHROPIC_AUTH_TOKEN=sk-...
```

The agent reads configuration from the profile HOME, ensuring:

- Configuration files are isolated
- Conversation history is separate
- Settings don't leak between profiles

### What Gets Isolated

| Isolated | Not Isolated |
|----------|--------------|
| Agent config files | System binaries |
| API credentials | Shell configuration |
| Conversation history | Network access |
| Agent settings | File system access |

---

## Data Flow

### Profile Run Workflow

```
1. User: clown profiles run my-project

2. CLI:
   - Connects to daemon (or starts it)
   - Sends ProfileRunRequest

3. Daemon:
   - Loads profile configuration
   - Retrieves API key from keychain
   - Creates/validates profile home
   - If proxy enabled, starts ultrallm
   - Spawns agent process with:
     - HOME set to profile home
     - Environment variables injected
     - CLI arguments forwarded

4. Agent runs with isolated configuration

5. Daemon tracks:
   - Session start/end times
   - Token usage (from agent files)
   - Runtime duration
```

### Registry Sync Workflow

```
1. CLI sends RegistrySyncRequest to daemon

2. Daemon:
   - Acquires per-channel lock
   - Checks registry.lock for cached data
   - If force or cache stale:
     - Downloads registry.json
     - Verifies checksums
     - Fetches missing artifacts
     - Stages under commits/<sha>/
   - Updates registry.lock
   - Publishes RegistryUpdated event

3. CLI receives response with:
   - Resolved commit
   - Channel
   - Downloaded artifact count
   - Cache/network indicator
```

---

## Persistence Layout

```
~/.config/clown/
├── config.toml               # User preferences
├── daemon-endpoint           # Active daemon endpoint
├── agents.d/                 # Custom agent manifests
├── providers.d/              # Custom provider manifests
├── scripts/                  # Custom Rhai scripts
├── profiles/                 # Profile definitions
│   └── my-project.json
├── registry/                 # Cached registry data
│   ├── current -> commits/f4a12c3
│   ├── registry.lock
│   ├── litellm-pricing.json
│   └── commits/
│       └── f4a12c3/
├── cache/
│   └── agent-detections.json
├── telemetry/
│   ├── sessions.jsonl
│   └── aggregates.json
└── logs/
    └── clownd.log
```

---

## Cross-Platform Support

| Platform | Config Path | IPC Endpoint |
|----------|-------------|--------------|
| macOS | `~/.config/clown/` | `/tmp/clownd.sock` |
| Linux | `~/.config/clown/` or XDG | `/tmp/clownd.sock` |
| Windows | `%APPDATA%\clown\` | `%LOCALAPPDATA%\clown\clownd.ipc` |

Clown uses the `dirs` crate to resolve paths and keeps launcher scripts optional.

---

## Future Directions

- **UI Layer** - Desktop/web frontend connecting to daemon
- **Delta syncs** - Download only changed registry entries
- **Prometheus/OpenTelemetry** - Enterprise observability endpoints
- **Plugin system** - User-defined hooks and extensions
