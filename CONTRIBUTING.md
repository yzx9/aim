# Contributing to AIM

Thank you for your interest in contributing to AIM! This document provides guidelines and
instructions for contributing to the project.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Commit Messages](#commit-messages)
- [Pull Requests](#pull-requests)
- [Documentation](#documentation)

## Getting Started

### Prerequisites

- **Rust**: Latest stable version (Rust 2024 edition)
- **Git**: For version control
- **just** (optional): Command runner
- **Nix** (optional): Declarative builds and deployments
- **direnv** (optional): Automatic environment management for Nix

The project supports Nix for reproducible development environments, and we recommend giving it a
try:

- Automatic Rust toolchain provisioning
- Shell completion installation
- SQLite system library support
- Development shell with all dependencies
- Works with direnv for automatic shell activation

### Setting Up Development Environment

1. **Clone the repository:**

```bash
git clone https://github.com/yzx9/aim.git
cd aim
```

2. **Set up Nix development environment (recommended):**

The Nix development shell automatically sets these environment variables:

- **AIM_DEV**: Set to `1` to enforce explicit config specification
  - In development mode, config must be provided via `--config` flag or `AIM_CONFIG` env var
  - Prevents accidentally using production configs during development
  - Recognized values: `1`, `true`, `yes` (enables), `0`, `false`, `no` (disables)
  - Case-insensitive
  - Unrecognized values are ignored with a warning

- **AIM_CONFIG**: Set to point to development config at `cli/config.dev.toml`

Using Nix (recommended):

```bash
nix develop
```

Or with **direnv** (for automatic shell activation):

```bash
# Install direnv (if not already installed)
nix-env -iA nixpkgs.direnv

# Hook direnv into your shell
echo 'eval "$(direnv hook bash)"' >> ~/.bashrc
# or for zsh:
echo 'eval "$(direnv hook zsh)"' >> ~/.zshrc

# Reload your shell or run:
source ~/.bashrc  # or source ~/.zshrc

# Allow the .envrc
direnv allow
```

With direnv, the development environment is automatically loaded when you `cd` into the project
directory, and unloaded when you leave it.

Both methods provide a complete development environment with all dependencies and the environment
variables above automatically set.

3. **Or install dependencies manually:**

```bash
cargo build
```

When not using Nix, set the environment variables manually:

```bash
export AIM_DEV=1
export AIM_CONFIG="$(pwd)/cli/config.dev.toml"
```

### Project Structure

The project is organized as a Cargo workspace with four crates:

- **core/** - Core library with calendar logic and database operations
- **cli/** - Command-line interface with TUI support
- **ical/** - iCalendar (RFC 5545) parser
- **aimcal/** - Minimal public API facade

See [architecture.md](docs/architecture.md) for detailed architecture documentation.

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

- Write code following the [Code Standards](#code-standards)
- Add tests for new functionality
- Update documentation as needed

### 3. Run Tests and Linters

Before committing, always run:

```bash
# Run all tests
just test

# Format code
cargo fmt

# Run linter
just lint
```

**Important**: Ensure all tests pass and there are no clippy warnings before submitting a PR.

### 4. Commit Changes

Follow [Gitmoji Commit Standard](https://gitmoji.dev/) for commit messages.

```bash
git add .
git commit -m "✨ Add new feature"
```

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub following the [PR guidelines](#pull-requests).

## Code Standards

### General Principles

- **Async/await**: Full async support throughout the codebase
- **Error handling**: Comprehensive error handling with descriptive messages
- **Type Safety**: Leverage Rust's type system for correctness

### Code Quality

We maintain high code quality standards:

- **rustfmt**: All code must be formatted with rustfmt
- **Clippy**: All clippy warnings must be addressed or justified
- **Tests**: Comprehensive test coverage for new features
- **Documentation**: Public APIs must have rustdoc comments

### Naming Conventions

- **Code**: Follow Rust standard naming conventions
- Use descriptive names that convey purpose
- Prefer clarity over brevity

### Code Organization

- **Clear separation**: Keep business logic separate from presentation
- **Single responsibility**: Each module should have one clear purpose

### Language

- Always write code and comments in **English**
- Use clear, concise explanations in documentation
- Avoid jargon unless well-known in the Rust community

## Testing

For detailed testing guidelines, see [docs/testing.md](docs/testing.md).

## Styling

For detailed style guidelines, see [docs/styling.md](docs/styling.md).

## Pull Requests

### Before Submitting

1. **Update documentation**: Ensure docs reflect your changes
2. **Add tests**: Include tests for new functionality
3. **Run checks**: `just test` and `just lint` must pass
4. **Update CHANGELOG**: Add user-facing changes to CHANGELOG.md
5. **Rebase**: Keep your branch up to date with main

### PR Title Format

Use the same Gitmoji format as commit messages:

```
✨ Brief description of changes
🐛 Brief description of the bug fix, link the issue if available
♻️ Brief description of the refactoring
```

### PR Description

Include:

- **Summary**: What changes were made and why
- **Testing**: How you tested the changes
- **Breaking changes**: Any breaking changes (if applicable)
- **Related issues**: Link to related issues

### Review Process

- Maintain a polite and constructive tone
- Address all review comments
- Request review from relevant maintainers
- Keep the PR focused and relatively small

### Merge Process

Maintainers will:

1. Ensure CI passes
2. Review the code
3. Squash and merge using the PR title

## Documentation

### Types of Documentation

1. **API Documentation** (rustdoc)
   - Document all public items
   - Include examples where helpful
   - Run `cargo doc --open` to view

2. **User Documentation** (README.md)
   - Installation instructions
   - Usage examples
   - Feature overview

3. **Developer Documentation** (CLAUDE.md)
   - Architecture overview
   - Module organization
   - Development guidelines

4. **Change Log** (CHANGELOG.md)
   - Document user-visible changes
   - Follow [Keep a Changelog](https://keepachangelog.com/) format

### Writing Good Documentation

- Start with a brief summary
- Use code examples for complex features
- Keep it up to date as code changes
- Consider the audience (users vs. developers)

## Development Tools

### Just Commands

```bash
just        # List all available commands
just build  # Build all crates
just test   # Run all tests
just lint   # Run clippy
```

## Getting Help

- **Documentation**: Start with [docs](docs) and crate-specific docs
- **Issues**: Search existing issues before creating new ones
- **Discussions**: Use GitHub Discussions for questions
- **RFC**: For major changes, consider writing an RFC first

## License

By contributing to AIM, you agree that your contributions will be licensed under the [Apache-2.0](LICENSE) License.

---

Thank you for contributing to AIM! Your contributions help make this project better for everyone.
