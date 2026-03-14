# Multi-Calendar Support

## Overview

Support multiple independent calendars, each with its own configuration (local, WebDAV, etc.). Events/todos belong to exactly one calendar, and queries aggregate results from all enabled calendars by default.

## Current Status

As of 2026-03-19, the core multi-calendar data model is implemented:

- `calendars`, `events.calendar_id`, `todos.calendar_id`, and `resources(uid, calendar_id)` exist in the database
- `[[calendars]]` and `default_calendar` are supported in config
- Item creation, update, and listing are calendar-aware
- Default aggregated queries exclude disabled calendars and sort by calendar priority
- CLI support exists for `aim calendar list`
- CLI support exists for `--calendar` on `event new|list` and `todo new|list`

Still pending relative to the design below:

- `aim calendar add`
- `aim calendar remove`
- `aim calendar default`
- Showing calendar metadata in event/todo output by default
- Broader CalDAV multi-calendar integration coverage in tests

## Design Principles

1. **One-to-one mapping**: Each event/todo belongs to exactly one calendar
2. **Independent calendars**: Each calendar has unique ID, name, configuration, and priority
3. **Aggregated views**: Default queries show items from all enabled calendars
4. **Priority ordering**: Calendar priority determines conflict resolution (lower number = higher priority)
5. **Config-driven activation**: Calendars are enabled or disabled through configuration without deleting stored data

## Architecture

```
┌─────────────┐
│   Aim       │
└──────┬──────┘
       │
       ▼
┌──────────────────────────┐
│  Calendars (config)      │
│  - personal (local)      │  priority 0
│  - work (caldav)         │  priority 1
│  - archive (local)       │  priority 2, disabled
└──────┬───────────────────┘
       │
       ▼
┌──────────────────────────┐
│  Db                      │
│  ├─ calendars (new)      │  ← calendar metadata
│  ├─ events               │  ← calendar_id
│  ├─ todos                │  ← calendar_id
│  └─ resources            │  ← calendar_id
└──────────────────────────┘
```

## Key Changes

### 1. Database Schema

**New table - `calendars`**:

- `id` (TEXT PRIMARY KEY): User-defined unique identifier
- `name` (TEXT): Display name
- `kind` (TEXT): Backend type (local, caldav, etc.)
- `priority` (INTEGER): Conflict resolution order
- `enabled` (INTEGER): Enable/disable flag
- `created_at`, `updated_at`: Timestamps

**Modified tables**:

- `events`, `todos`: Add `calendar_id` column
- `resources`: Change PK from `(uid, backend_kind)` to `(uid, calendar_id)`

**Migration strategy**:

- Create `calendars` table
- Add `calendar_id` with default value `'default'`
- Migrate existing items to default calendar
- Update `resources` to use `calendar_id`

### 2. Configuration

**TOML structure**:

```toml
# Array of calendars (replaces single calendar_path)
[[calendars]]
id = "personal"
name = "Personal"
kind = "local"
priority = 0
enabled = true
calendar_path = "~/calendar/personal"

[[calendars]]
id = "work"
name = "Work"
kind = "caldav"
priority = 1
enabled = true
base_url = "https://caldav.example.com"
calendar_href = "/dav/calendars/user/work/"
# ... caldav config

# Default calendar for new items
default_calendar = "personal"
```

**Backward compatibility**:

- Detect old format (single `calendar_path`)
- Auto-migrate to `[[calendars]]` array with id="default"

### 3. Query Behavior

**Aggregated queries** (default):

```
aim list events     # Shows items from ALL enabled calendars
aim list todos      # Shows items from ALL enabled calendars
```

**Priority-based ordering**:

```
Personal calendar (priority 0): events shown first
Work calendar (priority 1):       events shown second
```

**Calendar-specific filtering**:

```
aim list events --calendar work    # Only work calendar items
aim new event "Meeting" --calendar work   # Create in work calendar
```

**Disabled calendars**:

```
# Remove `archive` from config or set `enabled = false`
# Items are hidden from default queries, but data is retained
```

### 4. Calendar Management

New commands:

```bash
aim calendar list              # List all calendars
aim calendar add               # Add new calendar
aim calendar remove <id>       # Remove calendar
aim calendar default <id>      # Set default calendar
```

Current implementation:

```bash
aim calendar list
```

## Data Flow

### Creating an item

1. Determine target calendar:
   - If `--calendar <id>` specified → use that calendar
   - If not specified → use `default_calendar` from config

