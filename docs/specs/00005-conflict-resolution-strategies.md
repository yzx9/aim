# Conflict Resolution Strategies for Local ICS Backend

## Overview

Define flexible and configurable conflict resolution strategies for local ICS backend synchronization. This document focuses on Local ICS backend specifically, with strategies that can later be extended to other backends (CalDAV, jCal).

**Key Design Decisions**:

- Strategies are backend-specific (each backend type has its own defaults and options)
- Conflict resolution is configurable per-backend in config file
- Detailed logging of all conflicts and resolutions for audit trail
- Merge strategy is future enhancement, not in current scope
- Delete handling is user-configurable with sensible defaults

## Conflict Scenarios for Local ICS

### Scenario 1: Database Modified, ICS File Unchanged

```
┌─────────────────┐         ┌─────────────────┐
│   LocalDB       │         │   ICS File      │
│  (SQLite)       │         │  ~/calendar/    │
├─────────────────┤         ├─────────────────┤
│ UID: abc123     │         │ UID: abc123     │
│ Summary: "New"  │◄───────►│ Summary: "Old"  │
│ (CLI edited)    │  sync   │ (external edit) │
└─────────────────┘         └─────────────────┘
```

User modifies event via CLI (`aim event edit`), ICS file has old data.

### Scenario 2: ICS File Modified, Database Unchanged

```
┌─────────────────┐         ┌─────────────────┐
│   LocalDB       │         │   ICS File      │
│  (SQLite)       │         │  ~/calendar/    │
├─────────────────┤         ├─────────────────┤
│ UID: abc123     │◄───────►│ UID: abc123     │
│ Summary: "Old"  │  sync   │ Summary: "New"  │
│ (stale)         │         │ (external edit) │
└─────────────────┘         └─────────────────┘
```

External calendar app modifies ICS file, database has stale data.

### Scenario 3: Both Modified (Same UID, Different Data)

```
┌─────────────────┐         ┌─────────────────┐
│   LocalDB       │         │   ICS File      │
│  (SQLite)       │         │  ~/calendar/    │
├─────────────────┤         ├─────────────────┤
│ UID: abc123     │◄───────►│ UID: abc123     │
│ Summary: "DB"   │  sync   │ Summary: "ICS"  │
│ Updated: T1     │         │ Updated: T2     │
└─────────────────┘         └─────────────────┘
```

Both sides modified independently, need conflict resolution.

### Scenario 4: Deleted on One Side

```
┌─────────────────┐         ┌─────────────────┐
│   LocalDB       │         │   ICS File      │
│  (SQLite)       │         │  ~/calendar/    │
├─────────────────┤         ├─────────────────┤
│ UID: abc123     │◄───────►│ NOT FOUND       │
│ Summary: "A"    │  sync   │ (deleted externally)│
└─────────────────┘         └─────────────────┘
```

External app deleted event from ICS file, still exists in database.

### Scenario 5: Field-Level Conflicts

```
┌─────────────────┐         ┌─────────────────┐
│   LocalDB       │         │   ICS File      │
│  (SQLite)       │         │  ~/calendar/    │
├─────────────────┤         ├─────────────────┤
│ UID: abc123     │◄──────►│ UID: abc123    │
│ Summary: "A"    │  sync   │ Summary: "B"    │  ← Conflict
│ Desc: "same"    │         │ Desc: "same"    │  ← Same
│ Start: 10am     │         │ Start: 10am     │  ← Same
│ End: 11am       │         │ End: 11am       │  ← Same
└─────────────────┘         └─────────────────┘
```

Same UID exists on both sides, but only some fields differ. For example:

- User edited summary via CLI ("A")
- External app edited summary in ICS file ("B")
- Other fields (description, start/end) are identical

**Behavior with current strategies**:

- `ics_wins`: All fields from ICS (summary="B", overwrites description/start/end)
- `db_wins`: All fields from database (summary="A", keeps description/start/end)
- Both strategies provide no granular control over which fields to preserve

**Need**: Field-level merge strategy (future enhancement) to handle partial conflicts intelligently.

## Resolution Strategies for Local ICS

### Strategy 1: ICS Wins (Default)

**Principle**: ICS file is source of truth, always overwrite database.

**Behavior**:

```rust
if ics_file.exists(uid):
    parse ICS
    upsert to database (overwrite any local changes)
    record conflict: if local_modified, log as "local_overwritten"
```

