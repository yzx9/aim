use crate::sqlite_cache::SqliteCache;
use std::path::PathBuf;

pub struct Aim {
    cache: SqliteCache,
}

impl Aim {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let cache = SqliteCache::new(&config.calendar_path)
            .await
            .map_err(|e| format!("Failed to initialize cache: {}", e.to_string()))?;

        Ok(Self { cache })
    }

    pub async fn count_events(&self) -> Result<i64, sqlx::Error> {
        self.cache.count_events().await
    }

    pub async fn count_todos(&self) -> Result<i64, sqlx::Error> {
        self.cache.count_todos().await
    }

    pub async fn list_events(&self) -> Result<Vec<impl Event>, sqlx::Error> {
        self.cache.list_events().await
    }

    pub async fn list_todos(&self) -> Result<Vec<impl Todo>, sqlx::Error> {
        self.cache.list_todos().await
    }
}

pub trait Event {
    fn id(&self) -> i64;
    fn summary(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn start_at(&self) -> Option<&str>;
    fn start_has_time(&self) -> bool;
    fn end_at(&self) -> Option<&str>;
    fn end_has_time(&self) -> bool;
    fn status(&self) -> Option<&str>;
}

pub trait Todo {
    fn id(&self) -> i64;
    fn summary(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn due_at(&self) -> Option<&str>;
    fn due_has_time(&self) -> bool;
    fn completed(&self) -> Option<&str>;
    fn percent_complete(&self) -> Option<i64>;
    fn status(&self) -> Option<&str>;
}

pub struct Config {
    calendar_path: PathBuf,
}

impl Config {
    pub fn new(calendar_path: PathBuf) -> Self {
        Self { calendar_path }
    }
}
