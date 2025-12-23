# CLI Module

The CLI module provides a command-line interface for the AIM calendar application using the clap crate. It wraps the core functionality with intuitive commands and supports both traditional CLI operations and interactive TUI (Text User Interface) modes.

## Folder Structure

```
cli/src/
├── arg.rs              # CLI arguments utils
├── cli.rs              # Main CLI entry point and command routing
├── cmd_event.rs        # Event management commands (new, edit, list)
├── cmd_generate_completion.rs  # Shell completion generation
├── cmd_todo.rs         # Todo management commands (new, edit, done, etc.)
├── cmd_toplevel.rs     # Toplevel command (dashboard, delay, reschedule, flush)
├── cmd_tui.rs          # TUI mode commands (new, edit)
├── config.rs           # CLI-specific configuration handling
├── event_formatter.rs  # Event display formatting for different output modes
├── lib.rs              # Library exports and module declarations
├── main.rs             # Application entry point
├── table.rs            # Table formatting utilities for CLI output
├── todo_formatter.rs   # Todo display formatting for different output modes
├── tui/                # Text User Interface components
└── util.rs             # Shared utilities for CLI operations
```

## Main Components

### CLI (src/cli.rs)

The central command router that:

- Defines the complete command structure using clap
- Parses command-line arguments
- Routes commands to appropriate handlers
- Manages application lifecycle (configuration, initialization, cleanup)
- Supports aliases for common commands (e.g., "add" for "new")

## Command Structure

The CLI provides a hierarchical command structure:

```
aim [OPTIONS] [COMMAND] [SUBCOMMAND]
```

### Main Commands

- **`dashboard`** *(default)*: Show upcoming events and todos
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

Many commands support both direct CLI mode and interactive TUI mode. TUI mode activates automatically when not all required fields are provided.

### Formatting and Display

Modules that handle presentation of data:

#### EventFormatter (src/event_formatter.rs) and TodoFormatter (src/todo_formatter.rs):

- Formats events/todos for display in table or JSON format
- Color-coding for current/upcoming events and overdue and high-priority items
- Flexible column-based output

#### Table Utilities (src/table.rs):

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

## Dependencies

- **chrono**: Date and time handling
- **clap**: Command line argument parsing
- **tokio**: Async runtime
- **serde**: Serialization framework
- **uuid**: Unique identifier generation
- **tracing**: Logging and instrumentation
- **dirs**: Platform-specific directory handling

## Features

- **Dual Mode Operation**: Traditional CLI commands and interactive TUI modes
- **Flexible Input Parsing**: Multiple date/time formats and natural language parsing
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
