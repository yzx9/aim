# Undo/Redo Implementation Plan for AIM

## Overview

Implement a persistent undo/redo mechanism for AIM using the **Command Pattern with Persistent History Log** approach. This design provides medium granularity (one command = one undo step), persistent history across CLI invocations, and clean separation between core and CLI layers.

### Key Design Decisions

| Aspect                | Choice                       | Rationale                                                          |
| --------------------- | ---------------------------- | ------------------------------------------------------------------ |
| **Granularity**       | Medium (complete operations) | Matches user mental model - one command = one undo step            |
| **Persistence Layer** | Core crate                   | Aim struct manages undo log; business logic owns state history     |
| **Storage**           | Database (SQLite)            | Uses existing SQLite DB with new `undo_log` table; ACID guarantees |
| **Extensibility**     | Add Operation variants       | Simple enum pattern for new operations                             |

## Architecture

```
┌───────────────────────────────────────────────────────────┐
│ CLI Layer (Thin presentation layer)                       │
│                                                           │
│  Commands just call Aim methods:                          │
│    aim.record_undo(command)  → Save operation to history  │
│    aim.undo()                 → Undo last operation       │
│    aim.redo()                 → Redo last undone          │
│    aim.history()              → Get history for display   │
│                                                           │
│  No state management in CLI                               │
└───────────────────────────────────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────┐
│ Core Layer (Business logic + state persistence)           │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐  │
│  | Aim (core/src/aim.rs)                               │  │
│  |  ┌───────────────────────────────────────────────┐  │  │
│  |  | UndoManager (internal)                        │  │  │
│  |  |  - history: Vec<UndoableCommand>              │  │  │
│  |  |  - position: usize                            │  │  │
│  |  │                                               │  │  │
│  |  |  Database operations via db.undo_log:         │  │  │
│  |  |    - load()     → Load from SQLite            │  │  │
│  |  |    - save(cmd) → Insert into SQLite           │  │  │
│  |  |    - persist() → Update position in SQLite    │  │  │
│  |  └───────────────────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐  │
│  | UndoableCommand (core/src/undo.rs)                  │  │
│  |  - id: CommandId (UUID)                             │  │
│  |  - timestamp: i64                                   │  │
│  |  - op: Operation enum                               │  │
│  │                                                     │  │
│  |  Operation variants:                                │  │
│  │    - EventNew, EventUpdate, EventDelete             │  │
│  │    - TodoNew, TodoUpdate, TodoDelete                │  │
│  │    - Batch (for composite operations)               │  │
│  │                                                     │  │
│  |  Methods:                                           │  │
│  │    - forward(aim)   → Execute operation             │  │
│  │    - backward(aim)  → Undo operation                │  │
│  │    - describe()     → Human-readable description    │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                           │
│  Core operations:                                         │
│    - new_event_undoable() → (Event, UndoableCommand)      │
│    - update_event_undoable() → (Event, UndoableCommand)   │
│    - delete_event_undoable() → UndoableCommand            │
│    - (similar for todos)                                  │
└───────────────────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Core Undoable Operations + UndoManager

**Files to create/modify:**

- **NEW**: `core/src/undo.rs` - UndoableCommand, Operation types, and UndoManager
- **NEW**: `core/src/localdb/undo_log.rs` - Database operations for undo log
- **NEW**: `core/src/localdb/migrations/YYYYMMDDHHMMSS_add_undo_log.sql` - Database migration
- **MODIFY**: `core/src/aim.rs` - Add undoable operation methods + undo/redo/history methods
- **MODIFY**: `core/src/event.rs` - Ensure EventPatch captures full state
- **MODIFY**: `core/src/todo.rs` - Ensure TodoPatch captures full state
- **MODIFY**: `core/src/localdb/events.rs` - Add delete method if missing
- **MODIFY**: `core/src/localdb/todos.rs` - Add delete method if missing
- **MODIFY**: `core/src/localdb.rs` - Add undo_log module and field

**Core data structures:**

```rust
// core/src/undo.rs

