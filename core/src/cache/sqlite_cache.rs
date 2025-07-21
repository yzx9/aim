// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use super::{
    events::{EventRecord, Events},
    todos::{TodoRecord, Todos},
};
use icalendar::{Calendar, CalendarComponent};
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

    pub async fn add_ics(self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

        let path = path.to_str().ok_or("Invalid path encoding")?.to_string();
        for component in calendar.components {
            log::debug!("Processing component: {component:?}");
            match component {
                CalendarComponent::Event(event) => {
                    let record = EventRecord::from(path.clone(), event)?;
                    self.events.insert(record).await?
                }

                CalendarComponent::Todo(todo) => {
                    let record = TodoRecord::from(path.clone(), todo)?;
                    self.todos.insert(record).await?
                }
                _ => log::warn!("Ignoring unsupported component type: {component:?}"),
            }
        }

        Ok(())
    }
}
