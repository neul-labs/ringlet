# Contributing

Thank you for your interest in contributing to Clown! This guide will help you get started.

---

## Ways to Contribute

### Report Bugs

Before creating a bug report, check existing issues to avoid duplicates.

**Include in your report:**

- Clear, descriptive title
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, Clown version)
- Relevant logs or error messages

```bash
# Get version info
clown --version

# Get debug logs
clown --log-level debug <command>
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

- Rust 1.75 or later
- Cargo
- Git

### Clone and Build

```bash
git clone https://github.com/neul-labs/ccswitch.git
cd ccswitch
cargo build
```

### Run Tests

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p clown-core
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
clown/
├── crates/
│   ├── clown/           # CLI binary
│   ├── clownd/          # Background daemon
│   ├── clown-core/      # Core types and utilities
│   └── clown-scripting/ # Rhai scripting engine
├── docs/                # Developer documentation
├── documentation/       # User-facing MkDocs site
├── manifests/           # Built-in agent/provider manifests
└── scripts/             # Built-in Rhai scripts
```

### Key Crates

| Crate | Description |
|-------|-------------|
| `clown` | CLI binary - thin client for daemon |
| `clownd` | Background daemon - core orchestration |
| `clown-core` | Shared types, serialization, filesystem |
| `clown-scripting` | Rhai script engine and built-ins |

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
./target/debug/clown agents list
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

## Testing

### Unit Tests

Add tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

### Integration Tests

Add to `tests/` directory:

```rust
#[test]
fn test_profile_creation() {
    // Integration test
}
```

### Manual Testing

```bash
# Build and test
cargo build
./target/debug/clown agents list
./target/debug/clown providers list
```

---

## Documentation

### User Documentation

User-facing docs use MkDocs in `documentation/`:

```bash
cd documentation
pip install -r requirements.txt
mkdocs serve
# Open http://127.0.0.1:8000
```

### Developer Documentation

Developer docs are in `docs/` as Markdown.

### API Documentation

Generate Rust docs:

```bash
cargo doc --open
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
