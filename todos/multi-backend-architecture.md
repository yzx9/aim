# Multi-Backend Architecture Implementation Plan

## Overview

Support two backend types (Local ICS files and WebDAV/CalDAV) with LocalDB as a pure cache.

## User Choices

- **Migration**: Keep default behavior (local backend when unspecified)
- **Href generation**: Server decides via Location header
- **Sync strategy**: Manual sync command + startup sync
- **Database**: Separate tables for path/href, remove path from main tables, add backend_type

## Architecture Design

```
┌─────────────────────────────────────────────────────────┐
│                         Aim                              │
│  now, config, backend: Box<dyn Backend>, db, short_ids  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                    Backend Trait                         │
│  - create/update/delete/get Event/Todo                  │
│  - list/count Event/Todo                                 │
│  - sync_cache()                                          │
│  - uid_exists()                                          │
└────────────┬────────────────────────────┬────────────────┘
             │                            │
             ▼                            ▼
      ┌─────────────┐             ┌──────────────┐
      │ LocalBackend│             │WebdavBackend │
      │  ICS files  │             │ CalDAV API   │
      └─────────────┘             └──────────────┘
             │                            │
             └────────────┬───────────────┘
                          ▼
              ┌─────────────────────────┐
              │       LocalDb (cache)    │
              │  events, todos, short_ids│
              │  event_hrefs, todo_hrefs │
              └─────────────────────────┘
```

## Database Schema Changes

### New Migration: `20250123_add_backend_support`

```sql
-- Add backend_type to events and todos
ALTER TABLE events ADD COLUMN backend_type TEXT NOT NULL DEFAULT 'local';
ALTER TABLE todos ADD COLUMN backend_type TEXT NOT NULL DEFAULT 'local';

-- Create href mapping tables
CREATE TABLE IF NOT EXISTS event_hrefs (
    uid   TEXT PRIMARY KEY,
    href  TEXT NOT NULL,
    etag  TEXT,
    FOREIGN KEY (uid) REFERENCES events(uid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS todo_hrefs (
    uid   TEXT PRIMARY KEY,
    href  TEXT NOT NULL,
    etag  TEXT,
    FOREIGN KEY (uid) REFERENCES todos(uid) ON DELETE CASCADE
);

-- Copy existing paths to href tables for local backend
INSERT INTO event_hrefs (uid, href)
SELECT uid, 'file://' || path FROM events;

INSERT INTO todo_hrefs (uid, href)
SELECT uid, 'file://' || path FROM todos;

-- Remove path column (after verification)
-- ALTER TABLE events DROP COLUMN path;
-- ALTER TABLE todos DROP COLUMN path;
```

### Final Tables

**events**: uid, summary, description, status, start, end, backend_type
**todos**: uid, completed, description, percent, priority, status, summary, due, backend_type
**event_hrefs**: uid, href, etag
**todo_hrefs**: uid, href, etag
**short_ids**: short_id, uid, kind

## Implementation Plan

### Phase 1: Database Migration (core crate)

**Files**: `core/src/localdb/migrations/20250123_add_backend_support.{up,down}.sql`

1. Create migration to add backend_type and href tables
2. For existing users: set backend_type = 'local', copy paths to hrefs
3. **Note**: Keep path column initially, remove in separate migration after verification

### Phase 2: Backend Abstraction (core crate)

**New Files**:
- `core/src/backend.rs` - Backend trait, BackendError, BackendType, SyncResult
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
    fn backend_type(&self) -> BackendType;
}
```

**LocalBackend**:
- Uses `io.rs` functions (parse_ics, write_ics, add_calendar)
- Stores hrefs as `file:///absolute/path/to/{uid}.ics`
- sync_cache: scans directory, updates cache

**WebdavBackend**:
- Wraps `aimcal_caldav::CalDavClient`
- On create: PUT to `calendar_href/{uid}.ics`, read Location header for actual href
- Stores hrefs from server response
- Maintains ETag for optimistic concurrency
- sync_cache: calendar-query REPORT, update cache with ETags

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
#[serde(tag = "backend_type")]
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
backend_type = "local"
calendar_path = "calendar"

