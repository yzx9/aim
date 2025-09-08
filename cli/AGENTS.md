# CLI Module

The CLI module provides a command-line interface for the AIM calendar application using the clap crate. It wraps the core functionality with intuitive commands and supports both traditional CLI operations and interactive TUI (Text User Interface) modes.

## Folder Structure

```
cli/src/
├── cli.rs              # Main CLI entry point and command routing
├── cmd_dashboard.rs    # Dashboard command for overview display
├── cmd_event.rs        # Event management commands (new, edit, list)
├── cmd_generate_completion.rs  # Shell completion generation
├── cmd_todo.rs         # Todo management commands (new, edit, done, etc.)
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

The CLI supports a hierarchical command structure:

```
aim [OPTIONS] [COMMAND] [SUBCOMMAND]
├── dashboard                        # Show overview (default)
├── new                              # Interactive creation
├── edit <ID>                        # Interactive editing
├── event
│   ├── new/add [SUMMARY]            # Create event
│   ├── edit <ID>                    # Edit event
│   └── list                         # List events
└── todo
    ├── new/add [SUMMARY]            # Create todo
    ├── edit <ID>                    # Edit todo
    ├── done <ID>...                 # Mark done
    ├── undo <ID>...                 # Mark needs-action
    ├── cancel <ID>...               # Mark cancelled
    ├── delay <ID> <TIMEDELTA>       # Delay due date
    └── list                         # List todos
```

### Command Implementations

The CLI supports a hierarchical command structure:

```
aim [OPTIONS] [COMMAND] [SUBCOMMAND]
├── dashboard                        # Show overview (default)
├── new                              # Interactive creation
├── edit <ID>                        # Interactive editing
├── event
│   ├── new/add [SUMMARY]            # Create event
│   ├── edit <ID>                    # Edit event
│   └── list                         # List events
└── todo
    ├── new/add [SUMMARY]            # Create todo
    ├── edit <ID>                    # Edit todo
    ├── done <ID>...                 # Mark done
    ├── undo <ID>...                 # Mark needs-action
    ├── cancel <ID>...               # Mark cancelled
    ├── delay <ID> <TIMEDELTA>       # Delay due date
    └── list                         # List todos
```

Individual command modules that handle specific functionality:

#### Event Commands (src/cmd_event.rs)

- `event new/add` - Create new calendar event with optional TUI mode
- `event edit` - Modify existing event with optional TUI mode
- `event list` - Display upcoming events with filtering and formatting options

#### Todo Commands (src/cmd_todo.rs)

- `todo new/add` - Create new todo item with optional TUI mode
- `todo edit` - Modify existing todo item with optional TUI mode
- `todo done` - Mark todos as completed
- `todo undo` - Mark todos as needs-action
- `todo cancel` - Mark todos as cancelled
- `todo delay` - Postpone todo due dates
- `todo list` - Display todo lists with sorting and filtering

#### TUI Commands (src/cmd_tui.rs)

- `new` - Create new event or todo with optional TUI mode
- `edit` - Modify existing of existing items with optional TUI mode

#### Utility Commands

- `dashboard` - Overview display of upcoming events and todos
- `generate-completion` - Shell completion script generation

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
