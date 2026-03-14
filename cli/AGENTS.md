# CLI Module

The CLI module provides a command-line interface for the AIM calendar
application using the clap crate. It wraps the core functionality with
intuitive commands and supports both traditional CLI operations and interactive
TUI (Text User Interface) modes.

## Main Components

- **CLI (src/cli.rs)**: The central command router.

### Main Commands

- **`dashboard`** _(default)_: Show upcoming events and todos
- **`new`**/**`add`**: Create event/todo with optional TUI mode
- **`edit`**: Modify event/todo with optional TUI mode
- **`delay`**: Delay events/todos based on original times
- **`reschedule`**: Reschedule events/todos based on current time
- **`flush`**: Clear all short ID mappings
- **`event SUBCMD`**: Event Management
- **`todo SUBCMD`**: Todo Management

### TUI Mode

Many commands support both direct CLI mode and interactive TUI mode. TUI mode
activates automatically when not all required fields are provided.

### Text User Interface (src/tui/)

Interactive mode components:

- Form-based data entry for events and todos
- In-place editing capabilities
- State management for TUI applications

## Key design decisions

### Why SQLite?

Embedded database ideal for personal calendar applications:

- Zero-configuration
- Excellent Rust support (sqlx with compile-time validation)
- Reliable and cross-platform
- Sufficient for single-user workloads

### Why short IDs?

UUIDs are cumbersome for CLI usage. We map UUIDs to compact numeric IDs:

```bash
# Instead of:
aim event edit LONG_LONG_UID

# Use:
aim event edit 1
```

Bidirectional mapping is stored in the database for persistence.

### Why both CLI and TUI?

CLI Mode - Fast for experienced users:

```bash
aim event new "Meeting" --start "tomorrow 10am" --duration "1h"
```

TUI Mode - Friendly for complex inputs:

```bash
aim event new  # → Interactive form with all fields
```

## Extension points

### Adding new event properties

1. Update `VEvent` in `ical/src/semantic/vevent.rs`
2. Add property spec in `ical/src/typed/property_spec.rs`
3. Update `Event` trait in `core/src/event.rs`
4. Add database migration
5. Update CLI formatters

### Adding new commands

1. Add command definition in `cli/src/cli.rs`
2. Create handler function
3. Update completion generation
4. Add tests

## Features

- **Dual Mode Operation**: Traditional CLI commands and interactive TUI modes
- **Flexible Input Parsing**: Multiple date/time formats and natural language
  parsing
- **Rich Output Formatting**: Table and JSON output modes with color coding
- **Alias Support**: Shorter command names for frequently used operations
- **Shell Completion**: Auto-completion script generation for popular shells
- **Unicode Support**: Proper handling of multi-byte characters and emojis
- **Environment Integration**: Configuration via environment variables and files
- **Error Handling**: User-friendly error messages with colored output
- **Sorting and Filtering**: Configurable display options for events and todos

## Code Standards

- Clean separation between CLI presentation and core logic
- Colorized output for enhanced user experience
- Configuration-driven behavior with sensible defaults
