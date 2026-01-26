# Registry Management

Clown distributes agent manifests, profile templates, and model catalogs through a GitHub-hosted registry. This guide covers how to work with the registry system.

---

## Overview

The registry provides:

- **Agent manifests** - How to detect and configure each agent
- **Provider definitions** - API backends and their endpoints
- **Rhai scripts** - Configuration generation logic
- **Profile templates** - Pre-configured setups
- **Model catalog** - Available models with metadata

### Benefits

- **Single source of truth** - Consistent metadata across all installations
- **Open and reviewable** - Changes vetted through pull requests
- **Extensible** - Support multiple channels and private forks
- **Offline friendly** - Local cache keeps CLI working without network

---

## Repository Layout

```
registry/
├── registry.json               # Versioned index with channels
├── agents/
│   └── claude/
│       ├── manifest.toml       # Agent manifest
│       └── hooks/              # Optional scripts/assets
├── providers/
│   └── minimax.toml            # Provider definition
├── scripts/
│   ├── claude.rhai             # Config generation script
│   ├── codex.rhai
│   └── grok.rhai
├── profiles/
│   └── claude/
│       ├── minimax.toml        # Profile template
│       └── anthropic.toml
├── models/
│   └── catalog.json            # Model metadata
└── templates/
    └── README.md
```

### Key Files

| File | Description |
|------|-------------|
| `registry.json` | Index pointing to entries, includes checksums, maps to channels |
| `agents/<id>/manifest.toml` | Agent detection and configuration |
| `providers/<id>.toml` | API backend definitions |
| `scripts/<agent>.rhai` | Configuration generation scripts |
| `profiles/<agent>/<template>.toml` | Pre-configured profile templates |
| `models/catalog.json` | Model metadata for autocomplete |

---

## CLI Commands

### Sync Registry

Update local cache from GitHub:

```bash
# Normal sync (uses cache if fresh)
ringlet registry sync

# Force refresh
ringlet registry sync --force

# Use cached data only
ringlet registry sync --offline
```

### Inspect Registry

View current registry status:

```bash
ringlet registry inspect
```

**Output:**

```
Registry Status
───────────────
Channel: stable
Commit: f4a12c3
Last Sync: 2026-01-08T10:00:00Z
Cached: Yes
Artifacts: 25

Agents: 5
Providers: 4
Templates: 8
```

### Pin Version

Lock to a specific version:

```bash
# Pin to specific commit
ringlet registry pin f4a12c3

# Pin to tag
ringlet registry pin v1.2.3
```

---

## Sync Workflow

When you run `ringlet registry sync`:

1. **CLI** sends `RegistrySyncRequest` to daemon
2. **Daemon** acquires per-channel lock
3. **Checks** `registry.lock` for cached data
4. **If online and not cached/forced**:
   - Downloads `registry.json`
   - Verifies checksums/signatures
   - Fetches missing artifacts
   - Stages under `commits/<sha>/`
5. **Updates** `registry.lock` with resolved state
6. **Publishes** `RegistryUpdated` event
7. **Returns** summary to CLI

### Caching

The registry is cached under:

```
~/.config/ringlet/registry/
├── current -> commits/f4a12c3    # Symlink to active version
├── registry.lock                  # Lock file with state
├── litellm-pricing.json          # Model pricing data
└── commits/
    └── f4a12c3/
        ├── registry.json
        ├── agents/
        ├── providers/
        ├── scripts/
        └── models/
```

### Offline Mode

When GitHub is unreachable:

```bash
ringlet registry sync --offline
```

Returns the currently pinned commit with `offline=true` indicator.

---

## Profile Templates

Templates define common setups for agent+provider combinations.

### Using Templates

```bash
# Create profile from template
ringlet profiles create claude work --template minimax

# List available templates
ringlet registry templates
```

### Template Format

```toml
# profiles/claude/minimax.toml
name = "Claude + MiniMax"
description = "Claude Code with MiniMax API"
agent = "claude"
provider = "minimax"

[defaults]
model = "MiniMax-M2.1"
endpoint = "international"

[prompts]
api_key = "Enter your MiniMax API key"

[env]
API_TIMEOUT_MS = "3000000"
```

### Template Features

- Required prompts for API keys and settings
- Default environment variables and CLI args
- Optional hooks or setup tasks
- Recommended MCP servers

---

## Model Catalog

The model catalog provides metadata for autocomplete and validation.

### Format

```json
{
  "id": "MiniMax-M2.1",
  "provider": "minimax",
  "base_urls": {
    "international": "https://api.minimax.io/anthropic",
    "china": "https://api.minimaxi.com/anthropic"
  },
  "capabilities": ["code", "image"],
  "max_output_tokens": 64000,
  "notes": "Use with Claude-compatible CLIs"
}
```

### Pricing Data

LiteLLM pricing is downloaded during sync:

```bash
ls ~/.config/ringlet/registry/litellm-pricing.json
```

Contains per-token costs for 200+ models.

---

## Channels

The registry supports multiple channels:

| Channel | Description |
|---------|-------------|
| `stable` | Production-ready, thoroughly tested |
| `beta` | New features, may have bugs |
| `nightly` | Latest changes, unstable |

### Switching Channels

```bash
# Use environment variable
export CLOWN_REGISTRY_CHANNEL=beta
ringlet registry sync
```

---

## Private Registries

Enterprises can host their own registry:

### Setup

1. Fork the registry repository
2. Host on GitHub Enterprise or artifact server
3. Configure Clown to use your registry:

```bash
export CLOWN_REGISTRY_URL=https://github.example.com/org/ringlet-registry
ringlet registry sync
```

### Benefits

- Control which agents/providers are available
- Add internal-only configurations
- Audit changes through your review process
- Air-gapped environments support

---

## Contributing

### Adding a New Agent

1. Fork the registry repository
2. Create `agents/<id>/manifest.toml`:

```toml
id = "my-agent"
name = "My Agent"
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
supported = ["default-model"]
```

3. Create `scripts/my-agent.rhai`
4. Open a pull request

### Adding a Provider

1. Create `providers/<id>.toml`:

```toml
id = "my-provider"
name = "My Provider"
type = "anthropic-compatible"

[endpoints]
default = "https://api.example.com/v1"

[auth]
env_key = "MY_PROVIDER_API_KEY"
prompt = "Enter your API key"

[models]
available = ["model-a", "model-b"]
default = "model-a"
```

2. Open a pull request

### Validation

Before submitting:

```bash
# Validate manifests (planned)
ringlet registry lint

# Test locally
cp -r my-changes ~/.config/ringlet/registry/
ringlet agents list
```

---

## Security

### Verification

- Registry releases are tagged
- Checksums verified before caching
- Optional Sigstore attestations
- Only trusted artifacts served to clients

### Best Practices

- Review changes before syncing in production
- Pin to specific versions in CI/CD
- Use private registries for sensitive configurations
- Audit registry access in enterprise environments

---

## Troubleshooting

### Sync Fails

1. Check network connectivity
2. Try `--offline` to use cache
3. Check if GitHub is accessible
4. Review daemon logs

### Manifest Not Found

1. Verify manifest exists in registry
2. Force sync: `ringlet registry sync --force`
3. Check if using correct channel

### Custom Provider Not Detected

1. Place in `~/.config/ringlet/providers.d/`
2. Verify TOML syntax
3. Run `ringlet registry sync --force`
4. Check `ringlet providers list`
