# Core Module

The core module is the foundation of the AIM calendar application, providing all the essential functionality for managing events and todos. It handles calendar operations, data persistence, configuration management, and integration with the iCalendar standard.

## Folder Structure

```
core/src/
├── lib.rs              # Library exports and module declarations
├── aim.rs              # Main application interface (Aim struct)
├── config.rs           # Configuration management (Config struct)
├── datetime.rs         # DateTime module declaration
│   ├── anchor.rs       # DateTimeAnchor - date parsing
│   ├── loose.rs        # LooseDateTime - flexible datetime
│   └── util.rs         # DateTime utilities
├── event.rs            # Event trait, EventDraft, EventPatch, EventStatus
├── todo.rs             # Todo trait, TodoDraft, TodoPatch, TodoStatus
├── short_id.rs         # Short ID generation and mapping
├── types.rs            # Common types (Id, Kind, Pager, Priority, SortOrder)
├── io.rs               # File I/O operations
└── localdb.rs          # LocalDB module declaration
    ├── db.rs           # LocalDb struct, connection pooling
    ├── events.rs       # Event storage operations
    ├── todos.rs        # Todo storage operations
    ├── short_ids.rs    # Short ID mapping
    └── migrations/     # Database migrations
```

## Main Components

### Aim (src/aim.rs)

The central application interface that coordinates all calendar operations:

- Manages the application lifecycle and configuration
- Handles database initialization and connection
- Provides CRUD operations for events and todos
- Integrates with the file system for .ics file management
- Manages short ID assignment and lookup
- Implements calendar synchronization from disk

### Config (src/config.rs)

Handles application configuration:

- Parses TOML configuration files
- Manages calendar paths and state directories
- Handles default values for todos (due dates, priority)
- Supports path expansion with environment variables
- Validates and normalizes configuration settings

### DateTime Handling (src/datetime.rs)

Comprehensive date/time management:

- `LooseDateTime` enum for flexible date/time representation
- Handles different iCalendar date formats (date-only, floating, timezone-aware)
- Provides range positioning and comparison operations
- Implements relative date calculations (today, tomorrow, etc.)
- Supports stable serialization for database storage

### Event System (src/event.rs)

Manages calendar events and related operations:

- `Event` trait for event abstraction
- `EventDraft` for creating new events
- `EventPatch` for partial event updates
- `EventStatus` enum with standard iCalendar values
- `EventConditions` for filtering events

### Todo System (src/todo.rs)

Manages todo items and related operations:

- `Todo` trait for todo abstraction
- `TodoDraft` for creating new todos
- `TodoPatch` for partial todo updates
- `TodoStatus` enum with standard iCalendar values
- `TodoConditions` and sorting options for querying

### Short ID Management (src/short_id.rs)

Handles compact numeric identifiers:

- Maps UUIDs to small numeric IDs for user convenience
- Wraps entities with their short IDs
- Provides `ID` resolution from mixed input formats

### Common Types (src/types.rs)

Shared data structures used throughout the application:

- `Id` enum for flexible ID handling (UUID or short ID)
- `Kind` enum for entity classification
- `SortOrder` for query result ordering
- `Pager` for pagination support
- `Priority` enum with 1-9 scale and named levels
- Integration with `serde` for serialization/deserialization

## Dependencies

- **aimcal-ical** - iCalendar (RFC 5545) format parsing and formatting
- **jiff** - Date/time handling
- **sqlx** - Database operations with compile-time SQL validation
- **serde / serde_json** - Serialization
- **uuid** - Unique identifiers
- **tokio** - Async runtime
- **regex** - Pattern matching
- **bimap** - Bidirectional maps
- **dirs / xdg** - Platform-specific directories

**Features:**

- `sqlite` (default) - Bundled SQLite
- `sqlite-unbundled` - System SQLite

## Code Standards

- Full async/await support with Tokio runtime
- Comprehensive error handling with descriptive messages
- Extensive tracing instrumentation for debugging
- Strict adherence to iCalendar RFC 5545 standards
- Type-safe operations with compile-time validation
- Configuration-driven behavior with sensible defaults
- Comprehensive test coverage for critical components
- Clean separation of concerns between modules
- Always write code and comments in English

## Common Development Tasks

```
# Add a new database migration
just migrate-add <name>
```
