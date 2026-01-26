# Key Concepts

Understanding how Clown works will help you get the most out of it.

---

## The Big Picture

Clown solves a common problem: managing multiple AI coding agents with different configurations. Instead of manually switching API keys and config files, Clown lets you:

1. **Define profiles** that bind agents to providers
2. **Isolate configurations** so profiles don't interfere
3. **Track usage** across all your profiles
4. **Switch instantly** between different setups

---

## Core Concepts

### Agents

An **agent** is an AI coding assistant CLI tool like:

- **Claude Code** - Anthropic's agentic IDE
- **Codex CLI** - OpenAI's coding assistant
- **Grok CLI** - xAI's coding agent
- **OpenCode** - Open-source alternative

Clown detects installed agents automatically and knows how to configure them.

```bash
# See what agents are available
ringlet agents list
```

### Providers

A **provider** is an API backend that serves the AI models:

| Provider | Type | Example Models |
|----------|------|----------------|
| Anthropic | Native | Claude Sonnet, Claude Opus |
| MiniMax | Anthropic-compatible | Claude via proxy |
| OpenAI | Native | GPT-4o, o1 |
| OpenRouter | OpenAI-compatible | Many models |

Providers are separate from agents because:

- One agent can work with multiple providers
- You might use different providers for different projects
- Costs and limits vary by provider

```bash
# See available providers
ringlet providers list
```

### Profiles

A **profile** binds an agent to a provider with specific configuration:

```
Profile = Agent + Provider + API Key + Settings
```

For example:

- `work-claude` = Claude Code + MiniMax API + work API key
- `personal-claude` = Claude Code + Anthropic + personal API key
- `experiment` = Codex CLI + OpenRouter + test API key

```bash
# Create a profile
ringlet profiles create claude my-profile --provider anthropic
```

---

## How Isolation Works

### The Home Wrapper Strategy

When you run a profile, Clown creates an isolated environment:

```
~/.claude-profiles/my-profile/
├── .claude/
│   ├── settings.json    # Profile-specific settings
│   └── history/         # Conversation history
└── ...
```

The agent runs with `HOME` set to this directory, so:

- Configuration files are isolated
- History is separate per profile
- Settings don't leak between profiles

### What Gets Isolated

| Isolated | Not Isolated |
|----------|--------------|
| Agent config files | System binaries |
| API keys | Shell configuration |
| Conversation history | Other applications |
| Custom settings | Network access |

---

## The Daemon

Clown uses a background daemon (`ringletd`) that:

- **Manages profiles** - Stores and retrieves profile data
- **Tracks usage** - Monitors token consumption and costs
- **Runs proxies** - Handles request routing when enabled
- **Broadcasts events** - Notifies about profile changes

The daemon starts automatically when needed and stops after idle timeout.

```bash
# Check daemon status
ringlet daemon status

# Keep daemon running indefinitely
ringlet daemon start --stay-alive
```

### Why a Daemon?

- **Single source of truth** - No config file conflicts
- **Real-time tracking** - Usage updates as you work
- **Background services** - Proxies run independently
- **Web UI** - Access stats via browser at `http://127.0.0.1:8765`

---

## The Registry

Clown uses a GitHub-based registry to store:

- **Agent manifests** - How to configure each agent
- **Provider definitions** - API endpoints and auth
- **Profile templates** - Pre-configured setups
- **Pricing data** - Token costs for usage tracking

The registry syncs automatically, but you can control it:

```bash
# Force sync
ringlet registry sync --force

# Check status
ringlet registry inspect

# Pin to specific version
ringlet registry pin v1.2.3
```

---

## Data Flow

Here's how a typical profile run works:

```
1. You run: ringlet profiles run my-profile

2. CLI sends request to daemon (via IPC)

3. Daemon:
   - Loads profile configuration
   - Creates isolated home directory
   - Injects environment variables
   - Starts the agent process

4. Agent runs with:
   - Isolated HOME
   - API key in environment
   - Custom configuration

5. Daemon tracks:
   - Session start/end
   - Token usage (from agent files)
   - Runtime duration
```

---

## Common Patterns

### One Profile Per Project

```bash
ringlet profiles create claude project-a --provider anthropic
ringlet profiles create claude project-b --provider minimax
```

Each project gets isolated history and settings.

### Different Providers for Different Costs

```bash
# Cheap provider for experiments
ringlet profiles create claude experiment --provider openrouter

# Premium provider for production
ringlet profiles create claude production --provider anthropic
```

### Quick Switching with Aliases

```bash
# Install aliases for fast access
ringlet aliases install project-a
ringlet aliases install project-b

# Now just run:
project-a   # Starts Claude with project-a profile
project-b   # Starts Claude with project-b profile
```

---

## Summary

| Concept | What It Is | Example |
|---------|------------|---------|
| Agent | AI coding CLI tool | Claude Code, Codex |
| Provider | API backend | Anthropic, MiniMax |
| Profile | Agent + Provider binding | `work-claude` |
| Daemon | Background service | `ringletd` |
| Registry | Configuration source | GitHub repo |

---

## Next Steps

Now that you understand the concepts:

- [:octicons-arrow-right-24: Learn about Profiles](../guides/profiles.md) in depth
- [:octicons-arrow-right-24: Configure Providers](../guides/providers.md) for your needs
- [:octicons-arrow-right-24: Track Usage](../guides/usage.md) and costs
