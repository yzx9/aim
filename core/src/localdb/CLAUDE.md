# LocalDB Module

The `localdb` module provides a SQLite database wrapper using SQLx with a migration system. It
manages local storage of calendar events, todos, and short ID mappings

## Module Overview

## Database Schema

The database maintains three tables created through migrations:

### 1. events Table

```sql
CREATE TABLE IF NOT EXISTS events (
    uid         TEXT PRIMARY KEY,     -- Unique event identifier
    path        TEXT NOT NULL,        -- Source .ics file path
    summary     TEXT NOT NULL,        -- Event title/summary
    description TEXT NOT NULL,        -- Event description
    status      TEXT NOT NULL,        -- Event status (confirmed, tentative, etc.)
    start       TEXT NOT NULL,        -- Start datetime (ISO 8601)
    end         TEXT NOT NULL         -- End datetime (ISO 8601)
);
```

### 2. todos Table

```sql
CREATE TABLE IF NOT EXISTS todos (
    uid         TEXT PRIMARY KEY,     -- Unique todo identifier
    path        TEXT NOT NULL,        -- Source .ics file path
    summary     TEXT NOT NULL,        -- Todo title/summary
    description TEXT NOT NULL,        -- Todo description
    status      TEXT NOT NULL,        -- Todo status (needs-action, etc.)
    priority    INTEGER NOT NULL,     -- Priority level (0-9, 5 = medium)
    percent     INTEGER,              -- Percent complete (0-100, nullable)
    due         TEXT NOT NULL,        -- Due datetime (ISO 8601, empty if none)
    completed   TEXT NOT NULL         -- Completion datetime (ISO 8601, empty if none)
);
```

**Constraints**:

- `percent` is nullable (no value if not set)
- Empty strings represent `None` for optional fields (`due`, `completed`, `description`)

### 3. short_ids Table

```sql
CREATE TABLE short_ids (
    short_id INTEGER PRIMARY KEY,     -- Compact numeric ID (ROWID-based)
    uid      TEXT UNIQUE NOT NULL,    -- UUID being mapped
    kind     TEXT NOT NULL            -- Entity type ("event" or "todo")
);
```

**Constraints**:

- `short_id` uses ROWID (no AUTOINCREMENT) for compact IDs
- `uid` must be unique (one short_id per UUID)
- `kind` is either "event" or "todo"

## Migration History

1. `20250801070804_init_events_todos` - Initial schema with events and todos
2. `20250801095832_add_short_ids` - Added short_ids table with AUTOINCREMENT
3. `20250805075731_drop_autoincrement` - Removed AUTOINCREMENT (data-preserving migration)

## Code Standards

### Design Pattern

- All fields are `NOT NULL`, unless an empty value has a special meaning
- All database operations are `async`

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
