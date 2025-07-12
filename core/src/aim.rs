// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{Event, EventConditions, Pager, Todo, TodoConditions, TodoSort, cache::SqliteCache};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Aim {
    cache: SqliteCache,
}

impl Aim {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let cache = SqliteCache::open()
            .await
            .map_err(|e| format!("Failed to initialize cache: {}", e.to_string()))?;

        cache
            .add_calendar(&config.calendar_path)
            .await
            .map_err(|e| format!("Failed to add calendar files: {}", e.to_string()))?;

        Ok(Self { cache })
    }

    pub async fn list_events(
        &self,
        query: &EventConditions,
        pager: &Pager,
    ) -> Result<Vec<impl Event>, sqlx::Error> {
        self.cache.list_events(query, pager).await
    }

    pub async fn count_events(&self, query: &EventConditions) -> Result<i64, sqlx::Error> {
        self.cache.count_events(query).await
    }

    pub async fn list_todos(
        &self,
        query: &TodoConditions,
        sort: &Vec<TodoSort>,
        pager: &Pager,
    ) -> Result<Vec<impl Todo>, sqlx::Error> {
        self.cache.list_todos(query, sort, pager).await
    }

    pub async fn count_todos(&self, query: &TodoConditions) -> Result<i64, sqlx::Error> {
        self.cache.count_todos(query).await
    }
}

pub struct Config {
    calendar_path: PathBuf,
}

impl Config {
    pub fn new(calendar_path: PathBuf) -> Self {
        Self { calendar_path }
    }
}
