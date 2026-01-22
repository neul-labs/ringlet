# HTTP API Reference

The Clown daemon exposes an HTTP API on `http://127.0.0.1:8765` for UI integrations and external tools.

---

## Overview

The HTTP API provides:

- **RESTful endpoints** for profiles, agents, providers, and usage
- **JSON responses** with consistent structure
- **WebSocket support** for real-time events

### Base URL

```
http://127.0.0.1:8765
```

### Response Format

All responses follow this structure:

```json
{
  "success": true,
  "data": { ... }
}
```

Error responses:

```json
{
  "success": false,
  "error": "Error message"
}
```

---

## Profiles

### List Profiles

```http
GET /api/profiles
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `agent` | string | Filter by agent ID |

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "alias": "my-project",
      "agent_id": "claude",
      "provider_id": "anthropic",
      "endpoint_id": "default",
      "model": "claude-sonnet-4",
      "last_used": "2026-01-08T10:30:00Z"
    }
  ]
}
```

### Get Profile

```http
GET /api/profiles/{alias}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "alias": "my-project",
    "agent_id": "claude",
    "provider_id": "anthropic",
    "endpoint_id": "default",
    "model": "claude-sonnet-4",
    "created_at": "2026-01-05T10:00:00Z",
    "last_used": "2026-01-08T10:30:00Z",
    "profile_home": "~/.claude-profiles/my-project"
  }
}
```

### Create Profile

```http
POST /api/profiles
Content-Type: application/json
```

**Request Body:**

```json
{
  "agent_id": "claude",
  "alias": "new-project",
  "provider_id": "anthropic",
  "endpoint_id": "default",
  "model": "claude-sonnet-4",
  "api_key": "sk-..."
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "alias": "new-project",
    "message": "Profile created successfully"
  }
}
```

### Delete Profile

```http
DELETE /api/profiles/{alias}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "message": "Profile deleted"
  }
}
```

---

## Agents

### List Agents

```http
GET /api/agents
```

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "claude",
      "name": "Claude Code",
      "installed": true,
      "version": "1.0.0",
      "binary_path": "/usr/local/bin/claude",
      "profile_count": 3
    },
    {
      "id": "codex",
      "name": "Codex CLI",
      "installed": true,
      "version": "0.5.0",
      "binary_path": "/usr/local/bin/codex",
      "profile_count": 1
    }
  ]
}
```

### Get Agent

```http
GET /api/agents/{id}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "claude",
    "name": "Claude Code",
    "installed": true,
    "version": "1.0.0",
    "binary": "claude",
    "binary_path": "/usr/local/bin/claude",
    "profile_strategy": "home-wrapper",
    "supports_hooks": true,
    "default_model": "claude-sonnet-4",
    "supported_models": ["claude-sonnet-4", "claude-opus-4"]
  }
}
```

---

## Providers

### List Providers

```http
GET /api/providers
```

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "anthropic",
      "name": "Anthropic",
      "type": "anthropic",
      "default_model": "claude-sonnet-4"
    },
    {
      "id": "minimax",
      "name": "MiniMax",
      "type": "anthropic-compatible",
      "default_model": "MiniMax-M2.1"
    }
  ]
}
```

### Get Provider

```http
GET /api/providers/{id}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "minimax",
    "name": "MiniMax",
    "type": "anthropic-compatible",
    "endpoints": {
      "international": "https://api.minimax.io/anthropic",
      "china": "https://api.minimaxi.com/anthropic"
    },
    "default_endpoint": "international",
    "models": ["MiniMax-M2.1"],
    "default_model": "MiniMax-M2.1"
  }
}
```

---

## Usage

### Get Usage Statistics

```http
GET /api/usage
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `period` | string | Time period: today, yesterday, week, month, 7d, 30d, all |
| `profile` | string | Filter by profile alias |
| `model` | string | Filter by model |

**Response:**

```json
{
  "success": true,
  "data": {
    "period": "This week",
    "total_tokens": {
      "input_tokens": 125000,
      "output_tokens": 45000,
      "cache_creation_input_tokens": 10000,
      "cache_read_input_tokens": 50000
    },
    "total_cost": {
      "input_cost": 0.375,
      "output_cost": 0.675,
      "cache_creation_cost": 0.0375,
      "cache_read_cost": 0.015,
      "total_cost": 1.1025
    },
    "total_sessions": 42,
    "total_runtime_secs": 3600,
    "aggregates": {
      "by_profile": {
        "work-claude": {
          "profile": "work-claude",
          "tokens": {
            "input_tokens": 80000,
            "output_tokens": 30000
          },
          "cost": {
            "total_cost": 0.75
          },
          "sessions": 30,
          "runtime_secs": 2400,
          "last_used": "2026-01-20T10:30:00Z"
        }
      }
    }
  }
}
```

### Import Claude Data

```http
POST /api/usage/import-claude
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `claude_dir` | string | Path to .claude directory (optional) |

**Response:**

```json
{
  "success": true,
  "data": "Imported 125000 input tokens, 45000 output tokens from stats-cache.json. Found 42 session entries from JSONL files"
}
```

