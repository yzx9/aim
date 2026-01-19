# AIM Architecture

This document provides a high-level overview of the AIM system architecture and
design decisions.

## Overview

AIM is a modular calendar and task management application built with Rust,
following a layered architecture with clear separation between core business
logic, data persistence, and user interface.

## Workspace Structure

```
aim/
├── .github/        # GitHub configurations, including CI/CD workflows and dependabot
├── aimcal/         # Public API facade
├── cli/            # Command-line interface
├── core/           # Core business logic
├── caldav/         # CalDAV client
├── ical/           # iCalendar parser
├── docs/           # Documentation
├── Cargo.toml      # Workspace configuration
├── CHANGELOG.md    # Version history
├── CLAUDE.md       # Project overview
├── CONTRIBUTING.md # Contribution guidelines
├── flake.nix       # Nix flake
├── justfile        # Command runner recipes
├── LICENSE         # Apache-2.0 license
└── README.md       # User-facing documentation
```

### Crate Responsibilities

**aimcal** - Minimal facade that re-exports the CLI

**cli** - User interface providing:

- Command parsing with clap
- Interactive TUI with ratatui
- Table and JSON output formatting
- Shell completion generation

**core** - Foundation providing:

- Event and todo management
- SQLite database operations
- DateTime handling with timezone support
- Configuration management
- Short ID mapping

**caldav** - CalDAV (RFC 4791) client with WebDAV (RFC 4918) support

**ical** - iCalendar (RFC 5545) parser and formatter with:

- Three-phase parsing pipeline (syntax → typed → semantic)
- Type-safe representations with generic `StringStorage` trait for flexible string and span handling
- Comprehensive error reporting
- RFC 5545 formatter for serializing components, properties, parameters, and values

## Technology Stack

| Component         | Technology    |
| ----------------- | ------------- |
| **Language**      | Rust 2024     |
| **Async Runtime** | Tokio         |
| **Database**      | SQLite + sqlx |
| **CLI**           | clap          |
| **TUI**           | ratatui       |
| **Lexer**         | logos         |
| **Parser**        | chumsky       |
| **Date/Time**     | jiff          |

## Design Principles

- **Modularity**: Clear separation between layers
- **Async/Await**: Full async support throughout
- **Type Safety**: Leverage Rust's type system
- **RFC Compliance**: Adherence to iCalendar RFC 5545, CalDAV RFC 4791, and WebDAV RFC 4918

## Additional Resources

- **just**: [Just command runner](https://github.com/casey/just)
- **Nix**: [Nix package manager](https://nixos.org/)
