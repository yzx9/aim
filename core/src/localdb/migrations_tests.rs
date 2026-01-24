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
// Down Migration Tests
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
