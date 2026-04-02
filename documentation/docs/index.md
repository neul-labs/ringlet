# Ringlet

**Manage every coding agent from one CLI** — isolated profiles, provider switching, cost control, and security built in.

---

## What is Ringlet?

Ringlet gives you a single interface for all your AI coding agents. Instead of juggling separate configs, credentials, and billing across Claude Code, Codex, Grok, and others, Ringlet lets you:

- **Create isolated profiles** that bind an agent to a provider with its own credentials and history
- **Switch providers** without reconfiguring your agent — move from Anthropic to MiniMax in one command
- **Track usage and costs** across every profile in one place
- **Route requests intelligently** to optimize for cost or performance

---

## Quick Install

=== "Linux/macOS"

    ```bash
    curl -fsSL https://raw.githubusercontent.com/neul-labs/ringlet/main/install.sh | bash
    ```

=== "Cargo"

    ```bash
    cargo install ringlet
    ```

=== "From Source"

    ```bash
    git clone https://github.com/neul-labs/ringlet
    cd ringlet
    cargo build --release
    ```

---

## Features

<div class="grid cards" markdown>

-   :material-account-switch:{ .lg .middle } **Profile Isolation**

    ---

    Every profile gets its own HOME, credentials, config, and history. No cross-contamination between projects.

    [:octicons-arrow-right-24: Profiles guide](guides/profiles.md)

-   :material-api:{ .lg .middle } **Provider Switching**

    ---

    Bind any agent to Anthropic, MiniMax, OpenAI, OpenRouter, or your own gateway. Swap providers without touching agent config.

    [:octicons-arrow-right-24: Providers guide](guides/providers.md)

-   :material-chart-line:{ .lg .middle } **Usage Analytics**

    ---

    Track tokens, costs, and sessions across all profiles. Import Claude data. Export for reporting.

    [:octicons-arrow-right-24: Usage guide](guides/usage.md)

-   :material-router:{ .lg .middle } **Intelligent Routing**

    ---

    Route requests to different providers based on token count, tool usage, or custom rules. Optimize for cost or performance.

    [:octicons-arrow-right-24: Proxy guide](guides/proxy.md)

-   :material-shield-lock:{ .lg .middle } **Security**

    ---

    Keychain credential storage, sandboxed remote sessions, bearer-token auth, localhost-only daemon.

    [:octicons-arrow-right-24: Security](security.md)

-   :material-hook:{ .lg .middle } **Event Hooks**

    ---

    Trigger shell commands or webhooks on tool use, notifications, or agent events. Build audit logs and custom workflows.

    [:octicons-arrow-right-24: Hooks guide](guides/hooks.md)

-   :material-monitor-dashboard:{ .lg .middle } **Web Dashboard**

    ---

    Manage profiles, view usage, and launch terminal sessions from a visual UI.

    [:octicons-arrow-right-24: Terminal guide](guides/terminal.md)

-   :material-map-marker-path:{ .lg .middle } **Roadmap**

    ---

    See what's coming: Plugin SDK, team features, enterprise SSO, and more.

    [:octicons-arrow-right-24: Roadmap](roadmap.md)

</div>

---

## Quick Start

Create your first profile in under a minute:

```bash
# Interactive setup — detects agents, creates first profile
ringlet init

# Or manually
ringlet profiles create claude my-project --provider anthropic
ringlet profiles run my-project

# Check your usage
ringlet usage
```

[:octicons-arrow-right-24: Full quick start guide](getting-started/quickstart.md)

---

## Supported Agents

| Agent | Compatible Providers |
|-------|---------------------|
| **Claude Code** | Anthropic, MiniMax |
| **Codex CLI** | OpenAI, OpenRouter |
| **Grok CLI** | OpenAI-compatible |
| **Droid CLI** | Anthropic, MiniMax |
| **OpenCode** | Anthropic, MiniMax |

---

## Getting Help

- **Documentation**: You're reading it!
- **GitHub Issues**: [Report bugs or request features](https://github.com/neul-labs/ringlet/issues)
- **CLI Help**: Run `ringlet --help` or `ringlet <command> --help`

---

## License

Ringlet is open source software licensed under the MIT License.
