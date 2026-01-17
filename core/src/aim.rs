// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::path::PathBuf;

use aimcal_ical::{CalendarComponent, ICalendar};
use chrono::{DateTime, Local};
use tokio::fs;
use uuid::Uuid;

use crate::io::{add_calendar, parse_ics, write_ics};
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
    ///
    /// # Errors
    /// If initialization fails.
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn Error>> {
        let now = Local::now();

        config.normalize()?;
        prepare(&config).await?;

        let db = initialize_db(&config).await?;
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
    #[must_use]
    pub fn now(&self) -> DateTime<Local> {
        self.now
    }

    /// Refresh the current time to now.
    pub fn refresh_now(&mut self) {
        self.now = Local::now();
    }

    /// Create a default event draft based on the AIM configuration.
    #[must_use]
    pub fn default_event_draft(&self) -> EventDraft {
        EventDraft::default(&self.now)
    }

    /// Get a event by its id.
    ///
    /// # Errors
    /// If the event is not found or database access fails.
    pub async fn get_event(&self, id: &Id) -> Result<impl Event + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
        match self.db.events.get(&uid).await {
            Ok(Some(event)) => Ok(self.short_ids.event(event).await?),
            Ok(None) => Err("Event not found".into()),
            Err(e) => Err(e.into()),
        }
    }

    /// Add a new event from the given draft.
    ///
    /// # Errors
    /// If the event is not found, database or file system access fails.
    pub async fn new_event(
        &self,
        draft: EventDraft,
    ) -> Result<impl Event + 'static, Box<dyn Error>> {
        let uid = self.generate_uid(Kind::Event).await?;
        let event = draft.resolve(&self.now).into_ics(&uid);
        let path = self.get_path(&uid);

        // Create calendar with single event
        // TODO: consider reusing existing calendar if possible. see also: Todo
        let mut calendar = ICalendar::new();
        calendar.components.push(event.clone().into());

        write_ics(&path, &calendar).await?;
        self.db.upsert_event(&path, &event).await?;

        let event = self.short_ids.event(event).await?;
        Ok(event)
    }

    /// Upsert an event into the calendar.
    ///
    /// # Errors
    /// If the event is not found, database or file system access fails.
    pub async fn update_event(
        &self,
        id: &Id,
        patch: EventPatch,
    ) -> Result<impl Event + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
        let Some(event) = self.db.events.get(&uid).await? else {
            return Err("Event not found".into());
        };

        let path: PathBuf = event.path().into();
        let mut calendar = parse_ics(&path).await?;

        // Find and update the event by UID
        let mut found = false;
        for component in &mut calendar.components {
            if let CalendarComponent::Event(e) = component
                && e.uid.content.to_string() == event.uid()
            // PERF: avoid to_string() here
            {
                patch.resolve(self.now).apply_to(e);
                found = true;
                break;
            }
        }

        if !found {
            return Err("Event not found in calendar".into());
        }

        write_ics(&path, &calendar).await?;
        self.db.upsert_event(&path, &event).await?;

        let event_with_id = self.short_ids.event(event).await?;
        Ok(event_with_id)
    }

    /// Get the kind of the given id, which can be either an event or a todo.
    ///
    /// # Errors
    /// If the id is not found or database access fails.
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

    /// List events matching the given conditions.
    ///
    /// # Errors
    /// If database access fails.
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
    ///
    /// # Errors
    /// If database access fails.
    pub async fn count_events(&self, conds: &EventConditions) -> Result<i64, sqlx::Error> {
        let conds = conds.resolve(&self.now);
        self.db.events.count(&conds).await
    }

    /// Create a default todo draft based on the AIM configuration.
    #[must_use]
    pub fn default_todo_draft(&self) -> TodoDraft {
        TodoDraft::default(&self.config, &self.now)
    }

    /// Add a new todo from the given draft.
    ///
    /// # Errors
    /// If the todo is not found, database or file system access fails.
    pub async fn new_todo(&self, draft: TodoDraft) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.generate_uid(Kind::Todo).await?;
        let todo = draft.resolve(&self.config, &self.now).into_ics(&uid);
        let path = self.get_path(&uid);

        // Create calendar with single todo
        let mut calendar = ICalendar::new();
        calendar.components.push(todo.clone().into());

        write_ics(&path, &calendar).await?;
        self.db.upsert_todo(&path, &todo).await?;

        let todo_with_id = self.short_ids.todo(todo).await?;
        Ok(todo_with_id)
    }

    /// Upsert an event into the calendar.
    ///
    /// # Errors
    /// If the todo is not found, database or file system access fails.
    pub async fn update_todo(
        &self,
        id: &Id,
        patch: TodoPatch,
    ) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
        let Some(todo) = self.db.todos.get(&uid).await? else {
            return Err("Todo not found".into());
        };

        let path: PathBuf = todo.path().into();
        let mut calendar = parse_ics(&path).await?;

        // Find and update the todo by UID
        let mut found = false;
        for component in &mut calendar.components {
            if let CalendarComponent::Todo(t) = component
                && t.uid.content.to_string() == todo.uid()
            {
                patch.resolve(&self.now).apply_to(t);
                found = true;
                break;
            }
        }

        if !found {
            return Err("Todo not found in calendar".into());
        }

        write_ics(&path, &calendar).await?;
        self.db.upsert_todo(&path, &todo).await?;

        let todo = self.short_ids.todo(todo).await?;
        Ok(todo)
    }

    /// Get a todo by its id.
    ///
    /// # Errors
    /// If the todo is not found or database access fails.
    pub async fn get_todo(&self, id: &Id) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
        match self.db.todos.get(&uid).await {
            Ok(Some(todo)) => Ok(self.short_ids.todo(todo).await?),
            Ok(None) => Err("Event not found".into()),
            Err(e) => Err(e.into()),
        }
    }

    /// List todos matching the given conditions, sorted and paginated.
    ///
    /// # Errors
    /// If database access fails.
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
    ///
    /// # Errors
    /// If database access fails.
    pub async fn count_todos(&self, conds: &TodoConditions) -> Result<i64, sqlx::Error> {
        let conds = conds.resolve(&self.now);
        self.db.todos.count(&conds).await
    }

    /// Flush the short IDs to remove all entries.
    ///
    /// # Errors
    /// If database access fails.
    pub async fn flush_short_ids(&self) -> Result<(), Box<dyn Error>> {
        self.short_ids.flush().await
    }

    /// Close the AIM instance, saving any changes to the database.
    ///
    /// # Errors
    /// If closing the database fails.
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

async fn initialize_db(config: &Config) -> Result<LocalDb, Box<dyn Error>> {
    const NAME: &str = "aim.db";
    let db = if let Some(parent) = &config.state_dir {
        LocalDb::open(Some(&parent.join(NAME))).await
    } else {
        LocalDb::open(None).await
    }
    .map_err(|e| format!("Failed to initialize db: {e}"))?;

    Ok(db)
}
