// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use jiff::Zoned;
use tokio::fs;
use uuid::Uuid;

use crate::config::StoreDef;
use crate::db::{Db, calendars::CalendarRecord};
use crate::short_id::ShortIds;
use crate::store::{CaldavStore, LocalStore, Store, SyncResult};
use crate::{
    Config, Event, EventConditions, EventDraft, EventPatch, Id, Kind, Pager, Todo, TodoConditions,
    TodoDraft, TodoPatch, TodoSort,
};

/// Detailed information for a single calendar.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CalendarDetails {
    /// Unique calendar identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Store kind.
    pub kind: String,
    /// Lower numbers sort first.
    pub priority: i32,
    /// Whether the calendar is enabled.
    pub enabled: bool,
    /// Whether this calendar is used by default for new items.
    pub is_default: bool,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
    /// Store-specific configuration details, when available from config.
    pub store: Option<CalendarStoreDetails>,
}

/// Store-specific details for a calendar.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CalendarStoreDetails {
    /// Local filesystem-backed calendar details.
    Local {
        /// Path to the local calendar directory, if configured.
        calendar_path: Option<String>,
    },
    /// CalDAV-backed calendar details.
    Caldav {
        /// Base URL of the `CalDAV` server.
        base_url: String,
        /// Calendar home path on the server.
        calendar_home: String,
        /// Href of the calendar collection on the server.
        calendar_href: String,
        /// Authentication method kind.
        auth_method: String,
        /// Request timeout in seconds.
        timeout_secs: u64,
        /// User agent used for HTTP requests.
        user_agent: String,
    },
}

/// AIM calendar application core.
pub struct Aim {
    now: Zoned,
    config: Config,
    db: Db,
    short_ids: ShortIds,
    stores: HashMap<String, Box<dyn Store>>,
    default_calendar: String,
    startup_notices: Vec<String>,
}

struct InitializedStores {
    stores: HashMap<String, Box<dyn Store>>,
    default_calendar: String,
    startup_notices: Vec<String>,
}

impl fmt::Debug for Aim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Aim")
            .field("now", &self.now)
            .field("config", &self.config)
            .field("db", &self.db)
            .field("short_ids", &self.short_ids)
            .field("stores", &self.stores.len())
            .field("default_calendar", &self.default_calendar)
            .field("startup_notices", &self.startup_notices)
            .finish()
    }
}

impl Aim {
    fn calendar_store_details(
        entry: &crate::CalendarEntry,
        backend: &StoreDef,
    ) -> CalendarStoreDetails {
        match backend {
            StoreDef::Local { .. } => CalendarStoreDetails::Local {
                calendar_path: entry.calendar_path.clone(),
            },
            StoreDef::Caldav {
                base_url,
                calendar_home,
                auth,
                timeout_secs,
                user_agent,
            } => CalendarStoreDetails::Caldav {
                base_url: base_url.clone(),
                calendar_home: calendar_home.clone(),
                calendar_href: entry.calendar_href.clone().unwrap_or_default(),
                auth_method: match auth {
                    crate::AuthMethod::None => "none".to_string(),
                    crate::AuthMethod::Basic { .. } => "basic".to_string(),
                    crate::AuthMethod::Bearer { .. } => "bearer".to_string(),
                },
                timeout_secs: *timeout_secs,
                user_agent: user_agent.clone(),
            },
        }
    }

    /// Create a store from a store definition and calendar-specific fields.
    fn create_store(
        calendar_id: String,
        entry: &crate::CalendarEntry,
        store_def: &StoreDef,
        db: &Db,
        state_dir: Option<&std::path::Path>,
    ) -> Result<Box<dyn Store>, Box<dyn Error>> {
        match store_def {
            StoreDef::Local { .. } => {
                let calendar_path = entry.calendar_path.as_ref().map_or_else(
                    || {
                        state_dir.map_or_else(
                            || std::path::PathBuf::from("calendar"),
                            |p| p.join("calendar"),
                        )
                    },
                    std::path::PathBuf::from,
                );
                Ok(Box::new(LocalStore::with_db(
                    calendar_path,
                    db.clone(),
                    calendar_id,
                )))
            }
            StoreDef::Caldav {
                base_url,
                calendar_home,
                auth,
                timeout_secs,
                user_agent,
            } => {
                let calendar_href = entry.calendar_href.as_deref().ok_or_else(|| {
                    format!(
                        "Calendar '{calendar_id}' references caldav store but has no calendar_href"
                    )
                })?;
                let caldav_config = aimcal_caldav::CalDavConfig {
                    base_url: base_url.clone(),
                    calendar_home: calendar_home.clone(),
                    auth: auth.clone(),
                    timeout_secs: *timeout_secs,
                    user_agent: user_agent.clone(),
                };
                let backend = CaldavStore::new(
                    caldav_config,
                    calendar_href.to_string(),
                    db.clone(),
                    calendar_id,
                )
                .map_err(|e| format!("Failed to create CalDAV store: {e}"))?;
                Ok(Box::new(backend))
            }
        }
    }

