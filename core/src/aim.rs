// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use icalendar::{Calendar, CalendarComponent, Component};
use tokio::fs;
use uuid::Uuid;

use crate::io::{add_calendar, parse_ics};
use crate::localdb::LocalDb;
use crate::short_id::ShortIds;
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
        add_calendar(&db, &calendar_path)
            .await
            .map_err(|e| format!("Failed to add calendar files: {e}"))?;

        Ok(Self {
            now,
            config,
            db,
            short_ids,
            calendar_path,
        })
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
        let uid = self.generate_uid(Kind::Event).await?;
        let event = draft.resolve(self.now).into_ics(&uid);
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
            None => return Err("Event not found".into()),
        };

        let path: PathBuf = event.path().into();
        let mut calendar = parse_ics(&path).await?;
        let e = calendar
            .components
            .iter_mut()
            .filter_map(|a| match a {
                CalendarComponent::Event(a) => Some(a),
                _ => None,
            })
            .find(|a| a.get_uid() == Some(event.uid()))
            .ok_or("Event not found in calendar")?;

        patch.resolve().apply_to(e);
        let event = e.clone();
        fs::write(&path, calendar.done().to_string())
            .await
            .map_err(|e| format!("Failed to write calendar file: {e}"))?;

        self.db.upsert_event(&path, &event).await?;

        let todo = self.short_ids.event(event).await?;
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
        let conds = conds.resolve(&self.now);
        let events = self.db.events.list(&conds, pager).await?;
        let events = self.short_ids.events(events).await?;
        Ok(events)
    }

    /// Counts the number of events matching the given conditions.
    pub async fn count_events(&self, conds: &EventConditions) -> Result<i64, sqlx::Error> {
        let conds = conds.resolve(&self.now);
        self.db.events.count(&conds).await
    }

    /// Create a default todo draft based on the AIM configuration.
    pub fn default_todo_draft(&self) -> TodoDraft {
        TodoDraft::default(&self.config, &self.now)
    }

    /// Add a new todo from the given draft.
    pub async fn new_todo(&self, draft: TodoDraft) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.generate_uid(Kind::Todo).await?;
        let todo = draft.resolve(&self.config, &self.now).into_ics(&uid);
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

        patch.resolve(self.now).apply_to(t);
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
        let conds = conds.resolve(&self.now);
        let sort = TodoSort::resolve_vec(sort, &self.config);
        let todos = self.db.todos.list(&conds, &sort, pager).await?;
        let todos = self.short_ids.todos(todos).await?;
        Ok(todos)
    }

    /// Counts the number of todos matching the given conditions.
    pub async fn count_todos(&self, conds: &TodoConditions) -> Result<i64, sqlx::Error> {
        let conds = conds.resolve(&self.now);
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

    async fn generate_uid(&self, kind: Kind) -> Result<String, Box<dyn Error>> {
        for i in 0..16 {
            let uid = Uuid::new_v4().to_string(); // TODO: better uid
            tracing::debug!(
                ?uid,
                attempt = i + 1,
                "generated uid, checking for uniqueness"
            );

            let exists = match kind {
                Kind::Event => self.db.events.get(&uid).await?.is_some(),
                Kind::Todo => self.db.todos.get(&uid).await?.is_some(),
            };
            if exists {
                tracing::debug!(uid, ?kind, "uid already exists in db");
                continue;
            }

            let path = self.get_path(&uid);
            if fs::try_exists(&path).await? {
                tracing::debug!(uid, ?path, "uid already exists as a file");
                continue;
            }
            return Ok(uid);
        }

        tracing::warn!("failed to generate a unique uid after multiple attempts");
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
