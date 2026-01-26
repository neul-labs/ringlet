# Quick Start

Create and run your first Clown profile in under 5 minutes.

---

## Prerequisites

Before starting, ensure you have:

- [x] Clown installed ([Installation guide](installation.md))
- [x] An AI coding agent installed (e.g., Claude Code)
- [x] An API key from your provider

---

## Step 1: Discover Your Agents

First, see what agents Clown has detected:

```bash
ringlet agents list
```

Example output:

```
ID          Name         Installed   Version
claude      Claude Code  Yes         1.0.0
codex       Codex CLI    Yes         0.5.0
grok        Grok CLI     No          -
```

!!! tip "Agent Not Detected?"
    If your agent isn't showing, ensure it's in your PATH and try:
    ```bash
    ringlet registry sync --force
    ```

---

## Step 2: List Available Providers

See what providers are available for your agent:

```bash
ringlet providers list
```

Example output:

```
ID          Name        Type                 Default Model
anthropic   Anthropic   anthropic            claude-sonnet-4-20250514
minimax     MiniMax     anthropic-compatible claude-sonnet-4-20250514
openai      OpenAI      openai               gpt-4o
openrouter  OpenRouter  openai-compatible    anthropic/claude-3.5-sonnet
```

---

## Step 3: Create Your First Profile

Create a profile that binds Claude Code to Anthropic:

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

## Step 4: Run Your Profile

Start your agent with the new profile:

```bash
ringlet profiles run my-project
```

Claude Code will launch with:

- Isolated configuration (won't affect other profiles)
- Your Anthropic API key configured
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
| `ringlet profiles list` | List all profiles |
| `ringlet profiles run <alias>` | Run a profile |
| `ringlet profiles inspect <alias>` | View profile details |
| `ringlet usage` | Show today's usage |
| `ringlet aliases install <alias>` | Create shell alias |
| `ringlet daemon status` | Check daemon status |