use serde::{Deserialize, Serialize};
use aimcal_ical::{VEvent, VTodo};

/// Database ID for an undoable command (auto-incrementing)
pub type CommandId = i64;

/// Undoable command with forward and backward operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoableCommand {
    pub id: CommandId,  // Database primary key
    pub timestamp: i64,
    pub op: Operation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    EventNew {
        uid: String,
        path: String,
        event_data: VEvent<String>,
    },
    EventUpdate {
        uid: String,
        path: String,
        before: VEvent<String>,
        after: VEvent<String>,
    },
    EventDelete {
        uid: String,
        path: String,
        event_data: VEvent<String>,
    },
    // Similar variants for Todo...
    Batch {
        commands: Vec<UndoableCommand>,
    },
}

impl UndoableCommand {
    pub async fn forward(&self, aim: &Aim) -> Result<(), Box<dyn Error>> { /* ... */ }
    pub async fn backward(&self, aim: &Aim) -> Result<(), Box<dyn Error>> { /* ... */ }
    pub fn describe(&self) -> String { /* ... */ }
}
```

**UndoManager (in core/src/undo.rs):**

```rust
// core/src/undo.rs

use crate::localdb::undo_log::UndoLog;
use serde::{Deserialize, Serialize};

/// Manages undo/redo history within Aim
pub struct UndoManager {
    undo_log: UndoLog,
    history: Vec<UndoableCommand>,
    current_position: usize,  // Points to next command to redo
}

impl UndoManager {
    /// Load history from database (called during Aim::new)
    pub async fn load(undo_log: UndoLog) -> Result<Self, Box<dyn Error>> {
        // Load history and position from database
        let (history, position) = undo_log.load_all().await?;

        Ok(Self {
            undo_log,
            history,
            current_position: position,
        })
    }

    /// Save a new command to history
    pub async fn save(&mut self, mut command: UndoableCommand) -> Result<(), Box<dyn Error>> {
        // Truncate redo history in memory
        self.history.truncate(self.current_position);
        self.history.push(command.clone());
        self.current_position = self.history.len();

        // Persist to database (ID will be set during insert)
        self.undo_log.insert(&mut command).await?;
        self.undo_log.save_position(self.current_position).await?;
        Ok(())
    }

    /// Persist current position to database
    pub async fn persist(&self) -> Result<(), Box<dyn Error>> {
        self.undo_log.save_position(self.current_position).await
    }

    pub fn can_undo(&self) -> bool {
        self.current_position > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_position < self.history.len()
    }

    pub fn history(&self) -> &[UndoableCommand] {
        &self.history
    }
}
```

**UndoLog database operations (core/src/localdb/undo_log.rs):**

```rust
// core/src/localdb/undo_log.rs

use sqlx::SqlitePool;
use aimcal_core::undo::UndoableCommand;

pub struct UndoLog {
    pool: SqlitePool,
}

