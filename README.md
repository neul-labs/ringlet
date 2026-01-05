# ccswitch

ccswitch is a cross-platform orchestrator for CLI-based coding agents. It lets developers define named aliases for every agent/model pairing they rely on, display which agents are available on a workstation, and inspect how many profiles each agent owns. The core is written in Rust so that the same binaries can power lightweight command-line use today and a persistent background service with a UI later. The background daemon autostarts when the CLI needs it, then shuts down on idle unless `ccswitch daemon --stay-alive` keeps it resident.

## Why ccswitch exists

- **One switchboard for every agent** – track installations of tools such as Claude Code, Grok CLI, Codex CLI, Droid, OpenCode, and any future Anthropic-compatible agents without memorizing bespoke flags per tool.
- **Profiles per model** – quickly create aliases such as `claude-enterprise` or `grok-m2` that pin a CLI agent to a specific MiniMax, Anthropic, OpenAI, or future vendor model/credential set.
- **Executable aliases** – `ccswitch aliases install <profile>` creates real commands like `claude-minimax` or `grok-glm` so every profile launches its agent with isolated homes and env vars.
- **Prompted secrets** – `ccswitch profiles create` always asks for the model name, API keys, and any other manifest-required values per profile, storing them securely so nothing is assumed or silently re-used.
- **Immediate observability** – `ccswitch agents list` shows each installed agent, detected version, last-used timestamp, and how many profiles exist, highlighting gaps before hopping between projects.
- **Composable architecture** – extension manifests describe how to detect, configure, and run an agent, making it straightforward to add entirely new CLI coding agents.
- **Future-ready service** – the library-first Rust design will grow into a daemon that exposes APIs and drives a UI without rebuilding the orchestration logic.
- **GitHub-backed registry** – manifests, profile templates, and model catalogs live in a public repository so new agents/models can ship without rebuilding the CLI while remaining reviewable.

## Project status

The repository currently focuses on design docs and interface conventions so that implementation work can start from a shared understanding. Contributions should follow the documents in `docs/` until the first executable prototype lands.

## CLI preview

```text
$ ccswitch agents list
┌────────────┬──────────────┬────────────┬──────────────┐
│ Agent      │ Version      │ Profiles   │ Default Model│
├────────────┼──────────────┼────────────┼──────────────┤
│ claude     │ 0.5.4        │ 3          │ MiniMax-M2.1 │
│ codex      │ 0.11.0       │ 1          │ MiniMax-M2.1 │
│ opencode   │ 1.8.0        │ 2          │ MiniMax-M2.1 │
└────────────┴──────────────┴────────────┴──────────────┘

$ ccswitch profiles create claude work-sonnet \
    --model MiniMax-M2.1 \
    --env ANTHROPIC_BASE_URL=https://api.minimax.io/anthropic

$ ccswitch aliases install work-sonnet
Installed shim ~/bin/claude-work-sonnet -> claude --settings ~/.claude-profiles/work-sonnet/.claude/settings.json

$ claude-work-sonnet /settings strict.json

$ ccswitch env setup work-sonnet cli-remap
Executed manual env setup task \"cli-remap\" for profile work-sonnet

$ ccswitch registry sync
Fetched registry commit f4a12c3 (stable channel)
```

Commands such as `ccswitch agents inspect <id>` and `ccswitch profiles switch <alias>` will be detailed in `docs/profiles.md` as the implementation evolves.

The daemon is started transparently the first time it is needed (for example, when listing agents). When no requests arrive for a configurable idle period the daemon exits, keeping the footprint small. Passing `ccswitch daemon --stay-alive` will pin it in memory for UI integrations. While the preview above references MiniMax, the CLI remains model-provider agnostic; swap in any model fields the agent supports.

## Documentation map

- `docs/architecture.md` – component overview, data flow, and service plans.
- `docs/agents.md` – manifests for each supported CLI coding agent plus steps for onboarding new agents.
- `docs/profiles.md` – lifecycle of agent profiles and CLI workflows that manage them.
- `docs/registry.md` – GitHub registry layout, sync workflow, templates, and model catalog.

## Getting started

1. Install the latest stable Rust toolchain (`rustup install stable`).
2. Clone this repository and fetch dependencies once Cargo files are introduced.
3. During early development, run `cargo run -- agents list` to validate discovery logic as it lands.
4. Keep whichever model/API credentials you rely on (MiniMax, Anthropic, OpenAI-compatible, internal gateways, etc.) in environment variables per the guidance inside `docs/agents.md` for every CLI coding agent—`ccswitch profiles create` will prompt for them explicitly each time.
5. Run any optional environment setup hooks (e.g., CLI remappers) manually via `ccswitch env setup <alias> <task>` whenever the manifest offers such tasks.
6. Pull the latest official manifests/templates/models with `ccswitch registry sync` or point `CCSWITCH_REGISTRY_URL` at your own GitHub fork.

## Roadmap highlights

- MVP CLI that can discover known agents on macOS, Linux, and Windows via declarative manifests.
- Persisted profile registry stored under `~/.config/ccswitch/` (or platform equivalent) with optional synchronization.
- Background `ccswitchd` service exposing a local API/WebSocket interface used by both the CLI and a future UI.
- Plugin SDK so teams can publish third-party agent manifests without recompiling the core.

## Contributing

Please open design discussions before implementing major features so that the manifest formats, profile persistence, and service APIs remain consistent. Refer to `docs/` for authoritative requirements, keep changes documented, and accompany new functionality with updates to the relevant guides.