---

## Proxy

### Get Proxy Status

```http
GET /api/proxy/status
```

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "profile": "work",
      "port": 8081,
      "pid": 12345,
      "status": "running",
      "restarts": 0,
      "started_at": "2026-01-18T10:30:00Z"
    }
  ]
}
```

### Get Proxy Status for Profile

```http
GET /api/proxy/status/{alias}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "profile": "work",
    "enabled": true,
    "port": 8081,
    "pid": 12345,
    "status": "running"
  }
}
```

### Start Proxy

```http
POST /api/proxy/{alias}/start
```

### Stop Proxy

```http
POST /api/proxy/{alias}/stop
```

---

## Registry

### Get Registry Status

```http
GET /api/registry
```

**Response:**

```json
{
  "success": true,
  "data": {
    "channel": "stable",
    "commit": "f4a12c3",
    "last_sync": "2026-01-08T10:00:00Z",
    "cached": true
  }
}
```

### Sync Registry

```http
POST /api/registry/sync
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `force` | boolean | Force refresh |

---

## Daemon

### Get Daemon Status

```http
GET /api/daemon/status
```

**Response:**

```json
{
  "success": true,
  "data": {
    "status": "running",
    "pid": 12345,
    "uptime_secs": 3600,
    "version": "0.1.0",
    "profiles_count": 5,
    "active_proxies": 2
  }
}
```

---

## Terminal Sessions

Terminal sessions allow running agents in a PTY (pseudo-terminal) within the daemon, accessible via WebSocket for real-time interaction through the web UI or other clients.

### List Terminal Sessions

```http
GET /api/terminal/sessions
```

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "46e15057-abbb-42cd-ad0e-52471a76ef9f",
      "profile_alias": "my-project",
      "state": "running",
      "created_at": "2026-01-22T00:22:45Z",
      "pid": 12345,
      "cols": 80,
      "rows": 24,
      "client_count": 1
    }
  ]
}
```

### Get Terminal Session

```http
GET /api/terminal/sessions/{session_id}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "46e15057-abbb-42cd-ad0e-52471a76ef9f",
    "profile_alias": "my-project",
    "state": "running",
    "created_at": "2026-01-22T00:22:45Z",
    "pid": 12345,
    "cols": 80,
    "rows": 24,
    "client_count": 1
  }
}
```

### Create Terminal Session

```http
POST /api/terminal/sessions
Content-Type: application/json
```

**Request Body:**

```json
{
  "profile_alias": "my-project",
  "args": ["--dangerously-skip-permissions"],
  "cols": 80,
  "rows": 24,
  "working_dir": "/home/user/code"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `profile_alias` | string | Yes | Profile to run |
| `args` | string[] | No | Additional arguments for the agent |
| `cols` | number | No | Terminal columns (default: 80) |
| `rows` | number | No | Terminal rows (default: 24) |
| `working_dir` | string | No | Working directory (default: profile home) |

**Response:**

```json
{
  "success": true,
  "data": {
    "session_id": "46e15057-abbb-42cd-ad0e-52471a76ef9f",
    "ws_url": "/ws/terminal/46e15057-abbb-42cd-ad0e-52471a76ef9f"
  }
}
```

### Terminate Terminal Session

```http
DELETE /api/terminal/sessions/{session_id}
```

**Response:**

```json
{
  "success": true
}
```

### Cleanup Terminated Sessions

Remove terminated sessions from the list.

```http
POST /api/terminal/cleanup
```

**Response:**

```json
{
  "success": true
}
```

### Terminal WebSocket

Connect to a terminal session for real-time I/O.

```
WebSocket: ws://127.0.0.1:8765/ws/terminal/{session_id}
```

**Client Messages:**

- **Binary**: Raw terminal input (keystrokes)
- **JSON**: Control messages

```json
// Resize terminal
{ "type": "resize", "cols": 120, "rows": 40 }

// Send signal (e.g., SIGINT=2)
{ "type": "signal", "signal": 2 }
```

**Server Messages:**

- **Binary**: Raw terminal output
- **JSON**: Control messages

```json
// Connected successfully
{ "type": "connected", "session_id": "..." }

// State changed
{ "type": "state_changed", "state": "terminated", "exit_code": 0 }

// Error
{ "type": "error", "message": "Session not found" }
```

**Features:**

- When connecting, the server sends the full scrollback buffer (terminal history)
- Multiple clients can connect to the same session
- Terminal output is broadcast to all connected clients

---

## Web UI

The daemon also serves a web UI at the root path:

```http
GET /
```

This provides a browser-based interface for:

- Viewing profiles
- Monitoring usage
- Managing proxy configurations
- Viewing logs

---

## Error Codes

| HTTP Status | Description |
|-------------|-------------|
| 200 | Success |
| 400 | Bad request (invalid parameters) |
| 404 | Resource not found |
| 409 | Conflict (e.g., profile already exists) |
| 500 | Internal server error |

**Error Response:**

```json
{
  "success": false,
  "error": "Profile 'my-project' not found"
}
```
