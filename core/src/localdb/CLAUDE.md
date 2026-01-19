# LocalDB Module

The `localdb` module provides a SQLite database wrapper using SQLx for the AIM project. It manages local storage of calendar events and todos with a migration system.

## Folder Structure

```
localdb/
├── events.rs       # Event storage and retrieval operations
├── short_ids.rs    # Short ID generation and mapping
├── todos.rs        # Todo storage and retrieval operations
└── migrations/     # Database schema migrations
```

## Main Components

### LocalDb (localdb.rs)

The main database interface that:

- Handles SQLite connection pooling
- Manages database migrations using sqlx-migration
- Provides high-level operations for upserting events and todos
- Exposes specialized storage modules as public fields

### Events (events.rs)

Manages storage and retrieval of calendar events with:

- Insert/upsert operations with conflict resolution
- Query operations with filtering and pagination
- Record structure that implements the Event trait

### Todos (todos.rs)

Manages storage and retrieval of todo items with:

- Upsert operations with conflict resolution
- Query operations with filtering, sorting, and pagination
- Record structure that implements the Todo trait

### ShortIds (short_ids.rs)

Handles short ID generation and mapping:

- Maps UUIDs to compact numeric IDs for user convenience
- Tracks entity types (todos vs events)
- Uses SQLite's ROWID mechanism for compact ID generation

## Database Schema

The module maintains three tables:

1. **events** - Stores calendar event data
2. **todos** - Stores todo item data
3. **short_ids** - Maps UUIDs to compact numeric IDs

## Code Standards

- Full support for async/await operations
- Comprehensive error handling with descriptive messages
- Tracing instrumentation for debugging
- SQLx for compile-time SQL validation
- Migration-based schema management
- Implements project traits (Event, Todo) for interoperability
