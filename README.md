# clown

[![CI](https://github.com/neul-labs/clown/actions/workflows/ci.yml/badge.svg)](https://github.com/neul-labs/clown/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/clown.svg)](https://crates.io/crates/clown)
[![Docs](https://img.shields.io/badge/docs-neullabs.com-blue)](https://docs.neullabs.com/clown)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey)

**One CLI to rule all your coding agents.**

Stop juggling configs between Claude Code, Grok CLI, Codex, and others. clown gives you profiles, provider switching, and intelligent routing—so you can focus on shipping code.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/neul-labs/clown/main/install.sh | sh
```

Or build from source:

```bash
git clone https://github.com/neul-labs/clown.git
cd clown && cargo build --release
```

## Quick Start

```bash
# See what agents you have installed
clown agents list

# Create a profile with your preferred provider
clown profiles create claude work --provider anthropic
clown profiles run work

# Or use MiniMax for cost-effective Claude-compatible requests
clown profiles create claude cheap --provider minimax --model MiniMax-M2.1
clown profiles run cheap
```

## Why clown?

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
clown profiles create claude work-anthropic --provider anthropic
clown profiles create claude work-minimax --provider minimax

# Install as executable aliases
clown aliases install work-anthropic
claude-work-anthropic  # runs with isolated config and credentials
```

### Intelligent Request Routing

```bash
# Enable proxy for smart routing based on context
clown profiles create claude routed --provider anthropic --proxy
clown proxy enable routed

# Route long-context requests to cost-effective providers
clown proxy route add routed "long" "tokens > 100000" "minimax/MiniMax-M2.1" --priority 10
clown proxy route add routed "default" "always" "anthropic/claude-sonnet-4"
```

### Event Hooks

```bash
# Add hooks for logging, notifications, or custom workflows
clown hooks add work PreToolUse "Bash" "echo '$EVENT' >> ~/.clown/audit.log"
clown hooks add work Notification "*" "notify-send 'Claude' '$MESSAGE'"
```

### Usage Tracking

```bash
# Track tokens and costs across all profiles
clown usage
clown usage --period month --profile work

# Import existing Claude usage data
clown usage import-claude
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  CLI (clown)                                                │
│  └── Thin client, auto-starts daemon                        │
├─────────────────────────────────────────────────────────────┤
│  Daemon (clownd)                                            │
│  └── Profiles │ Registry │ Telemetry │ Web UI              │
├─────────────────────────────────────────────────────────────┤
│  Agents                                                     │
│  └── Claude Code │ Grok CLI │ Codex │ Droid │ OpenCode     │
├─────────────────────────────────────────────────────────────┤
│  Providers                                                  │
│  └── Anthropic │ MiniMax │ OpenRouter │ Custom             │
└─────────────────────────────────────────────────────────────┘
```

The daemon starts automatically on first CLI use and exits after idle timeout (or pin with `clown daemon --stay-alive`).

## CLI Reference

```bash
clown agents list              # Show installed agents and versions
clown profiles create          # Create a new profile
clown profiles list            # List all profiles
clown profiles run <name>      # Run agent with profile
clown aliases install <name>   # Create executable alias
clown proxy status             # Check proxy status
clown proxy route list <name>  # List routing rules
clown hooks list <name>        # List profile hooks
clown usage                    # View usage stats
clown registry sync            # Update agent manifests
clown daemon --stay-alive      # Keep daemon running
```

## Documentation

Full documentation at **[docs.neullabs.com/clown](https://docs.neullabs.com/clown)**

| Guide | Description |
|-------|-------------|
| [Architecture](https://docs.neullabs.com/clown/architecture) | Component overview and data flow |
| [Agents](https://docs.neullabs.com/clown/agents) | Supported agents and custom manifests |
| [Providers](https://docs.neullabs.com/clown/providers) | API backends and custom providers |
| [Profiles](https://docs.neullabs.com/clown/profiles) | Profile lifecycle and workflows |
| [Hooks](https://docs.neullabs.com/clown/hooks) | Event-driven automation |
| [Proxy](https://docs.neullabs.com/clown/proxy) | Request routing with ultrallm |
| [Usage](https://docs.neullabs.com/clown/usage) | Token tracking and analytics |
| [Scripting](https://docs.neullabs.com/clown/scripting) | Rhai configuration scripting |

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

Open an issue for design discussions before implementing major features. See the [contribution guide](https://docs.neullabs.com/clown/contributing) for details.

## License

MIT - see [LICENSE](LICENSE)
