// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use icalendar::{Calendar, CalendarComponent, Component};
use tokio::fs;
use uuid::Uuid;

use crate::event::ParsedEventConditions;
use crate::localdb::LocalDb;
use crate::short_id::ShortIds;
use crate::todo::{ParsedTodoConditions, ParsedTodoSort};
use crate::{
    Config, Event, EventConditions, EventDraft, EventPatch, Id, Kind, Pager, Todo, TodoConditions,
    TodoDraft, TodoPatch, TodoSort,
};

/// AIM calendar application core.
#[derive(Debug, Clone)]
pub struct Aim {
    now: DateTime<Local>,
    config: Config,
    db: LocalDb,
    short_ids: ShortIds,
    calendar_path: PathBuf,
}

impl Aim {
    /// Creates a new AIM instance with the given configuration.
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn Error>> {
        let now = Local::now();

        config.normalize()?;
        prepare(&config).await?;

        let db = LocalDb::open(&config.state_dir)
            .await
            .map_err(|e| format!("Failed to initialize db: {e}"))?;

        let short_ids = ShortIds::new(db.clone());
        let calendar_path = config.calendar_path.clone();
        let that = Self {
            now,
            config,
            db,
            short_ids,
            calendar_path,
        };
        that.add_calendar(&that.calendar_path)
            .await
            .map_err(|e| format!("Failed to add calendar files: {e}"))?;

        Ok(that)
    }

    /// The current time in the AIM instance.
    pub fn now(&self) -> DateTime<Local> {
        self.now
    }

    /// Refresh the current time to now.
    pub fn refresh_now(&mut self) {
        self.now = Local::now();
    }

    /// Create a default event draft based on the AIM configuration.
    pub fn default_event_draft(&self) -> EventDraft {
        EventDraft::default(self.now)
    }

    /// Add a new event from the given draft.
    pub async fn new_event(
        &self,
        draft: EventDraft,
    ) -> Result<impl Event + 'static, Box<dyn Error>> {
        let uid = self.generate_uid().await?;
        let event = draft.into_ics(&self.now, &uid);
        let path = self.get_path(&uid);

        let calendar = Calendar::new().push(event.clone()).done();
        fs::write(&path, calendar.to_string())
            .await
            .map_err(|e| format!("Failed to write calendar file: {e}"))?;

        self.db.upsert_event(&path, &event).await?;

