# LocalDB Module

The `localdb` module provides a SQLite database wrapper using SQLx with a migration system. It
manages local storage of calendar events, todos, and short ID mappings

## Module Overview

## Database Schema

The database maintains four tables created through migrations:

### 1. events Table

```sql
CREATE TABLE IF NOT EXISTS events (
    uid         TEXT PRIMARY KEY,     -- Unique event identifier
    summary     TEXT NOT NULL,        -- Event title/summary
    description TEXT NOT NULL,        -- Event description
    status      TEXT NOT NULL,        -- Event status (confirmed, tentative, etc.)
    start       TEXT NOT NULL,        -- Start datetime (ISO 8601)
    end         TEXT NOT NULL         -- End datetime (ISO 8601)
    calendar_id TEXT NOT NULL,      -- Owning calendar identifier
    backend_kind TINYINT NOT NULL DEFAULT 0,  -- Backend type (0=local, 1=caldav)
);
```

### 2. todos Table

```sql
CREATE TABLE IF NOT EXISTS todos (
    uid         TEXT PRIMARY KEY,     -- Unique todo identifier
    summary     TEXT NOT NULL,        -- Todo title/summary
    description TEXT NOT NULL,        -- Todo description
    status      TEXT NOT NULL,        -- Todo status (needs-action, etc.)
    priority    INTEGER NOT NULL,     -- Priority level (0-9, 5 = medium)
    percent     INTEGER,              -- Percent complete (0-100, nullable)
    due         TEXT NOT NULL,        -- Due datetime (ISO 8601, empty if none)
    completed   TEXT NOT NULL         -- Completion datetime (ISO 8601, empty if none)
    calendar_id TEXT NOT NULL,      -- Owning calendar identifier
    backend_kind TINYINT NOT NULL DEFAULT 0,  -- Backend type (0=local, 1=caldav)
);
```

**Constraints**:

- `percent` is nullable (no value if not set)
- Empty strings represent `None` for optional fields (`due`, `completed`, `description`)

### 3. calendars Table

```sql
CREATE TABLE IF NOT EXISTS calendars (
    id        TEXT PRIMARY KEY,     -- Unique calendar ID (user-configured)
    name      TEXT NOT NULL,        -- Display name (e.g., "Personal", "Work")
    kind      TEXT NOT NULL,        -- Backend type (local, caldav)
    priority  INTEGER NOT NULL DEFAULT 0,  -- Conflict resolution order
    enabled   INTEGER NOT NULL DEFAULT 1,  -- Enable/disable flag
    config    TEXT NOT NULL,        -- Calendar-specific config (JSON)
    created_at TEXT NOT NULL,     -- Creation timestamp
    updated_at TEXT NOT NULL,     -- Last update timestamp
);
```

### 4. resources Table

```sql
CREATE TABLE IF NOT EXISTS resources (
    uid          TEXT NOT NULL,      -- Event/todo UID
    calendar_id  TEXT NOT NULL,      -- Owning calendar identifier
    resource_id  TEXT NOT NULL,      -- Backend resource (file://path, /dav/href, etc.)
    metadata     TEXT,                -- Backend-specific metadata (JSON, e.g., etag for CalDAV)
    PRIMARY KEY (uid, calendar_id)
);
```

**Constraints**:

- One event/todo belongs to exactly one calendar (`calendar_id`)
- `resources` maps `(uid, calendar_id)` to backend resource
- No foreign key constraints (referential integrity enforced at application layer)
- `metadata` is nullable (not used for local backend)

## Migration History

1. `20250801070804_init_events_todos` - Initial schema with events and todos
2. `20250801095832_add_short_ids` - Added short_ids table with AUTOINCREMENT
3. `20250805075731_drop_autoincrement` - Removed AUTOINCREMENT (data-preserving migration)
4. `20260131235400_ics_optional` - Made ICS support optional:
   - Added `backend_kind` column to events/todos
   - Created unified `resources` table for multi-backend support
   - Removed `path` column from events/todos
   - Migrated existing paths to resources table

## Code Standards

### Design Pattern

- All fields are `NOT NULL`, unless an empty value has a special meaning
- All database operations are `async`
- **No foreign key constraints**: Referential integrity enforced at application layer
- **Calendar ownership via `calendar_id`**: Each event/todo belongs to exactly one calendar

### Test Coverage Requirements

**100% coverage is mandatory** for all public APIs and database operations:

1. **Unit Tests** - Every public function must have tests
2. **Database Operations** - All SQL queries must be tested (success and error paths)
3. **Edge Cases** - Empty results, null values, boundary conditions
4. **Integration Tests** - Cross-module operations (events + short_ids)

### Database Query Standards

1. **Use SQLx query macros** for compile-time SQL validation
2. **Parameterize all user input** to prevent SQL injection
3. **Use `?` placeholders** with `.bind()` for values
4. **Return descriptive errors** with context

```rust
// Good - parameterized query
sqlx::query("\
INSERT INTO events (uid, summary)
VALUES (?, ?);
")
    .bind(uid)
    .bind(summary)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to insert event {uid}: {e}"))?;

// Bad - string interpolation (SQL injection risk)
sqlx::query(&format!("INSERT INTO events VALUES ('{uid}', '{summary}')"))
    .execute(&pool)
    .await?;
```

### Error Handling

All database operations return `Result<T, E>` with descriptive error messages:

```rust
pub async fn insert(&self, record: EventRecord) -> Result<(), sqlx::Error> {
    sqlx::query("\
INSERT INTO events ...
")
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert event: {e}");
            e
        })?;
    Ok(())
}
```

## Adding New Features

1. **Write tests first** - Define the expected behavior
2. **Implement the feature** - Make tests pass
3. **Add documentation** - Document public APIs
4. **Run full test suite** - Ensure no regressions
5. **Check coverage** - Verify 100% coverage maintained

## Migration Guidelines

When modifying the schema:

1. **Create new migration** - Use `just migrate-add <name>`
2. **Write up/down SQL** - Both must work correctly
3. **Add migration tests** - Test schema changes and data preservation
4. **Update this document** - Document schema changes
5. **Test migration** - Verify up/down migrations work
6. **Avoid foreign key constraints** - Use application-layer referential integrity only
