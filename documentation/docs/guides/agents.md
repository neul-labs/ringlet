# Working with Agents

Agents are the AI coding assistants that Clown manages. This guide covers how to discover, configure, and add new agents.

---

## What is an Agent?

An agent is a CLI coding tool like:

- **Claude Code** - Anthropic's agentic IDE
- **Codex CLI** - OpenAI's coding assistant
- **Grok CLI** - xAI's coding agent
- **Droid CLI** - Factory.ai's engineering tool
- **OpenCode** - Open-source alternative

Each agent has a manifest that tells Clown:

- How to detect if it's installed
- How to isolate profiles
- What configuration it needs
- Which providers it works with

---

## Discovering Agents

### List Available Agents

```bash
ringlet agents list
```

**Example output:**

```
ID          Name          Installed   Version    Profiles
claude      Claude Code   Yes         1.0.0      3
codex       Codex CLI     Yes         0.5.0      1
grok        Grok CLI      No          -          0
droid       Droid CLI     Yes         2.1.0      0
opencode    OpenCode      No          -          0
```

### Inspect an Agent

```bash
ringlet agents inspect claude
```

**Example output:**

```yaml
ID: claude
Name: Claude Code
Binary: claude
Version: 1.0.0
Binary Path: /usr/local/bin/claude
Profile Strategy: home-wrapper
Profile Home: ~/.claude-profiles/{alias}
Supports Hooks: Yes
Default Model: claude-sonnet-4
Supported Models:
  - claude-sonnet-4
  - claude-opus-4
Compatible Providers:
  - anthropic
  - anthropic-compatible (minimax)
```

---

## Agent-Provider Compatibility

Not all agents work with all providers. Here's the compatibility matrix:

| Agent | anthropic | anthropic-compatible | openai | openai-compatible |
|-------|:---------:|:-------------------:|:------:|:-----------------:|
| Claude Code | ✅ | ✅ | ❌ | ❌ |
| Droid CLI | ✅ | ✅ | ❌ | ❌ |
| OpenCode | ✅ | ✅ | ❌ | ❌ |
| Codex CLI | ❌ | ❌ | ✅ | ✅ |
| Grok CLI | ❌ | ❌ | ✅ | ✅ |

!!! note "Provider Types"
    - **anthropic** - Native Anthropic API
    - **anthropic-compatible** - APIs that mimic Anthropic (MiniMax)
    - **openai** - Native OpenAI API
    - **openai-compatible** - APIs that mimic OpenAI (OpenRouter)

---

## Supported Agents

### Claude Code

Anthropic's official coding assistant.

**Installation:**

```bash
# macOS with Homebrew
brew install claude-code

# Or follow https://docs.claude.com/en/docs/claude-code/setup
```

**Compatible providers:** Anthropic, MiniMax

**Profile isolation:** Full HOME wrapper at `~/.claude-profiles/{alias}`

**Features:**

- Event hooks support (PreToolUse, PostToolUse, etc.)
- MCP server integration
- Conversation history isolation

**Create a profile:**

```bash
# With Anthropic
ringlet profiles create claude my-claude --provider anthropic

# With MiniMax
ringlet profiles create claude my-minimax --provider minimax
```

---

### Codex CLI

OpenAI's coding assistant.

**Installation:**

```bash
npm install -g @openai/codex
```

**Compatible providers:** OpenAI, OpenRouter, MiniMax (OpenAI-compatible endpoint)

**Profile isolation:** Full HOME wrapper at `~/.codex-profiles/{alias}`

**Create a profile:**

```bash
# With OpenAI
ringlet profiles create codex my-codex --provider openai

# With OpenRouter
ringlet profiles create codex my-router --provider openrouter
```

---

### Grok CLI

xAI's coding assistant.

**Installation:**

```bash
npm install -g @vibe-kit/grok-cli
```

**Compatible providers:** OpenAI-compatible providers

**Profile isolation:** Full HOME wrapper at `~/.grok-profiles/{alias}`

**Create a profile:**

```bash
ringlet profiles create grok my-grok --provider openrouter
```

---

### Droid CLI

Factory.ai's AI engineering tool.