2. Insert into database:
   - `events`/`todos` table with `calendar_id`
   - `resources` table maps `(uid, calendar_id)` to backend resource

3. Sync to backend (if enabled in config):
   - Local: Write ICS file to configured path
   - CalDAV: PUT to server URL (future)

### Listing items

1. Fetch all calendars enabled in config (ordered by priority)
2. For each calendar:
   - Query events/todos where `calendar_id = <id>`
   - Include in result set
3. Sort results:
   - By calendar priority first (lower = higher priority)
   - Then by item-specific criteria (date, status, etc.)

### Updating an item

1. Fetch item's `calendar_id` from database
2. Get calendar configuration
3. Sync to that calendar's backend
4. Update `resources` table

## Key Design Decisions

### Why one-to-one mapping?

**Choice**: Events/todos belong to exactly one calendar

**Alternatives considered**:

- Multi-calendar items (same UID in multiple calendars)
- Flexible assignment (items can move between calendars)

**Rationale**:

- Simpler data model and conflict resolution
- Clear ownership (users know which calendar an item belongs to)
- Easier to reason about sync behavior

### Why priority-based ordering?

**Choice**: Calendar priority determines display order

**Alternative**: Alphabetical by calendar name

**Rationale**:

- Users can decide which calendar is more important
- Matches mental model (work calendar > personal calendar)
- Deterministic and predictable

### Why `calendar_id` instead of `backend_kind`?

**Choice**: Use specific calendar identifier

**Rationale**:

- `backend_kind` only identifies type (local vs caldav)
- Multiple calendars can share same backend type
- `calendar_id` provides precise ownership

### Why enable/disable instead of delete?

**Choice**: Soft disable with `enabled` flag controlled by config reconciliation

**Rationale**:

- Temporary exclusion without data loss
- Easy to re-enable
- Useful for testing, migration, cleanup

## User Experience

### Work/Personal Separation

```bash
# Setup
aim calendar add --id work --name "Work" --priority 0
aim calendar add --id personal --name "Personal" --priority 1

# Use
aim list events                # See all events, work first
aim list events --calendar work  # Only work events
aim new event "Meeting" --calendar work  # Add to work calendar
```

### Multiple Local Calendars

```bash
# Multiple local directories
[[calendars]]
id = "personal"
kind = "local"
calendar_path = "~/calendar/personal"

[[calendars]]
id = "archive"
kind = "local"
calendar_path = "~/archive"
enabled = false  # Don't show unless needed
```

### Mixed Backends

```bash
# Local + CalDAV in one view
[[calendars]]
id = "local-backup"
kind = "local"
priority = 0

[[calendars]]
id = "cloud-sync"
kind = "caldav"
priority = 1  # Secondary, synced to cloud
```

## Implementation Phases

### Phase 1: Database

- Create migration for `calendars` table
- Add `calendar_id` to events/todos
- Update `resources` PK
- Handle existing data migration

### Phase 2: Configuration

- Replace `calendar_path` with `calendars` array
- Add `default_calendar` field
- Implement backward compatibility
- Update normalization logic

### Phase 3: Core Logic

- Add `Calendars` module (CRUD operations)
- Update `Events`/`Todos` to use `calendar_id`
- Refactor `Aim` to manage multiple calendars
- Implement aggregated queries with priority sorting

### Phase 4: CLI

- Add `aim calendar` management commands
  Status: complete for `aim calendar list`
- Add `--calendar` filter to list/create commands
  Status: complete for `event new|list` and `todo new|list`
- Update output to show calendar information
  Status: pending
- Add calendar-specific tests
  Status: partially complete

### Phase 5: Testing

- Test multi-calendar setup
  Status: complete
- Test priority ordering
  Status: complete
- Test enable/disable
  Status: complete
- Test backward compatibility
  Status: existing legacy coverage remains in place
- Update documentation
  Status: in progress

## Benefits

1. **Organization**: Separate contexts (work, personal, projects)
2. **Flexibility**: Mix different backend types seamlessly
3. **Aggregation**: Single view of all items
4. **Priority control**: Decide what's important
5. **Scalability**: Add calendars without core changes
6. **Safety**: Disable without deletion
7. **Compatibility**: Graceful migration from single-calendar

## Future Enhancements

- Per-calendar color schemes
- Calendar groups (folder-like organization)
- Multi-calendar sync (same item in multiple calendars)
- Conflict resolution UI (interactive vs automatic)
- Calendar import/export (share calendar configs)
- Per-calendar sync intervals
- Read-only external calendars
