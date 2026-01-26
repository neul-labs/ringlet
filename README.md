# ringlet

[![CI](https://github.com/neul-labs/ringlet/actions/workflows/ci.yml/badge.svg)](https://github.com/neul-labs/ringlet/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ringlet.svg)](https://crates.io/crates/ringlet)
[![Docs](https://img.shields.io/badge/docs-neullabs.com-blue)](https://docs.neullabs.com/ringlet)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey)

**One CLI to rule all your coding agents.**

Stop juggling configs between Claude Code, Grok CLI, Codex, and others. ringlet gives you profiles, provider switching, and intelligent routing—so you can focus on shipping code.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/neul-labs/ringlet/main/install.sh | sh
```

Or build from source:

```bash
git clone https://github.com/neul-labs/ringlet.git
cd ringlet && cargo build --release
```

## Quick Start

```bash
# See what agents you have installed
ringlet agents list

# Create a profile with your preferred provider
ringlet profiles create claude work --provider anthropic
ringlet profiles run work

# Or use MiniMax for cost-effective Claude-compatible requests
ringlet profiles create claude cheap --provider minimax --model MiniMax-M2.1
ringlet profiles run cheap
```

## Why ringlet?

| Problem | Solution |
|---------|----------|
| Different config files for each agent | One profile system for all agents |
| Manual API key management | Secure credential storage per profile |
| Can't easily switch providers | Swap Anthropic/MiniMax/OpenRouter without touching code |
| No visibility into usage | Built-in token tracking and cost analytics |
| Complex proxy setups for routing | Native ultrallm integration with rule-based routing |

## Core Features

### Profile-Based Workflows

```bash
# Create isolated profiles with different providers
ringlet profiles create claude work-anthropic --provider anthropic
ringlet profiles create claude work-minimax --provider minimax

# Install as executable aliases
ringlet aliases install work-anthropic
claude-work-anthropic  # runs with isolated config and credentials
```

### Intelligent Request Routing

```bash
# Enable proxy for smart routing based on context
ringlet profiles create claude routed --provider anthropic --proxy
ringlet proxy enable routed

# Route long-context requests to cost-effective providers
ringlet proxy route add routed "long" "tokens > 100000" "minimax/MiniMax-M2.1" --priority 10
ringlet proxy route add routed "default" "always" "anthropic/claude-sonnet-4"
```

### Event Hooks

```bash
# Add hooks for logging, notifications, or custom workflows
ringlet hooks add work PreToolUse "Bash" "echo '$EVENT' >> ~/.ringlet/audit.log"
ringlet hooks add work Notification "*" "notify-send 'Claude' '$MESSAGE'"
```

### Usage Tracking

```bash
# Track tokens and costs across all profiles
ringlet usage
ringlet usage --period month --profile work

# Import existing Claude usage data
ringlet usage import-claude
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  CLI (ringlet)                                              │
│  └── Thin client, auto-starts daemon                        │
├─────────────────────────────────────────────────────────────┤
│  Daemon (ringletd)                                          │
│  └── Profiles │ Registry │ Telemetry │ Web UI              │
├─────────────────────────────────────────────────────────────┤
│  Agents                                                     │
│  └── Claude Code │ Grok CLI │ Codex │ Droid │ OpenCode     │
├─────────────────────────────────────────────────────────────┤
│  Providers                                                  │
│  └── Anthropic │ MiniMax │ OpenRouter │ Custom             │
└─────────────────────────────────────────────────────────────┘
```

The daemon starts automatically on first CLI use and exits after idle timeout (or pin with `ringlet daemon --stay-alive`).

## CLI Reference

```bash
ringlet agents list              # Show installed agents and versions
ringlet profiles create          # Create a new profile
ringlet profiles list            # List all profiles
ringlet profiles run <name>      # Run agent with profile
ringlet aliases install <name>   # Create executable alias
ringlet proxy status             # Check proxy status
ringlet proxy route list <name>  # List routing rules
ringlet hooks list <name>        # List profile hooks
ringlet usage                    # View usage stats
ringlet registry sync            # Update agent manifests
ringlet daemon --stay-alive      # Keep daemon running
```

## Documentation

Full documentation at **[docs.neullabs.com/ringlet](https://docs.neullabs.com/ringlet)**

| Guide | Description |
|-------|-------------|
| [Architecture](https://docs.neullabs.com/ringlet/architecture) | Component overview and data flow |
| [Agents](https://docs.neullabs.com/ringlet/agents) | Supported agents and custom manifests |
| [Providers](https://docs.neullabs.com/ringlet/providers) | API backends and custom providers |
| [Profiles](https://docs.neullabs.com/ringlet/profiles) | Profile lifecycle and workflows |
| [Hooks](https://docs.neullabs.com/ringlet/hooks) | Event-driven automation |
| [Proxy](https://docs.neullabs.com/ringlet/proxy) | Request routing with ultrallm |
| [Usage](https://docs.neullabs.com/ringlet/usage) | Token tracking and analytics |
| [Scripting](https://docs.neullabs.com/ringlet/scripting) | Rhai configuration scripting |

## Status

| Component | Status |
|-----------|--------|
| CLI + Daemon with IPC | ✅ Stable |
| Agent detection (Claude, Grok, Codex, Droid, OpenCode) | ✅ Stable |
| Provider support (Anthropic, MiniMax, OpenRouter) | ✅ Stable |
| Profile hooks and Rhai scripting | ✅ Stable |
| Proxy routing with ultrallm | ✅ Stable |
| Web UI and HTTP/WebSocket APIs | ✅ Stable |
| Token/cost tracking | ✅ Stable |
| Plugin SDK | 🔜 Planned |
| Cross-device sync | 🔜 Planned |

## Contributing

Open an issue for design discussions before implementing major features. See the [contribution guide](https://docs.neullabs.com/ringlet/contributing) for details.

## License

MIT - see [LICENSE](LICENSE)
