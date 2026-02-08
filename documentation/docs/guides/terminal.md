# Remote Terminal Sessions

Run agent sessions remotely in the daemon and access them from anywhere through the web UI or multiple CLI clients.

---

## Overview

Remote terminal sessions allow you to:

- **Run agents in the background** - Sessions persist even when you disconnect
- **Access from web UI** - Full interactive terminal in your browser
- **Share sessions** - Multiple clients can view and interact with the same session
- **View history** - Reconnecting shows terminal scrollback (up to 1MB)
- **Specify working directory** - Start sessions in any folder

---

## Quick Start

### 1. Start the Daemon

```bash
ringlet daemon start --stay-alive
```

### 2. Create a Remote Session

**From CLI:**

```bash
ringlet profiles run my-project --remote
```

This outputs the session ID and runs the agent in the daemon's PTY.

**From Web UI:**

1. Open `http://127.0.0.1:8765` in your browser
2. Go to the **Terminal** section
3. Click **New Session**
4. Select a profile and click **Create Session**

### 3. Access the Terminal

Open the web UI at `http://127.0.0.1:8765` and select your session from the list. You'll see the full terminal with input/output.

---

## CLI Commands

### Start a Remote Session

```bash
ringlet profiles run <alias> --remote [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--remote` | Run in daemon with PTY (enables web access) |
| `--cols <N>` | Terminal columns (default: 80) |
| `--rows <N>` | Terminal rows (default: 24) |
| `--no-sandbox` | Disable sandboxing (sandbox enabled by default) |
| `--bwrap-flags <FLAGS>` | Custom bwrap flags (Linux only, comma-separated) |

**Examples:**

```bash
# Basic remote session (sandboxed by default)
ringlet profiles run my-project --remote

# With custom terminal size
ringlet profiles run my-project --remote --cols 120 --rows 40

# Without sandbox (full system access)
ringlet profiles run my-project --remote --no-sandbox

# With custom sandbox rules (Linux)
ringlet profiles run my-project --remote --bwrap-flags="--unshare-net"

# Pass additional arguments to the agent
ringlet profiles run my-project --remote -- --dangerously-skip-permissions
```

### List Sessions

```bash
ringlet terminal list
```

Output:

```
SESSION ID                            PROFILE          STATE       CLIENTS
--------------------------------------------------------------------------------
46e15057-abbb-42cd-ad0e-52471a76ef9f  my-project       running     1
843b2463-c3c8-457d-8473-e92cff18c954  work-profile     running     0
```

### Get Session Info

```bash
ringlet terminal info <session-id>
```

Output:

```
Session ID: 46e15057-abbb-42cd-ad0e-52471a76ef9f
Profile: my-project
State: running
PID: 12345
Size: 80x24
Clients: 1
Created: 2026-01-22T00:22:45Z
```

### Terminate a Session

```bash
ringlet terminal kill <session-id>
```

---

## Web UI

The web UI provides a full-featured terminal experience at `http://127.0.0.1:8765/terminal`.

### Features

- **Session sidebar** - List of all active and terminated sessions
- **Full terminal emulation** - Powered by xterm.js
- **Click to focus** - Click anywhere in the terminal to start typing
- **Scrollback history** - Scroll up to see previous output
- **Multi-client support** - Multiple browser tabs can view the same session
- **Session creation** - Create new sessions with profile selection and options

### Creating Sessions in Web UI

1. Click **New Session** button
2. Select a profile from the dropdown
3. Optionally add arguments (e.g., `--dangerously-skip-permissions`)
4. Optionally specify a working directory
5. Click **Create Session**

The terminal connects automatically and shows the agent starting up.

---

## Working Directory

By default, terminal sessions start in the profile's home directory. You can override this when creating a session.

**CLI:**

Working directory is inherited from the profile configuration.

**Web UI:**

Enter a path in the **Working Directory** field when creating a new session.

**API:**

```bash
curl -X POST http://127.0.0.1:8765/api/terminal/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "profile_alias": "my-project",
    "working_dir": "/home/user/code/my-app",
    "cols": 80,
    "rows": 24
  }'
```

---

## Security: Sandboxing

Remote terminal sessions are **sandboxed by default** for security. This isolates the agent process from your system, limiting what files and resources it can access.

### How It Works

```
┌───────────────────────────────────────────────────────┐
│                    Host System                         │
│  ┌─────────────────────────────────────────────────┐  │
│  │        Sandbox (bwrap / sandbox-exec)            │  │
│  │  ┌───────────────────────────────────────────┐  │  │
│  │  │          Agent Process (claude)            │  │  │
│  │  │                                            │  │  │
│  │  │  Read-only:  /usr, /bin, /lib, /etc       │  │  │
│  │  │  Read-write: ~/, working_dir, /tmp        │  │  │
│  │  │  Network:    allowed (API access)          │  │  │
│  │  └───────────────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────┘
```

