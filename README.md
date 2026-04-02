# ringlet

[![CI](https://github.com/neul-labs/ringlet/actions/workflows/ci.yml/badge.svg)](https://github.com/neul-labs/ringlet/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ringlet.svg)](https://crates.io/crates/ringlet)
[![Docs](https://img.shields.io/badge/docs-neullabs.com-blue)](https://docs.neullabs.com/ringlet)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey)

**Manage every coding agent from one CLI — with isolated profiles, cost control, and security built in.**

## Why ringlet?

- **One place for all agents** — Claude Code, Codex, Grok, Droid, and OpenCode managed through a single interface
- **Profile isolation** — each profile gets its own credentials, config, and history with zero leakage
- **Swap providers instantly** — switch between Anthropic, MiniMax, OpenRouter, or your own gateway without reconfiguring
- **Know what you're spending** — built-in token tracking and cost analytics across every session
- **Secure by default** — keychain credential storage, sandboxed remote sessions, bearer-token daemon auth

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/neul-labs/ringlet/main/install.sh | sh
```

Or with Cargo:

```bash
cargo install ringlet
```

## Quick Start

```bash
ringlet init              # interactive setup — detects agents, creates first profile
ringlet profiles run work # launch an agent session
ringlet usage             # see token spend
```

## Features

**Profile Isolation** — every profile runs in its own HOME with separate config, credentials, and conversation history. No cross-contamination between projects.

**Provider Switching** — bind any agent to any compatible API backend. Move from Anthropic to MiniMax or OpenRouter by creating a new profile — no agent reconfiguration needed.

**Intelligent Routing** — attach an [ultrallm](https://github.com/starbaser/ultrallm) proxy to any profile and define rules that route long-context requests to cheaper providers automatically.

**Usage Analytics** — track tokens, costs, and session time across all profiles. Import existing Claude Code usage data. Export as JSON or CSV for reporting.

**Remote Terminal** — run agent sessions in the daemon and access them from a browser-based terminal. Sessions are sandboxed by default with bwrap (Linux) or sandbox-exec (macOS).

**Event Hooks** — trigger shell commands or webhooks on tool use, notifications, or agent stop events. Build audit logs, Slack alerts, or custom validation pipelines.

**Web Dashboard** — manage profiles, view usage, and launch terminal sessions from a visual UI at `http://127.0.0.1:8765`.

**Desktop App** — native Tauri wrapper for a standalone experience without a browser tab.

## Supported Agents

| Agent | Providers | Status |
|-------|-----------|--------|
| **Claude Code** | Anthropic, MiniMax | Supported |
| **Codex CLI** | OpenAI, OpenRouter | Supported |
| **Grok CLI** | OpenAI-compatible | Supported |
| **Droid CLI** | Anthropic, MiniMax | Supported |
| **OpenCode** | Anthropic, MiniMax | Supported |

## Security

- **Keychain credential storage** — API keys are stored in your system keychain (macOS Keychain, GNOME Keyring), never in plain text
- **Profile isolation** — each profile runs with a separate HOME, preventing credential and config leakage
- **Sandboxed remote sessions** — remote terminal sessions run inside bwrap (Linux) or sandbox-exec (macOS) with read-only system mounts
- **Bearer-token daemon auth** — the HTTP API requires a bearer token from `~/.config/ringlet/http_token`
- **Localhost only** — the daemon binds to `127.0.0.1` by default

## Roadmap

**Current** — profiles, provider switching, usage tracking, proxy routing, hooks, remote terminal, web UI, desktop app

**Planned** — plugin SDK, cross-device sync, richer scripting API

**Team** — shared profiles, centralized provider policy, role-based access

**Enterprise** — SSO integration, audit trails, compliance reporting

## Documentation

Full docs at **[docs.neullabs.com/ringlet](https://docs.neullabs.com/ringlet)**

## Contributing

Open an issue for design discussions before implementing major features. See the [contribution guide](https://docs.neullabs.com/ringlet/contributing) for details.

## License

MIT — see [LICENSE](LICENSE)
