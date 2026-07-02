# ringlet-cli

[![npm version](https://img.shields.io/npm/v/ringlet-cli)](https://www.npmjs.com/package/ringlet-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/neul-labs/ringlet/blob/main/LICENSE)
[![Node.js 16+](https://img.shields.io/badge/node-%3E%3D16-brightgreen.svg)](https://nodejs.org/)

> **One CLI to orchestrate all your coding agents.**

Ringlet is a cross-platform orchestrator for CLI-based coding agents. It unifies Claude Code, Codex, Grok, Droid, OpenCode, and more behind a single profile manager with usage tracking, provider abstraction, and intelligent request routing.

**[Site](https://ringlet.neullabs.com) · [Docs](https://docs.neullabs.com/ringlet) · [Repository](https://github.com/neul-labs/ringlet)**

## Installation

```bash
npm install -g ringlet-cli
```

Or with your preferred package manager:

```bash
# pnpm
pnpm add -g ringlet-cli

# yarn
yarn global add ringlet-cli

# bun
bun add -g ringlet-cli
```

## Quick Start

```bash
# Initialize ringlet (detects installed agents)
ringlet init

# List available agents
ringlet agents list

# Create a profile for your project
ringlet profiles create --alias myproject --agent claude --provider anthropic

# Run the agent
ringlet profiles run myproject

# Check usage statistics
ringlet usage
```

## Features

- **Profile Management** — Isolated profiles per project with custom environment variables, working directories, and provider settings
- **Agent Discovery** — Auto-detects installed agents (Claude Code, Grok, Codex, Droid, OpenCode)
- **Provider Abstraction** — Switch between Anthropic, OpenAI, OpenRouter, MiniMax, and custom backends
- **Usage Tracking** — Real-time token and cost analytics across all profiles and agents
- **Proxy Integration** — Intelligent request routing with LiteLLM-compatible proxy layer
- **Remote Terminal** — WebSocket-based terminal sessions with sandboxing support

## Why Ringlet?

If you work across multiple AI coding agents, you know the pain of scattered configuration files, no cost visibility, and reconfiguring everything when switching providers. Ringlet solves this with a single daemon that manages profiles, tracks usage, and coordinates agent execution.

## Supported Agents

| Agent | Status | Install Command |
|-------|--------|-----------------|
| Claude Code | Supported | `npm install -g @anthropic-ai/claude-code` |
| Codex CLI | Supported | Built-in with OpenAI CLI |
| Grok CLI | Supported | `pip install grok-cli` |
| Droid | Supported | Standalone binary |
| OpenCode | Supported | Standalone binary |

## Documentation

For full documentation, visit: https://docs.neullabs.com/ringlet

## License

MIT License — see [LICENSE](https://github.com/neul-labs/ringlet/blob/main/LICENSE)

## Part of the Neul Labs toolchain

Ringlet is part of the Neul Labs orchestration toolchain:

| Project | Description |
|---------|-------------|
| [brat](https://github.com/neul-labs/brat) | Multi-agent harness for AI coding tools — crash-safe state, parallel execution. |
| [fastworker](https://github.com/neul-labs/fastworker) | Background tasks in Python with zero infrastructure — no Redis, no RabbitMQ. |
| [m9m](https://github.com/neul-labs/m9m) | The n8n alternative without the bugs — one Go binary. |
| [conductor](https://github.com/neul-labs/conductor) | Multi-agent CLI orchestrator for AI coding agents. |

Learn more at [neullabs.com](https://www.neullabs.com).
