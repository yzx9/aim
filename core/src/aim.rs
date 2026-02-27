// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::path::PathBuf;

use aimcal_ical::{CalendarComponent, ICalendar};
use jiff::Zoned;
use tokio::fs;
use uuid::Uuid;

use crate::db::Db;
use crate::event::reconstruct_event_from_db;
use crate::io::{add_calendar_if_enabled, parse_ics, write_ics};
use crate::short_id::ShortIds;
use crate::todo::reconstruct_todo_from_db;
use crate::{
    Config, Event, EventConditions, EventDraft, EventPatch, Id, Kind, Pager, Todo, TodoConditions,
    TodoDraft, TodoPatch, TodoSort,
};

/// AIM calendar application core.
#[derive(Debug, Clone)]
pub struct Aim {
    now: Zoned,
    config: Config,
    db: Db,
    short_ids: ShortIds,
}

impl Aim {
    /// Creates a new AIM instance with the given configuration.
    ///
    /// # Errors
    /// If initialization fails.
    pub async fn new(mut config: Config) -> Result<Self, Box<dyn Error>> {
        let now = Zoned::now();

        config.normalize()?;
        prepare(&config).await?;

        let db = initialize_db(&config).await?;
        let short_ids = ShortIds::new(db.clone());

        add_calendar_if_enabled(&db, config.calendar_path.as_ref())
            .await
            .map_err(|e| format!("Failed to add calendar files: {e}"))?;

        Ok(Self {
            now,
            config,
            db,
            short_ids,
        })
    }

    /// The current time in the AIM instance.
    #[must_use]
    pub fn now(&self) -> Zoned {
        self.now.clone()
    }

    /// Refresh the current time to now.
    pub fn refresh_now(&mut self) {
        self.now = Zoned::now();
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

        self.db.upsert_event(&uid, &event, 0).await?;

        if let Some(calendar_path) = &self.config.calendar_path {
            let path = calendar_path.join(format!("{uid}.ics"));

            let mut calendar = ICalendar::new();
            calendar.components.push(event.clone().into());

            write_ics(&path, &calendar).await?;

            let resource_id = format!("file://{}", path.display());
            self.db
                .resources
                .insert(&uid, 0, &resource_id, None)
                .await?;
        }

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

        let resource = self.db.resources.get(&uid, 0).await?;

        if let Some(res) = resource {
            let ics_path = res
                .resource_id
                .strip_prefix("file://")
                .ok_or("Invalid resource_id format")?;
            let ics_path_buf = PathBuf::from(ics_path);

            let mut calendar = parse_ics(&ics_path_buf).await?;

            let mut updated_event = None;
            for component in &mut calendar.components {
                if let CalendarComponent::Event(e) = component
                    && e.uid.content.to_string() == event.uid()
                {
                    patch.resolve(self.now.clone()).apply_to(e);
                    updated_event = Some(e.clone());
                    break;
                }
            }

            let updated_event = updated_event.ok_or("Event not found in calendar")?;

            write_ics(&ics_path_buf, &calendar).await?;

            self.db.upsert_event(&uid, &updated_event, 0).await?;

            let event_with_id = self.short_ids.event(updated_event).await?;
            Ok(event_with_id)
        } else if let Some(calendar_path) = &self.config.calendar_path {
            // DB-only event - handle based on calendar_path configuration

            // Generate ICS file for this DB-only event
            let p = calendar_path.join(format!("{uid}.ics"));

            // Reconstruct VEvent from database record and apply patch
            let mut event_ics = reconstruct_event_from_db(&event, &self.now);
            patch.resolve(self.now.clone()).apply_to(&mut event_ics);

            // Write ICS file
            let mut calendar = ICalendar::new();
            calendar.components.push(event_ics.clone().into());
            write_ics(&p, &calendar).await?;

            // Create resource record
            let resource_id = format!("file://{}", p.display());
            self.db
                .resources
                .insert(&uid, 0, &resource_id, None)
                .await?;

            // Update database
            self.db.upsert_event(&uid, &event_ics, 0).await?;

            self.short_ids.event(event_ics).await
        } else {
            // Pure DB-only update - apply patch directly
            let mut event_ics = reconstruct_event_from_db(&event, &self.now);
            patch.resolve(self.now.clone()).apply_to(&mut event_ics);

            // Update database only (no ICS file)
            self.db.upsert_event(&uid, &event_ics, 0).await?;

            self.short_ids.event(event_ics).await
        }
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
        let conds = conds.resolve(&self.now)?;
        let events = self.db.events.list(&conds, pager).await?;
        let events = self.short_ids.events(events).await?;
        Ok(events)
    }

