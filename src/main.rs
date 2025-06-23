use icalendar::{Calendar, CalendarComponent, Component};
use log::{debug, error, warn};
use sqlx::sqlite::SqlitePool;
use std::{env, fs, path::Path};

#[derive(sqlx::FromRow)]
struct EventRecord {
    summary: String,
    start_time: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TodoRecord {
    summary: String,
    due_date: Option<String>,
}

async fn process_ics_file(
    path: &Path,
    pool: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Processing file: {}", path.display());

    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    debug!("Successfully read file.");

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

    for component in parsed_calendar.components.iter() {
        match component {
            CalendarComponent::Event(event) => {
                debug!("Found event, inserting into DB.");
                let summary = event.get_summary();
                let description = event.get_description();
                let start_time = event.get_start();
                let end_time = event.get_end();

                sqlx::query("INSERT INTO events (summary, description, start_time, end_time) VALUES (?, ?, ?, ?)")
                    .bind(summary)
                    .bind(description)
                    // .bind(start_time)
                    // .bind(end_time)
                    .execute(pool)
                    .await?;
            }
            CalendarComponent::Todo(todo) => {
                debug!("Found todo, inserting into DB.");
                let summary = todo.get_summary();
                let description = todo.get_description();
                let due_date = todo.get_due();
                let completed = todo.get_completed();

                sqlx::query("INSERT INTO todos (summary, description, due_date, completed) VALUES (?, ?, ?, ?)")
                    .bind(summary)
                    .bind(description)
                    // .bind(due_date)
                    // .bind(completed)
                    .execute(pool)
                    .await?;
            }
            _ => warn!("Component is not an event or to-do item"),
        }
    }
    Ok(())
}

async fn list_events(pool: &SqlitePool) -> Result<Vec<EventRecord>, sqlx::Error> {
    sqlx::query_as("SELECT summary, start_time FROM events WHERE summary IS NOT NULL")
        .fetch_all(pool)
        .await
}

async fn list_todos(pool: &SqlitePool) -> Result<Vec<TodoRecord>, sqlx::Error> {
    sqlx::query_as("SELECT summary, due_date FROM todos WHERE summary IS NOT NULL")
        .fetch_all(pool)
        .await
}

async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // Open an in-memory SQLite database connection pool
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    debug!("In-memory SQLite DB opened.");
    let mut tx = pool.begin().await?;

    sqlx::query(
        "CREATE TABLE events ( \
            id INTEGER PRIMARY KEY, \
            summary TEXT, \
            description TEXT, \
            start_time TEXT, \
            end_time TEXT \
        );",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "CREATE TABLE todos ( \
            id INTEGER PRIMARY KEY, \
            summary TEXT, \
            description TEXT, \
            due_date TEXT, \
            completed TEXT \
        );",
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    debug!("Tables 'events' and 'todos' created.");
    Ok(pool)
}

async fn build_db(dir_path: &str) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let pool = init_db()
        .await
        .map_err(|e| format!("Failed to initialize database: {}", e.to_string()))?;

    let paths = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e.to_string()))?;

    let mut count_ics = 0;
    for path_result in paths {
        let path = match path_result {
            Ok(entry) => entry.path(),
            Err(e) => {
                error!("Failed to get path from directory entry: {}", e);
                continue;
            }
        };

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "ics" {
                    count_ics += 1;
                    if let Err(e) = process_ics_file(&path, &pool).await {
                        error!("Failed to process file {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    debug!("Total .ics files processed: {}", count_ics);
    Ok(pool)
}

async fn init() {
    env_logger::init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init().await;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("Usage: {} <path-to-directory>", &args[0]);
        return Ok(());
    }
    let dir_path = &args[1];
    debug!("Scanning directory: {}", dir_path);

    let pool = build_db(&dir_path).await?;

    // Query and print results to verify
    println!("\n--- Events found in DB ---");
    for event in list_events(&pool).await? {
        println!(
            "- Event: {} (Starts: {})",
            event.summary,
            event.start_time.unwrap_or_else(|| "N/A".to_string())
        );
    }

    println!("\n--- Todos found in DB ---");
    for todo in list_todos(&pool).await? {
        println!(
            "- Todo: {} (Due: {})",
            todo.summary,
            todo.due_date.unwrap_or_else(|| "N/A".to_string())
        );
    }
    Ok(())
}
