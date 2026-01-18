# Agent manifests and integrations

clown treats every CLI coding agent as a declarative manifest. The manifest states how to detect an agent, how to create a profile, which environment variables unlock model access, and how to launch the final process. Manifests ship inside the CLI *and* live in the GitHub registry described in `docs/registry.md`, so new versions can be distributed without cutting a new binary. Examples below use MiniMax credentials because they are a common request, but the same manifest structure works for Anthropic, OpenAI-compatible, or internal enterprise models—just swap the environment variables and model identifiers accordingly.

## Manifest anatomy

| Field | Purpose |
| --- | --- |
| `id` | Stable identifier used by profiles (e.g., `claude`, `grok`). |
| `name` | Human-friendly label displayed in tables. |
| `binary` | Executable name or absolute path used when spawning the agent. |
| `version_flag` | Optional flag used to detect the installed version. |
| `detect.commands` | Shell-safe commands run during discovery; success marks the agent as installed. |
| `detect.files` | Optional list of paths whose existence also signals installation. |
| `profile.strategy` | How to isolate profiles. All agents use `home-wrapper` for full isolation—each profile gets its own HOME directory with separate config files and credentials. |
| `profile.source_home` | Template path for `home-wrapper` strategy (e.g., `~/.claude-profiles/{alias}`). |
| `profile.script` | Rhai script that generates configuration files. See `docs/scripting.md`. |
| `profile.required_env` | Environment variables that must exist in every profile prior to launch. Each entry becomes a prompt during `clown profiles create`. |
| `profile.optional_env` | Optional environment variables that can be set but are not required. |
| `models.default` | Default model identifier for new profiles. |
| `models.supported` | List of allowed model identifiers for the agent. |
| `hooks.create` | Commands run when a profile is created. |
| `hooks.delete` | Commands run when a profile is deleted. |
| `hooks.pre_run` | Commands run before launching the agent. |
| `hooks.post_run` | Commands run after the agent exits. |
| `setup_tasks` | Optional manual environment tasks users can run via `clown env setup <alias> <task>` (e.g., remapping CLI shims). |
| `supports_hooks` | Whether the agent supports profile-level event hooks. See [Hooks](hooks.md). |

Note: Agent manifests define **what tool** to run and **how to detect/isolate it**. They do not define API endpoints or credentials—those come from **providers** (see `docs/providers.md`).

## Agents vs. Providers

clown separates two concepts:

- **Agent**: The CLI tool itself (Claude Code, Codex, Droid, etc.). Defines detection, isolation strategy, and hooks.
- **Provider**: The API backend (Anthropic, MiniMax, OpenRouter, etc.). Defines endpoints, authentication, and available models.

When creating a profile, you bind an **agent** to a **provider**:

```bash
# Same agent (claude), different providers
clown profiles create claude work-anthropic --provider anthropic
clown profiles create claude work-minimax --provider minimax
clown profiles create claude home-minimax --provider minimax

# Different agents, same provider
clown profiles create codex codex-minimax --provider minimax
clown profiles create grok grok-minimax --provider minimax
```

This separation means you can:
- Run Claude Code against MiniMax today and switch to Anthropic tomorrow
- Use the same MiniMax credentials across multiple agents
- Add new providers without modifying agent manifests

### Agent-Provider Compatibility

Not all agents work with all provider types. Here's the compatibility matrix:

| Agent | anthropic | anthropic-compatible | openai | openai-compatible |
|-------|-----------|---------------------|--------|-------------------|
| Claude Code | Yes | Yes (MiniMax) | No | No |
| Droid CLI | Yes | Yes (MiniMax) | No | No |
| OpenCode | Yes | Yes (MiniMax) | No | No |
| Codex CLI | No | No | Yes | Yes (MiniMax*) |
| Grok CLI | No | No | Yes | Yes (MiniMax*) |

*MiniMax provides both Anthropic-compatible and OpenAI-compatible endpoints.

### Example manifest (TOML)

```toml
id = "claude"
name = "Claude Code"
binary = "claude"
version_flag = "--version"

[detect]
commands = ["claude --version"]
files = ["~/.claude/settings.json"]

[profile]
strategy = "home-wrapper"
source_home = "~/.claude-profiles/{alias}"
script = "claude.rhai"  # Rhai script generates settings.json, hooks, MCP config

[models]
default = "claude-sonnet-4"
supported = ["claude-sonnet-4", "claude-opus-4", "MiniMax-M2.1"]

[hooks]
create = []
delete = []
pre_run = []
post_run = []
```