**Pros**:

- Simple, predictable behavior
- ICS files are primary storage
- Works well when external calendar app is main interface
- No timestamp tracking required

**Cons**:

- CLI modifications lost on sync
- Poor UX if user expects CLI changes to persist

**Default for**: Local ICS backend

### Strategy 2: Database Wins

**Principle**: Database (CLI) is source of truth, write changes to ICS files.

**Behavior**:

```rust
if database.has(uid):
    keep database version
    if ics_file.exists(uid):
        write ICS from database (overwrite external changes)
        record conflict: log as "ics_overwritten"
    else:
        create new ICS file from database
```

**Pros**:

- Protects CLI modifications
- CLI is primary interface
- Works for offline-first workflows

**Cons**:

- External app modifications lost
- Requires write access to calendar directory
- May conflict with other apps using same ICS files

**Default for**: None (opt-in configuration)

### Strategy 3: Last Modified Wins (Future)

**Principle**: Compare modification timestamps, keep most recent version.

**Note**: This strategy is a future enhancement, not in current scope.

**Future Requirements**:

- Track `last_modified` timestamp in database
- Use ICS `LAST-MODIFIED` or `DTSTAMP` properties
- Handle missing timestamps (treat as oldest)

**Pros**:

- Prevents data loss
- Intuitive for users
- Handles both CLI and external modifications

**Cons**:

- Requires timestamp tracking (database migration)
- File system timestamps vs ICS metadata
- More complex implementation

**Default for**: Future enhancement (not current scope)

### Strategy 4: Prompt User (Opt-in)

**Principle**: When conflict detected, ask user to choose resolution.

**Behavior** (CLI):

```
for each conflict:
    println!("Conflict detected for event: {}", event.summary())
    println!("  LocalDB: {}", local_data)
    println!("  ICS file: {}", ics_data)
    println!()
    println!("Choose resolution:")
    println!("  [1] Keep LocalDB (overwrite ICS file)")
    println!("  [2] Keep ICS file (overwrite LocalDB)")
    println!("  [3] Skip (resolve later)")

    match user_choice {
        1 => write_ics(local_data), record_conflict("user_kept_db")
        2 => update_database(ics_data), record_conflict("user_kept_ics")
        3 => record_conflict("user_skipped")
    }
```

**Pros**:

- User has full control
- Prevents accidental data loss
- Can review each conflict individually

**Cons**:

- Not suitable for automated sync (startup sync)
- Poor UX for large numbers of conflicts
- Requires interactive terminal session
- Can't use in scripts/cron

**Default for**: None (opt-in configuration)

## Delete Handling Options

When an item exists in database but not in ICS file (or vice versa):

### Option 1: Always Delete (ICS Wins for Deletions)

**Behavior**:

```rust
if database.has(uid) && !ics_file.exists(uid):
    delete from database
    record conflict: "item_deleted_in_ics, removed_from_db"
```

**Pros**:

- ICS file is source of truth
- Simple behavior
- Works well with external calendar app

**Cons**:

- Can lose data if external deletion was accidental
- No way to recover deleted items

### Option 2: Keep Database (DB Wins for Deletions)

**Behavior**:

```rust
if database.has(uid) && !ics_file.exists(uid):
    keep database record
    recreate ICS file from database
    record conflict: "item_missing_in_ics, restored_from_db"
```

**Pros**:

- Protects database records
- Recovers accidentally deleted items
- CLI is primary interface

**Cons**:

- May conflict with intentional deletions
- External app shows "ghost" items

### Option 3: Prompt User (Opt-in)

**Behavior**:

```
if database.has(uid) && !ics_file.exists(uid):
    println!("Event '{}' exists in database but not in ICS file", summary)
    println!("  [1] Keep (restore to ICS file)")
    println!("  [2] Delete (remove from database)")

    match user_choice:
        1 => recreate_ics(), record_conflict("user_kept_deleted")
        2 => delete_from_db(), record_conflict("user_confirmed_delete")
```

**Pros**:

- User decides on deletions
- Prevents accidental data loss
- Granular control

**Cons**:

- Interactive only (not for automated sync)
- Can be tedious with many deletions

**Default for**: Always Delete (ICS wins)

## Configuration Design

### Local ICS Backend Configuration