    /// Get a store by calendar ID.
    fn get_store(&self, calendar_id: &str) -> Result<&dyn Store, Box<dyn Error>> {
        self.stores
            .get(calendar_id)
            .map(Box::as_ref)
            .ok_or_else(|| format!("Store not found for calendar: {calendar_id}").into())
    }

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

        // Handle legacy vs multi-calendar format
        let InitializedStores {
            stores,
            default_calendar,
            startup_notices,
        } = if config.is_legacy_format() {
            Self::initialize_legacy_calendar(&config, &db).await?
        } else {
            Self::initialize_multi_calendars(&config, &db).await?
        };

        // Sync all stores with local cache
        for (calendar_id, backend) in &stores {
            backend.sync_cache().await.map_err(|e| {
                format!("Failed to sync store cache for calendar '{calendar_id}': {e}")
            })?;
        }

        Ok(Self {
            now,
            config,
            db,
            short_ids,
            stores,
            default_calendar,
            startup_notices,
        })
    }

    async fn initialize_legacy_calendar(
        config: &Config,
        db: &Db,
    ) -> Result<InitializedStores, Box<dyn Error>> {
        let default_calendar_id = "default".to_string();

        // Legacy mode: create a local store using calendar_path or state_dir
        let calendar_path = config
            .calendar_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        let entry = crate::CalendarEntry {
            id: default_calendar_id.clone(),
            name: "Default".to_string(),
            store: "local".to_string(),
            calendar_href: None,
            calendar_path,
            priority: 0,
            enabled: true,
        };
        let store_def = StoreDef::Local {
            calendar_path: None,
        };

        let backend = Self::create_store(
            default_calendar_id.clone(),
            &entry,
            &store_def,
            db,
            config.state_dir.as_deref(),
        )?;

        let calendar = CalendarRecord::new(
            default_calendar_id.clone(),
            "Default".to_string(),
            "local".to_string(),
            0,
            true,
        );
        db.calendars.upsert(calendar).await?;

        let mut stores = HashMap::new();
        stores.insert(default_calendar_id.clone(), backend);

        Ok(InitializedStores {
            stores,
            default_calendar: default_calendar_id,
            startup_notices: Vec::new(),
        })
    }

    async fn initialize_multi_calendars(
        config: &Config,
        db: &Db,
    ) -> Result<InitializedStores, Box<dyn Error>> {
        if config.calendars.is_empty() {
            return Err("No calendars configured".into());
        }

        let existing = db.calendars.list().await?;
        let configured_ids: HashSet<_> = config
            .calendars
            .iter()
            .map(|calendar| &calendar.id)
            .collect();
        let mut auto_disabled = Vec::new();
        for calendar in existing {
            if configured_ids.contains(&calendar.id) || !calendar.enabled {
                continue;
            }

            db.calendars.set_enabled(&calendar.id, false).await?;
            auto_disabled.push(calendar.id);
        }

        let mut effective = Vec::with_capacity(config.calendars.len());
        for calendar in &config.calendars {
            let store_def = config.stores.get(&calendar.store).ok_or_else(|| {
                format!(
                    "Store '{}' not found for calendar '{}'",
                    calendar.store, calendar.id
                )
            })?;
            let calendar_kind = match store_def {
                StoreDef::Local { .. } => "local",
                StoreDef::Caldav { .. } => "caldav",
            };
            let record = CalendarRecord::new(
                calendar.id.clone(),
                calendar.name.clone(),
                calendar_kind.to_string(),
                calendar.priority,
                calendar.enabled,
            );
            db.calendars.upsert(record).await?;
            effective.push((calendar, calendar.enabled));
        }

        let mut stores = HashMap::new();
        for (calendar, enabled) in &effective {
            if !enabled {
                continue;
            }

            let store_def = config
                .stores
                .get(&calendar.store)
                .ok_or_else(|| format!("Store '{}' not found", calendar.store))?;

            let backend = Self::create_store(
                calendar.id.clone(),
                calendar,
                store_def,
                db,
                config.state_dir.as_deref(),
            )?;
            stores.insert(calendar.id.clone(), backend);
        }

        if stores.is_empty() {
            return Err("No enabled calendars found in configuration".into());
        }

        let default_calendar = if stores.contains_key(&config.default_calendar) {
            config.default_calendar.clone()
        } else {
            config
                .calendars
                .iter()
                .filter(|calendar| stores.contains_key(&calendar.id))
                .min_by_key(|calendar| calendar.priority)
                .map(|calendar| calendar.id.clone())
                .ok_or("No enabled calendars found in configuration")?
        };

        let startup_notices = if auto_disabled.is_empty() {
            Vec::new()
        } else {
            vec![format!(
                "Disabled calendar(s) not present in config: {}. Existing data was kept.",
                auto_disabled.join(", ")
            )]
        };

        Ok(InitializedStores {
            stores,
            default_calendar,
            startup_notices,
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

        // Resolve calendar: use draft.calendar_id or fall back to default
        let calendar_id = draft
            .calendar_id
            .as_deref()
            .unwrap_or(&self.default_calendar);
        let backend = self.get_store(calendar_id)?;

        // Create event in store
        let resource_id = backend
            .create_event(&uid, &event)
            .await
            .map_err(|e| format!("Failed to create event in store: {e}"))?;

        // Store in database with resource mapping
        self.db.upsert_event(&uid, &event, calendar_id).await?;
        self.db
            .resources
            .insert(&uid, calendar_id, &resource_id, None)
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

        // Get calendar_id from event record
        let event_record = self.db.events.get(&uid).await?.ok_or("Event not found")?;
        let backend = self.get_store(&event_record.calendar_id)?;
        let calendar_id = backend.calendar_id();

        // Update event through backend
        let updated_event = backend
            .update_event(&uid, &patch)
            .await
            .map_err(|e| format!("Failed to update event in store: {e}"))?;

        // Update database
        self.db
            .upsert_event(&uid, &updated_event, calendar_id)
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

        // Resolve calendar: use draft.calendar_id or fall back to default
        let calendar_id = draft
            .calendar_id
            .as_deref()
            .unwrap_or(&self.default_calendar);
        let backend = self.get_store(calendar_id)?;

        // Create todo in store
        let resource_id = backend
            .create_todo(&uid, &todo)
            .await
            .map_err(|e| format!("Failed to create todo in store: {e}"))?;

        // Store in database with resource mapping
        self.db.upsert_todo(&uid, &todo, calendar_id).await?;
        self.db
            .resources
            .insert(&uid, calendar_id, &resource_id, None)
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

        // Get calendar_id from todo record
        let todo_record = self.db.todos.get(&uid).await?.ok_or("Todo not found")?;
        let backend = self.get_store(&todo_record.calendar_id)?;
        let calendar_id = backend.calendar_id();

        // Update todo through backend
        let updated_todo = backend
            .update_todo(&uid, &patch)
            .await
            .map_err(|e| format!("Failed to update todo in store: {e}"))?;

        // Update database
        self.db
            .upsert_todo(&uid, &updated_todo, calendar_id)
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

    /// List known calendars ordered by priority.
    ///
    /// # Errors
    /// If database access fails.
    pub async fn list_calendars(&self) -> Result<Vec<CalendarRecord>, Box<dyn Error>> {
        Ok(self.db.calendars.list().await?)
    }

    /// Get detailed information for a single calendar.
    ///
    /// # Errors
    /// If the calendar is not found or database access fails.
    pub async fn get_calendar_details(&self, id: &str) -> Result<CalendarDetails, Box<dyn Error>> {
        let record = self
            .db
            .calendars
            .get(id)
            .await?
            .ok_or_else(|| format!("Calendar not found: {id}"))?;

        let backend = self
            .config
            .calendars
            .iter()
            .find(|calendar| calendar.id == record.id)
            .and_then(|calendar| {
                self.config
                    .stores
                    .get(&calendar.store)
                    .map(|store_def| Self::calendar_store_details(calendar, store_def))
            });

        Ok(CalendarDetails {
            id: record.id.clone(),
            name: record.name.clone(),
            kind: record.kind.clone(),
            priority: record.priority,
            enabled: record.enabled,
            is_default: self.default_calendar == record.id,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            store: backend,
        })
    }

    /// Startup notices produced while reconciling config and database state.
    #[must_use]
    pub fn startup_notices(&self) -> &[String] {
        &self.startup_notices
    }

    /// Flush the short IDs to remove all entries.
    ///
    /// # Errors
    /// If database access fails.
    pub async fn flush_short_ids(&self) -> Result<(), Box<dyn Error>> {
        self.short_ids.flush().await
    }

    /// Synchronizes the store with the local cache.
    ///
    /// # Errors
    /// If synchronization fails.
    pub async fn sync(&self) -> Result<SyncResult, Box<dyn Error>> {
        let mut created = 0;
        let mut updated = 0;
        let mut deleted = 0;

        for (calendar_id, backend) in &self.stores {
            match backend.sync_cache().await {
                Ok(result) => {
                    created += result.created;
                    updated += result.updated;
                    deleted += result.deleted;
                }
                Err(e) => {
                    return Err(format!("Failed to sync calendar '{calendar_id}': {e}").into());
                }
            }
        }

        Ok(SyncResult {
            created,
            updated,
            deleted,
        })
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