### Platform Support

| Platform | Sandbox Tool | Status |
|----------|--------------|--------|
| Linux | [bwrap (bubblewrap)](https://github.com/containers/bubblewrap) | ✅ Supported |
| macOS | sandbox-exec | ✅ Supported |
| Windows | - | ❌ Not supported |

!!! note "Installing bwrap on Linux"
    On Ubuntu/Debian: `sudo apt install bubblewrap`
    On Fedora: `sudo dnf install bubblewrap`
    On Arch: `sudo pacman -S bubblewrap`

### Default Sandbox Rules

The default sandbox provides practical security:

- **Read-only root filesystem** - System binaries and libraries are protected
- **Read-write home directory** - Agent configs, caches, and settings work normally
- **Read-write working directory** - The agent can modify your project files
- **Read-write /tmp** - Temporary files work as expected
- **Network access allowed** - API calls to providers work normally
- **Process isolation** - Separate PID/IPC namespaces (Linux)

### Disabling Sandboxing

If your agent needs full system access (e.g., to install packages or modify system files), you can disable the sandbox.

**CLI:**

```bash
ringlet profiles run my-project --remote --no-sandbox
```

**Web UI:**

Check the **"Disable sandbox"** checkbox when creating a new session.

**API:**

```bash
curl -X POST http://127.0.0.1:8765/api/terminal/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "profile_alias": "my-project",
    "no_sandbox": true
  }'
```

!!! warning "Security Implications"
    Disabling the sandbox gives the agent full access to your user account. Only disable it when necessary and for trusted profiles.

### Custom Sandbox Rules (Linux)

You can provide custom bwrap flags for advanced use cases:

```bash
ringlet profiles run my-project --remote --bwrap-flags="--unshare-net,--ro-bind /data /data"
```

Common custom flags:

| Flag | Effect |
|------|--------|
| `--unshare-net` | Disable network access |
| `--ro-bind /path /path` | Mount path as read-only |
| `--bind /path /path` | Mount path as read-write |

---

## Multi-Client Sessions

Multiple clients can connect to the same terminal session simultaneously. This is useful for:

- **Pair programming** - Share your agent session with a colleague
- **Monitoring** - Watch an agent session from multiple devices
- **Debugging** - Open the same session in multiple browser tabs

All connected clients see the same output in real-time. Any client can send input, which is broadcast to the terminal.

---

## Session States

| State | Description |
|-------|-------------|
| `starting` | Session is initializing, agent is launching |
| `running` | Agent is running and interactive |
| `terminated` | Agent has exited (check exit code) |

When a session terminates, all connected clients receive a notification with the exit code.

---

## Scrollback Buffer

Terminal sessions maintain a scrollback buffer (up to 1MB) that stores recent terminal output. When you reconnect to a session or open it in a new browser tab, the scrollback is sent automatically so you can see what happened while disconnected.

---

## API Reference

### REST Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/terminal/sessions` | List all sessions |
| `GET` | `/api/terminal/sessions/{id}` | Get session details |
| `POST` | `/api/terminal/sessions` | Create new session |
| `DELETE` | `/api/terminal/sessions/{id}` | Terminate session |
| `POST` | `/api/terminal/cleanup` | Remove terminated sessions |

### WebSocket

Connect to `/ws/terminal/{session_id}` for real-time terminal I/O.

**Client messages:**

- **Binary** - Raw terminal input (keystrokes)
- **JSON** - Control messages: `{ "type": "resize", "cols": 120, "rows": 40 }`
- **JSON** - Signal messages: `{ "type": "signal", "signal": 2 }` (SIGINT)

**Server messages:**

- **Binary** - Raw terminal output
- **JSON** - State changes: `{ "type": "state_changed", "state": "terminated", "exit_code": 0 }`
- **JSON** - Errors: `{ "type": "error", "message": "Session not found" }`

See the [HTTP API Reference](../reference/api.md#terminal-sessions) for complete details.

---

## Troubleshooting

### Terminal not receiving input

1. Click on the terminal area to focus it
2. Check that the session state is `running`
3. Verify WebSocket connection in browser dev tools

### Session disconnects immediately

Check the daemon logs for errors:

```bash
tail -f ~/.config/ringlet/ringletd.log
```

Common causes:

- Profile not found
- Agent binary not installed
- Invalid working directory

### Scrollback not showing

Scrollback is only sent on initial connection. If you've been connected since the session started, all output is already visible. Try opening in a new browser tab to see the scrollback.

### "Session not found" error

The session may have been terminated. Run `ringlet terminal list` to see active sessions.
