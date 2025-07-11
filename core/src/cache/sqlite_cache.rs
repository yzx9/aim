// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use super::{event::EventRecord, todo::TodoRecord};
use crate::{aim::EventQuery, todo::TodoQuery, types::Pager};
use icalendar::{Calendar, CalendarComponent};
use sqlx::sqlite::SqlitePool;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct SqliteCache {
    pool: SqlitePool,
}

impl SqliteCache {
    pub async fn open() -> Result<Self, Box<dyn std::error::Error>> {
        log::debug!("Open an in-memory SQLite database connection pool");
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| format!("Failed to connect to SQLite database: {}", e.to_string()))?;

        log::debug!("Creating tables in the database");
        EventRecord::create_table(&pool)
            .await
            .map_err(|e| format!("Failed to create events table: {}", e.to_string()))?;
        TodoRecord::create_table(&pool)
            .await
            .map_err(|e| format!("Failed to create todos table: {}", e.to_string()))?;

        Ok(SqliteCache { pool })
    }

    pub async fn add_calendar(
        &self,
        calendar_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                    let pool_clone = self.pool.clone();
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = add_ics(&path, &pool_clone).await {
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
        Ok(())
    }

    pub async fn list_events(
        &self,
        query: &EventQuery,
        pager: &Pager,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        EventRecord::list(&self.pool, query, pager).await
    }

    pub async fn count_events(&self, query: &EventQuery) -> Result<i64, sqlx::Error> {
        EventRecord::count(&self.pool, query).await
    }

    pub async fn list_todos(
        &self,
        query: &TodoQuery,
        pager: &Pager,
    ) -> Result<Vec<TodoRecord>, sqlx::Error> {
        TodoRecord::list(&self.pool, query, pager).await
    }

    pub async fn count_todos(&self, query: &TodoQuery) -> Result<i64, sqlx::Error> {
        TodoRecord::count(&self.pool, query).await
    }
}

async fn add_ics(path: &Path, pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
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
        log::debug!("Processing component: {:?}", component);
        match component {
            CalendarComponent::Event(event) => {
                log::debug!("Found event, inserting into DB.");
                let record: EventRecord = event.into();
                record.insert(pool).await?
            }
            CalendarComponent::Todo(todo) => {
                log::debug!("Found todo, inserting into DB.");
                let record: TodoRecord = todo.into();
                record.insert(pool).await?
            }
            _ => log::warn!("Ignoring unsupported component type: {:?}", component),
        }
    }

    Ok(())
}