The `script` field points to a Rhai script that dynamically generates configuration files based on the selected provider and user preferences. This enables support for hooks, MCP servers, and other agent-specific features without modifying the manifest.

## Built-in agent notes

The following agents ship with curated manifests. Each section describes the underlying CLI, how to install it, and which configuration values your profiles need. Feel free to adapt the commands to your platform.

### Claude Code

- **Install**: Follow the upstream [Claude Code setup guide](https://docs.claude.com/en/docs/claude-code/setup). macOS users typically install via Homebrew (`brew install claude-code`).
- **Profile isolation**: Claude stores sessions beneath `HOME`, so clown uses the `home-wrapper` strategy—each profile points `HOME` to `~/.claude-profiles/<alias>` before launch. Consider adding shims such as `claude-profile` if you run the CLI manually.
- **Configure MiniMax**:
  1. Clear Anthropics defaults to avoid overriding MiniMax values:
     ```bash
     unset ANTHROPIC_AUTH_TOKEN
     unset ANTHROPIC_BASE_URL
     ```
  2. Edit `~/.claude/settings.json` (or your profile-specific copy) to add:
     ```json
     {
       "env": {
         "ANTHROPIC_BASE_URL": "https://api.minimax.io/anthropic",
         "ANTHROPIC_AUTH_TOKEN": "<MINIMAX_API_KEY>",
         "API_TIMEOUT_MS": "3000000",
         "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
         "ANTHROPIC_MODEL": "MiniMax-M2.1",
         "ANTHROPIC_SMALL_FAST_MODEL": "MiniMax-M2.1",
         "ANTHROPIC_DEFAULT_SONNET_MODEL": "MiniMax-M2.1",
         "ANTHROPIC_DEFAULT_OPUS_MODEL": "MiniMax-M2.1",
         "ANTHROPIC_DEFAULT_HAIKU_MODEL": "MiniMax-M2.1"
       }
     }
     ```
- **Using with clown**: Profiles capture the environment block above plus any optional flags (e.g., `--settings`). When `clown profiles run claude-work`, the HOME rewriting ensures each alias keeps independent auth sessions.
- **Hooks**: Claude Code exposes lifecycle hooks (see https://code.claude.com/docs/en/hooks-guide). Declare them in the manifest so clown can run project-specific `pre_run`, `post_run`, or tool-approval hooks per profile.
- **Profile Hooks**: Claude Code supports profile-level event hooks (`supports_hooks: true`). Use `clown hooks add|list|remove` to configure PreToolUse, PostToolUse, Notification, and Stop hooks. See [Hooks](hooks.md).

### Grok CLI

- **Install**:
  ```bash
  npm install -g @vibe-kit/grok-cli
  ```
- **Configure MiniMax**:
  1. Clear OpenAI-specific variables to prevent conflicts:
     ```bash
     unset OPENAI_API_KEY
     unset OPENAI_BASE_URL
     ```
  2. Export MiniMax credentials:
     ```bash
     export GROK_BASE_URL="https://api.minimax.io/v1"   # use https://api.minimaxi.com/v1 in China
     export GROK_API_KEY="<MINIMAX_API_KEY>"
     ```
  3. Launch the CLI with the MiniMax model:
     ```bash
     grok --model MiniMax-M2.1
     ```
- **Using with clown**: The Grok manifest uses the `home-wrapper` strategy. Each profile gets its own isolated home at `~/.grok-profiles/<alias>` where Grok stores its configuration. When a profile runs, clown sets `HOME` to the profile directory, injects `GROK_BASE_URL`, `GROK_API_KEY`, and optional default arguments such as `--model`. This ensures each profile (e.g., `grok-home-minimax`, `grok-work-openai`) has fully isolated settings and credentials.

### Codex CLI

- **Install**:
  ```bash
  npm i -g @openai/codex
  ```
- **Configure MiniMax**:
  1. Clear OpenAI defaults:
     ```bash
     unset OPENAI_API_KEY
     unset OPENAI_BASE_URL
     ```
  2. Edit `~/.codex/config.toml` and append:
     ```toml
     [model_providers.minimax]
     name = "MiniMax Chat Completions API"
     base_url = "https://api.minimax.io/v1"  # https://api.minimaxi.com/v1 if hosted in China
     env_key = "MINIMAX_API_KEY"
     wire_api = "chat"
     requires_openai_auth = false
     request_max_retries = 4
     stream_max_retries = 10
     stream_idle_timeout_ms = 300000

     [profiles.m21]
     model = "codex-MiniMax-M2.1"
     model_provider = "minimax"
     ```
  3. Export your key for the session:
     ```bash
     export MINIMAX_API_KEY="<MINIMAX_API_KEY>"
     ```
  4. Start Codex CLI via `codex --profile m21`.
- **Using with clown**: The Codex manifest uses the `home-wrapper` strategy. Each profile gets its own isolated home at `~/.codex-profiles/<alias>` containing its own `config.toml`. When a profile runs, clown sets `HOME` to the profile directory and injects `MINIMAX_API_KEY` (or the relevant provider's key). The profile's `config.toml` is pre-configured with the selected provider during `clown profiles create`. This ensures each profile (e.g., `codex-home-minimax`, `codex-work-openai`) has fully isolated configuration and credentials.

### Droid CLI

- **Install**:
  ```bash
  # macOS / Linux
  curl -fsSL https://app.factory.ai/cli | sh

  # Windows (PowerShell)
  irm https://app.factory.ai/cli/windows | iex
  ```
- **Configure MiniMax**:
  1. Clear Anthropic defaults on your shell:
     ```bash
     unset ANTHROPIC_AUTH_TOKEN
     unset ANTHROPIC_BASE_URL
     ```
  2. Edit `~/.factory/config.json` so it looks like:
     ```json
     {
       "custom_models": [
         {
           "model_display_name": "MiniMax-M2.1",
           "model": "MiniMax-M2.1",
           "base_url": "https://api.minimax.io/anthropic",
           "api_key": "<MINIMAX_API_KEY>",
           "provider": "anthropic",
           "max_tokens": 64000
         }
       ]
     }
     ```
- **Using with clown**: The Droid manifest uses the `home-wrapper` strategy. Each profile gets its own isolated home at `~/.droid-profiles/<alias>` containing its own `config.json`. When a profile runs, clown sets `HOME` to the profile directory. The profile's `config.json` is pre-configured with the selected provider's `custom_models` block during `clown profiles create`. This ensures each profile (e.g., `droid-home-minimax`, `droid-work-anthropic`) has fully isolated configuration and credentials.
- **Hooks**: Droid supports declarative hooks (https://docs.factory.ai/cli/configuration/hooks-guide). Capture them in the manifest so clown can run pre/post commands or validations whenever a profile is created, deleted, or executed.

### OpenCode

- **Install**:
  ```bash
  curl -fsSL https://opencode.ai/install | bash
  # or
  npm i -g opencode-ai
  ```
- **Configure MiniMax**:
  1. Clear Anthropic defaults:
     ```bash
     unset ANTHROPIC_AUTH_TOKEN
     unset ANTHROPIC_BASE_URL
     ```
  2. Edit `~/.config/opencode/opencode.json`:
     ```json
     {
       "$schema": "https://opencode.ai/config.json",
       "provider": {
         "minimax": {
           "npm": "@ai-sdk/anthropic",
           "options": {
             "baseURL": "https://api.minimax.io/anthropic/v1",
             "apiKey": "<MINIMAX_API_KEY>"
           },
           "models": {
             "MiniMax-M2.1": {
               "name": "MiniMax-M2.1"
             }
           }
         }
       }
     }
     ```
  3. Alternatively, run `opencode auth login`, choose provider **Other**, supply `minimax` as the provider ID, and paste your MiniMax API key when prompted. clown can capture the resulting token path inside the profile metadata.
- **Using with clown**: The OpenCode manifest uses the `home-wrapper` strategy. Each profile gets its own isolated home at `~/.opencode-profiles/<alias>` containing its own `opencode.json`. When a profile runs, clown sets `HOME` to the profile directory. The profile's config is pre-configured with the selected provider during `clown profiles create`. This ensures each profile (e.g., `opencode-home-minimax`, `opencode-work-anthropic`) has fully isolated configuration and credentials.

## Adding a new agent

1. Copy `docs/templates/agent.example.toml` or start from the manifest snippet above.
2. Define detection commands that succeed quickly and do not mutate user state.
3. List every environment variable the agent requires and specify whether clown should prompt for secrets during profile creation.
4. Document how the agent selects models. If it lacks flags, note which config file must be patched and add a hook so clown keeps it in sync.
5. Include version detection commands/flags so `clown agents list` can surface accurate installed versions alongside profile counts.
6. Contribute accompanying documentation mirroring the sections above so other maintainers understand the onboarding steps.

With this pattern, clown can manage any future CLI coding agent without altering core binaries—only new manifests and docs are required.

### Manual environment setup tasks

Some integrations need extra shell changes such as remapping CLI tools or editing files outside the profile home. Define those actions inside a `setup_tasks` block so `clown env setup <alias> <task>` can run them on demand. Because these tasks are opt-in, users stay in control of when remaps or complex scripts execute.
