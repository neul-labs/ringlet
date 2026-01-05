# GitHub-based registry

ccswitch distributes agent manifests, profile templates, and curated model catalogs through a GitHub-hosted registry (e.g., `github.com/ccswitch/registry`). The registry keeps official integrations versioned, reviewable, and easy to mirror, while allowing enterprises to point at their own forks when they need custom agents.

## Goals

- **Single source of truth** – publish agent CLI manifests, profile templates, and model catalogs in one repository so both the CLI and `ccswitchd` can pull consistent metadata.
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
- `profiles/<id>/<template>.toml` – parameterized profile definitions (env prompts, default arguments, HOME rewrites). Users can apply them via `ccswitch profiles create <agent> --template <name>`.
- `models/catalog.json` – describes available AI models (provider, identifier, requirements, default prompt settings) so prompts can be auto-completed.

## Sync workflow

1. The CLI ships with a baked-in commit hash for the registry.
2. Running `ccswitch registry sync` (or any command that requires fresh data) fetches `registry.json` from GitHub, verifies its checksum, and downloads referenced manifests/templates not already cached.
3. Files are cached under `~/.config/ccswitch/registry/<commit>/...` with metadata stored in `registry.lock`.
4. Users can pin to a specific tag or commit: `ccswitch registry pin v0.3.0`.
5. Offline machines rely on the cached copy until a sync succeeds; the CLI surfaces when the cache is stale.
6. `ccswitch export` includes the pinned commit and optional cached manifests so `ccswitch import` can reconstruct the same registry state on another machine.

## Profile templates

Profile templates define common setups for a given agent and model (e.g., `claude/minimax`, `claude/glm`). Each template file stores:

- Required prompts (model name, API key var, base URL).
- Default environment variables and CLI args.
- Optional hooks or setup tasks.

The CLI exposes `ccswitch profiles create claude client-a --template minimax` to load those defaults before prompting users. Templates can also specify which optional `env setup` tasks are recommended after creation.

## Model catalog

`models/catalog.json` aggregates metadata across providers so profile prompts can offer autocomplete and validation. Example fields:

```json
{
  "id": "MiniMax-M2.1",
  "provider": "minimax",
  "base_urls": {
    "default": "https://api.minimax.io/anthropic",
    "cn": "https://api.minimaxi.com/anthropic"
  },
  "capabilities": ["code", "image"],
  "max_output_tokens": 64000,
  "notes": "Use with Claude-compatible CLIs"
}
```

Manifests can reference catalog entries to ensure prompts stay up to date even if providers rename models.

## Security and verification

- Every registry release is tagged and optionally signed (e.g., GitHub release + Sigstore attestations).
- The CLI verifies checksums before accepting downloaded manifests/templates.
- Enterprises can mirror the repository internally and set `CCSWITCH_REGISTRY_URL` to their GitHub Enterprise or artifact server.

## Contribution workflow

1. Fork the registry repository on GitHub.
2. Add or update the relevant manifest/template/model files.
3. (Future) Run `cargo xtask registry-validate` once the validation utility lands; for now, use `cargo fmt`/`cargo clippy` and JSON/TOML linters to verify schema, and rely on `ccswitch registry lint` (planned) before publishing.
4. Open a pull request; once merged, a CI workflow bumps `registry.json` with new checksums and publishes a release.

## Future enhancements

- Delta syncs so clients only download changed entries.
- Issue tracker templates for requesting new models or profile templates.
- Compatibility metadata (e.g., minimum CLI version per manifest).
- Optional analytics (opt-in) to understand which templates are most popular.
