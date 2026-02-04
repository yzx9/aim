# Multi-Backend Architecture Implementation Plan

## Overview

Support multiple backend types (Local ICS files, WebDAV/CalDAV, and future jCal) with LocalDB as primary storage and unified backend mapping via `resources` table.

**Key Design Decision**: Use unified `resources` table instead of separate `event_hrefs` and `todo_hrefs` tables. This provides a generic, extensible schema for all backends with flexible JSON metadata.

## User Choices

- **Migration**: Keep default behavior (local backend when unspecified)
- **Resource ID generation**: Backend-specific (local: file://, caldav: href from server)
- **Sync strategy**: Manual sync command + startup sync
- **Database**: Unified `resources` table, remove path from main tables, add backend_kind

## Architecture Design

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
│  - uid_exists()                                         │
└────────────┬────────────────────────────┬───────────────┘
             │                            │
             ▼                            ▼
      ┌─────────────┐             ┌──────────────┐
      │ LocalBackend│             │WebdavBackend │
      │  ICS files  │             │ CalDAV API   │
      └─────────────┘             └──────────────┘
             │                            │
              └────────────┬───────────────┘
                           ▼
               ┌─────────────────────────────────────────┐
               │           LocalDb (primary)            │
               │  ┌───────────────────────────────┐     │
               │  │ events                    │     │
               │  │ uid, summary, ...         │     │
               │  │ backend_kind             │     │
               │  └───────────────────────────────┘     │
               │  ┌───────────────────────────────┐     │
               │  │ todos                     │     │
               │  │ uid, summary, ...         │     │
               │  │ backend_kind             │     │
               │  └───────────────────────────────┘     │
               │  ┌───────────────────────────────┐     │
               │  │ resources (unified)        │     │
               │  │ uid, backend_kind         │     │
               │  │ resource_id, metadata     │     │
               │  └───────────────────────────────┘     │
               │  ┌───────────────────────────────┐     │
               │  │ short_ids                 │     │
               │  └───────────────────────────────┘     │
               └─────────────────────────────────────────┘
```

## Database Schema Changes

### New Migration: `20250130_add_backend_support`

```sql
-- Add backend_kind to events and todos
ALTER TABLE events ADD COLUMN backend_kind TEXT NOT NULL DEFAULT 'local';
ALTER TABLE todos ADD COLUMN backend_kind TEXT NOT NULL DEFAULT 'local';

-- Create unified resources table for all backends
CREATE TABLE IF NOT EXISTS resources (
    uid TEXT NOT NULL,
    backend_kind TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    metadata TEXT,
    PRIMARY KEY (uid, backend_kind),
    FOREIGN KEY (uid) REFERENCES events(uid) ON DELETE CASCADE,
    FOREIGN KEY (uid) REFERENCES todos(uid) ON DELETE CASCADE
);

-- Create index for performance
CREATE INDEX IF NOT EXISTS idx_resources_backend_kind ON resources(backend_kind);

-- Copy existing paths to resources table for local backend
INSERT OR IGNORE INTO resources (uid, backend_kind, resource_id)
SELECT uid, 'local', 'file://' || path
FROM events
WHERE path IS NOT NULL AND path != '';

INSERT OR IGNORE INTO resources (uid, backend_kind, resource_id)
SELECT uid, 'local', 'file://' || path
FROM todos
WHERE path IS NOT NULL AND path != '';

-- Remove path column (after verification)
-- ALTER TABLE events DROP COLUMN path;
-- ALTER TABLE todos DROP COLUMN path;
```

### Final Tables

**events**: uid, summary, description, status, start, end, backend_kind
**todos**: uid, completed, description, percent, priority, status, summary, due, backend_kind
**resources**: uid, backend_kind, resource_id, metadata
**short_ids**: short_id, uid, kind

### Backend Metadata Schemas

#### Local Backend

```json
null
```

#### CalDAV Backend

```json
{
  "etag": "\"...\"",
  "last_modified": "2025-01-31T10:00:00Z"
}
```

#### jCal Backend (Future)

```json
{
  "version": "1.0",
  "schema": "urn:ietf:params:rfc:7265"
}
```

## Implementation Plan

### Phase 1: Database Migration (core crate)

**Files**: `core/src/localdb/migrations/20250130_add_backend_support.{up,down}.sql`

1. Create migration to add backend_kind and unified resources table
2. For existing users: set backend_kind = 'local', copy paths to resources
3. **Note**: Keep path column initially, remove in separate migration after verification

### Phase 2: Backend Abstraction (core crate)

**New Files**:

- `core/src/backend.rs` - Backend trait, BackendError, BackendKind, SyncResult
- `core/src/backend/local.rs` - LocalBackend implementation
- `core/src/backend/webdav.rs` - WebdavBackend implementation
- `core/src/backend/mod.rs` - Module exports

**Backend Trait**:

```rust
#[async_trait]
pub trait Backend: Send + Sync {
    // Event operations
    async fn create_event(&self, uid: &str, event: &VEvent<String>)
        -> Result<Box<dyn Event + 'static>, BackendError>;
    async fn update_event(&self, uid: &str, event: &VEvent<String>)
        -> Result<Box<dyn Event + 'static>, BackendError>;
    async fn delete_event(&self, uid: &str) -> Result<(), BackendError>;
    async fn get_event(&self, uid: &str)
        -> Result<Option<Box<dyn Event + 'static>>, BackendError>;
    async fn list_events(&self, conds: &ResolvedEventConditions, pager: &Pager)
        -> Result<Vec<Box<dyn Event + 'static>>, BackendError>;
    async fn count_events(&self, conds: &ResolvedEventConditions)
        -> Result<i64, BackendError>;

    // Todo operations (similar)
    async fn create_todo(...);
    async fn update_todo(...);
    async fn delete_todo(...);
    async fn get_todo(...);
    async fn list_todos(...);
    async fn count_todos(...);

    // Utility
    async fn uid_exists(&self, uid: &str, kind: Kind) -> Result<bool, BackendError>;
    async fn sync_cache(&self) -> Result<SyncResult, BackendError>;
    fn backend_kind(&self) -> BackendKind;
}
```

**LocalBackend**:

- Uses `io.rs` functions (parse_ics, write_ics, add_calendar)
- Stores resource_id as `file:///absolute/path/to/{uid}.ics`
- metadata is NULL (no etag needed for local files)
- sync_cache: scans directory, updates resources table

**WebdavBackend**:

- Wraps `aimcal_caldav::CalDavClient`
- On create: PUT to `calendar_href/{uid}.ics`, read Location header for actual href
- Stores resource_id (href) and metadata (etag JSON) in resources table
- Maintains ETag for optimistic concurrency via metadata: `{"etag": "...", "last_modified": "..."}`
- sync_cache: calendar-query REPORT, update resources table with resource_id and metadata

### Phase 3: Config Extension (core crate)

**File**: `core/src/config.rs`

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub backend: BackendConfig,

    #[serde(default)]
    pub state_dir: Option<PathBuf>,

    #[serde(default)]
    pub default_due: Option<DateTimeAnchor>,

    #[serde(default)]
    pub default_priority: Priority,

    #[serde(default)]
    pub default_priority_none_fist: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "backend_kind")]
