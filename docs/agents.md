# Agent manifests and integrations

ccswitch treats every CLI coding agent as a declarative manifest. The manifest states how to detect an agent, how to create a profile, which environment variables unlock model access, and how to launch the final process. Manifests ship inside the CLI *and* live in the GitHub registry described in `docs/registry.md`, so new versions can be distributed without cutting a new binary. Examples below use MiniMax credentials because they are a common request, but the same manifest structure works for Anthropic, OpenAI-compatible, or internal enterprise models—just swap the environment variables and model identifiers accordingly.

## Manifest anatomy

| Field | Purpose |
| --- | --- |
| `id` | Stable identifier used by profiles (e.g., `claude`, `grok`). |
| `name` | Human-friendly label displayed in tables. |
| `binary` | Executable name or absolute path used when spawning the agent. |
| `version_flag` | Optional flag used to detect the installed version. |
| `detect.commands` | Shell-safe commands run during discovery; success marks the agent as installed. |
| `detect.files` | Optional list of paths whose existence also signals installation. |
| `profile.strategy` | How to isolate profiles (`env-only`, `home-wrapper`, etc.). |
| `profile.required_env` | Environment variables that must exist in every profile prior to launch. Each entry becomes a prompt during `ccswitch profiles create`. |
| `models` | Allowed model identifiers for the agent plus defaults. |
| `hooks` | Optional setup commands ccswitch can run when creating/removing profiles. |
| `setup_tasks` | Optional manual environment tasks users can run via `ccswitch env setup <alias> <task>` (e.g., remapping CLI shims). |

### Example manifest (YAML)

```yaml
id: claude
name: Claude Code
binary: claude
version_flag: --version
profile:
  strategy: home-wrapper
  required_env:
    - ANTHROPIC_BASE_URL
    - ANTHROPIC_AUTH_TOKEN
models:
  default: MiniMax-M2.1
  supported:
    - MiniMax-M2.1
hooks:
  create:
    - claude-profile {{ alias }}
```

## Built-in agent notes

The following agents ship with curated manifests. Each section describes the underlying CLI, how to install it, and which configuration values your profiles need. Feel free to adapt the commands to your platform.

### Claude Code

- **Install**: Follow the upstream [Claude Code setup guide](https://docs.claude.com/en/docs/claude-code/setup). macOS users typically install via Homebrew (`brew install claude-code`).
- **Profile isolation**: Claude stores sessions beneath `HOME`, so ccswitch uses the `home-wrapper` strategy—each profile points `HOME` to `~/.claude-profiles/<alias>` before launch. Consider adding shims such as `claude-profile` if you run the CLI manually.
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
- **Using with ccswitch**: Profiles capture the environment block above plus any optional flags (e.g., `--settings`). When `ccswitch profiles run claude-work`, the HOME rewriting ensures each alias keeps independent auth sessions.
- **Hooks**: Claude Code exposes lifecycle hooks (see https://code.claude.com/docs/en/hooks-guide). Declare them in the manifest so ccswitch can run project-specific `pre_run`, `post_run`, or tool-approval hooks per profile.

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
- **Using with ccswitch**: The Grok manifest uses the `env-only` strategy. When a profile runs, ccswitch injects `GROK_BASE_URL`, `GROK_API_KEY`, and optional default arguments such as `--model` so aliases like `grok-m2-prod` are deterministic.

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
- **Using with ccswitch**: Profiles specify which Codex profile to call plus env overrides. `ccswitch profiles run codex-infra` eventually shells out to `codex --profile <name>` after ensuring the MiniMax key is present. Detection also records `codex --version` output so `ccswitch agents list` can display the installed version.

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
- **Using with ccswitch**: Profiles keep the `custom_models` block synchronized and pass `/model MiniMax-M2.1` to the Droid REPL when launching. Hooks can optionally verify the JSON before writes.
- **Hooks**: Droid supports declarative hooks (https://docs.factory.ai/cli/configuration/hooks-guide). Capture them in the manifest so ccswitch can run pre/post commands or validations whenever a profile is created, deleted, or executed.

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
  3. Alternatively, run `opencode auth login`, choose provider **Other**, supply `minimax` as the provider ID, and paste your MiniMax API key when prompted. ccswitch can capture the resulting token path inside the profile metadata.
- **Using with ccswitch**: The manifest points OpenCode profiles at the JSON file above and ensures `/models` defaults to MiniMax-M2.1. Additional env vars listed in a profile override JSON options per alias.

## Adding a new agent

1. Copy `docs/templates/agent.example.toml` or start from the manifest snippet above.
2. Define detection commands that succeed quickly and do not mutate user state.
3. List every environment variable the agent requires and specify whether ccswitch should prompt for secrets during profile creation.
4. Document how the agent selects models. If it lacks flags, note which config file must be patched and add a hook so ccswitch keeps it in sync.
5. Include version detection commands/flags so `ccswitch agents list` can surface accurate installed versions alongside profile counts.
6. Contribute accompanying documentation mirroring the sections above so other maintainers understand the onboarding steps.

With this pattern, ccswitch can manage any future CLI coding agent without altering core binaries—only new manifests and docs are required.

### Manual environment setup tasks

Some integrations need extra shell changes such as remapping CLI tools or editing files outside the profile home. Define those actions inside a `setup_tasks` block so `ccswitch env setup <alias> <task>` can run them on demand. Because these tasks are opt-in, users stay in control of when remaps or complex scripts execute.