**Installation:**

```bash
# macOS / Linux
curl -fsSL https://app.factory.ai/cli | sh

# Windows
irm https://app.factory.ai/cli/windows | iex
```

**Compatible providers:** Anthropic, MiniMax

**Profile isolation:** Full HOME wrapper at `~/.droid-profiles/{alias}`

**Create a profile:**

```bash
ringlet profiles create droid my-droid --provider anthropic
```

---

### OpenCode

Open-source coding agent.

**Installation:**

```bash
curl -fsSL https://opencode.ai/install | bash
# or
npm install -g opencode-ai
```

**Compatible providers:** Anthropic, MiniMax

**Profile isolation:** Full HOME wrapper at `~/.opencode-profiles/{alias}`

**Create a profile:**

```bash
ringlet profiles create opencode my-opencode --provider anthropic
```

---

## How Agent Detection Works

Clown detects agents using commands defined in their manifests:

```toml
[detect]
commands = ["claude --version"]
files = ["~/.claude/settings.json"]
```

Detection runs when:

1. You run `ringlet agents list`
2. You create a profile for an agent
3. The daemon starts

!!! tip "Agent Not Detected?"
    If an agent isn't showing up:

    1. Ensure the binary is in your PATH
    2. Run `ringlet registry sync --force` to refresh manifests
    3. Verify with `which <agent-binary>`

---

## Profile Isolation Strategy

All agents use the **home-wrapper** strategy:

```
Original HOME: /home/user
Profile HOME:  ~/.claude-profiles/my-project

When profile runs:
  HOME=/home/user/.claude-profiles/my-project
  claude <args>
```

This ensures:

- Configuration files are isolated per profile
- Credentials are separate
- History doesn't leak between profiles
- Settings can differ per project

---

## Agent Manifest Structure

Agent manifests define how Clown works with each tool:

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
script = "claude.rhai"

[models]
default = "claude-sonnet-4"
supported = ["claude-sonnet-4", "claude-opus-4"]

[hooks]
create = []
delete = []
pre_run = []
post_run = []

supports_hooks = true
```

| Field | Purpose |
|-------|---------|
| `id` | Unique identifier |
| `binary` | Executable name |
| `detect` | How to find the installed agent |
| `profile.strategy` | Isolation method |
| `profile.script` | Rhai script for config generation |
| `models` | Supported model identifiers |
| `hooks` | Lifecycle hooks |
| `supports_hooks` | Whether event hooks work |

---

## Adding a New Agent

To add support for a new coding agent:

### 1. Create the Manifest

Create `~/.config/ringlet/agents.d/my-agent.toml`:

```toml
id = "my-agent"
name = "My Coding Agent"
binary = "my-agent"
version_flag = "--version"

[detect]
commands = ["my-agent --version"]

[profile]
strategy = "home-wrapper"
source_home = "~/.my-agent-profiles/{alias}"
script = "my-agent.rhai"

[models]
default = "default-model"
supported = ["default-model", "advanced-model"]
```

### 2. Create the Configuration Script

Create `~/.config/ringlet/scripts/my-agent.rhai`:

```rhai
// Generate configuration for my-agent

let config = #{
    api_key: provider.api_key,
    model: profile.model,
    base_url: provider.endpoint_url
};

// Output
#{
    files: #{
        ".my-agent/config.json": json::encode(config)
    },
    env: #{
        "MY_AGENT_API_KEY": provider.api_key
    }
}
```

### 3. Test Detection

```bash
ringlet registry sync --force
ringlet agents list
```

### 4. Create a Profile

```bash
ringlet profiles create my-agent test --provider anthropic
ringlet profiles run test
```

---

## Troubleshooting

### Agent shows as "Not Installed"

1. Verify the binary is in PATH: `which claude`
2. Check if detection command works: `claude --version`
3. Sync registry: `ringlet registry sync --force`

### Wrong version displayed

1. Run `ringlet registry sync --force`
2. Check manifest version_flag matches agent's actual flag

### Profile creation fails

1. Verify agent-provider compatibility
2. Check if Rhai script exists for the agent
3. Look at daemon logs: `ringlet daemon status`
