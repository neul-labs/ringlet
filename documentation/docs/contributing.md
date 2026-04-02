# Contributing

Thank you for your interest in contributing to Ringlet! This guide will help you get started.

---

## Ways to Contribute

### Report Bugs

Before creating a bug report, check existing issues to avoid duplicates.

**Include in your report:**

- Clear, descriptive title
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, Ringlet version)
- Relevant logs or error messages

```bash
# Get version info
ringlet --version

# Get debug logs
ringlet --log-level debug <command>
```

### Suggest Features

Feature suggestions are welcome! Open an issue with:

- Clear description of the feature
- Problem it solves or use case
- Implementation ideas (optional)

### Submit Pull Requests

1. Fork and clone the repository
2. Create a branch from `main`
3. Make your changes
4. Test thoroughly
5. Submit a pull request

---

## Development Setup

### Prerequisites

- Rust 1.85 or later (2024 edition)
- Cargo
- Git
- Node.js (for frontend development)

### Clone and Build

```bash
git clone https://github.com/neul-labs/ringlet.git
cd ringlet
cargo build
```

### Run Tests

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p ringlet-core
```

### Check Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check formatting
cargo fmt --check
```

---

## Project Structure

```
ringlet/
├── crates/
│   ├── ringlet/           # Unified binary (CLI + daemon + desktop app)
│   ├── ringlet-core/      # Shared types and utilities
│   └── ringlet-scripting/ # Rhai scripting engine
├── documentation/         # MkDocs user documentation
├── ringlet-ui/            # Vue 3 + TypeScript frontend
├── packaging/             # Platform-specific packaging
└── scripts/               # Build and release scripts
```

### Key Crates

| Crate | Description |
|-------|-------------|
| `ringlet` | Unified binary — CLI client, background daemon, and desktop app |
| `ringlet-core` | Shared types, serialization, filesystem utilities |
| `ringlet-scripting` | Rhai script engine and built-in functions |

---

## Code Guidelines

### Style

- Follow Rust conventions and idioms
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Write documentation for public APIs

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add MiniMax provider support

- Add provider manifest for MiniMax
- Create Rhai script for configuration
- Update documentation

Closes #123
```

### Documentation

- Update docs for user-facing changes
- Add doc comments to public APIs
- Keep README current

---

## Adding Agents

To add support for a new AI coding agent:

### 1. Create Agent Manifest

Add to `manifests/agents/<id>.toml`:

```toml
id = "my-agent"
name = "My Agent"
binary = "my-agent"
version_flag = "--version"

[detect]
commands = ["my-agent --version"]

[profile]
strategy = "home-wrapper"
source_home = "~/.my-agent-profiles/{alias}"
script = "my-agent.rhai"

[models]
default = "default-model"
supported = ["default-model", "other-model"]
```

### 2. Create Rhai Script

Add to `scripts/<id>.rhai`:

```rhai
// Configuration generation for my-agent

let config = #{
    "api_key": provider.api_key,
    "model": provider.model,
    "base_url": provider.endpoint
};

#{
    "files": #{
        ".my-agent/config.json": json::encode(config)
    },
    "env": #{
        "MY_AGENT_API_KEY": provider.api_key
    }
}
```

### 3. Test Detection

```bash
cargo build
./target/debug/ringlet agents list
```

---

## Adding Providers

To add a new API provider:

### Create Provider Manifest

Add to `manifests/providers/<id>.toml`:

```toml
id = "my-provider"
name = "My Provider"
type = "anthropic-compatible"  # or "openai-compatible"

[endpoints]
default = "https://api.example.com/v1"

[auth]
env_key = "MY_PROVIDER_API_KEY"
prompt = "Enter your API key"

[models]
available = ["model-a", "model-b"]
default = "model-a"
```

---

## Frontend Development

The web UI is a Vue 3 + TypeScript SPA in `ringlet-ui/`:

```bash
cd ringlet-ui
npm install
npm run dev
```

The dev server proxies API requests to the daemon at `127.0.0.1:8765`.

---

## User Documentation

User-facing docs use MkDocs in `documentation/`:

```bash
cd documentation
pip install -r requirements.txt
mkdocs serve
# Open http://127.0.0.1:8000
```

---

## Pull Request Process

### Before Submitting

- [ ] Code compiles without errors
- [ ] All tests pass
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt --check` passes
- [ ] Documentation updated if needed
- [ ] Commit messages are clear

### PR Description

Include:

- What the change does
- Why it's needed
- How to test it
- Related issues (e.g., "Closes #123")

### Review Process

1. Automated CI checks run
2. Maintainer reviews code
3. Address feedback if needed
4. PR is merged when approved

---

## Getting Help

- **Documentation**: Read the docs first
- **Issues**: Open a GitHub issue
- **Discussions**: Use GitHub Discussions for questions

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