pub enum BackendConfig {
    #[serde(rename = "local")]
    Local {
        #[serde(default)]
        calendar_path: Option<PathBuf>,
    },
    #[serde(rename = "webdav")]
    Webdav {
        #[serde(flatten)]
        webdav: CalDavConfig,
        calendar_href: String,
    },
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self::Local { calendar_path: None }
    }
}
```

**Config examples**:

```toml
# Local backend (default, backward compatible)
backend_kind = "local"
calendar_path = "calendar"

# WebDAV backend
backend_kind = "webdav"
base_url = "https://caldav.example.com"
calendar_home = "/dav/calendars/user/"
calendar_href = "/dav/calendars/user/default/"
auth = { username = "user", password = "pass" }
```

### Phase 4: LocalDb Adaptation (core crate)

**Files**: `core/src/localdb.rs`, `core/src/localdb/events.rs`, `core/src/localdb/todos.rs`

**Changes**:

1. Add resources table management:

```rust
// localdb.rs
pub struct LocalDb {
    pool: SqlitePool,
    pub events: Events,
    pub todos: Todos,
    pub short_ids: ShortIds,
    pub resources: Resources,  // Unified for all backends
}

// new module: localdb/resources.rs
pub struct Resources {
    pool: SqlitePool,
}

impl Resources {
    pub async fn insert(
        &self,
        uid: &str,
        backend_kind: &str,
        resource_id: &str,
        metadata: Option<&str>,
    ) -> Result<()>;

    pub async fn get(
        &self,
        uid: &str,
        backend_kind: &str,
    ) -> Result<Option<ResourceRecord>, sqlx::Error>;

