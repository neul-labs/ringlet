# Quick Start

Create and run your first ringlet profile in under 2 minutes.

---

## Prerequisites

Before starting, ensure you have:

- [x] ringlet installed ([Installation guide](installation.md))
- [x] An AI coding agent installed (e.g., Claude Code)

---

## Option 1: Interactive Setup (Recommended)

The fastest way to get started is with the interactive wizard:

```bash
ringlet init
```

This will:

1. Detect your installed agents
2. Show available providers
3. Guide you through creating your first profile

**Example session:**

```
$ ringlet init

Welcome to Ringlet!
This wizard will help you get started with managing coding agents.

Checking daemon connection... connected.

Detecting installed agents...

Installed agents:
  [x] Claude Code (1.0.23)

Not installed:
  [ ] Grok CLI
  [ ] Codex CLI

Available providers:
  - self: Agent's Own Auth
  - anthropic: Anthropic API (requires API key)
  - minimax: MiniMax API (requires API key)
  - openrouter: OpenRouter (requires API key)

Would you like to create your first profile? [Y/n] y

--- Create Your First Profile ---

Select an agent: Claude Code (1.0.23)
Select a provider: self - Agent's Own Auth
Profile alias: my-project

Profile 'my-project' created successfully!

Run it with: ringlet profiles run my-project

==================================================
Setup complete!

Next steps:
  ringlet profiles list        View your profiles
  ringlet profiles run <alias> Run an agent session
  ringlet --help               See all available commands
```

!!! tip "Using agent's built-in auth"
    The `self` provider uses the agent's own authentication (e.g., Claude Code's built-in login).
    No API key needed - just sign in when the agent launches.

---

## Option 2: Manual Setup

If you prefer more control, create profiles manually:

### Step 1: Discover Your Agents

```bash
ringlet agents list
```

```
ID          Name         Installed   Version
claude      Claude Code  Yes         1.0.23
codex       Codex CLI    No          -
grok        Grok CLI     No          -
```

### Step 2: List Available Providers

```bash
ringlet providers list
```

```
ID          Name        Type                 Default Model
self        Self        self                 -
anthropic   Anthropic   anthropic            claude-sonnet-4-20250514
minimax     MiniMax     anthropic-compatible MiniMax-M2.1
openrouter  OpenRouter  openai-compatible    anthropic/claude-3.5-sonnet
```

### Step 3: Create Your Profile

```bash
ringlet profiles create claude my-project --provider anthropic
```

You'll be prompted for your API key:

```
Enter API key for Anthropic: ************************************
Profile 'my-project' created successfully!
```

!!! note "API Key Storage"
    Your API key is stored securely in your system's keychain, not in plain text files.

---

## Run Your Profile

Start your agent with the new profile:

```bash
ringlet profiles run my-project
```

Claude Code will launch with:

- Isolated configuration (won't affect other profiles)
- Your API key configured (if using a provider)
- Separate history and settings

---

## Step 5: Create a Quick Alias

For faster access, install a shell alias:

```bash
ringlet aliases install my-project
```

Now you can simply run:

```bash
my-project
```

---

## Step 6: Check Your Usage

After using your agent, check token usage:

```bash
ringlet usage
```

Example output:

```
Usage Summary (Today)
────────────────────
Total Tokens: 15,432
  Input:      12,100
  Output:      3,332

Estimated Cost: $0.04

By Profile:
  my-project    15,432 tokens    $0.04
```

---

## What You've Learned

You've now:

- [x] Discovered available agents and providers
- [x] Created an isolated profile
- [x] Run your agent with the profile
- [x] Set up a quick alias
- [x] Checked your usage statistics

---

## Next Steps

<div class="grid cards" markdown>

-   :material-account-multiple:{ .lg .middle } **Create More Profiles**

    ---

    Create profiles for different projects or providers.

    ```bash
    ringlet profiles create claude work --provider minimax
    ringlet profiles create claude personal --provider anthropic
    ```

-   :material-swap-horizontal:{ .lg .middle } **Switch Profiles**

    ---

    List and switch between profiles easily.

    ```bash
    ringlet profiles list
    ringlet profiles run work
    ```

-   :material-chart-bar:{ .lg .middle } **Track Usage**

    ---

    Monitor costs across profiles and time periods.

    ```bash
    ringlet usage --period week
    ringlet usage profiles
    ```

-   :material-book-open:{ .lg .middle } **Learn More**

    ---

    Explore the full documentation.

    [:octicons-arrow-right-24: Key Concepts](concepts.md)

</div>

---

## Common Commands

| Command | Description |
|---------|-------------|
| `ringlet init` | Interactive setup wizard |
| `ringlet profiles list` | List all profiles |
| `ringlet profiles run <alias>` | Run a profile |
| `ringlet profiles run <alias> --remote` | Run in remote mode (Web UI) |
| `ringlet profiles inspect <alias>` | View profile details |
| `ringlet usage` | Show today's usage |
| `ringlet aliases install <alias>` | Create shell alias |
| `ringlet terminal list` | List active terminal sessions |
| `ringlet daemon status` | Check daemon status |
