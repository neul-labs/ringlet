# ringlet (Homebrew)

[![Homebrew](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fneul-labs%2Fhomebrew-ringlet%2Fmain%2FFormula%2Fringlet.rb&query=%24.version&prefix=v&label=homebrew)](https://github.com/neul-labs/homebrew-ringlet)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/neul-labs/ringlet/blob/main/LICENSE)

> **One CLI to orchestrate all your coding agents.**

Ringlet is a cross-platform orchestrator for CLI-based coding agents. It unifies Claude Code, Codex, Grok, Droid, OpenCode, and more behind a single profile manager with usage tracking, provider abstraction, and intelligent request routing.

## Installation

```bash
# Add the tap
brew tap neul-labs/ringlet

# Install ringlet
brew install ringlet
```

Or in one line:

```bash
brew install neul-labs/ringlet/ringlet
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
