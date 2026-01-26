# Troubleshooting

Common issues and their solutions when working with Clown.

---

## Daemon Issues

### Daemon Not Starting

**Symptoms:**

- Commands hang or timeout
- "Connection refused" errors

**Solutions:**

1. Check if daemon is already running:
   ```bash
   ringlet daemon status
   ```

2. Check for stale socket file:
   ```bash
   # Linux/macOS
   rm /tmp/ringletd.sock

   # Windows
   del %LOCALAPPDATA%\ringlet\ringletd.ipc
   ```

3. Start daemon manually with debug logging:
   ```bash
   ringlet --log-level debug daemon start
   ```

4. Check daemon logs:
   ```bash
   cat ~/.config/ringlet/logs/ringletd.log
   ```

### Daemon Keeps Stopping

**Cause:** Daemon exits after idle timeout by default.

**Solution:** Keep daemon running indefinitely:

```bash
ringlet daemon start --stay-alive
```

---

## Profile Issues

### Profile Creation Fails

**Symptoms:**

- "Provider not compatible with agent" error
- "Failed to run Rhai script" error

**Solutions:**

1. Verify agent-provider compatibility:
   ```bash
   ringlet agents inspect <agent-id>
   # Check "Compatible Providers"
   ```

2. Sync registry for latest scripts:
   ```bash
   ringlet registry sync --force
   ```

3. Check if custom script has syntax errors:
   ```bash
   ringlet scripts test <agent>.rhai --provider <provider>
   ```

### Profile Won't Run

**Symptoms:**

- Agent exits immediately
- "Command not found" error

**Solutions:**

1. Verify agent is installed:
   ```bash
   ringlet agents list
   which <agent-binary>
   ```

2. Check profile configuration:
   ```bash
   ringlet profiles inspect <alias>
   ```

3. Verify API key is valid:
   ```bash
   # Check if key is stored
   ringlet profiles inspect <alias>
   # Should show "API Key: ****...****"
   ```

4. Run with debug logging:
   ```bash
   ringlet --log-level debug profiles run <alias>
   ```

### Wrong Model Being Used

**Cause:** Model specified in profile doesn't match expected.

**Solutions:**

1. Inspect profile:
   ```bash
   ringlet profiles inspect <alias>
   ```

2. Recreate with explicit model:
   ```bash
   ringlet profiles delete <alias>
   ringlet profiles create <agent> <alias> --provider <provider> --model <model>
   ```

---

## Agent Issues

### Agent Not Detected

**Symptoms:**

- Agent shows as "Not Installed" in `agents list`
- "Agent not found" error

**Solutions:**

1. Verify binary is in PATH:
   ```bash
   which claude
   # Should return path like /usr/local/bin/claude
   ```

2. Verify detection command works:
   ```bash
   claude --version
   ```

3. Sync registry to update manifests:
   ```bash
   ringlet registry sync --force
   ```

4. Check agent manifest detection config:
   ```bash
   ringlet agents inspect <agent-id>
   ```

### Wrong Version Displayed

**Cause:** Detection cache is stale.

**Solutions:**

1. Force registry sync:
   ```bash
   ringlet registry sync --force
   ```

2. Clear detection cache:
   ```bash
   rm ~/.config/ringlet/cache/agent-detections.json
   ringlet agents list
   ```

---

## Provider Issues

### API Key Not Working

**Symptoms:**

- Authentication errors from provider
- "Invalid API key" messages

**Solutions:**

1. Verify key is correct with provider directly
2. Recreate profile to re-enter key:
   ```bash
   ringlet profiles delete <alias>
   ringlet profiles create <agent> <alias> --provider <provider>
   ```

3. Check environment variables aren't overriding:
   ```bash
   env | grep -i api_key
   ```

### Wrong Endpoint Used

**Symptoms:**

- Connection errors
- Requests going to wrong region

**Solutions:**

1. Specify endpoint explicitly:
   ```bash
   ringlet profiles create <agent> <alias> --provider minimax --endpoint china
   ```

2. Check profile endpoint:
   ```bash
   ringlet profiles inspect <alias>
   ```

3. Verify provider endpoints:
   ```bash
   ringlet providers inspect <provider-id>
   ```

---

## Proxy Issues

### Proxy Not Starting

**Symptoms:**

- "Failed to start proxy" error
- Agent can't connect to local endpoint

**Solutions:**

1. Check if ultrallm is installed:
   ```bash
   which ultrallm
   # Or check ringlet's binary cache
   ls ~/.cache/ringlet/binaries/ultrallm
   ```

2. Check if port is in use:
   ```bash
   lsof -i :8081
   ```

3. View proxy logs:
   ```bash
   ringlet proxy logs <alias>
   ```

4. Start proxy manually to see errors:
   ```bash
   ringlet proxy start <alias>
   ```

### Routing Not Working

**Symptoms:**

