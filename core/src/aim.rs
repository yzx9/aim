// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{Event, EventConditions, Pager, Todo, TodoConditions, TodoSort, cache::SqliteCache};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Aim {
    cache: SqliteCache,
}

impl Aim {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let cache = SqliteCache::open()
            .await
            .map_err(|e| format!("Failed to initialize cache: {e}"))?;

        add_calendar(&cache, &config.calendar_path)
            .await
            .map_err(|e| format!("Failed to add calendar files: {e}"))?;

        Ok(Self { cache })
    }

    pub async fn list_events(
        &self,
        query: &EventConditions,
        pager: &Pager,
    ) -> Result<Vec<impl Event>, sqlx::Error> {
        self.cache.events.list(query, pager).await
    }

    pub async fn count_events(&self, query: &EventConditions) -> Result<i64, sqlx::Error> {
        self.cache.events.count(query).await
    }

    pub async fn list_todos(
        &self,
        query: &TodoConditions,
        sort: &[TodoSort],
        pager: &Pager,
    ) -> Result<Vec<impl Todo>, sqlx::Error> {
        self.cache.todos.list(query, sort, pager).await
    }

    pub async fn count_todos(&self, query: &TodoConditions) -> Result<i64, sqlx::Error> {
        self.cache.todos.count(query).await
    }
}

#[derive(Debug)]
pub struct Config {
    pub calendar_path: PathBuf,
}

async fn add_calendar(
    cache: &SqliteCache,
    calendar_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = tokio::fs::read_dir(calendar_path)
        .await
        .map_err(|e| format!("Failed to read directory: {e}"))?;

    let mut handles = vec![];
    let mut count_ics = 0;

    while let Some(entry) = reader.next_entry().await? {
        let path = entry.path();
        match path.extension() {
            Some(ext) if ext == "ics" => {
                count_ics += 1;
                let that = cache.clone();
                handles.push(tokio::spawn(async move {
                    if let Err(e) = that.add_ics(&path).await {
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

    log::debug!("Total .ics files processed: {count_ics}");
    Ok(())
}
