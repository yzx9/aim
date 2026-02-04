// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive database migration tests for the localdb module.
//!
//! This test module validates:
//! - Schema changes from migrations
//! - Migration idempotency
//! - Down migrations
//! - Data preservation during migrations
//! - Edge cases and special scenarios

use std::path::PathBuf;
use std::sync::atomic::Ordering;

use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

use crate::localdb::IN_MEMORY_DB_COUNTER;

/// Creates a database pool without running migrations automatically.
async fn create_pool_without_migrations() -> SqlitePool {
    let db_id = IN_MEMORY_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    let db_name = format!("file:memdb_{db_id}:?mode=memory&cache=shared");

    let conn_opts = SqliteConnectOptions::new()
        .filename(&db_name)
        .in_memory(true)
        .create_if_missing(true);

    SqlitePool::connect_with(conn_opts)
        .await
        .expect("Failed to create in-memory database pool")
}

/// Reads the content of a migration SQL file by name.
fn read_migration_file(name: &str) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let path = PathBuf::from(manifest_dir)
        .join("src")
        .join("localdb")
        .join("migrations")
        .join(name);

    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read migration file {path:?}: {e}"))
}

/// Manually applies a single migration by executing its SQL.
async fn apply_migration(pool: &SqlitePool, migration_name: &str) {
    let up_sql = read_migration_file(&format!("{migration_name}.up.sql"));
    sqlx::query(&up_sql)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to apply migration {migration_name}: {e}"));
}

/// Manually applies a single down migration.
async fn apply_down_migration(pool: &SqlitePool, migration_name: &str) {
    let down_sql = read_migration_file(&format!("{migration_name}.down.sql"));
    sqlx::query(&down_sql)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to apply down migration {migration_name}: {e}"));
}

/// Gets a list of all table names in the database.
async fn get_table_names(pool: &SqlitePool) -> Vec<String> {
    let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .fetch_all(pool)
        .await
        .expect("Failed to query table names");

    rows.iter()
        .map(|row| row.get::<String, _>("name"))
        .collect()
}

/// Gets the SQL used to create a table (for schema validation).
async fn get_table_sql(pool: &SqlitePool, table: &str) -> String {
    let sql = format!("SELECT sql FROM sqlite_master WHERE type='table' AND name='{table}'");
    sqlx::query_scalar::<_, Option<String>>(&sql)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|_| panic!("Table {table} not found"))
        .unwrap_or_default()
}

/// Column information for schema validation.
#[derive(Debug, Clone, PartialEq)]
struct ColumnInfo {
    name: String,
    data_type: String,
    is_pk: bool,
    not_null: bool,
}

