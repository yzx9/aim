# AIM Project Overview

AIM (Analyze. Interact. Manage) is a Rust-based calendar and task management
application that provides both a core library and a command-line interface. The
project follows a modular architecture with a clear separation between the core
functionality and the CLI presentation layer.

## Project map

@docs/architecture.md
@docs/testing.md
@docs/styling.md

## Development Commands

```bash
# Build the project
cargo build

# Run tests
just test

# Format code
cargo fmt

# Run linter
just lint

# See all available commands
just
```

## Compact Instructions

When compressing, preserve in priority order:

1. Architecture decisions (NEVER summarize)
2. Modified files and their key changes
3. Current verification status (pass/fail)
4. Open TODOs and rollback notes
5. Tool outputs (can delete, keep pass/fail only)
