# Installation

This guide covers all the ways to install Ringlet on your system.

---

## Quick Install (Recommended)

The easiest way to install Ringlet is using our install script:

=== "Linux/macOS"

    ```bash
    curl -fsSL https://raw.githubusercontent.com/neul-labs/ringlet/main/install.sh | bash
    ```

This will:

1. Detect your platform (Linux/macOS, x64/arm64)
2. Download the pre-built binary
3. Install to `~/.local/bin`
4. Provide instructions to add to your PATH

---

## Installation Methods

### Pre-built Binaries

Download the latest release directly:

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | [ringlet-linux-x64](https://github.com/neul-labs/ringlet/releases/latest) |
| Linux | arm64 | [ringlet-linux-arm64](https://github.com/neul-labs/ringlet/releases/latest) |
| macOS | x86_64 | [ringlet-darwin-x64](https://github.com/neul-labs/ringlet/releases/latest) |
| macOS | Apple Silicon | [ringlet-darwin-arm64](https://github.com/neul-labs/ringlet/releases/latest) |

After downloading:

```bash
# Extract
tar -xzf ringlet-*.tar.gz

# Move to PATH
mv ringlet ~/.local/bin/

# Make executable
chmod +x ~/.local/bin/ringlet
```

### From Source

Build from source using Cargo:

```bash
# Install from git
cargo install --git https://github.com/neul-labs/ringlet ringlet

# Or clone and build
git clone https://github.com/neul-labs/ringlet
cd ringlet
cargo build --release
cp target/release/ringlet ~/.local/bin/
```

!!! note "Requirements"
    Building from source requires:

    - Rust 1.85 or later (2024 edition)
    - A C compiler (for native dependencies)

### Local Build

If you have the repository cloned, you can build and install directly:

```bash
cd /path/to/ringlet
./install.sh --local
```

This is useful for:

- Development and testing
- Offline installation
- When GitHub releases are unavailable

---

## Verify Installation

After installation, verify everything works:

```bash
# Check version
ringlet --version

# List available agents
ringlet agents list

# Check daemon status
ringlet daemon status
```

Expected output:

```
ringlet 0.1.0
```

---

## PATH Configuration

If `ringlet` is not found after installation, add it to your PATH:

=== "Bash"

    ```bash
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
    source ~/.bashrc
    ```

=== "Zsh"

    ```bash
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
    source ~/.zshrc
    ```

=== "Fish"

    ```bash
    fish_add_path ~/.local/bin
    ```

---

## Installing Agents

Ringlet works with these coding agents:

### Claude Code

```bash
# Via npm
npm install -g @anthropic-ai/claude-code

# Verify
claude --version
```

### Codex CLI

```bash
# Via npm
npm install -g @openai/codex

# Verify
codex --version
```

### Grok CLI

```bash
# Follow installation from xAI
# https://grok.x.ai/cli
```

!!! tip "Agent Detection"
    Ringlet automatically detects installed agents. Run `ringlet agents list` to see what's available.

---

## Uninstalling

To remove Ringlet:

```bash
# Stop the daemon
ringlet daemon stop

# Remove binaries
rm ~/.local/bin/ringlet

# Remove configuration (optional)
rm -rf ~/.config/ringlet
```

---

## Next Steps

Now that Ringlet is installed:

1. [:octicons-arrow-right-24: Follow the Quick Start](quickstart.md) to create your first profile
2. [:octicons-arrow-right-24: Learn the Key Concepts](concepts.md) to understand how Ringlet works
