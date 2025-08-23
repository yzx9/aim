# AIM Project Overview

AIM (Analyze. Interact. Manage) is a Rust-based calendar and task management application that provides both a core library and a command-line interface. The project follows a modular architecture with a clear separation between the core functionality and the CLI presentation layer.

## Project Structure

The project is organized as a Cargo workspace with three main crates:

```
aim/            # Workspace root
├── core/       # Core library (aimcal-core)
├── cli/        # Command-line interface (aimcal-cli)
├── aimcal/     # Public API facade crate
├── Cargo.toml  # Workspace configuration
└── AGENTS.md   # This file
```

### Core Crate (`core/`)

The foundation of the application that provides:

- Calendar event and todo management
- Local database storage using SQLite
- iCalendar format parsing and generation
- Date/time handling with timezone support
- Configuration management
- Priority and status systems

### CLI Crate (`cli/`)

Command-line interface that provides:

- Intuitive command structure using clap
- Interactive TUI modes using ratatui

### Public API Crate (`aimcal/`)

Facade crate that exposes the public API, simplified interface for integration.

## Code Standards

### General Principles

- **Rust 2024 Edition**: Following the latest Rust edition for modern features
- **Zero-cost abstractions**: Leveraging Rust's performance characteristics
- **Safety first**: Using Rust's memory safety guarantees
- **Async/await**: Full async support throughout the codebase
- **Error handling**: Comprehensive error handling with descriptive messages
- **Documentation**: Complete API documentation for all public items
- **Testing**: Extensive unit tests for critical functionality

### Code Quality

- **clap::warn**: Enabling all Rust warnings for code quality
- **Tracing instrumentation**: Comprehensive logging for debugging
- **Type safety**: Strong typing to prevent runtime errors
- **Lifetime management**: Proper ownership and borrowing
- **Dead code elimination**: Removing unused code paths

### Formatting and Style

- **rustfmt**: Consistent code formatting
- **Clippy linting**: Static analysis for code quality
- **Line length**: 100 characters maximum
- **Naming conventions**: Following Rust standard naming
- **Module organization**: Clear separation of concerns

### Gitmoji Commit Standard

The project follows the Gitmoji standard using emojis to visually represent commit types, making the commit history more readable and expressive at a glance.

### Testing Standards

- **Unit tests**: Comprehensive coverage for individual functions
- **Integration tests**: End-to-end testing of major features
- **Property-based testing**: For complex algorithms
- **Mocking**: For external dependencies
- **Test organization**: Clear separation of test types

### Documentation Standards

- **API documentation**: All public functions, structs, and traits
- **Code examples**: Practical usage examples in documentation
- **README updates**: Keeping user documentation current
- **Inline comments**: Explaining complex algorithms or decisions
- **Module-level docs**: Overview of module purpose and usage

### Continuous Integration

- **GitHub Actions**: Automated testing and linting
- **Cross-platform testing**: Ensuring compatibility
- **Code coverage**: Monitoring test coverage
- **Security scanning**: Dependency vulnerability checks
- **Release automation**: Streamlined publishing process

## Dependencies and Features

### Core Dependencies

- **sqlx**: Type-safe SQL database access
- **chrono**: Date and time handling
- **icalendar**: iCalendar format support
- **tokio**: Async runtime
- **serde**: Serialization framework
- **uuid**: Unique identifier generation
- **tracing**: Logging and instrumentation
- **dirs**: Platform-specific directory handling

### CLI Dependencies

- **clap**: Command-line argument parsing
- **colored**: Terminal color support
- **comfy-table**: Table formatting in terminal
- **crossterm**: Cross-platform terminal handling
- **ratatui**: TUI framework

## Development Workflow

The project uses `just` as a command runner to simplify common development tasks. See the `justfile` for all available commands.

### Building

```bash
# Build all crates
cargo build

# Build release version
cargo build --release

# Build specific crate
cargo build -p aimcal-core
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p aimcal-core

# Run with specific features
cargo test --features "clap"

# Using just (runs all tests in workspace with all features)
just test
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for security issues
cargo audit

# Using just (runs clippy on workspace with all targets and features)
just lint
```

### Common Development Tasks

```bash
# Run the application (using cargo directly)
cargo run

# Generate documentation
cargo doc --workspace --all-features --no-deps --open

# Clean build artifacts
cargo clean

# Release new version (without publishing)
just release <version>

# Add a new database migration
just migrate-add <name>

# List all available just commands
just
```

## Project Goals

### Current Features

- Event and todo management via CLI or TUI
- Local SQLite storage with migration support
- iCalendar format compatibility
- Flexible date/time parsing and formatting
- Priority and status management
- Rich output formatting options

### Future Roadmap

- AI-powered scheduling assistance
- CalDAV server integration
- Webhook and REST API support
- Undo history editing
- Markdown support in descriptions
- Full text search capabilities
- Recurring event support
