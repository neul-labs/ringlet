# Provider Configuration

Providers define API backends that serve AI models. By separating providers from agents, you can run the same tool against different backends.

---

## What is a Provider?

A provider specifies:

- **API endpoint** - Where to send requests
- **Authentication** - How to authenticate (API key)
- **Models** - What models are available
- **Type** - API compatibility (Anthropic vs OpenAI style)

---

## Built-in Providers

Clown ships with these providers:

| ID | Name | Type | Endpoint |
|----|------|------|----------|
| `anthropic` | Anthropic API | anthropic | `api.anthropic.com` |
| `minimax` | MiniMax | anthropic-compatible | `api.minimax.io` (intl), `api.minimaxi.com` (China) |
| `openai` | OpenAI API | openai | `api.openai.com` |
| `openrouter` | OpenRouter | openai-compatible | `openrouter.ai` |

---

## CLI Commands

### List Providers

```bash
ringlet providers list
```

**Output:**

```
ID          Name           Type                  Default Model
anthropic   Anthropic API  anthropic             claude-sonnet-4
minimax     MiniMax        anthropic-compatible  MiniMax-M2.1
openai      OpenAI API     openai                gpt-4o
openrouter  OpenRouter     openai-compatible     auto
```

### Inspect a Provider

```bash
ringlet providers inspect minimax
```

**Output:**

```yaml
ID: minimax
Name: MiniMax
Type: anthropic-compatible
Endpoints:
  international: https://api.minimax.io/anthropic (default)
  china: https://api.minimaxi.com/anthropic
Auth: MINIMAX_API_KEY
Models: MiniMax-M2.1
```

---

## Using Providers

### Basic Usage

Specify a provider when creating a profile:

```bash
ringlet profiles create claude my-project --provider anthropic
```

### Multi-Region Providers

Some providers offer multiple endpoints for different regions:

```bash
# Use international endpoint (default)
ringlet profiles create claude intl-project --provider minimax

# Use China endpoint
ringlet profiles create claude china-project --provider minimax --endpoint china
```

### Same Credentials, Different Profiles

When creating multiple profiles with the same provider, Clown offers to reuse credentials:

```bash
$ ringlet profiles create claude project-a --provider minimax
Enter MiniMax API key: ****

$ ringlet profiles create claude project-b --provider minimax
? Reuse existing MiniMax credentials? [project-a] Yes
✔ Created profile using existing credentials
```

---

## Provider Compatibility

### Type Matrix

| Type | Description | Example Providers |
|------|-------------|-------------------|
| `anthropic` | Native Anthropic API | Anthropic |
| `anthropic-compatible` | APIs mimicking Anthropic | MiniMax |
| `openai` | Native OpenAI API | OpenAI |
| `openai-compatible` | APIs mimicking OpenAI | OpenRouter |

### Agent Compatibility

| Provider Type | Compatible Agents |
|---------------|-------------------|
| anthropic | Claude Code, Droid, OpenCode |
| anthropic-compatible | Claude Code, Droid, OpenCode |
| openai | Codex CLI, Grok CLI |
| openai-compatible | Codex CLI, Grok CLI |

---

## Adding Custom Providers

### Create a Provider Manifest

Create a TOML file in `~/.config/ringlet/providers.d/`:

```toml
# ~/.config/ringlet/providers.d/my-gateway.toml
id = "my-gateway"
name = "Internal API Gateway"
type = "anthropic-compatible"

[endpoints]
production = "https://api.internal.company.com/llm"
staging = "https://api-staging.internal.company.com/llm"
default = "production"

[auth]
env_key = "INTERNAL_API_KEY"
prompt = "Enter your internal gateway API key"

[models]
available = ["internal-claude-3", "internal-gpt-4"]
default = "internal-claude-3"
```

### Use Your Custom Provider

```bash
# Sync to detect new provider
ringlet registry sync --force

# Create a profile
ringlet profiles create claude internal --provider my-gateway
```

---

## Provider Manifest Reference

| Field | Description |
|-------|-------------|
| `id` | Unique identifier (e.g., `minimax`) |
| `name` | Display name |
| `type` | API compatibility type |
| `endpoints` | Named endpoints with URLs |
| `endpoints.default` | Default endpoint name |
| `auth.env_key` | Environment variable for API key |
| `auth.prompt` | Message shown when prompting for key |
| `models.available` | List of available model IDs |
| `models.default` | Default model for new profiles |

**Example:**

```toml
id = "minimax"
name = "MiniMax"
type = "anthropic-compatible"

[endpoints]
international = "https://api.minimax.io/anthropic"
china = "https://api.minimaxi.com/anthropic"
default = "international"

[auth]
env_key = "MINIMAX_API_KEY"
prompt = "Enter your MiniMax API key"

[models]
available = ["MiniMax-M2.1"]
default = "MiniMax-M2.1"
```

---

## Common Configurations

### Anthropic Direct

```bash
ringlet profiles create claude my-claude --provider anthropic
```

Uses:
- Endpoint: `api.anthropic.com`
- Key: `ANTHROPIC_API_KEY`
- Models: Claude Sonnet, Claude Opus

### MiniMax (International)

```bash
ringlet profiles create claude my-minimax --provider minimax
```

Uses:
- Endpoint: `api.minimax.io/anthropic`
- Key: `MINIMAX_API_KEY`
- Models: MiniMax-M2.1

### MiniMax (China)

```bash
ringlet profiles create claude china-minimax --provider minimax --endpoint china
```

Uses:
- Endpoint: `api.minimaxi.com/anthropic`
- Key: `MINIMAX_API_KEY`
- Models: MiniMax-M2.1

### OpenRouter

```bash
ringlet profiles create codex my-codex --provider openrouter
```

Uses:
- Endpoint: `openrouter.ai/api/v1`
- Key: `OPENROUTER_API_KEY`
- Models: Various (GPT-4, Claude, etc.)

---

## How Providers Work Internally

When you create a profile, Clown:

1. **Reads provider manifest** - Gets endpoint, auth requirements, models
2. **Prompts for API key** - Stores securely in system keychain
3. **Runs Rhai script** - Generates agent-specific configuration
4. **Creates profile** - Saves binding with all settings

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Agent     │     │  Provider   │     │   Rhai      │
│  Manifest   │  +  │  Manifest   │  →  │  Script     │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                                               ▼
                                        ┌─────────────┐
                                        │   Profile   │
                                        │  (JSON +    │
                                        │   Keychain) │
                                        └─────────────┘
```

---

## Troubleshooting

### Provider Not Found

1. Check if manifest exists: `ls ~/.config/ringlet/providers.d/`
2. Sync registry: `ringlet registry sync --force`
3. Verify TOML syntax is valid

### Authentication Errors

1. Verify API key is correct
2. Check if endpoint is reachable
3. Ensure provider type matches agent requirements

### Wrong Endpoint Used

1. Explicitly specify endpoint: `--endpoint china`
2. Check provider's default endpoint setting
3. Inspect profile: `ringlet profiles inspect <alias>`

### Model Not Available

1. Check provider's model list: `ringlet providers inspect <id>`
2. Verify model is supported by both provider and agent
3. Use `--model` flag to override
