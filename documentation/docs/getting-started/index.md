# Getting Started

Welcome to Clown! This section will help you get up and running quickly.

---

## Overview

Clown helps you manage multiple AI coding agents with different configurations. Here's what you'll learn:

1. **[Installation](installation.md)** - Install Clown on your system
2. **[Quick Start](quickstart.md)** - Create and run your first profile
3. **[Key Concepts](concepts.md)** - Understand how Clown works

---

## Prerequisites

Before installing Clown, ensure you have:

- **An AI coding agent** installed (Claude Code, Codex CLI, etc.)
- **An API key** from your provider (Anthropic, OpenAI, etc.)
- **Linux or macOS** (Windows support coming soon)

---

## 5-Minute Setup

If you want to dive right in:

```bash
# 1. Install Clown
curl -fsSL https://raw.githubusercontent.com/neul-labs/ccswitch/main/install.sh | bash

# 2. Create a profile
clown profiles create claude my-project --provider anthropic

# 3. Run it
clown profiles run my-project
```

That's it! Your agent will start with its own isolated configuration.

---

## What's Next?

<div class="grid cards" markdown>

-   :material-download:{ .lg .middle } **Installation**

    ---

    Detailed installation instructions for all platforms and methods.

    [:octicons-arrow-right-24: Install Clown](installation.md)

-   :material-rocket-launch:{ .lg .middle } **Quick Start**

    ---

    Step-by-step tutorial to create your first profile.

    [:octicons-arrow-right-24: Quick Start](quickstart.md)

-   :material-lightbulb:{ .lg .middle } **Key Concepts**

    ---

    Understand agents, providers, profiles, and how they work together.

    [:octicons-arrow-right-24: Learn Concepts](concepts.md)

</div>