- All requests go to same provider
- Rules seem to be ignored

**Solutions:**

1. Verify routing rules:
   ```bash
   ringlet proxy route list <alias>
   ```

2. Check generated config:
   ```bash
   cat ~/.claude-profiles/<alias>/.ultrallm/config.yaml
   ```

3. Ensure API keys are set for all target providers

### Connection Refused

**Symptoms:**

- Agent gets "connection refused" error
- Proxy appears to be running

**Solutions:**

1. Verify proxy status:
   ```bash
   ringlet proxy status <alias>
   ```

2. Check port matches profile config:
   ```bash
   ringlet profiles inspect <alias>
   ringlet proxy config <alias>
   ```

3. Test proxy health endpoint:
   ```bash
   curl http://127.0.0.1:8081/health
   ```

---

## Usage Tracking Issues

### No Usage Data Showing

**Symptoms:**

- `ringlet usage` shows zeros
- "No data for period" message

**Solutions:**

1. Verify daemon is running:
   ```bash
   ringlet daemon status
   ```

2. Check if profiles have been used:
   ```bash
   ringlet profiles list
   # Check "Last Used" column
   ```

3. Verify telemetry directory exists:
   ```bash
   ls ~/.config/ringlet/telemetry/
   ```

### Costs Showing as Null

**Cause:** Cost calculation only works for "self" provider.

**Solutions:**

1. Check provider type:
   ```bash
   ringlet profiles inspect <alias>
   ```

2. Verify pricing data is synced:
   ```bash
   ls ~/.config/ringlet/registry/litellm-pricing.json
   ```

3. Sync registry:
   ```bash
   ringlet registry sync
   ```

### Import Not Finding Data

**Symptoms:**

- "No data found" when importing Claude data
- Import shows 0 records

**Solutions:**

1. Verify Claude directory exists:
   ```bash
   ls ~/.claude
   ```

2. Check for stats-cache.json:
   ```bash
   ls ~/.claude/stats-cache.json
   ```

3. Specify path explicitly:
   ```bash
   ringlet usage import-claude --claude-dir ~/.claude
   ```

---

## Registry Issues

### Sync Fails

**Symptoms:**

- "Network error" during sync
- Timeout errors

**Solutions:**

1. Check internet connectivity
2. Use offline mode:
   ```bash
   ringlet registry sync --offline
   ```

3. Check if GitHub is accessible:
   ```bash
   curl https://github.com
   ```

4. Use environment variable to override URL:
   ```bash
   export CLOWN_REGISTRY_URL=<alternative-url>
   ```

### Custom Manifest Not Loaded

**Symptoms:**

- Custom agent/provider not appearing
- Changes not taking effect

**Solutions:**

1. Verify file location:
   ```bash
   ls ~/.config/ringlet/agents.d/
   ls ~/.config/ringlet/providers.d/
   ```

2. Check TOML syntax:
   ```bash
   # Use a TOML validator
   cat ~/.config/ringlet/agents.d/my-agent.toml | python3 -c "import sys, toml; toml.load(sys.stdin)"
   ```

3. Force sync:
   ```bash
   ringlet registry sync --force
   ```

---

## Hooks Issues

### Hook Not Firing

**Symptoms:**

- Command not executing
- No output from hook

**Solutions:**

1. Verify event type is correct (case-sensitive):
   ```bash
   # Correct: PreToolUse, PostToolUse, Notification, Stop
   # Wrong: pretooluse, pre_tool_use
   ```

2. Check matcher pattern:
   ```bash
   ringlet hooks list <alias>
   ```

3. Ensure agent supports hooks:
   ```bash
   ringlet agents inspect <agent-id>
   # Check "Supports Hooks: Yes"
   ```

4. Enable debug logging:
   ```bash
   ringlet --log-level debug profiles run <alias>
   ```

### Hook Command Errors

**Symptoms:**

- Hook runs but fails
- Partial output

**Solutions:**

1. Test command manually:
   ```bash
   echo '{"type":"PreToolUse","tool":"Bash"}' | your-command
   ```

2. Check `$EVENT` variable quoting:
   ```bash
   # Good: echo '$EVENT'
   # Bad: echo $EVENT (unquoted)
   ```

3. Increase timeout if needed:
   ```json
   {
     "type": "command",
     "command": "slow-command",
     "timeout": 30000
   }
   ```

---

## Getting Help

If you're still stuck:

1. **Check logs:**
   ```bash
   cat ~/.config/ringlet/logs/ringletd.log
   ```

2. **Enable debug mode:**
   ```bash
   ringlet --log-level debug <command>
   ```

3. **Report issues:**
   [https://github.com/neul-labs/ccswitch/issues](https://github.com/neul-labs/ccswitch/issues)

4. **Include in bug reports:**
   - Clown version (`ringlet --version`)
   - OS and version
   - Agent and provider being used
   - Relevant log output
   - Steps to reproduce
