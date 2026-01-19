# Contributing to Clown

Thank you for your interest in contributing to Clown! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates. When creating a bug report, include:

- A clear and descriptive title
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- Your environment (OS, Rust version, Clown version)
- Any relevant logs or error messages

### Suggesting Features

Feature suggestions are welcome! Please open an issue and include:

- A clear description of the feature
- The problem it solves or the use case it addresses
- Any implementation ideas you might have

### Pull Requests

1. **Fork the repository** and create your branch from `main`.

2. **Set up development environment:**
   ```bash
   git clone https://github.com/your-username/clown.git
   cd clown
   cargo build
   ```

3. **Make your changes:**
   - Follow the existing code style
   - Add tests for new functionality
   - Update documentation as needed

4. **Test your changes:**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Commit your changes:**
   - Use clear, descriptive commit messages
   - Reference any related issues

6. **Submit the pull request:**
   - Provide a clear description of the changes
   - Link any related issues

## Development Guidelines

### Code Style

- Follow Rust conventions and idioms
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Write documentation for public APIs

### Project Structure

```
clown/
├── crates/
│   ├── clown/          # CLI binary
│   ├── clownd/         # Background daemon
│   ├── clown-core/     # Core types and utilities
│   └── clown-scripting/# Scripting engine
├── docs/               # Documentation
└── manifests/          # Configuration manifests
```

### Testing

- Write unit tests for new functionality
- Ensure all existing tests pass
- Test on multiple platforms if possible

### Documentation

- Update relevant documentation for user-facing changes
- Add doc comments to public APIs
- Keep the README up to date

## Getting Help

If you have questions:

- Check the [documentation](docs/)
- Open a discussion or issue on GitHub

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