impl UndoLog {
    /// Load all commands and current position from database
    pub async fn load_all(&self) -> Result<(Vec<UndoableCommand>, usize), Box<dyn Error>> {
        // Load both ID and serialized data
        let rows: Vec<(i64, Vec<u8>)> = sqlx::query_as(
            "SELECT id, command_data FROM undo_log ORDER BY id ASC;"
        )
        .fetch_all(&self.pool)
        .await?;

        // Deserialize and set IDs
        let commands: Vec<UndoableCommand> = rows
            .into_iter()
            .map(|(id, data)| {
                let mut cmd: UndoableCommand = serde_json::from_slice(&data)?;
                cmd.id = id;  // Ensure ID matches database
                Ok(cmd)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let position: i64 = sqlx::query_scalar(
            "SELECT value FROM metadata WHERE key = 'undo_position';"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok((commands, position as usize))
    }

    /// Insert a new command into the database and return its ID
    pub async fn insert(&self, command: &mut UndoableCommand) -> Result<(), Box<dyn Error>> {
        let data = serde_json::to_vec(command)?;

        let result = sqlx::query(
            "INSERT INTO undo_log (timestamp, command_data) VALUES (?, ?);"
        )
        .bind(command.timestamp)
        .bind(&data)
        .execute(&self.pool)
        .await?;

        // Set the database-generated ID
        command.id = result.last_insert_rowid();
        Ok(())
    }

    /// Save the current undo position
    pub async fn save_position(&self, position: usize) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('undo_position', ?);"
        )
        .bind(position as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Prune old commands
    pub async fn prune_old(&self, older_than: i64) -> Result<u64, Box<dyn Error>> {
        let result = sqlx::query(
            "DELETE FROM undo_log WHERE timestamp < ?;"
        )
        .bind(older_than)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Limit to N most recent commands
    pub async fn prune_max(&self, max_count: usize) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "DELETE FROM undo_log WHERE id NOT IN (
                SELECT id FROM undo_log ORDER BY id DESC LIMIT ?
            );"
        )
        .bind(max_count as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

**Database migration (core/src/localdb/migrations/YYYYMMDDHHMMSS_add_undo_log.sql):**

```sql
-- Undo log table for storing undoable operations
CREATE TABLE IF NOT EXISTS undo_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    command_data BLOB NOT NULL  -- Serialized UndoableCommand (JSON)
);

-- Metadata table for storing undo position
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Initialize undo position
INSERT OR IGNORE INTO metadata (key, value) VALUES ('undo_position', '0');

-- Index for timestamp-based pruning
CREATE INDEX IF NOT EXISTS idx_undo_log_timestamp ON undo_log(timestamp);
```

**New Aim methods:**

```rust
// core/src/aim.rs additions

pub struct Aim {
    now: Zoned,
    config: Config,
    db: LocalDb,
    short_ids: ShortIds,
    calendar_path: PathBuf,
    undo_manager: UndoManager,  // NEW: embedded undo manager
}

impl Aim {
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn Error>> {
        // ... existing initialization ...

        let undo_manager = UndoManager::load(db.undo_log.clone()).await?;

        Ok(Self {
            now,
            config,
            db,
            short_ids,
            calendar_path,
            undo_manager,  // NEW
        })
    }

    // === Undoable operations ===

    pub async fn new_event_undoable(
        &self,
        draft: EventDraft,
    ) -> Result<(impl Event + 'static, UndoableCommand), Box<dyn Error>> {
        // 1. Generate UID
        // 2. Create command with forward state
        // 3. Execute forward operation
        // 4. Return (event, command)
    }

    pub async fn update_event_undoable(
        &self,
        id: &Id,
        patch: EventPatch,
    ) -> Result<(impl Event + 'static, UndoableCommand), Box<dyn Error>> {
        // 1. Load current event (before state)
        // 2. Apply patch to get after state
        // 3. Create command with before/after
        // 4. Execute update
        // 5. Return (updated_event, command)
    }

    pub async fn delete_event_undoable(
        &self,
        id: &Id,
    ) -> Result<UndoableCommand, Box<dyn Error>> {
        // 1. Load event to delete
        // 2. Create command with deleted state
        // 3. Execute delete
        // 4. Return command
    }

    // Similar methods for todos...

    // === Undo/redo/history methods ===

    /// Record an undoable command to history
    pub fn record_undo(&mut self, command: UndoableCommand) -> Result<(), Box<dyn Error>> {
        self.undo_manager.save(command)
    }

    /// Undo the last operation
    pub async fn undo(&mut self) -> Result<String, Box<dyn Error>> {
        if !self.undo_manager.can_undo() {
            return Err("Nothing to undo".into());
        }

        let command = &self.undo_manager.history()[self.undo_manager.current_position - 1];
        command.backward(self).await?;
        self.undo_manager.current_position -= 1;
        self.undo_manager.persist()?;

        Ok(format!("Undid: {}", command.describe()))
    }

    /// Redo the last undone operation
    pub async fn redo(&mut self) -> Result<String, Box<dyn Error>> {
        if !self.undo_manager.can_redo() {
            return Err("Nothing to redo".into());
        }

        let command = &self.undo_manager.history()[self.undo_manager.current_position];
        command.forward(self).await?;
        self.undo_manager.current_position += 1;
        self.undo_manager.persist()?;

        Ok(format!("Redid: {}", command.describe()))
    }

    /// Get undo history for display
    pub fn history(&self) -> &[UndoableCommand] {
        self.undo_manager.history()
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.undo_manager.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.undo_manager.can_redo()
    }
}
```

**Tests to add:**

- Test forward execution of each operation type
- Test backward execution (undo) of each operation type
- Test batch operations
- Test state consistency (file + DB sync)

### Phase 2: CLI Integration (Simplified - Core owns state)

**Files to create/modify:**

- **NEW**: `cli/src/cmd_undo.rs` - Undo/redo/history command handlers
- **MODIFY**: `cli/src/cli.rs` - Add undo/redo/history subcommands
- **MODIFY**: `cli/src/cmd_event.rs` - Integrate undo tracking
- **MODIFY**: `cli/src/cmd_todo.rs` - Integrate undo tracking
- **MODIFY**: `cli/src/cmd_toplevel.rs` - Batch operation support

**Command integration pattern (much simpler than before):**

```rust
// Example: cli/src/cmd_event.rs

pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
    // Use undoable operation
    let (event, command) = aim.new_event_undoable(draft).await?;

    // Record to Aim's internal history
    aim.record_undo(command)?;

    print_events(aim, &[event], output_format, verbose);
    Ok(())
}
```

**Undo/redo/history commands:**

```rust
// cli/src/cmd_undo.rs

pub struct CmdUndo;
pub struct CmdRedo;
pub struct CmdHistory;

impl CmdUndo {
    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        let msg = aim.undo().await?;
        println!("{}", msg);
        Ok(())
    }
}

impl CmdRedo {
    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        let msg = aim.redo().await?;
        println!("{}", msg);
        Ok(())
    }
}

impl CmdHistory {
    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        let history = aim.history();
        for (i, cmd) in history.iter().enumerate() {
            let marker = if i < aim.undo_manager().current_position() {
                "✓"
            } else {
                " "
            };
            println!("{} {} {}", marker, i, cmd.describe());
        }
        Ok(())
    }
}
```

**New CLI commands:**

```bash
aim undo              # Undo last operation
aim redo              # Redo last undone operation
aim history           # Show undo history
```

### Phase 3: Polish & Features

**Additional features:**

1. **Batch operations** - Group multiple operations as single undo step:

   ```rust
   // cli/src/cmd_toplevel.rs (delay multiple events)
   let mut commands = Vec::new();
   for id in ids {
       let (_, cmd) = aim.update_event_undoable(&id, patch).await?;
       commands.push(cmd);
   }
   let batch = UndoableCommand::batch(commands);
   aim.record_undo(batch)?;
   ```

2. **History pruning** - Keep history size bounded (in core):

   ```rust
   // core/src/aim.rs
   pub async fn prune_undo_history(&mut self, older_than: Duration) -> Result<()> {
       let cutoff = chrono::Utc::now() - older_than;
       self.undo_manager.prune_old(cutoff.timestamp()).await?;
       Ok(())
   }

   pub async fn limit_undo_history(&mut self, max_count: usize) -> Result<()> {
       self.undo_manager.prune_max(max_count).await?;
       Ok(())
   }
   ```

3. **Colored history display** - Enhance `aim history` output

4. **Edge case handling**:
   - File corruption recovery
   - Version skew detection
   - Concurrent instance safety (lock file on undo.log)

5. **Documentation** - Update README and add examples

## Comparison with Alternative Approaches

### Approach 1 (Chosen): Command Pattern + Persistent Log

- ✅ Medium granularity (operations)
- ✅ Persistent storage (JSONL file)
- ✅ Simple architecture
- ✅ Easy to extend
- ✅ O(1) undo/redo

### Approach 2: Memento Pattern + In-Memory Stack

- ❌ Lost on process exit
- ❌ Complex field comparison
- ❌ Doesn't match stateless CLI design
- ✅ Lower memory usage

### Approach 3: Event Sourcing + Database

- ❌ Complex architecture
- ❌ Performance overhead
- ❌ Overkill for single-user app
- ✅ Perfect audit trail
- ✅ Temporal queries

## Critical Files Reference

### Core Layer

- `/Users/yzx9/git/aim/core/src/aim.rs` - Add undoable operation methods + undo/redo/history methods
- `/Users/yzx9/git/aim/core/src/event.rs` - EventPatch state capture
- `/Users/yzx9/git/aim/core/src/todo.rs` - TodoPatch state capture
- `/Users/yzx9/git/aim/core/src/localdb/events.rs` - Event DB operations
- `/Users/yzx9/git/aim/core/src/localdb/todos.rs` - Todo DB operations
- `/Users/yzx9/git/aim/core/src/localdb.rs` - Add undo_log module and field
- **NEW**: `/Users/yzx9/git/aim/core/src/undo.rs` - UndoableCommand, Operation, UndoManager
- **NEW**: `/Users/yzx9/git/aim/core/src/localdb/undo_log.rs` - Database operations for undo log
- **NEW**: `/Users/yzx9/git/aim/core/src/localdb/migrations/YYYYMMDDHHMMSS_add_undo_log.sql` - Database migration

### CLI Layer

- `/Users/yzx9/git/aim/cli/src/cli.rs` - Add undo/redo/history subcommands
- `/Users/yzx9/git/aim/cli/src/cmd_event.rs` - Event command integration
- `/Users/yzx9/git/aim/cli/src/cmd_todo.rs` - Todo command integration
- `/Users/yzx9/git/aim/cli/src/cmd_toplevel.rs` - Batch operations
- **NEW**: `/Users/yzx9/git/aim/cli/src/cmd_undo.rs` - Undo/redo/history command handlers

## Verification

### End-to-End Testing

```bash
# Test 1: Basic undo/redo
aim event new "Meeting" --start "tomorrow 10am"
aim undo              # Should remove "Meeting"
aim redo              # Should restore "Meeting"

# Test 2: Update undo/redo
aim event edit 1 --summary "Updated Meeting"
aim undo              # Should revert to "Meeting"
aim redo              # Should restore "Updated Meeting"

# Test 3: Persistence across CLI invocations
aim event new "Test"
# (restart CLI)
aim undo              # Should work (history persisted)

# Test 4: Batch operations
aim event delay 1 2 3 --duration "1h"
aim undo              # Should undo all 3 delays at once

# Test 5: History display
aim history           # Should show all operations with timestamps
```

### Unit Tests

- Test each operation's forward/backward execution
- Test UndoManager save/load/persist
- Test batch operation undo/redo
- Test state consistency (file + DB)

### Integration Tests

- Test persistence across CLI restarts
- Test concurrent instance detection
- Test corrupted history recovery

## Migration Strategy

**Zero Breaking Changes:**

- Existing `new_event()`, `update_event()` methods remain unchanged
- New `*_undoable()` methods are additions
- Commands opt-in to undo tracking
- UndoManager is internal to Aim (not exposed publicly)
- Backward compatible configuration

```rust
// Old code continues to work
let event = aim.new_event(draft).await?;

// New code with undo support
let (event, command) = aim.new_event_undoable(draft).await?;
aim.record_undo(command)?;
```

## Future Extensibility

Easy to add new operations by extending the `Operation` enum:

```rust
pub enum Operation {
    // ... existing variants

    // Future: CalDAV sync
    CalDAVSync {
        server_url: String,
        before: Vec<VEvent<String>>,
        after: Vec<VEvent<String>>,
    },

    // Future: Recurring events
    RecurringEventCreated {
        series_uid: String,
        instances: Vec<(String, VEvent<String>)>,
    },

    // Future: Bulk import
    BulkImport {
        source: String,
        events: Vec<(String, VEvent<String>)>,
    },
}
```
