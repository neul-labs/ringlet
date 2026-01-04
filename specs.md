Perfect — this is the **correct choice** if you want *hard isolation*.

Below is a **clean, production-grade way** to run **multiple Claude Code CLIs** with **fully separate state** using separate `HOME` profiles.

This gives you:

* Separate auth / sessions
* Separate `~/.claude/settings.json`
* Separate `~/.claude.json`
* No accidental leakage between profiles
* One shared `claude` binary

---

# ✅ The Correct Mental Model

Claude Code stores **everything** relative to `HOME`:

```text
$HOME/
 ├─ .claude/
 │   └─ settings.json
 ├─ .claude.json
 ├─ .cache/
 └─ other runtime state
```

So:

> **Different HOME = different Claude Code “installation”**

---

# 🧱 Step 1 — Create profile homes

Pick a root directory:

```bash
mkdir -p ~/.claude-profiles/{work,personal,clientA}
```

Each profile becomes a **mini home directory**.

---

# 🧠 Step 2 — Minimal launcher (recommended)

Create `~/bin/claude-profile`:

```bash
#!/usr/bin/env bash
set -euo pipefail

profile="${1:?usage: claude-profile <profile> [claude args...]}"
shift || true

PROFILE_HOME="$HOME/.claude-profiles/$profile"

mkdir -p "$PROFILE_HOME"

HOME="$PROFILE_HOME" exec claude "$@"
```

Make it executable:

```bash
chmod +x ~/bin/claude-profile
```

---

# ▶️ Usage

```bash
claude-profile work
claude-profile personal
claude-profile clientA --settings ./strict.json
```

Each one:

* Prompts for auth separately
* Maintains its own sessions
* Has its own tool permissions
* Has its own MCP config and cache

---

# 🗂 Resulting layout (important)

After first run:

```text
~/.claude-profiles/work/
 ├─ .claude/
 │   └─ settings.json
 ├─ .claude.json
 ├─ .cache/
 └─ .config/

~/.claude-profiles/personal/
 ├─ .claude/
 │   └─ settings.json
 ├─ .claude.json
 └─ .cache/
```

This is **true isolation** — no shared files.

---

# 🔐 Step 3 — Optional: harden isolation (recommended)

Add this inside the wrapper to avoid accidental leakage:

```bash
unset ANTHROPIC_API_KEY
unset CLAUDE_API_KEY
unset XDG_CONFIG_HOME
unset XDG_CACHE_HOME
```

Claude will store everything under the profile HOME instead.

---

# 🧩 Optional: named commands (even cleaner UX)

If you want:

```bash
claude-work
claude-personal
```

Create tiny shims:

```bash
# ~/bin/claude-work
#!/usr/bin/env bash
exec "$HOME/bin/claude-profile" work "$@"
```

---

# 🧪 Verification checklist

Run inside each profile:

```bash
claude-profile work
> /settings
```

Confirm:

* Settings differ
* Auth state differs
* Tools permissions differ

Then switch profiles — nothing should carry over.

---

# 🚀 Why this is the *correct* architecture

This is exactly how:

* `kubectl` contexts
* `terraform` workspaces
* browser profiles
* SSH config isolation

**should** work.

It scales to:

* CI
* Agents
* Multi-tenant dev machines
* Compliance environments

---

# 🧠 Advanced (if you want to go further later)

You can later add:

* A **daemon** that enforces policy per profile
* Audit logs per HOME
* Profile-scoped filesystem mounts
* Profile-scoped tool allowlists

This is the same pattern you were circling around with:

* curl/wget wrappers
* kubectl enhancements
* infra-grade compliance tooling

---

If you want next:

* zsh/bash completion for profiles
* a `claude-profile use work` command
* CI-safe ephemeral profiles
* or a **formal spec** (this could be a real product)

Just say the word.