    pub async fn update_metadata(
        &self,
        uid: &str,
        backend_kind: &str,
        metadata: &str,
    ) -> Result<()>;

    pub async fn delete(&self, uid: &str, backend_kind: &str) -> Result<()>;
}

#[derive(Debug, sqlx::FromRow)]
pub struct ResourceRecord {
    pub uid: String,
    pub backend_kind: String,
    pub resource_id: String,
    pub metadata: Option<String>,
}

impl ResourceRecord {
    pub fn metadata_json<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        let metadata = self.metadata.as_ref()?;
        serde_json::from_str(metadata).ok()
    }
}
```

2. Modify upsert to accept resource_id and metadata:

```rust
// events.rs
impl Events {
    pub async fn upsert_with_backend(
        &self,
        uid: &str,
        event: &impl Event,
        backend_kind: &str,
        resource_id: Option<&str>,
        metadata: Option<&str>,
    ) -> Result<()>;

    // Old method for backward compatibility
    pub async fn upsert(&self, path: &Path, event: &impl Event) -> Result<()> {
        let backend_kind = "local";
        let resource_id = Some(format!("file://{}", path.display()).as_str());
        self.upsert_with_backend(event.uid(), event, backend_kind, resource_id, None).await
    }
}
```

### Phase 5: Aim Refactoring (core crate)

**File**: `core/src/aim.rs`

**Key changes**:

```rust
pub struct Aim {
    now: Zoned,
    config: Config,
    backend: Box<dyn Backend>,
    db: LocalDb,
    short_ids: ShortIds,
    // Remove: calendar_path: PathBuf
}

impl Aim {
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn Error>> {
        let now = Zoned::now();
        config.normalize()?;
        prepare(&config).await?;

        let db = initialize_db(&config).await?;
        let short_ids = ShortIds::new(db.clone());

        // Create backend based on config
        let backend = create_backend(&config, db.clone()).await?;

        // Initial sync
        let _sync_result = backend.sync_cache().await;

        Ok(Self { now, config, backend, db, short_ids })
    }

    pub async fn sync(&self) -> Result<SyncResult, Box<dyn Error>> {
        Ok(self.backend.sync_cache().await?)
    }

    // Remove: get_path(), add_calendar() direct calls
    // Delegate all operations to backend
}

async fn create_backend(config: &Config, db: LocalDb)
    -> Result<Box<dyn Backend>, Box<dyn Error>>
{
    match &config.backend {
        BackendConfig::Local { calendar_path } => {
            let path = calendar_path.clone()
                .unwrap_or_else(|| PathBuf::from("calendar"));
            Ok(Box::new(LocalBackend::new(path, db).await?))
        }
        BackendConfig::Webdav { webdav, calendar_href } => {
            Ok(Box::new(WebdavBackend::new(
                webdav.clone(),
                calendar_href.clone(),
                db,
            ).await?))
        }
    }
}
```

### Phase 6: CLI Sync Command (cli crate)

**File**: `cli/src/main.rs` or `cli/src/sync.rs`

Add new subcommand:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands
    Sync {
        /// Force full sync from backend
        #[arg(short, long)]
        force: bool,
    },
}
```

### Phase 7: Remove Path Column (optional, later)

**File**: `core/src/localdb/migrations/20250124_remove_path.{up,down}.sql`

After verifying href system works:

```sql
ALTER TABLE events DROP COLUMN path;
ALTER TABLE todos DROP COLUMN path;
```

## File Change Summary

### New Files

- `core/src/backend.rs`
- `core/src/backend/mod.rs`
- `core/src/backend/local.rs`
- `core/src/backend/webdav.rs`
- `core/src/localdb/resources.rs` (replaces hrefs.rs)
- `core/src/localdb/migrations/20250130_add_backend_support.up.sql`
- `core/src/localdb/migrations/20250130_add_backend_support.down.sql`
- `core/src/localdb/migrations/20250130_remove_path.up.sql` (optional)
- `core/src/localdb/migrations/20250130_remove_path.down.sql` (optional)
- `cli/src/sync.rs` (or add to main.rs)

### Modified Files

