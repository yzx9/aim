// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::sqlite_cache::SqliteCache;
use chrono::{DateTime, Duration, Utc};
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

    pub async fn list_events(
        &self,
        query: &EventQuery,
        pager: &Pager,
    ) -> Result<Vec<impl Event>, sqlx::Error> {
        self.cache.list_events(query, pager).await
    }

    pub async fn count_events(&self, query: &EventQuery) -> Result<i64, sqlx::Error> {
        self.cache.count_events(query).await
    }

    pub async fn list_todos(
        &self,
        query: &TodoQuery,
        pager: &Pager,
    ) -> Result<Vec<impl Todo>, sqlx::Error> {
        self.cache.list_todos(query, pager).await
    }

    pub async fn count_todos(&self, query: &TodoQuery) -> Result<i64, sqlx::Error> {
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

pub struct EventQuery {
    now: DateTime<Utc>,
}

impl EventQuery {
    pub fn new() -> Self {
        Self { now: Utc::now() }
    }
}

pub trait Todo {
    fn id(&self) -> i64;
    fn summary(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn due_at(&self) -> Option<DateTime<Utc>>;
    fn due_has_time(&self) -> bool;
    fn completed(&self) -> Option<&str>;
    fn percent_complete(&self) -> Option<i64>;
    fn status(&self) -> Option<TodoStatus>;
}

#[derive(Clone, Copy)]
pub enum TodoStatus {
    NeedsAction,
    Completed,
    InProcess,
    Cancelled,
}

impl From<TodoStatus> for &str {
    fn from(item: TodoStatus) -> &'static str {
        match item {
            TodoStatus::NeedsAction => "needs-action",
            TodoStatus::Completed => "completed",
            TodoStatus::InProcess => "in-process",
            TodoStatus::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for TodoStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "needs-action" => Ok(TodoStatus::NeedsAction),
            "completed" => Ok(TodoStatus::Completed),
            "in-process" => Ok(TodoStatus::InProcess),
            "cancelled" => Ok(TodoStatus::Cancelled),
            _ => Err(format!("Unknown TodoStatus: {}", value)),
        }
    }
}

impl From<TodoStatus> for icalendar::TodoStatus {
    fn from(item: TodoStatus) -> icalendar::TodoStatus {
        match item {
            TodoStatus::NeedsAction => icalendar::TodoStatus::NeedsAction,
            TodoStatus::Completed => icalendar::TodoStatus::Completed,
            TodoStatus::InProcess => icalendar::TodoStatus::InProcess,
            TodoStatus::Cancelled => icalendar::TodoStatus::Cancelled,
        }
    }
}

impl From<&icalendar::TodoStatus> for TodoStatus {
    fn from(status: &icalendar::TodoStatus) -> Self {
        match status {
            icalendar::TodoStatus::NeedsAction => TodoStatus::NeedsAction,
            icalendar::TodoStatus::Completed => TodoStatus::Completed,
            icalendar::TodoStatus::InProcess => TodoStatus::InProcess,
            icalendar::TodoStatus::Cancelled => TodoStatus::Cancelled,
        }
    }
}

pub struct TodoQuery {
    now: DateTime<Utc>,
    status: Option<TodoStatus>,
    due: Option<Duration>,
}

impl TodoQuery {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: now,
            status: None,
            due: None,
        }
    }

    pub fn with_status(mut self, status: TodoStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_due(mut self, duration: Duration) -> Self {
        self.due = Some(duration);
        self
    }

    pub fn status(&self) -> Option<TodoStatus> {
        self.status
    }

    pub fn due(&self) -> Option<DateTime<Utc>> {
        match self.due {
            Some(duration) => Some(self.now + duration),
            None => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Pager {
    pub limit: i64,
    pub offset: i64,
}

impl Into<Pager> for (i64, i64) {
    fn into(self) -> Pager {
        Pager {
            limit: self.0,
            offset: self.1,
        }
    }
}
