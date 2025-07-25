// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use super::{
    events::{EventRecord, Events},
    todos::{TodoRecord, Todos},
};
use sqlx::sqlite::SqlitePool;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SqliteCache {
    pub events: Events,
    pub todos: Todos,
}

impl SqliteCache {
    pub async fn open() -> Result<Self, Box<dyn std::error::Error>> {
        log::debug!("Open an in-memory SQLite database connection pool");
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| format!("Failed to connect to SQLite database: {e}"))?;

        log::debug!("Creating tables in the database");
        let events = Events::new(pool.clone()).await?;
        let todos = Todos::new(pool.clone()).await?;
        Ok(SqliteCache { events, todos })
    }

    pub async fn upsert_event(
        &self,
        path: &Path,
        event: &icalendar::Event,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.to_str().ok_or("Invalid path encoding")?.to_string();
        let record = EventRecord::from(path.clone(), event)?;
        self.events
            .insert(record)
            .await
            .map_err(|e| format!("Failed to insert event into cache: {e}").into())
    }

    pub async fn upsert_todo(
        &self,
        path: &Path,
        todo: &icalendar::Todo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.to_str().ok_or("Invalid path encoding")?.to_string();
        let record = TodoRecord::from(path.clone(), todo)?;
        self.todos
            .upsert(&record)
            .await
            .map_err(|e| format!("Failed to insert todo into cache: {e}").into())
    }
}
