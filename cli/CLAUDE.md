# CLI Module

The CLI module provides a command-line interface for the AIM calendar
application using the clap crate. It wraps the core functionality with
intuitive commands and supports both traditional CLI operations and interactive
TUI (Text User Interface) modes.

## Folder Structure

```
cli/src/
├── lib.rs              # Library exports
├── main.rs             # Binary entry point
├── cli.rs              # Main CLI entry point
├── config.rs           # CLI-specific configuration
├── arg.rs              # CLI argument utilities
├── prompt.rs           # Interactive prompts
├── util.rs             # Shared utilities
├── table.rs            # Table formatting
├── event_formatter.rs  # Event display formatting
├── todo_formatter.rs   # Todo display formatting
├── cmd_toplevel.rs     # Dashboard, delay, reschedule, flush commands
├── cmd_event.rs        # Event commands
├── cmd_todo.rs         # Todo commands
├── cmd_tui.rs          # TUI mode commands
├── cmd_generate_completion.rs  # Shell completion generation
└── tui/                # TUI components
    ├── app.rs                  # TUI application orchestration
    ├── component.rs            # Base TUI component
    ├── component_form.rs       # Form component
    ├── component_form_util.rs  # Form utilities
    ├── component_page.rs       # Page component
    ├── dispatcher.rs           # Event dispatcher
    ├── event_editor.rs         # Event editing UI
    ├── event_store.rs          # Event state management
    ├── event_todo_editor.rs    # Unified event/todo editor
    ├── todo_editor.rs          # Todo editing UI
    └── todo_store.rs           # Todo state management
```

## Main Components

- **CLI (src/cli.rs)**: The central command router.

The CLI provides a hierarchical command structure:

```
aim [OPTIONS] [COMMAND] [SUBCOMMAND]
```

### Main Commands

- **`dashboard`** _(default)_: Show upcoming events and todos
- **`new`**/**`add`**: Create event/todo with optional TUI mode
- **`edit`**: Modify event/todo with optional TUI mode
- **`delay`**: Delay events/todos based on original times
- **`reschedule`**: Reschedule events/todos based on current time
- **`flush`**: Clear all short ID mappings

### Event Management (`event` or `e`)

- **`event new/add`**: Create event (start/end times, description, status, summary)
- **`event edit`**: Modify existing event
- **`event delay`**: Postpone event based on original start time
- **`event reschedule`**: Reschedule event based on current time
- **`event list`**: Display events with filtering options

### Todo Management (`todo` or `t`)

- **`todo new/add`**: Create todo (due date, description, priority, status, summary)
- **`todo edit`**: Modify existing todo
- **`todo done`**: Mark todos as completed
- **`todo undo`**: Mark todos as needs-action
- **`todo cancel`**: Mark todos as cancelled
- **`todo delay`**: Postpone todo due dates based on original due dates
- **`todo reschedule`**: Reschedule todos based on current time
- **`todo list`**: Display todos with sorting/filtering options

### Global Options

- **`-c`**/**`--config`**: Configuration file path
- **`--output-format`**: `table` (default) or `json`
- **`--verbose`**: Additional output details

### TUI Mode

Many commands support both direct CLI mode and interactive TUI mode. TUI mode
activates automatically when not all required fields are provided.

### Formatting and Display

Modules that handle presentation of data:

EventFormatter (src/event_formatter.rs) and TodoFormatter (src/todo_formatter.rs):

- Formats events/todos for display in table or JSON format
- Color-coding for current/upcoming events and overdue and high-priority items
- Flexible column-based output

Table Utilities (src/table.rs):

- Generic table formatting engine
- Support for both basic text tables and JSON output
- Column alignment and padding management

### Text User Interface (src/tui/)

Interactive mode components:

- Form-based data entry for events and todos
- In-place editing capabilities
- State management for TUI applications

### Utilities (src/util.rs)

Shared helper functions:

- Date/time parsing and formatting
- Unicode string width calculation
- Grapheme cluster handling
- Mathematical utilities

### Configuration (src/config.rs)

CLI-specific configuration management:

- Configuration file parsing
- Integration with core configuration
- Environment variable handling

## Key Design Decisions

### Why SQLite?

Embedded database ideal for personal calendar applications:

- Zero-configuration
- Excellent Rust support (sqlx with compile-time validation)
- Reliable and cross-platform
- Sufficient for single-user workloads

### Why Short IDs?

UUIDs are cumbersome for CLI usage. We map UUIDs to compact numeric IDs:

```bash
# Instead of:
aim event edit 550e8400-e29b-41d4-a716-446655440000

# Use:
aim event edit 1
```

Bidirectional mapping is stored in the database for persistence.

### Why Both CLI and TUI?

**CLI Mode** - Fast for experienced users:

```bash
aim event new "Meeting" --start "tomorrow 10am" --duration "1h"
```

**TUI Mode** - Friendly for complex inputs:

```bash
aim event new  # → Interactive form with all fields
```

## Dependencies

- **clap** - CLI argument parsing
- **ratatui** - TUI framework
- **cliclack** - Interactive prompts
- **colored** - Terminal colors
- **tokio** - Async runtime
- **futures** - Async utilities
- **unicode-segmentation** - Grapheme handling
- **aimcal-core** - Core functionality

**Features:**

- `sqlite` (default) - Bundled SQLite
- `sqlite-unbundled` - System SQLite

## Extension Points

### Adding New Event Properties

1. Update `VEvent` in `ical/src/semantic/vevent.rs`
2. Add property spec in `ical/src/typed/property_spec.rs`
3. Update `Event` trait in `core/src/event.rs`
4. Add database migration
5. Update CLI formatters

### Adding New Commands

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

- Async/await support throughout with proper error handling
- Extensive tracing instrumentation for debugging
- Unit tests for command parsing and utility functions
- Clean separation between CLI presentation and core logic
- Consistent error handling with user-friendly messages
- Colorized output for enhanced user experience
- Configuration-driven behavior with sensible defaults
- Always write code and comments in English
