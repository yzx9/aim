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
├── .github/            # GitHub configurations, including CI/CD workflows and dependabot
├── aimcal/             # Public API facade
├── core/               # Core business logic
├── cli/                # Command-line interface
├── ical/               # iCalendar parser
├── docs/               # Documentation
│   ├── architecture.md # System architecture
│   └── testing.md      # Testing guide
├── Cargo.toml          # Workspace configuration
├── CHANGELOG.md        # Version history
├── CLAUDE.md           # Project overview
├── CONTRIBUTING.md     # Contribution guidelines
├── flake.nix           # Nix flake
├── justfile            # Command runner recipes
├── LICENSE             # Apache-2.0 license
└── README.md           # User-facing documentation
```

### Crate Responsibilities

**aimcal** - Minimal facade that re-exports the CLI

**core** - Foundation providing:

- Event and todo management
- SQLite database operations
- DateTime handling with timezone support
- Configuration management
- Short ID mapping

**cli** - User interface providing:

- Command parsing with clap
- Interactive TUI with ratatui
- Table and JSON output formatting
- Shell completion generation

**ical** - iCalendar (RFC 5545) parser with:

- Four-phase parsing pipeline (lexer → syntax → typed → semantic)
- Three-pass typed analysis (parameter → value → property)
- Type-safe representations with generic `StringStorage` trait for flexible string handling
- Zero-copy parsing support with `SpannedSegments<'src>` for borrowed data
- Comprehensive error reporting

## Technology Stack

| Component         | Technology                |
| ----------------- | ------------------------- |
| **Language**      | Rust 2024                 |
| **Async Runtime** | Tokio                     |
| **Database**      | SQLite + sqlx             |
| **CLI**           | clap                      |
| **TUI**           | ratatui                   |
| **Lexer**         | logos                     |
| **Parser**        | chumsky                   |
| **Date/Time**     | jiff / chrono + chrono-tz |

## Design Principles

- **Modularity**: Clear separation between layers
- **Async/Await**: Full async support throughout
- **Type Safety**: Leverage Rust's type system
- **RFC Compliance**: Adherence to iCalendar RFC 5545
- **No mod.rs**: Module declarations in parent modules, e.g. `src/typed.rs`
  instead of `src/typed/mod.rs`

## Additional Resources

- **RFC 5545**: [iCalendar specification](https://tools.ietf.org/html/rfc5545)
- **Gitmoji**: [Gitmoji commit standard](https://gitmoji.dev/)
- **just**: [Just command runner](https://github.com/casey/just)
- **Nix**: [Nix package manager](https://nixos.org/)