```toml
[backend.local]
calendar_path = "~/calendar"

[backend.local.sync]
# Enable/disable automatic sync
enabled = true

# Sync on startup?
on_startup = true

# Conflict resolution strategy
# Options: "ics_wins", "db_wins", "prompt"
strategy = "ics_wins"

# Delete handling
# Options: "always_delete", "keep_db", "prompt"
delete_handling = "always_delete"

# Enable detailed conflict logging
conflict_logging = true

# Log file path (optional, defaults to state_dir/conflicts.log)
conflict_log_file = "conflicts.log"
```

### Configuration Examples

**Example 1: Default Behavior (ICS is source of truth)**

```toml
[backend.local]
calendar_path = "~/calendar"

[backend.local.sync]
enabled = true
on_startup = true
strategy = "ics_wins"
delete_handling = "always_delete"
conflict_logging = true
```

- ICS files always win
- Deletions propagate to database
- All conflicts logged

**Example 2: CLI-First Workflow**

```toml
[backend.local]
calendar_path = "~/calendar"

[backend.local.sync]
enabled = true
on_startup = false  # manual sync only
strategy = "db_wins"
delete_handling = "keep_db"
conflict_logging = true
```

- CLI is primary interface
- Database modifications never lost
- External edits overwritten
- Deleted items restored to ICS

**Example 3: Interactive Mode**

```toml
[backend.local]
calendar_path = "~/calendar"

[backend.local.sync]
enabled = false  # manual only
on_startup = false
strategy = "prompt"
delete_handling = "prompt"
conflict_logging = true
```

- User resolves conflicts interactively
- Full control over all changes
- Not for automated workflows

### CLI Overrides

```bash
# Override config for one-off sync
$ aim sync --strategy db_wins

# Interactive conflict resolution
$ aim sync --strategy prompt

# Force ICS to win (ignore config)
$ aim sync --strategy ics_wins

# Use different delete handling
$ aim sync --delete-handling keep_db
```

## Conflict Logging

### Log Format

```log
2025-02-04T10:30:15Z CONFLICT uid=abc123 type=data_mismatch
  strategy=ics_wins resolution=ics_overwritten
  db_summary="Meeting with John" db_modified=2025-02-04T10:00:00Z
  ics_summary="Meeting with Jane" ics_modified=2025-02-04T10:15:00Z
  action=updated_database

2025-02-04T10:31:22Z CONFLICT uid=def456 type=deleted_in_ics
  strategy=ics_wins resolution=deleted_from_db
  db_summary="Old task" db_modified=2025-02-01T14:30:00Z
  action=deleted_from_database

2025-02-04T10:32:10Z CONFLICT uid=ghi789 type=user_resolution
  strategy=prompt resolution=user_kept_db
  db_summary="Important meeting" db_modified=2025-02-04T09:00:00Z
  ics_summary="Important meeting - UPDATED" ics_modified=2025-02-04T10:00:00Z
  action=updated_ics_file
```

### Log Levels

```toml
[backend.local.sync]
# Log detail level
# Options: "minimal", "normal", "verbose"
log_level = "normal"
```

- **minimal**: Only conflicts, no sync operations
- **normal**: Conflicts + sync operations (default)
- **verbose**: Everything including field-by-field diffs

### Log Rotation

```toml
[backend.local.sync]
# Max log file size before rotation (default: 10MB)
max_log_size = "10MB"

# Number of log files to keep (default: 5)
max_log_files = 5
```

## Integration with Current Architecture

### Database Schema Changes

```sql
-- Migration: 20250204_add_conflict_tracking
-- Track which backend last updated the record
ALTER TABLE events ADD COLUMN updated_by TEXT DEFAULT 'local';
ALTER TABLE todos ADD COLUMN updated_by TEXT DEFAULT 'local';

-- For delete handling soft delete
ALTER TABLE events ADD COLUMN deleted INTEGER DEFAULT 0;
ALTER TABLE todos ADD COLUMN deleted INTEGER DEFAULT 0;

-- Future: Track last modification timestamp (for last_modified_wins strategy)
-- ALTER TABLE events ADD COLUMN last_modified TEXT;
-- ALTER TABLE todos ADD COLUMN last_modified TEXT;
```

### Sync Flow with Conflict Resolution

