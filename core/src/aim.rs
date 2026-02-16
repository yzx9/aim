// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::fmt;

use jiff::Zoned;
use tokio::fs;
use uuid::Uuid;

use crate::backend::{Backend, CaldavBackend, LocalBackend, SyncResult};
use crate::db::Db;
use crate::short_id::ShortIds;
use crate::{
    BackendConfig, Config, Event, EventConditions, EventDraft, EventPatch, Id, Kind, Pager, Todo,
    TodoConditions, TodoDraft, TodoPatch, TodoSort,
};

/// AIM calendar application core.
pub struct Aim {
    now: Zoned,
    config: Config,
    db: Db,
    short_ids: ShortIds,
    backend: Box<dyn Backend>,
}

impl fmt::Debug for Aim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Aim")
            .field("now", &self.now)
            .field("config", &self.config)
            .field("db", &self.db)
            .field("short_ids", &self.short_ids)
            .field("backend", &"Box<dyn Backend>")
            .finish()
    }
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

        // Create backend based on configuration
        let backend: Box<dyn Backend> = match &config.backend {
            BackendConfig::Local { calendar_path } => {
                let calendar_path = calendar_path.as_ref().map_or_else(
                    || {
                        config.state_dir.as_ref().map_or_else(
                            || std::path::PathBuf::from("calendar"),
                            |p| p.join("calendar"),
                        )
                    },
                    std::path::PathBuf::from,
                );
                Box::new(LocalBackend::with_db(calendar_path, db.clone()))
            }
            BackendConfig::Caldav {
                base_url,
                calendar_home,
                calendar_href,
                auth,
                timeout_secs,
                user_agent,
            } => {
                let caldav_config = aimcal_caldav::CalDavConfig {
                    base_url: base_url.clone(),
                    calendar_home: calendar_home.clone(),
                    auth: auth.clone(),
                    timeout_secs: *timeout_secs,
                    user_agent: user_agent.clone(),
                };
                Box::new(
                    CaldavBackend::new(caldav_config, calendar_href.clone(), db.clone())
                        .map_err(|e| format!("Failed to create CalDAV backend: {e}"))?,
                )
            }
        };

        // Sync backend with local cache
        backend
            .sync_cache()
            .await
            .map_err(|e| format!("Failed to sync backend cache: {e}"))?;

        Ok(Self {
            now,
            config,
            db,
            short_ids,
            backend,
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
    /// If the event is not found, database or backend access fails.
    pub async fn new_event(
        &self,
        draft: EventDraft,
    ) -> Result<impl Event + 'static, Box<dyn Error>> {
        let uid = self.generate_uid(Kind::Event).await?;
        let event = draft.resolve(&self.now).into_ics(&uid);

        // Create event in backend
        let resource_id = self
            .backend
            .create_event(&uid, &event)
            .await
            .map_err(|e| format!("Failed to create event in backend: {e}"))?;

        // Store in database with resource mapping
        self.db
            .upsert_event(&uid, &event, self.backend.backend_kind())
            .await?;
        self.db
            .resources
            .insert(&uid, self.backend.backend_kind(), &resource_id, None)
            .await?;

        let event = self.short_ids.event(event).await?;
        Ok(event)
    }

    /// Upsert an event into the calendar.
    ///
    /// # Errors
    /// If the event is not found, database or backend access fails.
    pub async fn update_event(
        &self,
        id: &Id,
        patch: EventPatch,
    ) -> Result<impl Event + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
        let Some(_event) = self.db.events.get(&uid).await? else {
            return Err("Event not found".into());
        };

        // Update event through backend
        let updated_event = self
            .backend
            .update_event(&uid, &patch)
            .await
            .map_err(|e| format!("Failed to update event in backend: {e}"))?;

        // Update database
        self.db
            .upsert_event(&uid, &updated_event, self.backend.backend_kind())
            .await?;

        let event_with_id = self.short_ids.event(updated_event).await?;
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
    /// If the todo is not found, database or backend access fails.
    pub async fn new_todo(&self, draft: TodoDraft) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.generate_uid(Kind::Todo).await?;
        let todo = draft.resolve(&self.config, &self.now).into_ics(&uid);

        // Create todo in backend
        let resource_id = self
            .backend
            .create_todo(&uid, &todo)
            .await
            .map_err(|e| format!("Failed to create todo in backend: {e}"))?;

        // Store in database with resource mapping
        self.db
            .upsert_todo(&uid, &todo, self.backend.backend_kind())
            .await?;
        self.db
            .resources
            .insert(&uid, self.backend.backend_kind(), &resource_id, None)
            .await?;

        let todo_with_id = self.short_ids.todo(todo).await?;
        Ok(todo_with_id)
    }

    /// Upsert a todo into the calendar.
    ///
    /// # Errors
    /// If the todo is not found, database or backend access fails.
    pub async fn update_todo(
        &self,
        id: &Id,
        patch: TodoPatch,
    ) -> Result<impl Todo + 'static, Box<dyn Error>> {
        let uid = self.short_ids.get_uid(id).await?;
        let Some(_todo) = self.db.todos.get(&uid).await? else {
            return Err("Todo not found".into());
        };

        // Update todo through backend
        let updated_todo = self
            .backend
            .update_todo(&uid, &patch)
            .await
            .map_err(|e| format!("Failed to update todo in backend: {e}"))?;

        // Update database
        self.db
            .upsert_todo(&uid, &updated_todo, self.backend.backend_kind())
            .await?;

        let todo = self.short_ids.todo(updated_todo).await?;
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

    /// Synchronizes the backend with the local cache.
    ///
    /// # Errors
    /// If synchronization fails.
    pub async fn sync(&self) -> Result<SyncResult, Box<dyn Error>> {
        self.backend
            .sync_cache()
            .await
            .map_err(|e| format!("Failed to sync: {e}").into())
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
