# ringlet (Debian / Ubuntu)

[![Debian package](https://img.shields.io/badge/debian-package-blue.svg)](https://github.com/neul-labs/ringlet/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/neul-labs/ringlet/blob/main/LICENSE)

> **One CLI to orchestrate all your coding agents.**

Ringlet is a cross-platform orchestrator for CLI-based coding agents. It unifies Claude Code, Codex, Grok, Droid, OpenCode, and more behind a single profile manager with usage tracking, provider abstraction, and intelligent request routing.

## Installation

### From .deb release

Download the latest `.deb` from the [releases page](https://github.com/neul-labs/ringlet/releases) and install:

```bash
sudo dpkg -i ringlet_*.deb
sudo apt-get install -f  # fix any missing dependencies
```

### From repository (coming soon)

```bash
# Add the repository (instructions will be provided once available)
# sudo add-apt-repository ppa:neul-labs/ringlet
# sudo apt update
# sudo apt install ringlet
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

For full documentation, visit: https://github.com/neul-labs/ringlet

## License

MIT License — see [LICENSE](https://github.com/neul-labs/ringlet/blob/main/LICENSE)
