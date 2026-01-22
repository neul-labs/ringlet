# Event Hooks

Hooks allow you to execute custom commands or webhooks in response to events during agent execution. This enables logging, auditing, notifications, and integration with external systems.

---

## Overview

Hooks are profile-level configurations that trigger actions when specific events occur during agent execution. Each profile can have its own set of hooks.

### Supported Events

| Event | Description | When it fires |
|-------|-------------|---------------|
| `PreToolUse` | Before a tool is executed | Just before Bash, Write, Edit, etc. |
| `PostToolUse` | After a tool completes | After tool execution finishes |
| `Notification` | On agent notifications | When agent sends a notification |
| `Stop` | When agent stops | On normal or error termination |

!!! note "Agent Support"
    The agent must support hooks (`supports_hooks: true` in manifest). Currently, **Claude Code** is the primary agent with hooks support.

---

## CLI Commands

### Add a Hook

```bash
clown hooks add <alias> <event> <matcher> <command>
```

**Parameters:**

| Parameter | Description |
|-----------|-------------|
| `alias` | Profile alias |
| `event` | Event type (PreToolUse, PostToolUse, Notification, Stop) |
| `matcher` | Tool pattern to match (e.g., "Bash\|Write" or "*" for all) |
| `command` | Shell command to execute |

**Examples:**

```bash
# Log all Bash commands before execution
clown hooks add myprofile PreToolUse "Bash" "echo 'Running: $EVENT' >> /tmp/clown.log"

# Log all tool usage after completion
clown hooks add myprofile PostToolUse "*" "logger -t clown '$EVENT'"

# Send notification to webhook on stop
clown hooks add myprofile Stop "*" "curl -X POST https://hooks.example.com/clown -d '$EVENT'"

# Match multiple tools
clown hooks add myprofile PreToolUse "Bash|Write|Edit" "echo 'File operation: $EVENT'"
```

### List Hooks

```bash
clown hooks list <alias>
```

**Example output:**

```
Hooks for profile 'myprofile':

PreToolUse:
  [0] matcher: "Bash"
      command: echo 'Running: $EVENT' >> /tmp/clown.log

PostToolUse:
  [0] matcher: "*"
      command: logger -t clown '$EVENT'

Stop:
  [0] matcher: "*"
      command: curl -X POST https://hooks.example.com/clown -d '$EVENT'
```

### Remove a Hook

```bash
clown hooks remove <alias> <event> <index>
```

**Parameters:**

- `alias` - Profile alias
- `event` - Event type
- `index` - Rule index (0-based, as shown in `hooks list`)

**Example:**

```bash
# Remove the first PreToolUse hook
clown hooks remove myprofile PreToolUse 0
```

### Import/Export Hooks

```bash
# Import hooks from JSON file
clown hooks import <alias> <file>

# Export hooks to JSON
clown hooks export <alias> > hooks-backup.json

# View current configuration
clown hooks export myprofile | jq .
```

---

## Configuration Format

Hooks are stored in JSON format:

```json
{
  "PreToolUse": [
    {
      "matcher": "Bash|Write",
      "hooks": [
        {
          "type": "command",
          "command": "echo $EVENT >> /tmp/tool-log.txt",
          "timeout": 5000
        }
      ]
    }
  ],
  "PostToolUse": [
    {
      "matcher": "*",
      "hooks": [
        {
          "type": "command",
          "command": "logger -t clown 'Tool completed'"
        }
      ]
    }
  ],
  "Notification": [],
  "Stop": [
    {
      "matcher": "*",
      "hooks": [
        {
          "type": "url",
          "url": "https://hooks.example.com/agent-stopped"
        }
      ]
    }
  ]
}
```

### Hook Rule Structure

| Field | Type | Description |
|-------|------|-------------|
| `matcher` | string | Tool pattern (pipe-separated names or "*") |
| `hooks` | array | Actions to execute when matched |

### Hook Action Types

**Command Action:**

```json
{
  "type": "command",
  "command": "echo $EVENT",
  "timeout": 5000
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be "command" |
| `command` | Yes | Shell command to execute |
| `timeout` | No | Timeout in milliseconds |

**URL Action (Webhook):**

```json
{
  "type": "url",
  "url": "https://hooks.example.com/webhook"
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be "url" |
| `url` | Yes | Webhook URL to POST to |

---

## Event Data

The `$EVENT` variable in commands contains JSON data about the event:

```json
{
  "type": "PreToolUse",
  "tool": "Bash",
  "timestamp": "2025-01-18T10:30:00Z",
  "profile": "myprofile",
  "details": { ... }
}
```

---

## Use Cases

### Audit Logging

Log all tool usage to a file:

```bash
clown hooks add work PostToolUse "*" \
  "echo '[$(date -Iseconds)] $EVENT' >> ~/.clown/audit.log"
```

### Slack Notifications

Send notifications when agent stops:

```bash
clown hooks add work Stop "*" \
  "curl -X POST -H 'Content-type: application/json' \
   --data '{\"text\":\"Agent stopped\"}' \
   $SLACK_WEBHOOK_URL"
```

### Custom Validation

Block certain operations with pre-tool hooks:

```bash
clown hooks add work PreToolUse "Bash" \
  "echo '$EVENT' | jq -e '.command | contains(\"rm -rf\") | not' || exit 1"
```

### Auto-Formatting

Run formatters after file edits:

```bash
clown hooks add work PostToolUse "Edit|Write" \
  "prettier --write \"$CLAUDE_FILE_PATHS\" 2>/dev/null || true"
```

### Integration with Monitoring

Send metrics to monitoring systems:

```bash
clown hooks add work PostToolUse "*" \
  "curl -X POST http://localhost:9091/metrics/job/clown \
   -d 'tool_execution_count{tool=\"'$(echo $EVENT | jq -r .tool)'\"} 1'"
```

---

## Storage

Hooks are stored in the profile's metadata:

```
~/.config/clown/profiles/{alias}.json
```

The `hooks_config` field in profile metadata contains the full hooks configuration.

---

## Troubleshooting

### Hook not firing

1. Verify the event type is correct (case-sensitive: `PreToolUse`, not `pretooluse`)
2. Check the matcher pattern matches the tool name
3. Ensure the agent supports hooks

### Command errors

1. Test the command manually in a shell
2. Check the `$EVENT` variable is properly quoted
3. Verify command timeout is sufficient

### Viewing hook execution

Enable debug logging:

```bash
clown --log-level debug profiles run myprofile
```