        let todo = self.short_ids.event(event).await?;
        Ok(todo)
    }

    /// Upsert an event into the calendar.
    pub async fn update_event(
        &self,
        id: &Id,
        patch: EventPatch,
    ) -> Result<impl Event + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
        let event = match self.db.events.get(&uid).await? {
            Some(todo) => todo,
            None => return Err("Todo not found".into()),
        };

        let path: PathBuf = event.path().into();
        let mut calendar = parse_ics(&path).await?;
        let t = calendar
            .components
            .iter_mut()
            .filter_map(|a| match a {
                CalendarComponent::Event(a) => Some(a),
                _ => None,
            })
            .find(|a| a.get_uid() == Some(event.uid()))
            .ok_or("Event not found in calendar")?;

        patch.apply_to(t);
        let todo = t.clone();
        fs::write(&path, calendar.done().to_string())
            .await
            .map_err(|e| format!("Failed to write calendar file: {e}"))?;

        self.db.upsert_event(&path, &todo).await?;

        let todo = self.short_ids.event(todo).await?;
        Ok(todo)
    }

    /// Get the kind of the given id, which can be either an event or a todo.
    pub async fn get_kind(&self, id: &Id) -> Result<Kind, Box<dyn Error>> {
        tracing::debug!(?id, "getting kind of id");
        if let Some(data) = self.short_ids.get(id).await? {
            return Ok(data.kind);
        }

        let uid = id.as_uid();

        tracing::debug!(uid, "checking if id is an event");
        if self.db.events.get(uid).await?.is_some() {
            return Ok(Kind::Event);
        }

        tracing::debug!(uid, "checking if id is a todo");
        if self.db.todos.get(uid).await?.is_some() {
            return Ok(Kind::Todo);
        }

        Err("Id not found".into())
    }

    /// Get a event by its id.
    pub async fn get_event(&self, id: &Id) -> Result<impl Event + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
        match self.db.events.get(&uid).await {
            Ok(Some(event)) => Ok(self.short_ids.event(event).await?),
            Ok(None) => Err("Event not found".into()),
            Err(e) => Err(e.into()),
        }
    }

    /// List events matching the given conditions.
    pub async fn list_events(
        &self,
        conds: &EventConditions,
        pager: &Pager,
    ) -> Result<Vec<impl Event + 'static>, Box<dyn Error>> {
        let conds = ParsedEventConditions::parse(&self.now, conds);
        let events = self.db.events.list(&conds, pager).await?;
        let events = self.short_ids.events(events).await?;
        Ok(events)
    }

    /// Counts the number of events matching the given conditions.
    pub async fn count_events(&self, conds: &EventConditions) -> Result<i64, sqlx::Error> {
        let conds = ParsedEventConditions::parse(&self.now, conds);
        self.db.events.count(&conds).await
    }

    /// Create a default todo draft based on the AIM configuration.
    pub fn default_todo_draft(&self) -> TodoDraft {
        TodoDraft::default(&self.config, &self.now)
    }

    /// Add a new todo from the given draft.
    pub async fn new_todo(&self, draft: TodoDraft) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.generate_uid().await?;
        let todo = draft.into_ics(&self.config, &self.now, &uid);
        let path = self.get_path(&uid);

        let calendar = Calendar::new().push(todo.clone()).done();
        fs::write(&path, calendar.to_string())
            .await
            .map_err(|e| format!("Failed to write calendar file: {e}"))?;

        self.db.upsert_todo(&path, &todo).await?;

        let todo = self.short_ids.todo(todo).await?;
        Ok(todo)
    }

    /// Upsert an event into the calendar.
    pub async fn update_todo(
        &self,
        id: &Id,
        patch: TodoPatch,
    ) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
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

        patch.apply_to(&self.now, t);
        let todo = t.clone();
        fs::write(&path, calendar.done().to_string())
            .await
            .map_err(|e| format!("Failed to write calendar file: {e}"))?;

        self.db.upsert_todo(&path, &todo).await?;

        let todo = self.short_ids.todo(todo).await?;
        Ok(todo)
    }

    /// Get a todo by its id.
    pub async fn get_todo(&self, id: &Id) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
        match self.db.todos.get(&uid).await {
            Ok(Some(todo)) => Ok(self.short_ids.todo(todo).await?),
            Ok(None) => Err("Event not found".into()),
            Err(e) => Err(e.into()),
        }
    }

    /// List todos matching the given conditions, sorted and paginated.
    pub async fn list_todos(
        &self,
        conds: &TodoConditions,
        sort: &[TodoSort],
        pager: &Pager,
    ) -> Result<Vec<impl Todo + 'static>, Box<dyn Error>> {
        let conds = ParsedTodoConditions::parse(&self.now, conds);
        let sort = ParsedTodoSort::parse_vec(&self.config, sort);
        let todos = self.db.todos.list(&conds, &sort, pager).await?;
        let todos = self.short_ids.todos(todos).await?;
        Ok(todos)
    }

    /// Counts the number of todos matching the given conditions.
    pub async fn count_todos(&self, conds: &TodoConditions) -> Result<i64, sqlx::Error> {
        let conds = ParsedTodoConditions::parse(&self.now, conds);
        self.db.todos.count(&conds).await
    }

    /// Flush the short IDs to remove all entries.
    pub async fn flush_short_ids(&self) -> Result<(), Box<dyn Error>> {
        self.short_ids.flush().await
    }

    /// Close the AIM instance, saving any changes to the database.
    pub async fn close(self) -> Result<(), Box<dyn Error>> {
        self.db.close().await
    }

    #[tracing::instrument(skip(self))]
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
                        if let Err(err) = that.add_ics(&path).await {
                            tracing::error!(path = %path.display(), err, "failed to process file");
                        }
                    }));
                }
                _ => {}
            }
        }

        for handle in handles {
            handle.await?;
        }

        tracing::debug!(count = count_ics, "total .ics files processed");
        Ok(())
    }

    async fn add_ics(self, path: &Path) -> Result<(), Box<dyn Error>> {
        tracing::debug!(path = %path.display(), "parsing file");
        let calendar = parse_ics(path).await?;

        tracing::debug!(path = %path.display(), components = calendar.components.len(), "found components");
        for component in calendar.components {
            tracing::debug!(?component, "processing component");
            match component {
                CalendarComponent::Event(event) => self.db.upsert_event(path, &event).await?,
                CalendarComponent::Todo(todo) => self.db.upsert_todo(path, &todo).await?,
                _ => tracing::warn!(?component, "ignoring unsupported component type"),
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

async fn prepare(config: &Config) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = &config.state_dir {
        tracing::debug!(path = %parent.display(), "ensuring state directory exists");
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

async fn parse_ics(path: &Path) -> Result<Calendar, Box<dyn Error>> {
    fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?
        .parse()
        .map_err(|e| format!("Failed to parse calendar: {e}").into())
}