```
┌────────────────────────────────────────────────────────┐
│                   sync()                               │
│  (called by 'aim sync' or startup if on_startup=true)  │
└────────────────────────────────────────────────────────┘
                  │
                  ▼
      ┌─────────────────────────┐
      │  Scan ICS directory     │
      │  Read all .ics files    │
      └─────────────────────────┘
                  │
                  ▼
      ┌─────────────────────────┐
      │  For each component     │
      │  - Parse ICS            │
      │  - Get UID              │
      └─────────────────────────┘
                  │
           ┌──────┴──────┐
           │             │
           ▼             ▼
    UID exists    UID new
    in DB?       in DB?
           │             │
           ▼             ▼
    ┌─────────────────┐  ┌─────────────────┐
    │ Conflict Check  │  │ Insert to DB    │
    │ - Data changed? │  │ - Create ICS    │
    │ - Delete?       │  │ - Log success   │
    └─────────────────┘  └─────────────────┘
           │
           ▼
    ┌─────────────────────────┐
    │ Apply Strategy          │
    │ - ics_wins?             │
    │ - db_wins?              │
    │ - prompt?               │
    └─────────────────────────┘
           │
           ▼
    ┌─────────────────────────┐
    │ Record Conflict Log     │
    │ - Timestamp             │
    │ - UID                   │
    │ - Type                  │
    │ - Resolution            │
    └─────────────────────────┘
```

## Migration Path

### Phase 1: Database Schema Migration

**Files**: `core/src/localdb/migrations/20250204_add_conflict_tracking.{up,down}.sql`

1. Add `updated_by`, `deleted` columns to events/todos
2. Set default values for existing records
3. Test migration with existing data

**Future**: Add `last_modified` columns when implementing last_modified_wins strategy

### Phase 2: Configuration Extension

**Files**: `core/src/config.rs`

1. Add `LocalSyncConfig` struct with strategy, delete_handling, logging options
2. Add `sync` section to `Local` backend config
3. Set sensible defaults (ics_wins, always_delete, logging enabled)

### Phase 3: Sync Command Implementation

**Files**: `core/src/aim.rs`, `cli/src/cmd_sync.rs`

1. Add `pub async fn sync(&self)` method to Aim
2. Add conflict detection logic
3. Implement strategy application (ics_wins, db_wins, prompt)
4. Add conflict logging

### Phase 4: Logging Infrastructure

**Files**: `core/src/conflict_logger.rs` (new)

1. Create structured conflict log format
2. Implement log rotation
3. Add support for different log levels
4. Integrate with tracing

### Phase 5: Testing

**Files**: `core/src/conflict_test.rs`, `cli/src/sync_test.rs`

1. Unit tests for each strategy
2. Integration tests with test ICS directory
3. Test log format and rotation
4. Test CLI overrides

## Risk & Mitigations

| Risk                             | Mitigation                                                         |
| -------------------------------- | ------------------------------------------------------------------ |
| Data loss on sync                | Default strategy (ics_wins) + detailed logging for recovery        |
| Poor UX with default strategy    | Clear documentation, opt-in alternatives (db_wins, prompt)         |
| Log file grows too large         | Automatic rotation, configurable size limits                       |
| Performance overhead             | Logging is async, minimal impact on sync speed                     |
| User confusion on conflict types | Clear log messages with type and resolution                        |
| Breaking changes                 | Default behavior unchanged (ics_wins), opt-in for other strategies |
| Accidental deletions propagated  | Delete handling is configurable (keep_db, prompt)                  |
| File permission issues           | Check write access before sync, clear error messages               |
| Concurrent syncs                 | File locking on ICS files, database transactions                   |

## Decision Matrix for Users

| Workflow                          | Recommended Strategy    | Delete Handling              | Rationale                 |
| --------------------------------- | ----------------------- | ---------------------------- | ------------------------- |
| ICS is primary, CLI is occasional | `ics_wins`              | `always_delete`              | Simple, predictable       |
| CLI is primary, ICS is export     | `db_wins`               | `keep_db`                    | Protect CLI work          |
| Mix of both tools                 | `ics_wins` or `db_wins` | Depends on primary           | Choose based on main tool |
| Careful with deletions            | Any                     | `prompt`                     | User oversight            |
| Offline-first                     | `db_wins`               | `keep_db`                    | Local changes never lost  |
| Automated sync (cron)             | `ics_wins` or `db_wins` | `always_delete` or `keep_db` | No interactive prompts    |
| Testing/development               | `ics_wins`              | `always_delete`              | Predictable behavior      |
| Production with important data    | `prompt`                | `prompt`                     | User confirms changes     |