- `core/src/lib.rs` - Export backend module
- `core/src/aim.rs` - Refactor to use backend trait
- `core/src/config.rs` - Add BackendConfig enum
- `core/src/localdb.rs` - Add resources table, modify upsert
- `core/src/localdb/events.rs` - Remove path, add backend_kind
- `core/src/localdb/todos.rs` - Remove path, add backend_kind
- `core/src/io.rs` - Keep but only used by LocalBackend
- `cli/src/main.rs` - Add sync command

### Files to Check/Update

- `core/Cargo.toml` - Add async-trait, serde_json dependencies
- Ensure aimcal-caldav functions are compatible

## Testing Strategy

1. **Unit tests**:
   - Backend trait operations
   - LocalBackend with test directory
   - Resources table CRUD operations
   - Metadata JSON serialization/deserialization
   - Multiple backend_kind entries for same uid
   - Backend-specific metadata structures (CalDavMetadata, etc.)

2. **Integration tests**:
   - LocalBackend: create/read/update/delete events
   - WebdavBackend: use wiremock from caldav crate
   - Sync: verify resources table updates correctly
   - Verify resource_id format for each backend
   - Verify metadata JSON parsing for CalDAV

3. **Migration tests**:
   - Test existing data migration to resources table
   - Test backward compatibility with old configs

## Verification Steps

1. Create test config with local backend
2. Create test config with webdav backend (using wiremock)
3. Run `aim list events` - should work with both
4. Run `aim new event "Test"` - should create via correct backend
5. Run `aim sync` - should update resources table from backend
6. Verify database:
   - resources table populated with correct resource_id values
   - metadata JSON correctly serialized for CalDAV
   - Multiple backend_kind entries work for same uid (if testing multi-backend)
7. Verify existing users: no breaking changes
8. Verify metadata queries: can parse CalDAV etags from JSON

## Benefits of Unified Resources Table

### Why Not Separate Tables?

A simpler approach might use `event_hrefs` and `todo_hrefs` tables like the original design. However, this has limitations:

1. **HTTP-specific naming**: `href` and `etag` are specific to WebDAV/CalDAV, not generic
2. **Fixed schema**: Adding new metadata fields requires schema changes
3. **Separate tables**: Two tables mean duplicate code and migrations
4. **Not extensible**: jCal backend might need different metadata structure
5. **Multi-backend difficulty**: Hard to support same item in multiple backends

### Why Unified Resources Table?

The unified `resources` table solves all these problems:

1. **Simpler codebase**: Single `Resources` module instead of `Hrefs<Kind>` generics
2. **Single migration**: One table creation instead of two separate tables
3. **Consistent API**: Same methods for all backends (insert, get, update_metadata, delete)
4. **Future-proof**: JSON metadata supports any backend-specific data
5. **Multi-backend ready**: Can have entries for same uid across different backends
6. **Clear semantics**: `resource_id` is generic (file://, /dav/, urn:uuid all work)
7. **Performance**: Single indexed table instead of two tables
8. **jCal ready**: Easy to add JSON-based calendar backend in future

### Example Resource Records

```sql
-- Local backend (ICS files)
INSERT INTO resources (uid, backend_kind, resource_id, metadata)
VALUES ('abc123', 'local', 'file:///home/user/calendar/abc123.ics', NULL);

-- CalDAV backend
INSERT INTO resources (uid, backend_kind, resource_id, metadata)
VALUES ('abc123', 'caldav', '/dav/calendars/user/default/abc123.ics',
  '{"etag": "\"abc123\"", "last_modified": "2025-01-31T10:00:00Z"}');

-- jCal backend (future)
INSERT INTO resources (uid, backend_kind, resource_id, metadata)
VALUES ('abc123', 'jcal', 'urn:uuid:abc123',
  '{"version": "1.0", "schema": "urn:ietf:params:rfc:7265"}');
```

## Risks & Mitigations

| Risk                       | Mitigation                                   |
| -------------------------- | -------------------------------------------- |
| Breaking existing configs  | Default to local backend, auto-migrate       |
| WebDAV href conflicts      | Let server decide via Location header        |
| Cache desync               | Manual sync command + startup sync           |
| Path column removal issues | Keep initially, remove in separate migration |
| Performance                | LocalDB still provides fast queries          |
| JSON metadata complexity   | Document schemas clearly, use typed structs  |

## Future Enhancements

- Watch file system for local backend changes
- Auto-sync interval configuration
- Sync-token REPORT support for efficient sync
- Conflict resolution strategies
- Offline mode with queue
- **jCal backend support**: Use resources table with JSON metadata
- **Multi-backend per item**: Support same item in both local and caldav backends
