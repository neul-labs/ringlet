# WebSocket Events Reference

The Clown daemon publishes real-time events over WebSocket connections, enabling live updates in UI clients and CLI watch modes.

---

## Overview

Events are published when:

- Profiles are created, updated, or deleted
- Agents are detected or updated
- Registry is synced
- Proxy instances start or stop
- Usage statistics are updated

### Connection

```
ws://127.0.0.1:8765/ws
```

### Message Format

All events are JSON objects:

```json
{
  "type": "event_type",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": { ... }
}
```

---

## Profile Events

### ProfileCreated

Emitted when a new profile is created.

```json
{
  "type": "ProfileCreated",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "alias": "my-project",
    "agent_id": "claude",
    "provider_id": "anthropic",
    "model": "claude-sonnet-4"
  }
}
```

### ProfileUpdated

Emitted when a profile is modified.

```json
{
  "type": "ProfileUpdated",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "alias": "my-project",
    "changes": ["model", "hooks_config"]
  }
}
```

### ProfileDeleted

Emitted when a profile is deleted.

```json
{
  "type": "ProfileDeleted",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "alias": "my-project"
  }
}
```

### ProfileStarted

Emitted when a profile starts running.

```json
{
  "type": "ProfileStarted",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "alias": "my-project",
    "pid": 12345
  }
}
```

### ProfileStopped

Emitted when a profile stops running.

```json
{
  "type": "ProfileStopped",
  "timestamp": "2026-01-08T10:45:00Z",
  "data": {
    "alias": "my-project",
    "exit_code": 0,
    "runtime_secs": 900
  }
}
```

---

## Agent Events

### AgentDetected

Emitted when an agent is detected on the system.

```json
{
  "type": "AgentDetected",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "id": "claude",
    "name": "Claude Code",
    "version": "1.0.0",
    "binary_path": "/usr/local/bin/claude"
  }
}
```

### AgentUpdated

Emitted when an agent's detection info changes.

```json
{
  "type": "AgentUpdated",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "id": "claude",
    "old_version": "0.9.0",
    "new_version": "1.0.0"
  }
}
```

---

## Registry Events

### RegistrySyncStarted

Emitted when registry sync begins.

```json
{
  "type": "RegistrySyncStarted",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "channel": "stable",
    "force": false
  }
}
```

### RegistrySyncCompleted

Emitted when registry sync completes.

```json
{
  "type": "RegistrySyncCompleted",
  "timestamp": "2026-01-08T10:30:05Z",
  "data": {
    "channel": "stable",
    "commit": "f4a12c3",
    "from_cache": false,
    "artifacts_downloaded": 5
  }
}
```

### RegistrySyncFailed

Emitted when registry sync fails.

```json
{
  "type": "RegistrySyncFailed",
  "timestamp": "2026-01-08T10:30:05Z",
  "data": {
    "error": "Network timeout"
  }
}
```

---

## Proxy Events

### ProxyStarted

Emitted when a proxy instance starts.

```json
{
  "type": "ProxyStarted",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "profile": "work",
    "port": 8081,
    "pid": 12345
  }
}
```

### ProxyStopped

Emitted when a proxy instance stops.

```json
{
  "type": "ProxyStopped",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "profile": "work",
    "exit_code": 0
  }
}
```

### ProxyError

Emitted when a proxy encounters an error.

```json
{
  "type": "ProxyError",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "profile": "work",
    "error": "Port already in use"
  }
}
```

---

## Usage Events

### UsageUpdated

Emitted when usage statistics are updated.

```json
{
  "type": "UsageUpdated",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "profile": "my-project",
    "tokens": {
      "input_tokens": 5000,
      "output_tokens": 2000
    },
    "cost": {
      "total_cost": 0.045
    }
  }
}
```

### UsageImported

Emitted when usage data is imported.

```json
{
  "type": "UsageImported",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "source": "claude",
    "records_imported": 42
  }
}
```

---

## Daemon Events

### DaemonStarted

Emitted when the daemon starts.

```json
{
  "type": "DaemonStarted",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "version": "0.1.0",
    "pid": 12345
  }
}
```

### DaemonShuttingDown

Emitted when the daemon begins shutdown.

```json
{
  "type": "DaemonShuttingDown",
  "timestamp": "2026-01-08T10:30:00Z",
  "data": {
    "reason": "idle_timeout"
  }
}
```

---

## Client Implementation

### JavaScript Example

```javascript
const ws = new WebSocket('ws://127.0.0.1:8765/ws');

ws.onopen = () => {
  console.log('Connected to clown daemon');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);

  switch (message.type) {
    case 'ProfileCreated':
      console.log(`Profile ${message.data.alias} created`);
      break;
    case 'UsageUpdated':
      console.log(`Usage updated for ${message.data.profile}`);
      break;
    // Handle other events...
  }
};

ws.onclose = () => {
  console.log('Disconnected from clown daemon');
  // Implement reconnection logic
};
```

### Python Example

```python
import asyncio
import websockets
import json

async def listen_events():
    uri = "ws://127.0.0.1:8765/ws"
    async with websockets.connect(uri) as websocket:
        async for message in websocket:
            event = json.loads(message)
            print(f"Event: {event['type']}")
            print(f"Data: {event['data']}")

asyncio.run(listen_events())
```

---

## Event Subscriptions

By default, clients receive all events. Future versions may support filtering:

```json
{
  "action": "subscribe",
  "events": ["ProfileCreated", "ProfileDeleted", "UsageUpdated"]
}
```

---

## Reconnection

The daemon may restart due to idle timeout or updates. Clients should implement reconnection logic:

1. Detect disconnection
2. Wait with exponential backoff
3. Reconnect and refresh state from HTTP API
4. Resume event listening
