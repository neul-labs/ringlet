# Profile management

Profiles are the heart of ccswitch. Each one binds an agent installation to a specific model, set of credentials, and optional launch arguments. The CLI surfaces how many profiles exist per agent so you can spot missing environments before starting work.

## Lifecycle

1. **Create** – `ccswitch profiles create <agent-id> <alias>` prompts for the model/provider you plan to use, required environment variables (API keys, base URLs, etc.), and any agent-specific options (e.g., HOME path for Claude). Prompts always fire per profile so secrets are never assumed or silently re-used.
2. **Inspect** – `ccswitch profiles inspect <alias>` prints the stored configuration and redacts secrets by default.
3. **List** – `ccswitch profiles list --agent <agent-id>` summarizes aliases per agent and feeds the aggregate counts shown by `ccswitch agents list`.
4. **Run** – `ccswitch profiles run <alias> -- <agent args>` launches the selected agent with the stored configuration, then streams stdout/stderr directly to the caller.
5. **Switch (shell helper)** – `eval "$(ccswitch profiles env <alias>)"` exports environment variables into the current shell when you want to run the agent manually.
6. **Delete** – `ccswitch profiles delete <alias>` removes the JSON file and executes any teardown hooks defined in the manifest.
7. **Optional env setup** – `ccswitch env setup <alias> <task>` runs manual environment adjustments (e.g., remapping CLI shims) defined by the manifest.

## Schema reference

| Field | Description |
| --- | --- |
| `alias` | Unique key used when referencing the profile. |
| `agent_id` | Links to a manifest (`claude`, `codex`, etc.). |
| `model` | Default model (e.g., `MiniMax-M2.1`, `claude-3`, `gpt-4o-mini`). |
| `env` | Key/value map of environment variables injected before launch. Secrets live in the OS keychain when available. |
| `args` | Default CLI arguments appended when running the agent. |
| `working_dir` | Optional override for the process working directory. |
| `metadata` | Arbitrary manifest-specific fields (e.g., JSON path, profile home, last-used timestamps). |

Profiles are serialized as JSON for readability, but the manager exposes a typed API so other persistence backends (SQLite, remote sync) can be added later.

### Templates

Profile templates stored in the GitHub registry (see `docs/registry.md`) provide opinionated defaults for each agent/model combination. Use `ccswitch profiles create <agent> <alias> --template <name>` to pre-populate prompts with registry data (env vars, args, setup task recommendations) before entering secrets.

## Example

```text
$ ccswitch profiles create claude client-a --model MiniMax-M2.1
✔ Agent detected at /usr/local/bin/claude
✔ Created profile home ~/.claude-profiles/client-a
✔ Stored env vars (ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN)

$ ccswitch aliases install client-a
✔ Installed shim ~/bin/claude-client-a

$ claude-client-a /settings strict.json

$ ccswitch agents list
Agent      Version   Profiles   Default Model
claude     0.5.4     4          MiniMax-M2.1
codex      0.11.0    1          MiniMax-M2.1
```

## Hooks

Agent manifests may define hooks that run during profile events. Common cases include:

- `create`: initialize directories, copy template config files, or generate wrapper scripts.
- `delete`: remove temporary homes or revoke credentials.
- `pre-run`: validate connectivity or refresh tokens.

Hooks are executed with a short timeout and receive environment variables describing the alias, agent path, and model.

## CLI aliases

- `ccswitch aliases install <alias>` installs a per-profile shim (e.g., `claude-client-a`) inside the user’s preferred `bin` directory so the agent can be launched directly without prepending `ccswitch profiles run`.
- Each shim rewrites `HOME` (when required), injects the stored environment variables, and forwards all arguments to the underlying agent—so every alias keeps its own safe home/profile automatically.
- `ccswitch aliases uninstall <alias>` removes the shim without touching the stored profile, letting you recreate or rename aliases freely.

## Manual environment setup commands

Some teams need extra shell manipulation (remapping CLI tool paths, creating symlinks, patching config files) beyond what ccswitch performs automatically. Agent manifests may declare named setup tasks, but they never run implicitly.

- `ccswitch env setup <alias> <task>` executes the requested task with the profile’s environment (e.g., `ccswitch env setup claude-client-a cli-remap`).
- Tasks can run scripts, copy files, or rewrite symlinks. They must be idempotent and will be audited like other hooks.
- Because these scripts may be destructive, users must call them manually each time they are required; the CLI will never execute them automatically during profile creation or alias installation.

## Background service interplay

When `ccswitchd` is active (the CLI bootstraps it automatically and it shuts down after an idle timeout unless pinned), the daemon takes ownership of profile persistence and publishes change events:

- CLI commands proxy through the daemon, avoiding concurrent writes.
- UI clients subscribe to `/profiles/stream` (SSE/WebSocket) to update automatically when counts change.
- Portable export/import endpoints (`GET /profiles/<alias>` / `POST /profiles`) allow cross-device sync workflows.

## Best practices

- Keep API keys in environment variables or OS keychains rather than committing them to profile files for shared machines.
- Use descriptive aliases (`<agent>-<project>-<model>`) so table summaries and exports remain readable.
- Leverage the `--json` output flag when integrating with scripts or CI systems that orchestrate multiple profiles or backup/import cycles.
