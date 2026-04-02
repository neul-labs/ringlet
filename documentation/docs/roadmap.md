# Roadmap

Ringlet's development roadmap — where we are now and where we're headed.

---

## Current (Stable)

These features are available today and considered production-ready:

- **Profile management** — create, run, inspect, delete isolated profiles
- **Multi-agent support** — Claude Code, Codex CLI, Grok CLI, Droid CLI, OpenCode
- **Provider switching** — Anthropic, MiniMax, OpenAI, OpenRouter, custom providers
- **Usage tracking** — token and cost analytics with import/export
- **Proxy routing** — rule-based request routing with ultrallm
- **Event hooks** — shell commands and webhooks on agent events
- **Remote terminal** — sandboxed browser-based agent sessions
- **Web dashboard** — visual management UI
- **Desktop app** — native Tauri wrapper
- **Registry system** — GitHub-hosted agent/provider metadata with offline support
- **Rhai scripting** — extensible configuration generation

---

## Planned

Features currently in development or design:

- **Plugin SDK** — extend Ringlet with custom plugins for new agent types, providers, and workflows without modifying core code
- **Cross-device sync** — synchronize profiles and settings across machines
- **CLI attach** — attach to remote terminal sessions directly from the CLI (currently web UI only)
- **Richer scripting API** — more built-in functions and context available to Rhai scripts

---

## Team

Features for small teams and organizations:

- **Shared profiles** — define profiles centrally and distribute to team members
- **Centralized provider policy** — set allowed providers, models, and spending limits from a single config
- **Role-based access (RBAC)** — control who can create profiles, run agents, or modify settings
- **Team usage dashboards** — aggregate usage and cost reporting across team members

---

## Enterprise

Features for larger organizations with compliance requirements:

- **SSO integration** — authenticate with your identity provider (SAML, OIDC)
- **Audit trails** — immutable logs of all profile operations, agent sessions, and configuration changes
- **Compliance reporting** — exportable reports for security and compliance reviews
- **Private registry hosting** — fully self-hosted registry with access controls
- **Air-gapped deployment** — operate without internet access using bundled manifests

---

## Influence the Roadmap

Have a feature request or use case that isn't listed here? [Open an issue](https://github.com/neul-labs/ringlet/issues) and let us know what you need.
