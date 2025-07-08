use crate::{
    Event, Todo, TodoStatus,
    aim::{EventQuery, Pager, TodoQuery},
};
use icalendar::{
    Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime, EventStatus,
};
use sqlx::sqlite::SqlitePool;
use std::path::{Path, PathBuf};

pub struct SqliteCache {
    pool: SqlitePool,
}

impl SqliteCache {
    pub async fn new(calendar_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = new_db()
            .await
            .map_err(|e| format!("Failed to initialize database: {}", e.to_string()))?;

        let mut reader = tokio::fs::read_dir(calendar_path)
            .await
            .map_err(|e| format!("Failed to read directory: {}", e.to_string()))?;

        let mut handles = vec![];
        let mut count_ics = 0;

        while let Some(entry) = reader.next_entry().await? {
            let path = entry.path();
            match path.extension() {
                Some(ext) if ext == "ics" => {
                    count_ics += 1;
                    let pool_clone = pool.clone();
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = process_ics_file(&path, &pool_clone).await {
                            log::error!("Failed to process file {}: {}", path.display(), e);
                        }
                    }));
                }
                _ => {}
            }
        }

        for handle in handles {
            handle.await?;
        }

        log::debug!("Total .ics files processed: {}", count_ics);
        Ok(SqliteCache { pool })
    }

    pub async fn list_events(
        &self,
        query: &EventQuery,
        pager: &Pager,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM events ORDER BY id LIMIT ? OFFSET ?")
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count_events(&self, query: &EventQuery) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn list_todos(
        &self,
        query: &TodoQuery,
        pager: &Pager,
    ) -> Result<Vec<TodoRecord>, sqlx::Error> {
        let mut sql = "SELECT * FROM todos".to_string();
        let mut where_clauses: Vec<&str> = Vec::new();

        if query.status().is_some() {
            where_clauses.push("status = ?");
        }
        if query.due().is_some() {
            where_clauses.push("due_at <= ?");
        }

        if !where_clauses.is_empty() {
            sql += " WHERE ";
            sql += &where_clauses.join(" AND ");
        }

        sql += " ORDER BY id LIMIT ? OFFSET ?";

        let mut executable = sqlx::query_as(&sql);
        if let Some(s) = query.status() {
            let status: &str = s.into();
            executable = executable.bind(status);
        }
        if let Some(due_at) = query.due() {
            executable = executable.bind(due_at.to_rfc3339());
        }

        executable
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count_todos(&self, query: &TodoQuery) -> Result<i64, sqlx::Error> {
        let mut sql = "SELECT COUNT(*) FROM todos".to_string();
        let mut where_clauses: Vec<&str> = Vec::new();

        if query.status().is_some() {
            where_clauses.push("status = ?");
        }
        if query.due().is_some() {
            where_clauses.push("due_at <= ?");
        }

        if !where_clauses.is_empty() {
            sql += " WHERE ";
            sql += &where_clauses.join(" AND ");
        }

        let mut executable = sqlx::query_as(&sql);
        if let Some(s) = query.status() {
            let status: &str = s.into();
            executable = executable.bind(status);
        }
        if let Some(due_at) = query.due() {
            executable = executable.bind(due_at.to_rfc3339());
        }
        let row: (i64,) = executable.fetch_one(&self.pool).await?;
        Ok(row.0)
    }
}

#[derive(sqlx::FromRow)]
pub struct EventRecord {
    id: i64,
    summary: String,
    description: Option<String>,
    start_at: Option<String>,
    start_has_time: bool,
    end_at: Option<String>,
    end_has_time: bool,
    status: Option<String>,
}

impl EventRecord {
    const fn sql_create_table() -> &'static str {
        "
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    summary TEXT,
    description TEXT,
    start_at TEXT,
    start_has_time BOOLEAN,
    end_at TEXT,
    end_has_time BOOLEAN,
    status TEXT
);
    "
    }
}

impl Event for EventRecord {
    fn id(&self) -> i64 {
        self.id
    }

    fn summary(&self) -> &str {
        &self.summary
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn start_at(&self) -> Option<&str> {
        self.start_at.as_deref()
    }

    fn start_has_time(&self) -> bool {
        self.start_has_time
    }

    fn end_at(&self) -> Option<&str> {
        self.end_at.as_deref()
    }

    fn end_has_time(&self) -> bool {
        self.end_has_time
    }

    fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
}

#[derive(sqlx::FromRow)]
pub struct TodoRecord {
    id: i64,
    summary: String,
    description: Option<String>,
    due_at: Option<String>,
    due_has_time: bool,
    completed: Option<String>,
    percent_complete: Option<i64>,
    status: Option<String>,
}

impl TodoRecord {
    const fn sql_create_table() -> &'static str {
        "
CREATE TABLE todos (
    id INTEGER PRIMARY KEY,
    summary TEXT,
    description TEXT,
    due_at TEXT,
    due_has_time BOOLEAN,
    completed TEXT,
    percent_complete INTEGER,
    status TEXT
);
    "
    }
}