/// Gets column information for a table.
async fn get_table_columns(pool: &SqlitePool, table: &str) -> Vec<ColumnInfo> {
    let sql = format!("PRAGMA table_info({table})");
    let rows = sqlx::query(&sql)
        .fetch_all(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to get column info for {table}: {e}"));

    rows.iter()
        .map(|row| ColumnInfo {
            name: row.get("name"),
            data_type: row.get::<String, _>("type"),
            is_pk: row.get::<i64, _>("pk") != 0,
            not_null: row.get::<i64, _>("notnull") != 0,
        })
        .collect()
}

/// Checks if a table was created with AUTOINCREMENT.
async fn has_autoincrement(pool: &SqlitePool, table: &str) -> bool {
    let sql = get_table_sql(pool, table).await;
    sql.contains("AUTOINCREMENT")
}

/// Asserts that a table exists in the database.
async fn assert_table_exists(pool: &SqlitePool, table: &str) {
    let tables = get_table_names(pool).await;
    assert!(
        tables.contains(&table.to_string()),
        "Table '{table}' should exist but was not found. Available tables: {tables:?}"
    );
}

/// Asserts that a table does not exist in the database.
async fn assert_table_not_exists(pool: &SqlitePool, table: &str) {
    let tables = get_table_names(pool).await;
    assert!(
        !tables.contains(&table.to_string()),
        "Table '{table}' should not exist but was found. Available tables: {tables:?}"
    );
}

/// Asserts that a table has AUTOINCREMENT on its primary key.
async fn assert_has_autoincrement(pool: &SqlitePool, table: &str) {
    assert!(
        has_autoincrement(pool, table).await,
        "Table '{table}' should have AUTOINCREMENT but does not"
    );
}

/// Asserts that a table does not have AUTOINCREMENT on its primary key.
async fn assert_no_autoincrement(pool: &SqlitePool, table: &str) {
    assert!(
        !has_autoincrement(pool, table).await,
        "Table '{table}' should not have AUTOINCREMENT but does"
    );
}

/// Gets row count for a table.
async fn get_row_count(pool: &SqlitePool, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    sqlx::query_scalar(&sql)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to get row count for {table}: {e}"))
}

// =============================================================================
// Schema Validation Tests
// =============================================================================

#[tokio::test]
async fn migrations_init_events_todos_creates_tables() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    assert_table_exists(&pool, "events").await;
    let events_columns = get_table_columns(&pool, "events").await;
    assert_eq!(
        events_columns.len(),
        7,
        "events table should have 7 columns"
    );

    let event_col_names: Vec<_> = events_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(event_col_names.contains(&"uid"));
    assert!(event_col_names.contains(&"path"));
    assert!(event_col_names.contains(&"summary"));
    assert!(event_col_names.contains(&"description"));
    assert!(event_col_names.contains(&"status"));
    assert!(event_col_names.contains(&"start"));
    assert!(event_col_names.contains(&"end"));

    assert_table_exists(&pool, "todos").await;
    let todos_columns = get_table_columns(&pool, "todos").await;
    assert_eq!(todos_columns.len(), 9, "todos table should have 9 columns");

    let todo_col_names: Vec<_> = todos_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(todo_col_names.contains(&"uid"));
    assert!(todo_col_names.contains(&"path"));
    assert!(todo_col_names.contains(&"completed"));
    assert!(todo_col_names.contains(&"description"));
    assert!(todo_col_names.contains(&"percent"));
    assert!(todo_col_names.contains(&"priority"));
    assert!(todo_col_names.contains(&"status"));
    assert!(todo_col_names.contains(&"summary"));
    assert!(todo_col_names.contains(&"due"));
}

#[tokio::test]
async fn migrations_add_short_ids_creates_table() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250801095832_add_short_ids").await;

    assert_table_exists(&pool, "short_ids").await;

    let columns = get_table_columns(&pool, "short_ids").await;
    assert_eq!(columns.len(), 3, "short_ids table should have 3 columns");

    let col_names: Vec<_> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(col_names.contains(&"short_id"));
    assert!(col_names.contains(&"uid"));
    assert!(col_names.contains(&"kind"));

    assert_has_autoincrement(&pool, "short_ids").await;
}

#[tokio::test]
async fn migrations_drop_autoincrement_removes_autoincrement() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250801095832_add_short_ids").await;
    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    assert_table_exists(&pool, "short_ids").await;
    assert_no_autoincrement(&pool, "short_ids").await;
}

// =============================================================================
// Idempotency Tests
// =============================================================================

