# Core Module

The core module is the foundation of the AIM calendar application, providing all the essential functionality for managing events and todos. It handles calendar operations, data persistence, configuration management, and integration with the iCalendar standard.

## Main Components

### Aim (src/aim.rs)

The central application interface that coordinates all calendar operations:

- Manages the application lifecycle and configuration
- Handles database initialization and connection
- Provides CRUD operations for events and todos
- Integrates with the file system for .ics file management
- Manages short ID assignment and lookup
- Implements calendar synchronization from disk

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

## Code Standards

- Extensive tracing instrumentation for debugging
- Strict adherence to iCalendar RFC 5545 standards

## Common Development Tasks

```
# Add a new database migration
just migrate-add <name>
```