## Future Enhancements

Beyond current scope (local ICS only):

- **Last modified wins** strategy with timestamp tracking
- **Field-level merge strategy**:
  - Per-field conflict resolution (ics_wins vs db_wins per field)
  - Example config:
    ```toml
    [backend.local.sync.merge]
    summary = "ics_wins"     # or "db_wins"
    description = "db_wins"
    start = "ics_wins"
    end = "ics_wins"
    status = "ics_wins"
    ```
  - No timestamp comparison needed
  - Simple binary choice per field
- **Interactive TUI conflict resolver**: Visual diff comparison
- **Three-way merge**: Use common ancestor for better decisions
- **Conflict history API**: Programmatic access for tools/extensions
- **Conflict statistics**: Dashboard of sync health
- **Auto-sync interval**: Periodic sync without user trigger
- **File system watching**: Auto-sync on file changes
- **CalDAV backend**: ETag-based conflict detection, 412 Precondition Failed handling
- **jCal backend**: JSON-based conflict metadata

## Examples

### Example 1: Standard ICS-First Workflow

```bash
# Config (default)
$ cat ~/.config/aim/config.toml
[backend.local]
calendar_path = "~/Documents/Calendar"

[backend.local.sync]
enabled = true
on_startup = true
strategy = "ics_wins"
delete_handling = "always_delete"
conflict_logging = true

# User workflow
$ aim event new "Meeting" --start "tomorrow 10am" --duration "1h"
# Event created in DB, ICS file written to ~/Documents/Calendar/

# External app modifies ICS file (Google Calendar, etc.)

# Next AIM command
$ aim event list
# Sync on startup: reads ICS file, updates DB
# Log shows: CONFLICT uid=xxx type=external_change resolution=ics_overwritten
```

### Example 2: CLI-First Workflow

```bash
# Config
$ cat ~/.config/aim/config.toml
[backend.local]
calendar_path = "~/Documents/Calendar"

[backend.local.sync]
enabled = true
on_startup = false  # manual sync only
strategy = "db_wins"
delete_handling = "keep_db"
conflict_logging = true

# User workflow
$ aim sync --strategy db_wins
# Sync: reads DB, writes/overwrites ICS files
# Any external changes in ICS are overwritten
# Log shows: CONFLICT uid=xxx type=external_overwritten resolution=db_kept

# If external app deletes ICS file
$ aim sync --strategy db_wins
# Deleted item is restored to ICS file
# Log shows: CONFLICT uid=xxx type=deleted_in_ics resolution=restored_from_db
```

### Example 3: Interactive Mode

```bash
# Config
$ cat ~/.config/aim/config.toml
[backend.local.sync]
strategy = "prompt"
delete_handling = "prompt"
conflict_logging = true

# User workflow
$ aim sync
Conflict detected for event: Team Standup
  LocalDB: Team Standup at 10:00am
  ICS file: Team Standup at 11:00am

Choose resolution:
  [1] Keep LocalDB (overwrite ICS file)
  [2] Keep ICS file (overwrite LocalDB)
  [3] Skip (resolve later)
> 2

# Log shows: CONFLICT uid=xxx type=user_resolution resolution=user_kept_ics

Conflict detected for event: Old Meeting
  LocalDB: Old Meeting exists
  ICS file: NOT FOUND

Choose resolution:
  [1] Keep (restore to ICS file)
  [2] Delete (remove from database)
> 2

# Log shows: CONFLICT uid=yyy type=user_resolution resolution=user_confirmed_delete
```

## Benefits of This Design

1. **Flexibility**: Users can choose behavior that matches their workflow
2. **Safety**: Detailed logging provides audit trail and recovery options
3. **Predictability**: Default behavior (ics_wins) is simple and well-defined
4. **No breaking changes**: Existing workflows continue to work unchanged
5. **Future-proof**: Architecture supports extension to CalDAV/jCal backends
6. **User control**: CLI overrides allow one-off changes to behavior
7. **Transparency**: Detailed logs help users understand what's happening
8. **Configurable deletes**: Users can choose how deletions are handled