#[tokio::test]
async fn migrations_all_idempotent() {
    let pool = create_pool_without_migrations().await;

    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250801095832_add_short_ids").await;
    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    sqlx::query(
        "INSERT INTO events (uid, path, summary, description, status, start, end) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("test-event-1")
    .bind("/path/to/event.ics")
    .bind("Test Event")
    .bind("Description")
    .bind("confirmed")
    .bind("2025-01-01T00:00:00Z")
    .bind("2025-01-01T01:00:00Z")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
        .bind("test-event-1")
        .bind("event")
        .execute(&pool)
        .await
        .unwrap();

    let tables_before = get_table_names(&pool).await;
    let events_count_before = get_row_count(&pool, "events").await;
    let short_ids_count_before = get_row_count(&pool, "short_ids").await;

    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    let tables_after = get_table_names(&pool).await;
    let events_count_after = get_row_count(&pool, "events").await;
    let short_ids_count_after = get_row_count(&pool, "short_ids").await;

    assert_eq!(
        tables_before, tables_after,
        "Table list should be unchanged"
    );
    assert_eq!(
        events_count_before, events_count_after,
        "Events count should be unchanged"
    );
    assert_eq!(
        short_ids_count_before, short_ids_count_after,
        "Short IDs count should be unchanged"
    );
}

#[tokio::test]
async fn migrations_init_idempotent() {
    let pool = create_pool_without_migrations().await;

    apply_migration(&pool, "20250801070804_init_events_todos").await;
    let tables_after_first = get_table_names(&pool).await;

    apply_migration(&pool, "20250801070804_init_events_todos").await;
    let tables_after_second = get_table_names(&pool).await;

    assert_eq!(
        tables_after_first, tables_after_second,
        "CREATE IF NOT EXISTS should make migration idempotent"
    );
}

// =============================================================================
// ICS Optional Migration Tests
// =============================================================================

#[tokio::test]
async fn migrations_ics_optional_creates_resources_table() {
    let pool = create_pool_without_migrations().await;

    // Apply init migration first to create events/todos tables
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    // Apply migration
    apply_migration(&pool, "20260131235400_ics_optional").await;

    // Verify backend_kind column was added to events table
    let events_columns = get_table_columns(&pool, "events").await;
    assert_eq!(events_columns.len(), 7);
    assert!(events_columns.iter().any(|c| c.name == "backend_kind"));
    assert!(
        events_columns
            .iter()
            .filter(|c| c.name == "backend_kind")
            .all(|c| c.not_null)
    );

    // Verify backend_kind column was added to todos table
    let todos_columns = get_table_columns(&pool, "todos").await;
    assert_eq!(todos_columns.len(), 9);
    assert!(todos_columns.iter().any(|c| c.name == "backend_kind"));
    assert!(
        todos_columns
            .iter()
            .filter(|c| c.name == "backend_kind")
            .all(|c| c.not_null)
    );

    // Drop test database
}

#[tokio::test]
async fn migrations_ics_optional_migrates_existing_path_data() {
    let pool = create_pool_without_migrations().await;

    // Apply init migration first to create tables with path column
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    // Add test event and todo
    sqlx::query(
            "INSERT INTO events (uid, path, summary, description, status, start, end) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("test-event-1")
        .bind("/path/to/test-event-1.ics")
        .bind("Test Event 1")
        .bind("")
        .bind("confirmed")
        .bind("2025-01-01T00:00:00Z")
        .bind("2025-01-01T01:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
            "INSERT INTO todos (uid, path, completed, description, percent, priority, status, summary, due) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("test-todo-1")
        .bind("/path/to/test-todo-1.ics")
        .bind("")
        .bind("")
        .bind(0)
        .bind(0)
        .bind("needs-action")
        .bind("Test Todo 1")
        .bind("2025-01-01T10:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    // Apply migration
    apply_migration(&pool, "20260131235400_ics_optional").await;

    // Verify data was migrated to resources table
    let event_resource = sqlx::query_as::<_, (String, i8, String)>(
        "SELECT uid, backend_kind, resource_id FROM resources WHERE uid = ? AND backend_kind = 0",
    )
    .bind("test-event-1")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        event_resource,
        (
            "test-event-1".to_string(),
            0i8,
            "file:///path/to/test-event-1.ics".to_string()
        )
    );

    let todo_resource = sqlx::query_as::<_, (String, i8, String)>(
        "SELECT uid, backend_kind, resource_id FROM resources WHERE uid = ? AND backend_kind = 0",
    )
    .bind("test-todo-1")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        todo_resource,
        (
            "test-todo-1".to_string(),
            0i8,
            "file:///path/to/test-todo-1.ics".to_string()
        )
    );

    // Drop test database
}

#[tokio::test]
async fn migrations_ics_optional_removes_path_columns() {
    let pool = create_pool_without_migrations().await;

    // Apply init migration first to create tables with path column
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    // Add test data with path column
    sqlx::query(
            "INSERT INTO events (uid, path, summary, description, status, start, end) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("test-event-2")
        .bind("/path/to/test-event-2.ics")
        .bind("Test Event 2")
        .bind("")
        .bind("confirmed")
        .bind("2025-01-01T00:00:00Z")
        .bind("2025-01-01T01:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
            "INSERT INTO todos (uid, path, completed, description, percent, priority, status, summary, due) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("test-todo-2")
        .bind("/path/to/test-todo-2.ics")
        .bind("")
        .bind("")
        .bind(0)
        .bind(0)
        .bind("needs-action")
        .bind("Test Todo 2")
        .bind("2025-01-01T10:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    // Apply migration
    apply_migration(&pool, "20260131235400_ics_optional").await;

    // Verify path column was removed from events table
    let events_columns = get_table_columns(&pool, "events").await;
    assert_eq!(events_columns.len(), 7); // Should be 7 columns (removed path, added backend_kind)

    let event_col_names: Vec<_> = events_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(!event_col_names.contains(&"path"));
    assert!(event_col_names.contains(&"backend_kind"));

    // Verify path column was removed from todos table
    let todos_columns = get_table_columns(&pool, "todos").await;
    assert_eq!(todos_columns.len(), 9); // Should be 9 columns (removed path, added backend_kind)

    let todo_col_names: Vec<_> = todos_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(!todo_col_names.contains(&"path"));
    assert!(todo_col_names.contains(&"backend_kind"));

    // Verify data still exists (migration should preserve all event/todo data)
    let event_count = get_row_count(&pool, "events").await;
    assert_eq!(event_count, 1);

    let todo_count = get_row_count(&pool, "todos").await;
    assert_eq!(todo_count, 1);

    // Drop test database
}

#[tokio::test]
async fn migrations_ics_optional_idempotency_preserves_data() {
    let pool = create_pool_without_migrations().await;

    // Apply init migration first to create tables with path column
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    // Add test event and todo
    sqlx::query(
            "INSERT INTO events (uid, path, summary, description, status, start, end) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("test-event-3")
        .bind("/path/to/test-event-3.ics")
        .bind("Test Event 3")
        .bind("")
        .bind("confirmed")
        .bind("2025-01-01T00:00:00Z")
        .bind("2025-01-01T01:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
            "INSERT INTO todos (uid, path, completed, description, percent, priority, status, summary, due) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("test-todo-3")
        .bind("/path/to/test-todo-3.ics")
        .bind("")
        .bind("")
        .bind(0)
        .bind(0)
        .bind("needs-action")
        .bind("Test Todo 3")
        .bind("2025-01-01T10:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    // Apply migration
    apply_migration(&pool, "20260131235400_ics_optional").await;

    // Verify all data preserved in events
    let events_columns = get_table_columns(&pool, "events").await;
    assert_eq!(events_columns.len(), 7);

    let event_col_names: Vec<_> = events_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(event_col_names.contains(&"uid"));
    assert!(event_col_names.contains(&"summary"));
    assert!(event_col_names.contains(&"description"));
    assert!(event_col_names.contains(&"status"));
    assert!(event_col_names.contains(&"start"));
    assert!(event_col_names.contains(&"end"));
    assert!(event_col_names.contains(&"backend_kind"));

    let event_count = get_row_count(&pool, "events").await;
    assert_eq!(event_count, 1);

    // Verify resources table has the migrated data
    let event_resource = sqlx::query_as::<_, (String, i8, String)>(
        "SELECT uid, backend_kind, resource_id FROM resources WHERE uid = ? AND backend_kind = 0",
    )
    .bind("test-event-3")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        event_resource,
        (
            "test-event-3".to_string(),
            0i8,
            "file:///path/to/test-event-3.ics".to_string()
        )
    );

    // Verify all data preserved in todos
    let todos_columns = get_table_columns(&pool, "todos").await;
    assert_eq!(todos_columns.len(), 9);

    let todo_col_names: Vec<_> = todos_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(todo_col_names.contains(&"uid"));
    assert!(todo_col_names.contains(&"summary"));
    assert!(todo_col_names.contains(&"description"));
    assert!(todo_col_names.contains(&"status"));
    assert!(todo_col_names.contains(&"priority"));
    assert!(todo_col_names.contains(&"due"));
    assert!(todo_col_names.contains(&"completed"));
    assert!(todo_col_names.contains(&"backend_kind"));

    let todo_count = get_row_count(&pool, "todos").await;
    assert_eq!(todo_count, 1);

    // Verify resources table has the migrated data
    let todo_resource = sqlx::query_as::<_, (String, i8, String)>(
        "SELECT uid, backend_kind, resource_id FROM resources WHERE uid = ? AND backend_kind = 0",
    )
    .bind("test-todo-3")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        todo_resource,
        (
            "test-todo-3".to_string(),
            0i8,
            "file:///path/to/test-todo-3.ics".to_string()
        )
    );

    // Drop test database
}

#[tokio::test]
async fn migrations_ics_optional_creates_index() {
    let pool = create_pool_without_migrations().await;

    // Apply init migration first
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    // Apply migration
    apply_migration(&pool, "20260131235400_ics_optional").await;

    // Verify index was created
    let indexes =
        sqlx::query("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='resources'")
            .fetch_all(&pool)
            .await
            .unwrap();

    assert_eq!(indexes.len(), 2);

    let index_names: Vec<_> = indexes.iter().map(|i| i.get::<_, _>("name")).collect();
    assert!(index_names.contains(&"idx_resources_backend_kind"));

    // Drop test database
}

#[tokio::test]
async fn migrations_ics_optional_full_cycle() {
    let pool = create_pool_without_migrations().await;

    // Apply init migration first
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    // Add test data BEFORE up migration (with path column)
    sqlx::query(
            "INSERT INTO events (uid, path, summary, description, status, start, end) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("full-cycle-event")
        .bind("/path/to/full-cycle.ics")
        .bind("Full Cycle Event")
        .bind("")
        .bind("confirmed")
        .bind("2025-01-01T00:00:00Z")
        .bind("2025-01-01T01:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
            "INSERT INTO todos (uid, path, completed, description, percent, priority, status, summary, due) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("full-cycle-todo")
        .bind("/path/to/full-cycle.ics")
        .bind("")
        .bind("")
        .bind(0)
        .bind(0)
        .bind("needs-action")
        .bind("Full Cycle Todo")
        .bind("2025-01-01T10:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    // Apply up migration
    apply_migration(&pool, "20260131235400_ics_optional").await;

    // Verify changes
    let tables = get_table_names(&pool).await;
    assert!(tables.iter().any(|t| t == "resources"));
    assert!(tables.iter().any(|t| t == "events"));
    assert!(tables.iter().any(|t| t == "todos"));

    // Verify resources table
    let columns = get_table_columns(&pool, "resources").await;
    assert_eq!(columns.len(), 4);
    assert!(columns.iter().any(|c| c.name == "uid"));
    assert!(columns.iter().any(|c| c.name == "backend_kind"));
    assert!(columns.iter().any(|c| c.name == "resource_id"));
    assert!(columns.iter().any(|c| c.name == "metadata"));

    // Verify backend_kind in events
    let events_columns = get_table_columns(&pool, "events").await;
    assert!(events_columns.iter().any(|c| c.name == "backend_kind"));
    let col_names: Vec<_> = events_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(col_names.contains(&"uid"));
    assert!(col_names.contains(&"backend_kind"));

    // Verify backend_kind in todos
    let todos_columns = get_table_columns(&pool, "todos").await;
    assert!(todos_columns.iter().any(|c| c.name == "backend_kind"));
    let todo_col_names: Vec<_> = todos_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(todo_col_names.contains(&"uid"));
    assert!(todo_col_names.contains(&"backend_kind"));

    // Verify data migrated to resources
    let event_resource = sqlx::query_as::<_, (String, i8, String)>(
        "SELECT uid, backend_kind, resource_id FROM resources WHERE uid = ? AND backend_kind = 0",
    )
    .bind("full-cycle-event")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        event_resource,
        (
            "full-cycle-event".to_string(),
            0i8,
            "file:///path/to/full-cycle.ics".to_string()
        )
    );

    let todo_resource = sqlx::query_as::<_, (String, i8, String)>(
        "SELECT uid, backend_kind, resource_id FROM resources WHERE uid = ? AND backend_kind = 0",
    )
    .bind("full-cycle-todo")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        todo_resource,
        (
            "full-cycle-todo".to_string(),
            0i8,
            "file:///path/to/full-cycle.ics".to_string()
        )
    );

    // Apply down migration
    apply_down_migration(&pool, "20260131235400_ics_optional").await;

    // Verify path columns are restored
    let events_columns = get_table_columns(&pool, "events").await;
    assert_eq!(events_columns.len(), 7); // Should have 7 columns again (original structure)

    let event_col_names: Vec<_> = events_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(event_col_names.contains(&"path"));
    assert!(!event_col_names.contains(&"backend_kind")); // backend_kind should be preserved

    let todos_columns = get_table_columns(&pool, "todos").await;
    assert_eq!(todos_columns.len(), 9); // Should have 9 columns again (original structure)

    let todos_column_names: Vec<_> = todos_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(todos_column_names.contains(&"path"));
    assert!(!todos_column_names.contains(&"backend_kind")); // backend_kind should be preserved

    // Verify resources table was dropped
    let tables = get_table_names(&pool).await;
    assert!(!tables.iter().any(|t| t == "resources"));

    // Drop test database
}

// =============================================================================
// Down Migration Tests
// =============================================================================

#[tokio::test]
async fn migrations_ics_optional_down_restores_path() {
    let pool = create_pool_without_migrations().await;

    // Create tables with path column
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    // Add test data
    sqlx::query(
            "INSERT INTO events (uid, path, summary, description, status, start, end) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("down-event-1")
        .bind("/down-event-1")
        .bind("Down Event 1")
        .bind("")
        .bind("confirmed")
        .bind("2025-01-01T00:00:00Z")
        .bind("2025-01-01T01:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
            "INSERT INTO todos (uid, path, completed, description, percent, priority, status, summary, due) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("down-todo-1")
        .bind("/down-todo-1")
        .bind("")
        .bind("")
        .bind(0)
        .bind(0)
        .bind("needs-action")
        .bind("Down Todo 1")
        .bind("2025-01-01T10:00:00Z")
        .execute(&pool)
        .await
        .unwrap();

    // Apply up migration first (to get to state where down migration works)
    apply_migration(&pool, "20260131235400_ics_optional").await;

    // Verify up migration worked
    let tables = get_table_names(&pool).await;
    assert!(tables.iter().any(|t| t == "resources"));

    let events_columns = get_table_columns(&pool, "events").await;
    assert!(!events_columns.iter().any(|c| c.name == "path"));
    assert!(events_columns.iter().any(|c| c.name == "backend_kind"));

    let todos_columns = get_table_columns(&pool, "todos").await;
    assert!(!todos_columns.iter().any(|c| c.name == "path"));
    assert!(todos_columns.iter().any(|c| c.name == "backend_kind"));

    // Apply down migration
    apply_down_migration(&pool, "20260131235400_ics_optional").await;

    // Verify path columns are restored
    let events_columns = get_table_columns(&pool, "events").await;
    assert!(events_columns.iter().any(|c| c.name == "path"));
    assert!(!events_columns.iter().any(|c| c.name == "backend_kind"));

    let todos_columns = get_table_columns(&pool, "todos").await;
    assert!(todos_columns.iter().any(|c| c.name == "path"));
    assert!(!todos_columns.iter().any(|c| c.name == "backend_kind"));

    // Verify resources table was dropped
    let tables = get_table_names(&pool).await;
    assert!(!tables.iter().any(|t| t == "resources"));

    // Verify data still exists (down migration should preserve all event/todo data)
    let event_count = get_row_count(&pool, "events").await;
    assert_eq!(event_count, 1);

    let todo_count = get_row_count(&pool, "todos").await;
    assert_eq!(todo_count, 1);

    // Drop test database
}

#[tokio::test]
async fn migrations_ics_optional_down_drops_resources_table() {
    let pool = create_pool_without_migrations().await;

    // Apply init migration first
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    // Apply up migration (to get to state where down migration works)
    apply_migration(&pool, "20260131235400_ics_optional").await;

    // Apply down migration
    apply_down_migration(&pool, "20260131235400_ics_optional").await;

    // Verify resources table was dropped
    let tables = get_table_names(&pool).await;
    assert!(!tables.iter().any(|t| t == "resources"));

    // Verify events table exists without backend_kind
    let events_columns = get_table_columns(&pool, "events").await;
    assert_eq!(events_columns.len(), 7); // Should be 7 columns (original structure)

    let events_col_names: Vec<_> = events_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(events_col_names.contains(&"uid"));
    assert!(events_col_names.contains(&"path"));
    assert!(!events_col_names.contains(&"backend_kind"));

    // Verify todos table exists without backend_kind
    let todos_columns = get_table_columns(&pool, "todos").await;
    assert_eq!(todos_columns.len(), 9); // Should be 9 columns (original structure)

    let todos_column_names: Vec<_> = todos_columns.iter().map(|c| c.name.as_str()).collect();
    assert!(todos_column_names.contains(&"uid"));
    assert!(todos_column_names.contains(&"path"));
    assert!(!todos_column_names.contains(&"backend_kind"));

    // Drop test database
}

// =============================================================================
// Data Preservation Tests (ensure no data loss during migration)
// =============================================================================

#[tokio::test]
async fn migrations_add_short_ids_down_reverts_schema() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    apply_migration(&pool, "20250801095832_add_short_ids").await;
    assert_table_exists(&pool, "short_ids").await;

    apply_down_migration(&pool, "20250801095832_add_short_ids").await;
    assert_table_not_exists(&pool, "short_ids").await;

    assert_table_exists(&pool, "events").await;
    assert_table_exists(&pool, "todos").await;
}

#[tokio::test]
async fn migrations_drop_autoincrement_down_restores_autoincrement() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    apply_migration(&pool, "20250801095832_add_short_ids").await;
    assert_has_autoincrement(&pool, "short_ids").await;

    apply_migration(&pool, "20250805075731_drop_autoincrement").await;
    assert_no_autoincrement(&pool, "short_ids").await;

    apply_down_migration(&pool, "20250805075731_drop_autoincrement").await;
    assert_has_autoincrement(&pool, "short_ids").await;
}

#[tokio::test]
async fn migrations_init_down_drops_tables() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    assert_table_exists(&pool, "events").await;
    assert_table_exists(&pool, "todos").await;

    apply_down_migration(&pool, "20250801070804_init_events_todos").await;

    assert_table_not_exists(&pool, "events").await;
    assert_table_not_exists(&pool, "todos").await;
}

#[tokio::test]
async fn migrations_full_down_sequence() {
    let pool = create_pool_without_migrations().await;

    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250801095832_add_short_ids").await;
    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
        .bind("test-uid")
        .bind("event")
        .execute(&pool)
        .await
        .unwrap();

    assert!(get_row_count(&pool, "short_ids").await > 0);

    apply_down_migration(&pool, "20250805075731_drop_autoincrement").await;
    apply_down_migration(&pool, "20250801095832_add_short_ids").await;
    apply_down_migration(&pool, "20250801070804_init_events_todos").await;

    assert_table_not_exists(&pool, "events").await;
    assert_table_not_exists(&pool, "todos").await;
    assert_table_not_exists(&pool, "short_ids").await;
}

// =============================================================================
// Data Migration Tests
// =============================================================================

#[tokio::test]
async fn migrations_drop_autoincrement_preserves_data() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250801095832_add_short_ids").await;

    sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
        .bind("uid-1")
        .bind("event")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
        .bind("uid-2")
        .bind("todo")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
        .bind("uid-3")
        .bind("event")
        .execute(&pool)
        .await
        .unwrap();

    let before: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT short_id, uid, kind FROM short_ids ORDER BY short_id")
            .fetch_all(&pool)
            .await
            .unwrap();

    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    let after: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT short_id, uid, kind FROM short_ids ORDER BY short_id")
            .fetch_all(&pool)
            .await
            .unwrap();

    assert_eq!(
        before, after,
        "All data should be preserved after migration"
    );
    assert_eq!(after.len(), 3, "Should have 3 rows");
    #[allow(clippy::indexing_slicing)]
    {
        assert_eq!(after[0].1, "uid-1");
        assert_eq!(after[1].1, "uid-2");
        assert_eq!(after[2].1, "uid-3");
    }
}

#[tokio::test]
async fn migrations_drop_autoincrement_handles_empty_table() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250801095832_add_short_ids").await;

    let count_before = get_row_count(&pool, "short_ids").await;
    assert_eq!(count_before, 0, "Table should be empty");

    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    assert_table_exists(&pool, "short_ids").await;
    let count_after = get_row_count(&pool, "short_ids").await;
    assert_eq!(count_after, 0, "Table should still be empty");
    assert_no_autoincrement(&pool, "short_ids").await;
}

#[tokio::test]
async fn migrations_add_short_ids_to_drop_autoincrement_roundtrip() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;

    apply_migration(&pool, "20250801095832_add_short_ids").await;

    sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
        .bind("test-uid")
        .bind("event")
        .execute(&pool)
        .await
        .unwrap();

    let data_at_add: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT short_id, uid, kind FROM short_ids")
            .fetch_all(&pool)
            .await
            .unwrap();

    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    let data_at_drop: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT short_id, uid, kind FROM short_ids")
            .fetch_all(&pool)
            .await
            .unwrap();

    apply_down_migration(&pool, "20250805075731_drop_autoincrement").await;

    let data_after_revert: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT short_id, uid, kind FROM short_ids")
            .fetch_all(&pool)
            .await
            .unwrap();

    assert_eq!(
        data_at_add, data_at_drop,
        "Data should match after drop_autoincrement"
    );
    assert_eq!(
        data_at_add, data_after_revert,
        "Data should match after reverting drop_autoincrement"
    );
}

#[tokio::test]
async fn migrations_preserves_all_data_types() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250801095832_add_short_ids").await;
    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    sqlx::query(
        "INSERT INTO events (uid, path, summary, description, status, start, end) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("event-1")
    .bind("/path/to/event.ics")
    .bind("Event Summary")
    .bind("Event Description")
    .bind("confirmed")
    .bind("2025-01-15T10:00:00Z")
    .bind("2025-01-15T11:00:00Z")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO todos (uid, path, summary, description, status, priority, \
         percent, due, completed) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("todo-1")
    .bind("/path/to/todo.ics")
    .bind("Todo Summary")
    .bind("Todo Description")
    .bind("needs-action")
    .bind(5)
    .bind(50)
    .bind("2025-01-20T00:00:00Z")
    .bind("")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
        .bind("event-1")
        .bind("event")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
        .bind("todo-1")
        .bind("todo")
        .execute(&pool)
        .await
        .unwrap();

    let events_before: Vec<(String, String, String)> =
        sqlx::query_as("SELECT uid, summary, status FROM events")
            .fetch_all(&pool)
            .await
            .unwrap();

    let todos_before: Vec<(String, String, i64)> =
        sqlx::query_as("SELECT uid, summary, priority FROM todos")
            .fetch_all(&pool)
            .await
            .unwrap();

    apply_down_migration(&pool, "20250805075731_drop_autoincrement").await;
    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    let events_after: Vec<(String, String, String)> =
        sqlx::query_as("SELECT uid, summary, status FROM events")
            .fetch_all(&pool)
            .await
            .unwrap();

    let todos_after: Vec<(String, String, i64)> =
        sqlx::query_as("SELECT uid, summary, priority FROM todos")
            .fetch_all(&pool)
            .await
            .unwrap();

    assert_eq!(events_before, events_after, "Events should be preserved");
    assert_eq!(todos_before, todos_after, "Todos should be preserved");
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[tokio::test]
async fn migrations_handles_special_characters_in_data() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250801095832_add_short_ids").await;

    let test_cases = vec![
        ("uid-with'quote", "event"),
        ("uid-with\"double-quote", "todo"),
        ("uid-with-semicolon-;-", "event"),
        ("uid-with-newline-\n-escape", "todo"),
        ("uid-with-emoji-🎉", "event"),
        ("uid-with-中文字符", "todo"),
        ("uid-with-backslash-\\-", "event"),
    ];

    for (uid, kind) in &test_cases {
        sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
            .bind(uid)
            .bind(kind)
            .execute(&pool)
            .await
            .unwrap();
    }

    let before: Vec<(String, String)> =
        sqlx::query_as("SELECT uid, kind FROM short_ids ORDER BY short_id")
            .fetch_all(&pool)
            .await
            .unwrap();

    apply_migration(&pool, "20250805075731_drop_autoincrement").await;

    let after: Vec<(String, String)> =
        sqlx::query_as("SELECT uid, kind FROM short_ids ORDER BY short_id")
            .fetch_all(&pool)
            .await
            .unwrap();

    assert_eq!(
        before, after,
        "Special characters should be preserved correctly"
    );
    assert_eq!(after.len(), test_cases.len());
}

#[tokio::test]
async fn migrations_large_dataset_performance() {
    let pool = create_pool_without_migrations().await;
    apply_migration(&pool, "20250801070804_init_events_todos").await;
    apply_migration(&pool, "20250801095832_add_short_ids").await;

    let batch_size = 1000;
    for i in 0..batch_size {
        sqlx::query("INSERT INTO short_ids (uid, kind) VALUES (?, ?)")
            .bind(format!("uid-{i:05}"))
            .bind(if i % 2 == 0 { "event" } else { "todo" })
            .execute(&pool)
            .await
            .unwrap();
    }

    let count_before = get_row_count(&pool, "short_ids").await;
    assert_eq!(count_before, batch_size);

    let start = std::time::Instant::now();
    apply_migration(&pool, "20250805075731_drop_autoincrement").await;
    let duration = start.elapsed();

    let count_after = get_row_count(&pool, "short_ids").await;

    assert_eq!(count_before, count_after, "All rows should be preserved");

    let first_row: (i64, String, String) =
        sqlx::query_as("SELECT short_id, uid, kind FROM short_ids WHERE uid = ?")
            .bind("uid-00000")
            .fetch_one(&pool)
            .await
            .unwrap();

    let last_uid = batch_size - 1;
    let last_row: (i64, String, String) =
        sqlx::query_as("SELECT short_id, uid, kind FROM short_ids WHERE uid = ?")
            .bind(format!("uid-{last_uid:05}"))
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(first_row.1, "uid-00000");
    assert_eq!(first_row.2, "event");
    assert_eq!(last_row.1, format!("uid-{last_uid:05}"));

    println!("Migration of {batch_size} rows took: {duration:?}");
}
