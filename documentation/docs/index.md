# Clown

**CLI orchestrator for coding agents** - manage multiple AI coding assistants with isolated profiles, seamless provider switching, and unified usage tracking.

---

## What is Clown?

Clown is a command-line tool that helps you manage multiple AI coding agents (like Claude Code, Codex CLI, Grok CLI) with different API providers and configurations. It provides:

- **Profile Isolation** - Each profile gets its own isolated environment, preventing configuration conflicts
- **Provider Abstraction** - Switch between Anthropic, MiniMax, OpenAI, and other providers without reconfiguring your agent
- **Usage Tracking** - Monitor token usage and costs across all your profiles in one place
- **Intelligent Routing** - Route requests to different providers based on cost, latency, or custom rules

---

## Quick Install

=== "Linux/macOS"

    ```bash
    curl -fsSL https://raw.githubusercontent.com/neul-labs/ccswitch/main/install.sh | bash
    ```

=== "From Source"

    ```bash
    cargo install --git https://github.com/neul-labs/ccswitch ringlet
    ```

=== "Local Build"

    ```bash
    git clone https://github.com/neul-labs/ccswitch
    cd ccswitch
    ./install.sh --local
    ```

---

## Features

<div class="grid cards" markdown>

-   :material-account-switch:{ .lg .middle } **Profile Management**

    ---

    Create isolated profiles that bind agents to providers. Switch between configurations instantly.

    [:octicons-arrow-right-24: Learn more](guides/profiles.md)

-   :material-api:{ .lg .middle } **Multi-Provider Support**

    ---

    Use Anthropic, MiniMax, OpenAI, OpenRouter, or add custom providers. Each agent works with compatible APIs.

    [:octicons-arrow-right-24: Providers](guides/providers.md)

-   :material-chart-line:{ .lg .middle } **Usage Tracking**

    ---

    Track tokens, costs, and sessions across all profiles. Import existing Claude data and export for analysis.

    [:octicons-arrow-right-24: Usage guide](guides/usage.md)

-   :material-router:{ .lg .middle } **Intelligent Routing**

    ---

    Route requests based on token count, tool usage, or custom rules. Optimize for cost or performance.

    [:octicons-arrow-right-24: Proxy guide](guides/proxy.md)

-   :material-hook:{ .lg .middle } **Event Hooks**

    ---

    Execute commands or webhooks on tool usage, notifications, or agent events. Build custom workflows.

    [:octicons-arrow-right-24: Hooks guide](guides/hooks.md)

-   :material-script:{ .lg .middle } **Extensible Scripting**

    ---

    Customize agent configuration with Rhai scripts. Add new agents without recompiling.

    [:octicons-arrow-right-24: Scripting guide](guides/scripting.md)

</div>

---

## Quick Start

Create your first profile in under a minute:

```bash
# List available agents
ringlet agents list

# Create a profile binding Claude Code to Anthropic
ringlet profiles create claude my-claude --provider anthropic

# Run your profile
ringlet profiles run my-claude

# Check your usage
ringlet usage
```

[:octicons-arrow-right-24: Full quick start guide](getting-started/quickstart.md)

---

## Supported Agents

| Agent | Description | Provider Compatibility |
|-------|-------------|----------------------|
| **Claude Code** | Anthropic's agentic IDE | Anthropic, MiniMax |
| **Codex CLI** | OpenAI-based coding agent | OpenAI, OpenRouter |
| **Grok CLI** | xAI's coding assistant | OpenAI-compatible |
| **OpenCode** | Open-source coding agent | Multiple providers |

---

## Architecture Overview

Clown uses a daemon-first architecture where a background service manages all state:

```
┌─────────┐     ┌─────────┐     ┌──────────────┐
│  CLI    │────▶│ ringletd  │────▶│  Agent       │
│         │     │ daemon  │     │  (isolated)  │
└─────────┘     └────┬────┘     └──────────────┘
                     │
              ┌──────┴──────┐
              │             │
        ┌─────▼────┐  ┌─────▼────┐
        │ Profiles │  │  Usage   │
        │  Store   │  │ Tracking │
        └──────────┘  └──────────┘
```

[:octicons-arrow-right-24: Architecture details](advanced/architecture.md)

---

## Getting Help

- **Documentation**: You're reading it!
- **GitHub Issues**: [Report bugs or request features](https://github.com/neul-labs/ccswitch/issues)
- **CLI Help**: Run `ringlet --help` or `ringlet <command> --help`

---

## License

Clown is open source software licensed under the MIT License.
