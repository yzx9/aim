# Add CalDAV Support

## Overview

Add CalDAV/WebDAV backend support alongside the existing Local ICS backend. The database foundation (`backend_kind`, `resources` table) is already in place.

**Key Design**: Use a unified `resources` table with JSON metadata instead of backend-specific tables. This provides a generic, extensible schema for all backend types.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                         Aim                             │
│  now, config, backend: Box<dyn Backend>, db, short_ids  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                    Backend Trait                        │
│  - create/update/delete/get Event/Todo                  │
│  - list/count Event/Todo                                │
│  - sync_cache()                                         │
└────────────┬────────────────────────────┬───────────────┘
             │                            │
             ▼                            ▼
      ┌─────────────┐             ┌──────────────┐
      │ LocalBackend│             │CaldavBackend │
      │  ICS files  │             │ CalDAV API   │
      └─────────────┘             └──────────────┘
             │                            │
             └────────────┬───────────────┘
                          ▼
      ┌─────────────────────────────────────────┐
      │           db (primary)                  │
      │  events, todos, resources, short_ids    │
      └─────────────────────────────────────────┘
```

## Components

### Backend Trait

Abstract interface defining operations for all backend types:

- Event operations: create, update, delete, get, list, count
- Todo operations: create, update, delete, get, list, count
- Utilities: uid_exists, sync_cache, backend_kind

### LocalBackend

Manages local ICS file storage:

- Resource ID: `file:///absolute/path/to/{uid}.ics`
- Metadata: `null` (no etag needed for local files)
- Sync: scans directory, updates resources table

### CaldavBackend

Manages CalDAV server storage via `aimcal_caldav` crate:

- Resource ID: href from server (e.g., `/dav/calendars/user/default/abc123.ics`)
- Metadata: `{"etag": "\"...\"", "last_modified": "..."}`
- Sync: calendar-query REPORT, updates resources table with hrefs and etags

### Database Schema

**events**: uid, summary, description, status, start, end, backend_kind
**todos**: uid, completed, description, percent, priority, status, summary, due, backend_kind
**resources**: uid, backend_kind, resource_id, metadata
**short_ids**: short_id, uid, kind

The `resources` table stores backend-specific resource identifiers and metadata:

- Local backend: `file://` URLs, no metadata
- CalDAV backend: server hrefs with etag/last-modified JSON metadata
- Future backends: any resource_id format with custom JSON metadata

### Configuration

```toml
# Local backend (default)
backend_kind = "local"
calendar_path = "calendar"

# CalDAV backend
backend_kind = "caldav"
base_url = "https://caldav.example.com"
calendar_home = "/dav/calendars/user/"
calendar_href = "/dav/calendars/user/default/"
auth = { username = "user", password = "pass" }
```

## Implementation Status

### ✅ Complete

- Database migration: `backend_kind` column, `resources` table, removed `path` columns
- Resources table implementation with JSON metadata support
- ICS support made optional (DB-only mode supported)
- `BackendKind` enum (Local only, CalDav stub)

### ❌ Remaining

1. **Backend Abstraction Layer**
   - Create `Backend` trait with operations for events/todos
   - Implement `LocalBackend` (extract ICS logic from `aim.rs`)
   - Implement `CaldavBackend` (wrap `aimcal_caldav` client)

2. **Config Extension**
   - Add `BackendConfig` enum (Local/Caldav variants)
   - Update config parsing

3. **Aim Refactoring**
   - Add `backend: Box<dyn Backend>` field
   - Delegate storage operations to backend
   - Add `sync()` method

4. **CLI Sync Command**
   - Add `aim sync` subcommand with force option

## Design Rationale

### Why Unified Resources Table?

A simpler approach might use `event_hrefs` and `todo_hrefs` tables. However, the unified `resources` table provides:

1. **Generic design**: Works for any backend type (local, CalDAV, future jCal)
2. **Flexible metadata**: JSON supports backend-specific data without schema changes
3. **Multi-backend ready**: Can support same item across different backends
4. **Simpler codebase**: Single module instead of separate href tables
5. **Clear semantics**: `resource_id` is generic (file://, /dav/, urn:uuid all work)

### Resource ID Examples

```sql
-- Local backend (ICS files)
('abc123', 'local', 'file:///home/user/calendar/abc123.ics', NULL)

-- CalDAV backend
('abc123', 'caldav', '/dav/calendars/user/default/abc123.ics',
 '{"etag": "\"abc123\"", "last_modified": "2025-01-31T10:00:00Z"}')

-- jCal backend (future)
('abc123', 'jcal', 'urn:uuid:abc123',
 '{"version": "1.0", "schema": "urn:ietf:params:rfc:7265"}')
```

## Risks & Mitigations

| Risk                      | Mitigation                             |
| ------------------------- | -------------------------------------- |
| Breaking existing configs | Default to local backend, auto-migrate |
| CalDAV href conflicts     | Let server decide via Location header  |
| Cache desync              | Manual sync command + startup sync     |
| Performance               | Db still provides fast queries         |
| JSON metadata complexity  | Document schemas, use typed structs    |

## Future Enhancements

- File system watching for local backend changes
- Auto-sync interval configuration
- Sync-token REPORT for efficient sync
- Conflict resolution strategies
- Offline mode with queue
- jCal backend support
- Multi-backend per item (same item in local + CalDAV)
