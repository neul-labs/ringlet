# Architecture Checklist

This document tracks the architecture work needed to make Ringlet structurally coherent and easier to extend.

Status keys:
- [ ] Not started
- [~] In progress
- [x] Done

## M1. Lock The Runtime Model

- [x] Choose the authoritative runtime model
- [x] Document the boundary between daemon-owned planning and client-owned process attachment
- [ ] Remove or refactor codepaths that still violate the chosen model
- [x] Update the primary architecture docs to describe the same execution flow

Notes:
- Decision: the daemon owns state, validation, profile resolution, proxy startup, script execution, config generation, and execution planning.
- Local interactive CLI runs may still spawn the final agent process in the CLI process so the agent inherits the user's TTY.
- No surface may reconstruct profile execution state independently once a prepared execution context exists.

## M2. Define Core Service Boundaries

- [x] Split `ProfileManager` into persistence, secret storage, and profile lifecycle concerns
- [x] Split `ExecutionAdapter` into config rendering, execution planning, and process launching concerns
- [x] Define a single dependency direction between handlers, services, and storage
- [x] Remove cross-layer policy logic from HTTP routes

Notes:
- Profile persistence now lives in `ProfileStore`, keychain access lives in `SecretStore`, and `ProfileManager` is reduced to create/delete lifecycle orchestration.
- Execution flow now separates `ConfigRenderer`, `ExecutionPlanner`, and `ProcessLauncher` under the existing adapter boundary.
- Workspace inspection now lives in `WorkspaceService`, and fs/git HTTP routes no longer perform direct local filesystem or git work.
- Terminal session orchestration now flows through `handlers::terminal`, and proxy restart no longer assembles stop/start logic in the HTTP route.
- HTTP routes now depend on handlers/HTTP policy helpers instead of reaching directly into daemon internals, restoring a route -> handler -> service/storage dependency direction.

## M3. Unify All Run Paths

- [x] Make terminal profile sessions reuse the canonical execution preparation path
- [x] Make HTTP profile runs share the same execution planning path
- [x] Make CLI profile runs and alias/shim execution share the same execution launching abstraction
- [x] Make alias/shim execution reuse the same planning path
- [x] Ensure proxy startup and profile usage marking happen consistently across all run surfaces

Notes:
- Installed shims execute `ringlet profiles run <alias>`, so alias invocation reuses the same CLI prepare/spawn/complete path as direct CLI runs.
- Proxy startup, usage marking, and telemetry now flow through the same prepared execution path for daemon-hosted runs, CLI-attached runs, and profile-backed terminal sessions.

## M4. Normalize Public Contracts

- [x] Audit RPC requests and remove or implement placeholders
- [x] Audit HTTP routes against the supported UI/CLI surface
- [x] Stop returning placeholder "not implemented" responses for public commands
- [x] Establish one source of truth for frontend API types

Notes:
- `EnvSetup` is now implemented through the daemon for manifest-defined setup tasks.
- Route-local HTTP request/response structs have been moved into `ringlet_core::http_api`, so the daemon's public HTTP contract is no longer embedded inside individual route modules.
- `ringlet-core` now owns the canonical TypeScript API contract in `crates/ringlet-core/typescript/api-types.ts`; `cargo xtask api-types` writes the generated UI copy to `ringlet-ui/src/api/generated.ts`, and `cargo xtask api-types --check` verifies it has not drifted.
- The UI-facing `ringlet-ui/src/api/types.ts` now re-exports the generated contract and only keeps UI-local workspace bookmark/recent-workspace types.

## M5. Fix Telemetry And Usage

- [x] Define one canonical session record for all run paths
- [x] Record agent, provider, model, runtime, tokens, and cost on every completed session
- [x] Implement profile/provider/model/date usage aggregations fully
- [x] Make legacy stats derive from the same underlying telemetry data

Notes:
- Session records now include `source` and `model`.
- Session records now include a daemon-owned `session_id` and are written for CLI-attached, daemon-hosted, terminal, and shell session flows.
- Standard profile runs, CLI-attached prepared runs, and profile-backed terminal sessions now write richer telemetry records with native usage deltas when available.
- Usage aggregates now preserve telemetry-backed `by_date`, `by_model`, and provider-aware `by_profile` data instead of dropping them.
- Filtered `usage` and `stats` responses now rebuild from recorded sessions instead of relying on lossy pre-aggregated caches.
- Agent-native usage is now explicitly treated as non-attributable to Ringlet profile aliases unless a stable join key exists.

## M6. Fix Workspace Access Model

- [x] Define the security policy for workspace browsing and git inspection
- [x] Replace the current `HOME`/`/tmp` hard limit with an authenticated local path policy
- [x] Make workspace browsing support common external repo locations
- [x] Document the resulting local-access security model

Notes:
- Current policy: authenticated loopback HTTP routes may access any existing local path after canonicalization.

## M7. Complete Script-Driven Configuration

- [x] Move provider-specific auth env behavior into manifests/scripts
- [x] Ensure terminal, CLI, and daemon-hosted runs consume the same generated files/env
- [x] Centralize hook, MCP, and proxy-related config generation
- [x] Remove remaining hardcoded agent/provider special cases

Notes:
- Provider auth env injection is now owned by agent scripts instead of backend fallback logic.
- Script-generated runtime config now flows through files/env/args only; dead `hooks`/`mcp_servers` script outputs and Claude's legacy compatibility branch have been removed from the execution path.
- Hook, MCP, and proxy behavior now comes from the same script-driven config generation path instead of mixed backend/script assembly.

## M8. Verification And Hardening

- [ ] Add smoke coverage for daemon auto-start
- [ ] Add smoke coverage for profile creation and local run
- [ ] Add smoke coverage for remote terminal runs
- [ ] Add smoke coverage for proxy lifecycle
- [ ] Add smoke coverage for usage import and aggregation
- [x] Add smoke coverage for workspace access outside `HOME`
- [ ] Reduce the current warning volume from `cargo check`

Notes:
- Workspace service tests now cover directory listing, path completion, hidden-file filtering, and non-repository git inspection without relying on route-local filesystem logic.
- `cargo check` warning volume is down to 30 `ringlet` warnings after removing obvious unused imports/re-exports and dead local variables; the remaining warnings are mostly larger dead-code or not-yet-wired feature surfaces.
