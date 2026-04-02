# Security

Ringlet is designed with security as a default. This page covers how credentials, profiles, remote sessions, and daemon communication are protected.

---

## Credential Management

API keys are stored in your operating system's keychain — never in plain-text config files.

| Platform | Backend |
|----------|---------|
| macOS | Keychain |
| Linux | GNOME Keyring / Secret Service |
| Windows | Windows Credential Manager |

When you create a profile, Ringlet prompts for the API key and writes it directly to the keychain. The profile JSON on disk contains only non-secret metadata.

```bash
# Credentials are redacted in inspect output
ringlet profiles inspect my-project
# API Key: ****...****
```

!!! tip "Re-entering credentials"
    To rotate a key, delete the profile and recreate it. The old key is removed from the keychain automatically.

---

## Profile Isolation

Each profile runs inside its own HOME directory:

```
~/.claude-profiles/my-project/
├── .claude/
│   ├── settings.json    # Profile-specific settings
│   └── history/         # Conversation history
└── ...
```

When a profile launches, Ringlet sets `HOME` to the profile directory so the agent reads and writes configuration there — not in your real home.

### What is isolated

| Isolated | Not isolated |
|----------|--------------|
| Agent config files | System binaries |
| API credentials | Shell configuration |
| Conversation history | Network access |
| Agent settings | File system (working dir) |

This prevents one profile's credentials, history, or settings from leaking into another.

---

## Sandbox Architecture

Remote terminal sessions are **sandboxed by default**. The sandbox restricts what the agent process can read and write on the host.

```
┌───────────────────────────────────────────────────┐
│                   Host System                      │
│  ┌─────────────────────────────────────────────┐  │
│  │        Sandbox (bwrap / sandbox-exec)        │  │
│  │  ┌───────────────────────────────────────┐  │  │
│  │  │        Agent Process (claude)          │  │  │
│  │  │                                        │  │  │
│  │  │  Read-only:  /usr, /bin, /lib, /etc   │  │  │
│  │  │  Read-write: ~/, working_dir, /tmp    │  │  │
│  │  │  Network:    allowed (API access)      │  │  │
│  │  └───────────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

### Platform support

| Platform | Tool | Status |
|----------|------|--------|
| Linux | [bwrap (bubblewrap)](https://github.com/containers/bubblewrap) | Supported |
| macOS | sandbox-exec | Supported |
| Windows | — | Not yet supported |

### Default rules

- **Read-only root filesystem** — system binaries and libraries are protected
- **Read-write home directory** — agent configs, caches, and settings work normally
- **Read-write working directory** — the agent can modify project files
- **Read-write /tmp** — temporary files work as expected
- **Network allowed** — API calls to providers work normally
- **Process isolation** — separate PID/IPC namespaces (Linux)

### Disabling the sandbox

If you need full system access (e.g. installing packages):

```bash
ringlet profiles run my-project --remote --no-sandbox
```

### Custom sandbox rules (Linux)

Pass additional bwrap flags for tighter restrictions:

```bash
# Disable network access
ringlet profiles run my-project --remote --bwrap-flags="--unshare-net"

# Add a read-only bind mount
ringlet profiles run my-project --remote --bwrap-flags="--ro-bind /data /data"
```

!!! warning "Security implications"
    Disabling the sandbox gives the agent full access to your user account. Only disable it for trusted profiles.

---

## Daemon Authentication

The daemon HTTP API is protected by a bearer token stored at:

```
~/.config/ringlet/http_token
```

All API and WebSocket requests must include this token:

```
Authorization: Bearer <token>
```

WebSocket connections use the `Sec-WebSocket-Protocol` header:

```
Sec-WebSocket-Protocol: bearer, <token>
```

The token is generated automatically on first daemon start with a cryptographically secure random value.

---

## Network Security

By default the daemon binds to **`127.0.0.1:8765`** — only accessible from your local machine.

- No ports are exposed to the network unless you explicitly configure it
- The web UI and API are local-only
- Remote terminal WebSocket connections are also local-only

!!! note "Remote access"
    If you want to access the dashboard from another device, use SSH port forwarding rather than binding to `0.0.0.0`.

---

## Best Practices for Teams

1. **Use separate profiles per project** — prevents credential sharing across projects
2. **Rotate API keys regularly** — delete and recreate profiles to update stored keys
3. **Keep the sandbox enabled** — only disable for trusted, well-understood workloads
4. **Use SSH tunnels for remote access** — never expose the daemon port directly
5. **Pin registry versions in CI** — prevent unexpected manifest changes from affecting builds
6. **Review hooks before running** — hooks execute arbitrary shell commands and should be audited
7. **Use private registries** — enterprises should host their own registry to control available agents and providers