    /// Counts the number of events matching the given conditions.
    ///
    /// # Errors
    /// If database access fails.
    pub async fn count_events(&self, conds: &EventConditions) -> Result<i64, Box<dyn Error>> {
        let conds = conds.resolve(&self.now)?;
        Ok(self.db.events.count(&conds).await?)
    }

    /// Create a default todo draft based on the AIM configuration.
    ///
    /// # Errors
    /// If date/time resolution fails.
    pub fn default_todo_draft(&self) -> Result<TodoDraft, String> {
        TodoDraft::default(&self.config, &self.now)
    }

    /// Add a new todo from the given draft.
    ///
    /// # Errors
    /// If the todo is not found, database or file system access fails.
    pub async fn new_todo(&self, draft: TodoDraft) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.generate_uid(Kind::Todo).await?;
        let todo = draft.resolve(&self.config, &self.now).into_ics(&uid);

        self.db.upsert_todo(&uid, &todo, 0).await?;

        if let Some(calendar_path) = &self.config.calendar_path {
            let path = calendar_path.join(format!("{uid}.ics"));

            let mut calendar = ICalendar::new();
            calendar.components.push(todo.clone().into());

            write_ics(&path, &calendar).await?;

            let resource_id = format!("file://{}", path.display());
            self.db
                .resources
                .insert(&uid, 0, &resource_id, None)
                .await?;
        }

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

        let resource = self.db.resources.get(&uid, 0).await?;

        if let Some(res) = resource {
            let ics_path = res
                .resource_id
                .strip_prefix("file://")
                .ok_or("Invalid resource_id format")?;
            let ics_path_buf = PathBuf::from(ics_path);

            let mut calendar = parse_ics(&ics_path_buf).await?;

            let mut updated_todo = None;
            for component in &mut calendar.components {
                if let CalendarComponent::Todo(t) = component
                    && t.uid.content.to_string() == todo.uid()
                {
                    patch.resolve(&self.now).apply_to(t);
                    updated_todo = Some(t.clone());
                    break;
                }
            }

            let updated_todo = updated_todo.ok_or("Todo not found in calendar")?;

            write_ics(&ics_path_buf, &calendar).await?;

            self.db.upsert_todo(&uid, &updated_todo, 0).await?;

            let todo = self.short_ids.todo(updated_todo).await?;
            Ok(todo)
        } else if let Some(calendar_path) = &self.config.calendar_path {
            // DB-only todo - handle based on calendar_path configuration

            // Generate ICS file for this DB-only todo
            let p = calendar_path.join(format!("{uid}.ics"));

            // Reconstruct VTodo from database record and apply patch
            let mut todo_ics = reconstruct_todo_from_db(&todo, &self.now);
            patch.resolve(&self.now).apply_to(&mut todo_ics);

            // Write ICS file
            let mut calendar = ICalendar::new();
            calendar.components.push(todo_ics.clone().into());
            write_ics(&p, &calendar).await?;

            // Create resource record
            let resource_id = format!("file://{}", p.display());
            self.db
                .resources
                .insert(&uid, 0, &resource_id, None)
                .await?;

            // Update database
            self.db.upsert_todo(&uid, &todo_ics, 0).await?;

            self.short_ids.todo(todo_ics).await
        } else {
            // Pure DB-only update - apply patch directly
            let mut todo_ics = reconstruct_todo_from_db(&todo, &self.now);
            patch.resolve(&self.now).apply_to(&mut todo_ics);

            // Update database only (no ICS file)
            self.db.upsert_todo(&uid, &todo_ics, 0).await?;

            self.short_ids.todo(todo_ics).await
        }
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
        let conds = conds.resolve(&self.now)?;
        let sort = TodoSort::resolve_vec(sort, &self.config);
        let todos = self.db.todos.list(&conds, &sort, pager).await?;
        let todos = self.short_ids.todos(todos).await?;
        Ok(todos)
    }

    /// Counts the number of todos matching the given conditions.
    ///
    /// # Errors
    /// If database access fails.
    pub async fn count_todos(&self, conds: &TodoConditions) -> Result<i64, Box<dyn Error>> {
        let conds = conds.resolve(&self.now)?;
        Ok(self.db.todos.count(&conds).await?)
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

            return Ok(uid);
        }

        tracing::warn!("failed to generate a unique uid after multiple attempts");
        Err("Failed to generate a unique UID after multiple attempts".into())
    }
}

async fn prepare(config: &Config) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = &config.state_dir {
        tracing::debug!(path = %parent.display(), "ensuring state directory exists");
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

async fn initialize_db(config: &Config) -> Result<Db, Box<dyn Error>> {
    const NAME: &str = "aim.db";
    let db = if let Some(parent) = &config.state_dir {
        Db::open(Some(&parent.join(NAME))).await
    } else {
        Db::open(None).await
    }
    .map_err(|e| format!("Failed to initialize db: {e}"))?;

    Ok(db)
}
