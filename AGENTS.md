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

Facade crate that exposes the public API.
This crate can be ignored unless specifically requested by users.

## Code Standards

### General Principles

- **Async/await**: Full async support throughout the codebase
- **Error handling**: Comprehensive error handling with descriptive messages

### Code Quality

- **clap::warn**: Enabling all Rust warnings for code quality
- **rustfmt**: Consistent code formatting
- **Clippy linting**: Static analysis for code quality
- **Unit tests**: Comprehensive coverage for individual functions
- **Naming conventions**: Following Rust standard naming
- **Tracing instrumentation**: Comprehensive logging for debugging

### Documentation Standards

- **API documentation**: All public functions, structs, and traits
- **AGENTS.md reading and updates**: Read the AGNETS.md when needed and keep it up to date
- **README.md updates**: Keeping user documentation current
- **Inline comments**: Explaining complex algorithms or decisions
- **Module-level docs**: Overview of module purpose and usage

### Misc

- **Rust 2024 Edition**: Following the latest Rust edition for modern features
- **Safety first**: Using Rust's memory safety guarantees
- **Gitmoji Commit Standard**: Using emojis to visually represent commit types
- **GitHub Actions**: Automated testing, linting, and streamlined publishing process

## Development Workflow

The project uses `just` as a command runner to simplify common development tasks.
See the `justfile` for all available commands.

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
# Using just (runs all tests in workspace with all features)
just test
```

### Code Quality

```bash
# Run the application (using cargo directly)
cargo run

# Format code
cargo fmt

# Runs clippy on workspace with all targets and features
just lint
```

### Common Development Tasks

```bash
# Add a new database migration
just migrate-add <name>

# List all available just commands
just
```
