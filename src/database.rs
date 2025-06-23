use icalendar::{
    Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime, Event, EventStatus,
    Todo, TodoStatus,
};
use log::{debug, error, warn};
use sqlx::sqlite::SqlitePool;
use std::path::Path;
use tokio::fs;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(dir_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = new_db()
            .await
            .map_err(|e| format!("Failed to initialize database: {}", e.to_string()))?;

        let mut reader = fs::read_dir(dir_path)
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
                            error!("Failed to process file {}: {}", path.display(), e);
                        }
                    }));
                }
                _ => {}
            }
        }

        for handle in handles {
            handle.await?;
        }

        debug!("Total .ics files processed: {}", count_ics);
        Ok(Database { pool })
    }

    pub async fn count_events(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn count_todos(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM todos")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn list_events(&self) -> Result<Vec<EventRecord>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM events WHERE summary IS NOT NULL")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn list_todos(&self) -> Result<Vec<TodoRecord>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM todos WHERE summary IS NOT NULL")
            .fetch_all(&self.pool)
            .await
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

impl std::fmt::Display for EventRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Event #{}: {} (Starts: {})",
            self.id,
            self.summary,
            self.start_at.as_deref().unwrap_or("N/A")
        )
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

impl std::fmt::Display for TodoRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Todo #{}: {} (Due: {})",
            self.id,
            self.summary,
            self.due_at.as_deref().unwrap_or("N/A")
        )
    }
}

async fn new_db() -> Result<SqlitePool, sqlx::Error> {
    // Open an in-memory SQLite database connection pool
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    debug!("In-memory SQLite DB opened.");
    let mut tx = pool.begin().await?;

    sqlx::query(EventRecord::sql_create_table())
        .execute(&mut *tx)
        .await?;

    sqlx::query(TodoRecord::sql_create_table())
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    debug!("Tables created.");
    Ok(pool)
}

async fn process_ics_file(
    path: &Path,
    pool: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Processing file: {}", path.display());

    let contents = fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    debug!("Parsing calendar...");
    let parsed_calendar: Calendar = match contents.parse() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse calendar from {}: {}", path.display(), e);
            return Ok(()); // Continue with next file
        }
    };
    debug!("Calendar parsed successfully.");
    debug!(
        "Found {} components in {}.",
        parsed_calendar.components.len(),
        path.display()
    );

    for component in parsed_calendar.components {
        match component {
            CalendarComponent::Event(event) => add_event(pool, &event).await?,
            CalendarComponent::Todo(todo) => add_todo(pool, &todo).await?,
            _ => warn!("Component is not an event or to-do item"),
        }
    }
    Ok(())
}

async fn add_event(pool: &SqlitePool, event: &Event) -> Result<(), sqlx::Error> {
    debug!("Found event, inserting into DB.");
    let summary = event.get_summary();
    let description = event.get_description();

    let (start_at, start_has_time) = to_db_time(event.get_start());
    let (end_at, end_has_time) = to_db_time(event.get_end());

    let status = event.get_status().map(|s| match s {
        EventStatus::Tentative => Some("TENTATIVE".to_string()),
        EventStatus::Confirmed => Some("CONFIRMED".to_string()),
        EventStatus::Cancelled => Some("CANCELLED".to_string()),
    });

    sqlx::query(
        "
INSERT INTO events (summary, description, start_at, start_has_time, end_at, end_has_time, status)
VALUES (?, ?, ?, ?, ?, ?, ?);
        ",
    )
    .bind(summary)
    .bind(description)
    .bind(start_at)
    .bind(start_has_time)
    .bind(end_at)
    .bind(end_has_time)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(())
}

async fn add_todo(pool: &SqlitePool, todo: &Todo) -> Result<(), sqlx::Error> {
    debug!("Found todo, inserting into DB.");
    let summary = todo.get_summary();
    let description = todo.get_description();

    let (due_date, due_has_time) = to_db_time(todo.get_due());

    let completed = todo.get_completed().map(|d| d.to_rfc3339());
    let percent_complete = todo.get_percent_complete();
    let status = todo.get_status().map(|s| match s {
        TodoStatus::NeedsAction => "NEEDS-ACTION".to_string(),
        TodoStatus::InProcess => "IN-PROCESS".to_string(),
        TodoStatus::Completed => "COMPLETED".to_string(),
        TodoStatus::Cancelled => "CANCELLED".to_string(),
    });

    sqlx::query(
        "
INSERT INTO todos (summary, description, due_at, due_has_time, completed, percent_complete, status)
VALUES (?, ?, ?, ?, ?, ?, ?);
        ",
    )
    .bind(summary)
    .bind(description)
    .bind(due_date)
    .bind(due_has_time)
    .bind(completed)
    .bind(percent_complete)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(())
}

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
