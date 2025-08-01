// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Event, EventConditions, Pager, Priority, Todo, TodoConditions, TodoDraft, TodoSort,
    event::ParsedEventConditions,
    localdb::LocalDb,
    short_id::{EventWithShortId, ShortIdMap, TodoWithShortId},
    todo::{ParsedTodoConditions, ParsedTodoSort, TodoPatch},
};
use chrono::{DateTime, Duration, Local};
use icalendar::{Calendar, CalendarComponent, Component};
use std::{
    error::Error,
    path::{Path, PathBuf},
};
use tokio::fs;
use uuid::Uuid;

/// AIM calendar application core.
#[derive(Debug, Clone)]
pub struct Aim {
    now: DateTime<Local>,
    config: Config,
    db: LocalDb,
    map: ShortIdMap,
    calendar_path: PathBuf,
}

impl Aim {
    /// Creates a new AIM instance with the given configuration.
    pub async fn new(config: Config) -> Result<Self, Box<dyn Error>> {
        let now = Local::now();

        if let Some(parent) = &config.state_dir {
            log::info!("Ensuring state directory exists: {}", parent.display());
            fs::create_dir_all(parent).await?;
        }

        let db = LocalDb::open(&config.state_dir)
            .await
            .map_err(|e| format!("Failed to initialize db: {e}"))?;

        let map = ShortIdMap::load_or_new(&config).await?;

        let calendar_path = config.calendar_path.clone();
        let that = Self {
            now,
            config,
            db,
            map,
            calendar_path,
        };
        that.add_calendar(&that.calendar_path)
            .await
            .map_err(|e| format!("Failed to add calendar files: {e}"))?;

        Ok(that)
    }

    /// Returns the current time in the AIM instance.
    pub fn now(&self) -> DateTime<Local> {
        self.now
    }

    /// Refresh the current time to now.
    pub fn refresh_now(&mut self) {
        self.now = Local::now();
    }

    /// List events matching the given conditions.
    pub async fn list_events(
        &self,
        conds: &EventConditions,
        pager: &Pager,
    ) -> Result<Vec<impl Event>, Box<dyn Error>> {
        let conds = ParsedEventConditions::parse(&self.now, conds);
        let events = self.db.events.list(&conds, pager).await?;

        let mut with_id = Vec::with_capacity(events.len());
        for event in events {
            with_id.push(EventWithShortId::with(&self.map, event)?);
        }
        Ok(with_id)
    }

    /// Counts the number of events matching the given conditions.
    pub async fn count_events(&self, conds: &EventConditions) -> Result<i64, sqlx::Error> {
        let conds = ParsedEventConditions::parse(&self.now, conds);
        self.db.events.count(&conds).await
    }

    /// Create a default todo draft based on the AIM configuration.
    pub fn default_todo_draft(&self) -> TodoDraft {
        TodoDraft::default(&self.config)
    }

    /// Add a new todo from the given draft.
    pub async fn new_todo(&self, draft: TodoDraft) -> Result<impl Todo, Box<dyn Error>> {
        let uid = self.generate_uid().await?;
        let todo = draft.into_todo(&self.config, self.now, &uid);
        let path = self.get_path(&uid);

        let calendar = Calendar::new().push(todo.clone()).done();
        fs::write(&path, calendar.to_string())
            .await
            .map_err(|e| format!("Failed to write calendar file: {e}"))?;

        self.db.upsert_todo(&path, &todo).await?;

        let todo = TodoWithShortId::with(&self.map, todo)?;
        Ok(todo)
    }

    /// Upsert an event into the calendar.
    pub async fn update_todo(&self, id: Id, patch: TodoPatch) -> Result<impl Todo, Box<dyn Error>> {
        let uid = self.map.get_uid(id);
        let todo = match self.db.todos.get(&uid).await? {
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

        self.db.upsert_todo(&path, &todo).await?;

        let todo = TodoWithShortId::with(&self.map, todo)?;
        Ok(todo)
    }

    /// Get a todo by its UID.
    pub async fn get_todo(&self, id: &Id) -> Result<Option<impl Todo>, sqlx::Error> {
        let uid = self.map.get_uid(id.clone());
        self.db.todos.get(&uid).await
    }

    /// List todos matching the given conditions, sorted and paginated.
    pub async fn list_todos(
        &self,
        conds: &TodoConditions,
        sort: &[TodoSort],
        pager: &Pager,
    ) -> Result<Vec<impl Todo>, Box<dyn Error>> {
        let conds = ParsedTodoConditions::parse(&self.now, conds);

        let sort: Vec<_> = sort
            .iter()
            .map(|s| ParsedTodoSort::parse(&self.config, *s))
            .collect();

        let todos = self.db.todos.list(&conds, &sort, pager).await?;

        let mut with_id = Vec::with_capacity(todos.len());
        for todo in todos {
            with_id.push(TodoWithShortId::with(&self.map, todo)?);
        }
        Ok(with_id)
    }

    /// Counts the number of todos matching the given conditions.
    pub async fn count_todos(&self, conds: &TodoConditions) -> Result<i64, sqlx::Error> {
        let conds = ParsedTodoConditions::parse(&self.now, conds);
        self.db.todos.count(&conds).await
    }

    /// Close the AIM instance, saving any changes to the database and short ID map.
    pub async fn close(self) -> Result<(), Box<dyn Error>> {
        self.db.close().await?;
        self.map.dump(&self.config).await?;
        Ok(())
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
                CalendarComponent::Event(event) => self.db.upsert_event(path, &event).await?,
                CalendarComponent::Todo(todo) => self.db.upsert_todo(path, &todo).await?,
                _ => log::warn!("Ignoring unsupported component type: {component:?}"),
            }
        }

        Ok(())
    }

    async fn generate_uid(&self) -> Result<String, Box<dyn Error>> {
        for _ in 0..16 {
            let uid = Uuid::new_v4().to_string(); // TODO: better uid
            if self.db.todos.get(&uid).await?.is_some()
                || fs::try_exists(&self.get_path(&uid)).await?
            {
                continue;
            }
            return Ok(uid);
        }

        Err("Failed to generate a unique UID after multiple attempts".into())
    }

    fn get_path(&self, uid: &str) -> PathBuf {
        self.calendar_path.join(format!("{uid}.ics"))
    }
}

/// Configuration for the AIM application.
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the calendar directory.
    pub calendar_path: PathBuf,

    /// Directory for storing application state.
    pub state_dir: Option<PathBuf>,

    /// Default due time for new tasks.
    pub default_due: Option<Duration>,

    /// Default priority for new tasks.
    pub default_priority: Priority,

    /// If true, items with no priority will be listed first.
    pub default_priority_none_fist: bool,
}

async fn parse_ics(path: &Path) -> Result<Calendar, Box<dyn Error>> {
    fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?
        .parse()
        .map_err(|e| format!("Failed to parse calendar: {e}").into())
}

/// The unique identifier for a todo item, which can be either a UID or a short ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Id {
    /// The unique identifier for the todo item.
    Uid(String),
    /// Either a short identifier or a unique identifier.
    ShortIdOrUid(String),
}
