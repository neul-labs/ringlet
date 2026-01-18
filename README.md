# clown

clown is a cross-platform orchestrator for CLI-based coding agents, built around a central daemon (`clownd`) that manages profiles, tracks usage, and coordinates agent execution. The CLI is a thin client that **automatically starts the daemon** on first use—developers never need to manage the service manually. The daemon collects telemetry (profile invocations, session durations, resource consumption) and exposes it via `clown stats` commands. After an idle timeout the daemon exits unless pinned with `clown daemon --stay-alive`. This daemon-first architecture ensures consistent state, enables future UI integrations, and centralizes observability.

## Why clown exists

- **One switchboard for every agent** – track installations of tools such as Claude Code, Grok CLI, Codex CLI, Droid, OpenCode, and any future Anthropic-compatible agents without memorizing bespoke flags per tool.
- **Provider abstraction** – configure API backends (Anthropic, MiniMax, OpenRouter, etc.) once, then bind any agent to any provider. Run Claude Code against MiniMax today and Anthropic tomorrow without editing config files.
- **Profiles per agent+provider** – quickly create aliases such as `claude-work-minimax` or `grok-home-anthropic` that pin a CLI agent to a specific provider/model/credential set.
- **Executable aliases** – `clown aliases install <profile>` creates real commands like `claude-minimax` or `grok-glm` so every profile launches its agent with isolated homes and env vars.
- **Prompted secrets** – `clown profiles create` always asks for the model name, API keys, and any other manifest-required values per profile, storing them securely so nothing is assumed or silently re-used.
- **Immediate observability** – `clown agents list` shows each installed agent, detected version, last-used timestamp, and how many profiles exist, highlighting gaps before hopping between projects.
- **Composable architecture** – extension manifests describe how to detect, configure, and run an agent, making it straightforward to add entirely new CLI coding agents.
- **Future-ready service** – the library-first Rust design will grow into a daemon that exposes APIs and drives a UI without rebuilding the orchestration logic.
- **GitHub-backed registry** – manifests, profile templates, and model catalogs live in a public repository so new agents/models can ship without rebuilding the CLI while remaining reviewable.

## Project status

The repository currently focuses on design docs and interface conventions so that implementation work can start from a shared understanding. Contributions should follow the documents in `docs/` until the first executable prototype lands.

## CLI preview

```text
$ clown agents list
┌────────────┬──────────────┬────────────┬──────────────┐
│ Agent      │ Version      │ Profiles   │ Default Model│
├────────────┼──────────────┼────────────┼──────────────┤
│ claude     │ 0.5.4        │ 3          │ MiniMax-M2.1 │
│ grok       │ 0.3.2        │ 1          │ grok-3       │
│ codex      │ 0.11.0       │ 1          │ MiniMax-M2.1 │
│ droid      │ 1.2.0        │ 0          │ gemini-2.5   │
│ opencode   │ 1.8.0        │ 2          │ MiniMax-M2.1 │
└────────────┴──────────────┴────────────┴──────────────┘

$ clown profiles create claude work-sonnet \
    --provider minimax \
    --model MiniMax-M2.1

$ clown aliases install work-sonnet
Installed shim ~/bin/claude-work-sonnet -> claude --settings ~/.claude-profiles/work-sonnet/.claude/settings.json

$ claude-work-sonnet /settings strict.json

$ clown env setup work-sonnet cli-remap
Executed manual env setup task \"cli-remap\" for profile work-sonnet

$ clown registry sync
Fetched registry commit f4a12c3 (stable channel)

# Create profile with proxy for intelligent routing
$ clown profiles create claude work-proxy --provider anthropic --proxy

# Add event hooks to a profile
$ clown hooks add work-sonnet PreToolUse "Bash" "echo 'Running: $EVENT' >> /tmp/clown.log"
$ clown hooks list work-sonnet

$ clown profiles list --agent claude
Alias              Provider    Endpoint       Model           Last Used
work-sonnet        minimax     international  MiniMax-M2.1    2024-05-04T11:23:51Z
work-sre           minimax     international  MiniMax-M2.1    2024-05-03T09:18:12Z
```

Commands such as `clown agents inspect <id>` and `clown profiles env <alias>` (for shell integration) will be detailed in `docs/profiles.md` as the implementation evolves. See `docs/providers.md` for how providers (MiniMax, Anthropic, OpenRouter, etc.) are configured separately from agents.

The daemon is started transparently the first time it is needed (for example, when listing agents). When no requests arrive for a configurable idle period the daemon exits, keeping the footprint small. Passing `clown daemon --stay-alive` will pin it in memory for UI integrations. While the preview above references MiniMax, the CLI remains model-provider agnostic; swap in any model fields the agent supports.

## Documentation map

- `docs/architecture.md` – component overview, data flow, and service plans.
- `docs/agents.md` – manifests for each supported CLI coding agent plus steps for onboarding new agents.
- `docs/providers.md` – API backend definitions (Anthropic, MiniMax, OpenRouter) and how to add custom providers.
- `docs/profiles.md` – lifecycle of agent profiles and CLI workflows that manage them.
- `docs/hooks.md` – event-driven hooks for logging, auditing, and integration.
- `docs/proxy.md` – intelligent request routing via ultrallm proxy.
- `docs/scripting.md` – Rhai scripting guide for configuration generation.
- `docs/registry.md` – GitHub registry layout, sync workflow, templates, and model catalog.

## Getting started

1. Install the latest stable Rust toolchain (`rustup install stable`).
2. Clone this repository and fetch dependencies once Cargo files are introduced.
3. During early development, run `cargo run -- agents list` to validate discovery logic as it lands.
4. Keep whichever model/API credentials you rely on (MiniMax, Anthropic, OpenAI-compatible, internal gateways, etc.) in environment variables per the guidance inside `docs/agents.md` for every CLI coding agent—`clown profiles create` will prompt for them explicitly each time.
5. Run any optional environment setup hooks (e.g., CLI remappers) manually via `clown env setup <alias> <task>` whenever the manifest offers such tasks.
6. Pull the latest official manifests/templates/models with `clown registry sync` or point `CLOWN_REGISTRY_URL` at your own GitHub fork.

## Roadmap highlights

- Core `clownd` daemon that owns profile state, usage telemetry, and agent coordination—auto-started by the CLI via `async-nng`.
- Thin CLI that discovers agents on macOS, Linux, and Windows via declarative manifests and forwards all commands to the daemon.
- Telemetry collection: profile invocations, session durations, and resource consumption surfaced via `clown stats`.
- Persisted profile registry stored under `~/.config/clown/` (or platform equivalent) with optional synchronization.
- Optional HTTP/WebSocket endpoints for future UI integrations connecting to the daemon.
- Plugin SDK so teams can publish third-party agent manifests without recompiling the core.

## Contributing

Please open design discussions before implementing major features so that the manifest formats, profile persistence, and service APIs remain consistent. Refer to `docs/` for authoritative requirements, keep changes documented, and accompany new functionality with updates to the relevant guides.