impl Todo for TodoRecord {
    fn id(&self) -> i64 {
        self.id
    }

    fn summary(&self) -> &str {
        &self.summary
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn due_at(&self) -> Option<&str> {
        self.due_at.as_deref()
    }

    fn due_has_time(&self) -> bool {
        self.due_has_time
    }

    fn completed(&self) -> Option<&str> {
        self.completed.as_deref()
    }

    fn percent_complete(&self) -> Option<i64> {
        self.percent_complete
    }

    fn status(&self) -> Option<TodoStatus> {
        self.status.as_ref()?.as_str().try_into().ok()
    }
}

/**
 * Database
 */

async fn new_db() -> Result<SqlitePool, sqlx::Error> {
    // Open an in-memory SQLite database connection pool
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    log::debug!("In-memory SQLite DB opened.");
    let mut tx = pool.begin().await?;

    sqlx::query(EventRecord::sql_create_table())
        .execute(&mut *tx)
        .await?;

    sqlx::query(TodoRecord::sql_create_table())
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    log::debug!("Tables created.");
    Ok(pool)
}

async fn process_ics_file(
    path: &Path,
    pool: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Parsing file: {}", path.display());

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    let calendar: Calendar = content.parse()?;
    log::debug!(
        "Found {} components in {}.",
        calendar.components.len(),
        path.display()
    );

    for component in calendar.components {
        match component {
            CalendarComponent::Event(event) => insert_event(pool, &event).await?,
            CalendarComponent::Todo(todo) => insert_todo(pool, &todo).await?,
            _ => log::warn!("Ignoring unsupported component type: {:?}", component),
        }
    }

    Ok(())
}

async fn insert_event(pool: &SqlitePool, event: &icalendar::Event) -> Result<(), sqlx::Error> {
    log::debug!("Found event, inserting into DB.");

    let status = event.get_status().map(|s| match s {
        EventStatus::Tentative => "TENTATIVE",
        EventStatus::Confirmed => "CONFIRMED",
        EventStatus::Cancelled => "CANCELLED",
    });

    let (start_at, start_has_time) = to_db_time(event.get_start());
    let (end_at, end_has_time) = to_db_time(event.get_start());

    sqlx::query(
        "
INSERT INTO events (summary, description, start_at, start_has_time, end_at, end_has_time, status)
VALUES (?, ?, ?, ?, ?, ?, ?);
        ",
    )
    .bind(event.get_summary())
    .bind(event.get_description())
    .bind(start_at)
    .bind(start_has_time)
    .bind(end_at)
    .bind(end_has_time)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_todo(pool: &SqlitePool, todo: &icalendar::Todo) -> Result<(), sqlx::Error> {
    log::debug!("Found todo, inserting into DB.");

    let (due_at, due_has_time) = to_db_time(todo.get_due());
    let status: Option<&str> = todo.get_status().map(|s| TodoStatus::from(&s).into());

    sqlx::query(
        "
INSERT INTO todos (summary, description, due_at, due_has_time, completed, percent_complete, status)
VALUES (?, ?, ?, ?, ?, ?, ?);
        ",
    )
    .bind(todo.get_summary().unwrap_or(""))
    .bind(todo.get_description())
    .bind(due_at)
    .bind(due_has_time)
    .bind(todo.get_completed().map(|d| d.to_rfc3339()))
    .bind(todo.get_percent_complete().map(i64::from))
    .bind(status)
    .execute(pool)
    .await?;

    Ok(())
}

/**
* Help functions
 */

fn to_db_time(date: Option<DatePerhapsTime>) -> (Option<String>, bool) {
    match date {
        Some(DatePerhapsTime::DateTime(dt)) => match dt {
            CalendarDateTime::Floating(dt) => {
                (Some(dt.format("%Y-%m-%dT%H:%M:%S").to_string()), true)
            }
            CalendarDateTime::Utc(dt) => (Some(dt.to_rfc3339()), true),
            CalendarDateTime::WithTimezone { date_time, tzid: _ } => (
                Some(date_time.format("%Y-%m-%dT%H:%M:%S").to_string()),
                true,
            ),
        },
        Some(DatePerhapsTime::Date(d)) => (Some(d.to_string()), false),
        None => (None, false),
    }
}
