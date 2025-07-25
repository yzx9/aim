// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Event, EventConditions, Pager, Todo, TodoConditions, TodoDraft, TodoSort, cache::SqliteCache,
    todo::TodoPatch,
};
use icalendar::{Calendar, CalendarComponent, Component};
use std::{
    error::Error,
    path::{Path, PathBuf},
};
use tokio::fs;

/// AIM calendar application core.
#[derive(Debug, Clone)]
pub struct Aim {
    cache: SqliteCache,
    calendar_path: PathBuf,
}

impl Aim {
    /// Creates a new AIM instance with the given configuration.
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let cache = SqliteCache::open()
            .await
            .map_err(|e| format!("Failed to initialize cache: {e}"))?;

        let that = Self {
            cache,
            calendar_path: config.calendar_path.clone(),
        };
        that.add_calendar(&config.calendar_path)
            .await
            .map_err(|e| format!("Failed to add calendar files: {e}"))?;

        Ok(that)
    }

    /// List events matching the given conditions.
    pub async fn list_events(
        &self,
        conds: &EventConditions,
        pager: &Pager,
    ) -> Result<Vec<impl Event>, sqlx::Error> {
        self.cache.events.list(conds, pager).await
    }

    /// Counts the number of events matching the given conditions.
    pub async fn count_events(&self, conds: &EventConditions) -> Result<i64, sqlx::Error> {
        self.cache.events.count(conds).await
    }

    /// Add a new todo from the given draft.
    pub async fn new_todo(&self, draft: TodoDraft) -> Result<impl Todo, Box<dyn Error>> {
        if self.cache.todos.get(&draft.uid).await?.is_some() {
            return Err("Todo with this UID already exists".into());
        }

        let path = self.calendar_path.join(format!("{}.ics", draft.uid));
        if fs::try_exists(&path).await? {
            return Err(format!("File already exists: {}", path.display()).into());
        }

        let todo = draft.into_todo()?;
        let calendar = Calendar::new().push(todo.clone()).done();
        fs::write(&path, calendar.to_string())
            .await
            .map_err(|e| format!("Failed to write calendar file: {e}"))?;

        self.cache.upsert_todo(&path, &todo).await?;
        Ok(todo)
    }

    /// Upsert an event into the calendar.
    pub async fn update_todo(&self, patch: TodoPatch) -> Result<impl Todo, Box<dyn Error>> {
        let todo = match self.cache.todos.get(&patch.uid).await? {
            Some(todo) => todo,
            None => return Err("Todo not found".into()),
        };

        let path: PathBuf = todo.path().into();
        let mut calendar = parse_ics(&path).await?;
        let t = calendar
            .components
            .iter_mut()
            .filter_map(|a| match a {
                CalendarComponent::Todo(a) => Some(a),
                _ => None,
            })
            .find(|a| a.get_uid() == Some(todo.uid()))
            .ok_or("Todo not found in calendar")?;

        patch.apply_to(t);
        let todo = t.clone();
        fs::write(&path, calendar.done().to_string())
            .await
            .map_err(|e| format!("Failed to write calendar file: {e}"))?;

        self.cache.upsert_todo(&path, &todo).await?;
        Ok(todo)
    }

    /// Get a todo by its UID.
    pub async fn get_todo(&self, uid: &str) -> Result<Option<impl Todo>, sqlx::Error> {
        self.cache.todos.get(uid).await
    }

    /// List todos matching the given conditions, sorted and paginated.
    pub async fn list_todos(
        &self,
        conds: &TodoConditions,
        sort: &[TodoSort],
        pager: &Pager,
    ) -> Result<Vec<impl Todo>, sqlx::Error> {
        self.cache.todos.list(conds, sort, pager).await
    }

    /// Counts the number of todos matching the given conditions.
    pub async fn count_todos(&self, conds: &TodoConditions) -> Result<i64, sqlx::Error> {
        self.cache.todos.count(conds).await
    }

    async fn add_calendar(&self, calendar_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let mut reader = fs::read_dir(calendar_path)
            .await
            .map_err(|e| format!("Failed to read directory: {e}"))?;

        let mut handles = vec![];
        let mut count_ics = 0;

        while let Some(entry) = reader.next_entry().await? {
            let path = entry.path();
            match path.extension() {
                Some(ext) if ext == "ics" => {
                    count_ics += 1;
                    let that = self.clone();
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

    async fn add_ics(self, path: &Path) -> Result<(), Box<dyn Error>> {
        log::debug!("Parsing file: {}", path.display());
        let calendar = parse_ics(path).await?;
        log::debug!(
            "Found {} components in {}.",
            calendar.components.len(),
            path.display()
        );

        for component in calendar.components {
            log::debug!("Processing component: {component:?}");
            match component {
                CalendarComponent::Event(event) => self.cache.upsert_event(path, &event).await?,
                CalendarComponent::Todo(todo) => self.cache.upsert_todo(path, &todo).await?,
                _ => log::warn!("Ignoring unsupported component type: {component:?}"),
            }
        }

        Ok(())
    }
}

/// Configuration for the AIM application.
#[derive(Debug)]
pub struct Config {
    /// Path to the calendar directory.
    pub calendar_path: PathBuf,
}

async fn parse_ics(path: &Path) -> Result<Calendar, Box<dyn Error>> {
    fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?
        .parse()
        .map_err(|e| format!("Failed to parse calendar: {e}").into())
}
