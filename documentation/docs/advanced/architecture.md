# Architecture

How Ringlet is designed and how its components work together.

---

## Overview

Ringlet is a single binary (`ringlet`) that contains both the CLI client and a background daemon. The daemon manages all persistent state — profiles, agent discovery, usage tracking, and real-time events. The CLI communicates with the daemon over IPC.

```
┌─────────────────────────────────────────────────────────────────┐
│                        User / UI                                 │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                      ringlet CLI                                 │
│    (Thin client — auto-starts daemon, forwards commands)         │
└──────────────────────────────┬──────────────────────────────────┘
                               │ IPC / HTTP
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                      ringlet daemon                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│  │   Profile    │ │    Agent     │ │   Registry   │            │
│  │   Manager    │ │   Registry   │ │    Client    │            │
│  └──────────────┘ └──────────────┘ └──────────────┘            │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│  │    Proxy     │ │    Usage     │ │   Scripting  │            │
│  │   Manager    │ │   Tracking   │ │    Engine    │            │
│  └──────────────┘ └──────────────┘ └──────────────┘            │
│  ┌──────────────┐ ┌──────────────┐                             │
│  │  Terminal    │ │   HTTP/WS    │                             │
│  │   Manager    │ │   Server     │                             │
│  └──────────────┘ └──────────────┘                             │
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

- **Single entry point** — one binary that detects, configures, and manages all supported coding agents
- **Profile isolation** — every profile has completely separate configuration and state
- **Multi-provider support** — run the same agent against different API backends
- **Centralized state** — the daemon is the single source of truth
- **Usage tracking** — monitor token consumption and costs across all profiles
- **Extensibility** — add new agents through manifests and scripts, not code changes

---

## Unified Binary

Ringlet ships as a single binary. When you run a CLI command, it auto-starts the daemon if one isn't already running. The daemon shuts down after an idle timeout unless pinned with `--stay-alive`.

```bash
# CLI mode — runs a command and exits
ringlet profiles list

# Daemon mode — starts the background service
ringlet daemon --stay-alive --foreground

# The daemon also starts automatically on first CLI use
```

This eliminates the need for separate binaries or manual daemon management.

---

## Core Components

### CLI

A thin client that:

- Parses commands (`agents list`, `profiles create`, `profiles run`)
- Auto-starts the daemon if not running
- Forwards requests over IPC
- Renders responses as tables or JSON

The CLI never performs stateful operations directly — everything goes through the daemon.

### Daemon

The heart of Ringlet. Runs as a long-lived background process and manages:

| Responsibility | Description |
|----------------|-------------|
| Profile persistence | Stores and retrieves profile configurations |
| Agent discovery | Detects installed agents and their versions |
| Usage tracking | Tracks tokens, sessions, and costs |
| Event distribution | Publishes changes via WebSocket |
| Proxy management | Spawns and monitors ultrallm instances |
| Terminal sessions | Manages remote PTY sessions with sandboxing |
| HTTP API | Serves REST endpoints and the web UI |

### Profile Manager

Handles the profile lifecycle:

1. **Creation** — validates agent + provider pairing, prompts for API key, runs Rhai script
2. **Storage** — writes profile JSON to `~/.config/ringlet/profiles/`
3. **Execution** — creates isolated environment, injects variables, spawns agent
4. **Tracking** — updates `last_used` timestamps

### Agent Registry

Loads and manages agent manifests:

- Built-in manifests compiled into the binary
- User manifests from `~/.config/ringlet/agents.d/`
- Registry manifests from GitHub

Detection probes run in parallel to find installed agents and their versions.

### Scripting Engine

Embeds [Rhai](https://rhai.rs/) for configuration generation:

- Receives context (provider, profile, preferences)
- Outputs configuration files and environment variables
- Resolution order: user override → registry → built-in

### Proxy Manager

Manages ultrallm proxy instances:

- Spawns processes on dedicated ports
- Generates routing configuration from profile rules
- Monitors health and handles graceful shutdown
- Auto-starts proxies when profiles with proxy enabled run

### Terminal Manager

Manages remote PTY sessions:

- Creates sandboxed agent processes (bwrap on Linux, sandbox-exec on macOS)
- Streams terminal I/O over WebSocket
- Supports multiple concurrent clients per session
- Maintains scrollback buffer for reconnection

---

## Communication

### CLI to Daemon

The CLI and daemon communicate through IPC:

- **Request/Reply** — commands serialized via JSON, sent over IPC sockets
- **Endpoint** — `/tmp/ringlet.sock` (Unix) or `%LOCALAPPDATA%\ringlet\ringlet.ipc` (Windows)

### HTTP API

For UI integrations:

- REST endpoints at `http://127.0.0.1:8765`
- WebSocket at `ws://127.0.0.1:8765/ws` for real-time events
- Terminal WebSocket at `ws://127.0.0.1:8765/ws/terminal/{id}`
- Embedded web UI served at the root path
- Bearer-token authentication from `~/.config/ringlet/http_token`

---

## Profile Isolation

### Home Wrapper Strategy

When a profile runs, Ringlet creates an isolated environment:

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

## Sandbox Architecture

Remote terminal sessions are sandboxed by default:

```
┌───────────────────────────────────────────────────┐
│                   Host System                      │
│  ┌─────────────────────────────────────────────┐  │
│  │        Sandbox (bwrap / sandbox-exec)        │  │
│  │  ┌───────────────────────────────────────┐  │  │
│  │  │        Agent Process (claude)          │  │  │
│  │  │                                        │  │  │
│  │  │  Read-only:  /usr, /bin, /lib, /etc   │  │  │
│  │  │  Read-write: ~/, working_dir, /tmp    │  │  │
│  │  │  Network:    allowed (API access)      │  │  │
│  │  └───────────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

See [Security](../security.md) for full details on sandbox configuration and custom rules.

---

## Data Flow

### Profile Run

```
1. User: ringlet profiles run my-project

2. CLI:
   - Connects to daemon (or starts it)
   - Sends run request

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
   - Token usage
   - Runtime duration
```

---

## Persistence Layout

```
~/.config/ringlet/
├── config.toml               # User preferences
├── http_token                 # Daemon auth token
├── agents.d/                 # Custom agent manifests
├── providers.d/              # Custom provider manifests
├── scripts/                  # Custom Rhai scripts
├── profiles/                 # Profile definitions
│   └── my-project.json
├── registry/                 # Cached registry data
│   ├── current -> commits/f4a12c3
│   ├── registry.lock
│   └── litellm-pricing.json
├── cache/
│   └── agent-detections.json
├── telemetry/                # Usage data
└── logs/
    └── daemon.log
```

---

## Cross-Platform Support

| Platform | Config Path | IPC Endpoint |
|----------|-------------|--------------|
| macOS | `~/.config/ringlet/` | `/tmp/ringlet.sock` |
| Linux | `~/.config/ringlet/` or XDG | `/tmp/ringlet.sock` |
| Windows | `%APPDATA%\ringlet\` | `%LOCALAPPDATA%\ringlet\ringlet.ipc` |
