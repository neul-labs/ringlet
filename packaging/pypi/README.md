# ringlet-cli

[![PyPI version](https://img.shields.io/pypi/v/ringlet-cli)](https://pypi.org/project/ringlet-cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/neul-labs/ringlet/blob/main/LICENSE)
[![Python 3.8+](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)

> **One CLI to orchestrate all your coding agents.**

Ringlet is a cross-platform orchestrator for CLI-based coding agents. It unifies Claude Code, Codex, Grok, Droid, OpenCode, and more behind a single profile manager with usage tracking, provider abstraction, and intelligent request routing.

**[Site](https://ringlet.neullabs.com) · [Docs](https://docs.neullabs.com/ringlet) · [Repository](https://github.com/neul-labs/ringlet)**

## Why Ringlet?

If you work across multiple AI coding agents, you know the pain:
- Different configuration files scattered everywhere
- No visibility into token usage or costs across agents
- Switching providers means reconfiguring everything
- No unified way to manage profiles per project

Ringlet solves this with a single daemon that manages profiles, tracks usage, and coordinates agent execution.

## Installation

```bash
pip install ringlet-cli
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

- **Profile Management** — Create isolated profiles per project with custom environment variables, working directories, and provider settings
- **Agent Discovery** — Auto-detects installed agents (Claude Code, Grok, Codex, Droid, OpenCode)
- **Provider Abstraction** — Switch between Anthropic, OpenAI, OpenRouter, MiniMax, and custom backends without reconfiguring agents
- **Usage Tracking** — Real-time token and cost analytics across all profiles and agents
- **Proxy Integration** — Intelligent request routing with LiteLLM-compatible proxy layer
- **Remote Terminal** — WebSocket-based terminal sessions with sandboxing support
- **Hooks System** — Event-driven hooks for custom integrations

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
