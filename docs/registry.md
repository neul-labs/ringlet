# GitHub-based registry

clown distributes agent manifests, profile templates, and curated model catalogs through a GitHub-hosted registry (e.g., `github.com/clown/registry`). The registry keeps official integrations versioned, reviewable, and easy to mirror, while allowing enterprises to point at their own forks when they need custom agents.

## Goals

- **Single source of truth** – publish agent CLI manifests, profile templates, and model catalogs in one repository so both the CLI and `clownd` can pull consistent metadata.
- **Open, reviewable changes** – use pull requests/issues to vet new agents or templates, making it easy to audit what runs on developer machines.
- **Extensible distribution** – support multiple channels (stable, beta, nightly) and allow private forks without code changes.
- **Offline friendly** – cache the registry locally so the CLI can keep working when GitHub is not reachable.

## Repository layout

```text
registry/
├── registry.json               # versioned index with channels/timestamps
├── agents/
│   └── claude/
│       ├── manifest.toml       # canonical agent manifest
│       └── hooks/              # optional reference scripts or assets
├── providers/
│   └── minimax.toml            # provider manifest (endpoints, auth, models)
├── scripts/
│   ├── claude.rhai             # Rhai script for Claude Code config generation
│   ├── codex.rhai              # Rhai script for Codex CLI
│   └── grok.rhai               # Rhai script for Grok CLI
├── profiles/
│   └── claude/
│       ├── minimax.toml        # template for MiniMax profile
│       └── glm.toml            # template for alternative model/provider
├── models/
│   └── catalog.json            # curated model metadata (name, provider, max tokens, regions)
└── templates/
    └── README.md               # guidelines for creating new profile templates
```

Key files:

- `registry.json` – points to each agent/profile/model entry, includes checksums, and maps them to channels (`stable`, `beta`, `nightly`).
- `agents/<id>/manifest.toml` – the same manifest format described in `docs/agents.md`, stored so we can update support without releasing a new binary.
- `providers/<id>.toml` – provider manifests defining API backends (endpoints, auth, models). See `docs/providers.md`.
- `scripts/<agent-id>.rhai` – Rhai scripts that generate agent-specific configuration. See `docs/scripting.md`.
- `profiles/<id>/<template>.toml` – parameterized profile definitions (env prompts, default arguments, HOME rewrites). Users can apply them via `clown profiles create <agent> --template <name>`.
- `models/catalog.json` – describes available AI models (provider, identifier, requirements, default prompt settings) so prompts can be auto-completed.

## Sync workflow

1. The CLI bakes in a fallback commit hash that guarantees every install can bootstrap even before the first sync runs.
2. `clown registry sync` (or any command that needs fresh metadata) serializes a `RegistrySyncRequest` and sends it to `clownd` over the `async-nng` request socket, including channel overrides, explicit refs, and flags such as `--offline` or `--force`.
3. The daemon acquires a per-channel lock, reads `~/.config/clown/registry/registry.lock`, honors overrides like `CLOWN_REGISTRY_URL`/`CLOWN_REGISTRY_CHANNEL`, and skips network work when the cache already satisfies the request (unless `--force` is present).
4. When online, the daemon downloads `registry.json`, verifies checksums/signatures, fetches any referenced manifests/templates/models not yet cached, and stages the artifacts under `~/.config/clown/registry/commits/<sha>/` before updating the `registry/current` symlink.
5. Once the new commit is durable, `registry.lock` is rewritten with the resolved commit/channel/timestamp/etag plus a list of cached artifacts, and a `RegistryUpdated` pub/sub event notifies CLIs or UI watchers that data changed.
6. `clown registry pin <ref>` updates the lock to track a chosen commit/tag without running a sync, while offline requests simply return the currently pinned commit with an explicit `offline=true` indicator so callers know they are using cached data.
7. `clown export` optionally bundles `registry.lock`, the pinned commit, and the cached `registry/commits/<sha>` tree so `clown import` can recreate the same registry state on another machine with zero network access.

## Profile templates

Profile templates define common setups for a given agent and model (e.g., `claude/minimax`, `claude/glm`). Each template file stores:

- Required prompts (model name, API key var, base URL).
- Default environment variables and CLI args.
- Optional hooks or setup tasks.

The CLI exposes `clown profiles create claude work-minimax --template minimax` to load those defaults before prompting users. Templates can also specify which optional `env setup` tasks are recommended after creation.

## Model catalog

`models/catalog.json` aggregates metadata across providers so profile prompts can offer autocomplete and validation. Example fields:

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

Manifests can reference catalog entries to ensure prompts stay up to date even if providers rename models.

## Security and verification

- Every registry release is tagged and optionally signed (e.g., GitHub release + Sigstore attestations).
- `clownd` verifies checksums/signatures before caching manifests/templates so any CLI or UI client reading from the daemon only receives trusted artifacts.
- Enterprises can mirror the repository internally and set `CLOWN_REGISTRY_URL` to their GitHub Enterprise or artifact server.

## Contribution workflow

1. Fork the registry repository on GitHub.
2. Add or update the relevant manifest/template/model files.
3. (Future) Run `cargo xtask registry-validate` once the validation utility lands; for now, use `cargo fmt`/`cargo clippy` and JSON/TOML linters to verify schema, and rely on `clown registry lint` (planned) before publishing.
4. Open a pull request; once merged, a CI workflow bumps `registry.json` with new checksums and publishes a release.

## Future enhancements

- Delta syncs so clients only download changed entries.
- Issue tracker templates for requesting new models or profile templates.
- Compatibility metadata (e.g., minimum CLI version per manifest).
- Optional analytics (opt-in) to understand which templates are most popular.