# WebDAV backend
backend_type = "webdav"
base_url = "https://caldav.example.com"
calendar_home = "/dav/calendars/user/"
calendar_href = "/dav/calendars/user/default/"
auth = { username = "user", password = "pass" }
```

### Phase 4: LocalDb Adaptation (core crate)

**Files**: `core/src/localdb.rs`, `core/src/localdb/events.rs`, `core/src/localdb/todos.rs`

**Changes**:
1. Add href table management:

```rust
// localdb.rs
pub struct LocalDb {
    pool: SqlitePool,
    pub events: Events,
    pub todos: Todos,
    pub short_ids: ShortIds,
    pub event_hrefs: Hrefs<Kind::Event>,
    pub todo_hrefs: Hrefs<Kind::Todo>,
}

// new module: localdb/hrefs.rs
pub struct Hrefs<const KIND: Kind>;

impl Hrefs<Kind::Event> {
    pub async fn insert(&self, uid: &str, href: &str, etag: Option<&str>) -> Result<()>;
    pub async fn get(&self, uid: &str) -> Result<Option<(String, Option<String>)>>;
    pub async fn update_etag(&self, uid: &str, etag: &str) -> Result<()>;
    pub async fn delete(&self, uid: &str) -> Result<()>;
}
```

2. Modify upsert to accept href:

```rust
// events.rs
impl Events {
    pub async fn upsert_with_href(
        &self,
        uid: &str,
        event: &impl Event,
        backend_type: BackendType,
        href: &str,
    ) -> Result<()>;

    // Old method for backward compatibility
    pub async fn upsert(&self, path: &Path, event: &impl Event) -> Result<()> {
        let backend_type = BackendType::Local;
        let href = format!("file://{}", path.display());
        self.upsert_with_href(event.uid(), event, backend_type, &href).await
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
- `core/src/localdb/hrefs.rs`
- `core/src/localdb/migrations/20250123_add_backend_support.up.sql`
- `core/src/localdb/migrations/20250123_add_backend_support.down.sql`
- `core/src/localdb/migrations/20250124_remove_path.up.sql` (optional)
- `core/src/localdb/migrations/20250124_remove_path.down.sql` (optional)
- `cli/src/sync.rs` (or add to main.rs)

### Modified Files
- `core/src/lib.rs` - Export backend module
- `core/src/aim.rs` - Refactor to use backend trait
- `core/src/config.rs` - Add BackendConfig enum
- `core/src/localdb.rs` - Add href tables, modify upsert
- `core/src/localdb/events.rs` - Remove path, add backend_type
- `core/src/localdb/todos.rs` - Remove path, add backend_type
- `core/src/io.rs` - Keep but only used by LocalBackend
- `cli/src/main.rs` - Add sync command

### Files to Check/Update
- `core/Cargo.toml` - Add async-trait dependency
- Ensure aimcal-caldav functions are compatible

## Testing Strategy

1. **Unit tests**:
   - Backend trait operations
   - LocalBackend with test directory
   - Href table management

2. **Integration tests**:
   - LocalBackend: create/read/update/delete events
   - WebdavBackend: use wiremock from caldav crate
   - Sync: verify cache updates correctly

3. **Migration tests**:
   - Test existing data migration to href tables
   - Test backward compatibility with old configs

## Verification Steps

1. Create test config with local backend
2. Create test config with webdav backend (using wiremock)
3. Run `aim list events` - should work with both
4. Run `aim new event "Test"` - should create via correct backend
5. Run `aim sync` - should update cache from backend
6. Verify database: href tables populated correctly
7. Verify existing users: no breaking changes

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking existing configs | Default to local backend, auto-migrate |
| WebDAV href conflicts | Let server decide via Location header |
| Cache desync | Manual sync command + startup sync |
| Path column removal issues | Keep initially, remove in separate migration |
| Performance | LocalDB still provides fast queries |

## Future Enhancements

- Watch file system for local backend changes
- Auto-sync interval configuration
- Sync-token REPORT support for efficient sync
- Conflict resolution strategies
- Offline mode with queue
