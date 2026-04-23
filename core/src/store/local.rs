// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::path::{Path, PathBuf};

use aimcal_ical::{
    self as ical, CalendarComponent, Completed, Description, DtEnd, DtStamp, DtStart, Due,
    ICalendar, PercentComplete, Summary, Uid,
};
use async_trait::async_trait;
use jiff::Zoned;
use tokio::fs;

use crate::db::Db;
use crate::store::{StoreError, SyncResult};
use crate::{Event, EventPatch, LooseDateTime, Todo, TodoPatch};

/// Convert `Box<dyn Error>` (non-Send+Sync) to `StoreError` by wrapping in a String.
///
/// Use this for errors from parsing or database operations that return `Box<dyn Error>`.
#[allow(clippy::borrowed_box, clippy::needless_pass_by_value)]
fn into_store_error(e: Box<dyn Error>) -> StoreError {
    format!("{e}").into()
}

/// Reconstructs a [`aimcal_ical::VEvent`] from an Event trait object for database-only updates.
fn reconstruct_event_from_db<E: Event>(event: &E, now: &Zoned) -> aimcal_ical::VEvent<String> {
    let utc_now = now.with_time_zone(jiff::tz::TimeZone::UTC);
    let dt_stamp = DtStamp::new(utc_now.datetime());

    let dt_start = event.start().map_or_else(
        || {
            let default_start: LooseDateTime = now.clone().into();
            DtStart::new(default_start)
        },
        DtStart::new,
    );

    aimcal_ical::VEvent {
        uid: Uid::new(event.uid().into_owned()),
        dt_stamp,
        dt_start,
        dt_end: event.end().map(DtEnd::new),
        duration: None,
        summary: Some(Summary::new(event.summary().into_owned())),
        description: event
            .description()
            .map(|d| Description::new(d.into_owned())),
        status: event.status().map(|s| ical::EventStatus::new(s.into())),
        location: None,
        geo: None,
        url: None,
        organizer: None,
        attendees: Vec::new(),
        last_modified: None,
        transparency: None,
        sequence: None,
        priority: None,
        classification: None,
        resources: None,
        categories: None,
        rrule: None,
        rdates: Vec::new(),
        ex_dates: Vec::new(),
        x_properties: Vec::new(),
        retained_properties: Vec::new(),
        alarms: Vec::new(),
    }
}

/// Reconstructs a [`aimcal_ical::VTodo`] from a Todo trait object for database-only updates.
fn reconstruct_todo_from_db<T: Todo>(todo: &T, now: &Zoned) -> aimcal_ical::VTodo<String> {
    let utc_now = now.with_time_zone(jiff::tz::TimeZone::UTC);
    let dt_stamp = DtStamp::new(utc_now.datetime());

    aimcal_ical::VTodo {
        uid: Uid::new(todo.uid().into_owned()),
        dt_stamp,
        dt_start: None,
        due: todo.due().map(Due::new),
        completed: todo
            .completed()
            .map(|c| Completed::new(c.with_time_zone(jiff::tz::TimeZone::UTC).datetime())),
        duration: None,
        summary: Some(Summary::new(todo.summary().into_owned())),
        description: todo.description().map(|d| Description::new(d.into_owned())),
        status: Some(ical::TodoStatus::new(todo.status().into())),
        percent_complete: todo.percent_complete().map(PercentComplete::new),
        priority: Some(ical::Priority::new(Into::<u8>::into(todo.priority()))),
        location: None,
        geo: None,
        url: None,
        organizer: None,
        attendees: Vec::new(),
        last_modified: None,
        sequence: None,
        classification: None,
        resources: None,
        categories: None,
        rrule: None,
        rdates: Vec::new(),
        ex_dates: Vec::new(),
        x_properties: Vec::new(),
        retained_properties: Vec::new(),
        alarms: Vec::new(),
    }
}

/// Local file-based store for storing events and todos as ICS files.
///
/// This store stores each event/todo as a separate ICS file in the configured
/// calendar directory. Resource IDs use the `file://` URL scheme.
#[derive(Debug, Clone)]
pub struct LocalStore {
    /// Path to the calendar directory containing ICS files
    calendar_path: PathBuf,
    /// Database reference for syncing (optional, set when attached to Aim)
    db: Option<Db>,
    /// The calendar identifier.
    calendar_id: String,
}

impl LocalStore {
    /// Creates a new `LocalStore` with the specified calendar path.
    #[must_use]
    pub fn new(calendar_path: PathBuf, calendar_id: String) -> Self {
        Self {
            calendar_path,
            db: None,
            calendar_id,
        }
    }

    /// Creates a new `LocalStore` with the specified calendar path and database.
    #[must_use]
    pub fn with_db(calendar_path: PathBuf, db: Db, calendar_id: String) -> Self {
        Self {
            calendar_path,
            db: Some(db),
            calendar_id,
        }
    }

    /// Gets the file path for a given UID.
    fn file_path(&self, uid: &str) -> PathBuf {
        self.calendar_path.join(format!("{uid}.ics"))
    }

    /// Gets the resource ID (file:// URL) for a given UID.
    fn resource_id(&self, uid: &str) -> String {
        format!("file://{}", self.file_path(uid).display())
    }

    /// Scans the calendar directory for .ics files and syncs with the database.
    ///
    /// This is the implementation of `sync_cache` for the local store.
    async fn sync_from_directory(&self, db: &Db) -> Result<SyncResult, StoreError> {
        let mut created = 0;

        // Read directory and process each .ics file
        let mut entries = match fs::read_dir(&self.calendar_path).await {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Directory doesn't exist yet, nothing to sync
                return Ok(SyncResult {
                    created: 0,
                    updated: 0,
                    deleted: 0,
                });
            }
            Err(e) => {
                return Err(format!("Failed to read calendar directory: {e}").into());
            }
        };

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            match path.extension() {
                Some(ext) if ext == "ics" => {
                    // Parse the ICS file - log errors and continue
                    let calendar = match parse_ics(&path).await {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!(path = %path.display(), err = %e, "failed to parse ICS file");
                            continue;
                        }
                    };

                    // Process each component in the calendar
                    for component in calendar.components {
                        match component {
                            CalendarComponent::Event(event) => {
                                let uid = event.uid.content.to_string();

                                if let Err(e) =
                                    db.upsert_event(&uid, &event, &self.calendar_id).await
                                {
                                    tracing::error!(path = %path.display(), uid = %uid, err = %e, "failed to upsert event");
                                    continue;
                                }
                                if let Err(e) = db
                                    .resources
                                    .insert(&uid, &self.calendar_id, &self.resource_id(&uid), None)
                                    .await
                                {
                                    tracing::error!(path = %path.display(), uid = %uid, err = %e, "failed to insert resource");
                                    continue;
                                }
                                created += 1;
                            }
                            CalendarComponent::Todo(todo) => {
                                let uid = todo.uid.content.to_string();

                                if let Err(e) = db.upsert_todo(&uid, &todo, &self.calendar_id).await
                                {
                                    tracing::error!(path = %path.display(), uid = %uid, err = %e, "failed to upsert todo");
                                    continue;
                                }
                                if let Err(e) = db
                                    .resources
                                    .insert(&uid, &self.calendar_id, &self.resource_id(&uid), None)
                                    .await
                                {
                                    tracing::error!(path = %path.display(), uid = %uid, err = %e, "failed to insert resource");
                                    continue;
                                }
                                created += 1;
                            }
                            _ => {
                                tracing::warn!(
                                    path = %path.display(),
                                    "Unsupported component type in ICS file"
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(SyncResult {
            created,
            updated: 0,
            deleted: 0, // Local backend doesn't track deletions
        })
    }
}

#[async_trait]
impl crate::store::Store for LocalStore {
    async fn create_event(
        &self,
        uid: &str,
        event: &aimcal_ical::VEvent<String>,
    ) -> Result<String, StoreError> {
        let path = self.file_path(uid);

        // Wrap event in an ICalendar
        let calendar = ICalendar {
            components: vec![CalendarComponent::Event(event.clone())],
            ..Default::default()
        };

        // Write to file
        write_ics(&path, &calendar).await?;

        Ok(self.resource_id(uid))
    }

    async fn get_event(&self, uid: &str) -> Result<aimcal_ical::VEvent<String>, StoreError> {
        let path = self.file_path(uid);
        let calendar = parse_ics(&path).await.map_err(into_store_error)?;

        // Extract the first event component
        for component in calendar.components {
            if let CalendarComponent::Event(event) = component {
                return Ok(event);
            }
        }

        Err(format!("Event not found in file: {}", path.display()).into())
    }

    async fn update_event(
        &self,
        uid: &str,
        patch: &EventPatch,
    ) -> Result<aimcal_ical::VEvent<String>, StoreError> {
        let now = Zoned::now();

        // Try to get existing event from file
        match self.get_event(uid).await {
            Ok(mut event) => {
                // Case 1: File exists - apply patch and write back
                patch.resolve(now.clone()).apply_to(&mut event);

                let file_path = self.file_path(uid);
                let calendar = ICalendar {
                    components: vec![CalendarComponent::Event(event.clone())],
                    ..Default::default()
                };
                write_ics(&file_path, &calendar).await?;

                Ok(event)
            }
            Err(_) if self.db.is_some() => {
                // Case 2/3: File doesn't exist but we have DB - reconstruct from DB
                let db = self.db.as_ref().unwrap();
                let db_event = db
                    .events
                    .get(uid)
                    .await
                    .map_err(|e| StoreError::from(format!("{e}")))?
                    .ok_or_else(|| StoreError::from("Event not found in database"))?;

                let mut event = reconstruct_event_from_db(&db_event, &now);
                patch.resolve(now.clone()).apply_to(&mut event);

                // Write to file
                let file_path = self.file_path(uid);
                let calendar = ICalendar {
                    components: vec![CalendarComponent::Event(event.clone())],
                    ..Default::default()
                };
                write_ics(&file_path, &calendar).await?;

                // Update resource record and database
                db.resources
                    .insert(uid, &self.calendar_id, &self.resource_id(uid), None)
                    .await
                    .map_err(|e| StoreError::from(format!("{e}")))?;
                db.upsert_event(uid, &event, &self.calendar_id)
                    .await
                    .map_err(|e| StoreError::from(format!("{e}")))?;

                Ok(event)
            }
            Err(e) => Err(e),
        }
    }

    async fn delete_event(&self, uid: &str) -> Result<(), StoreError> {
        let file_path = self.file_path(uid);
        fs::remove_file(&file_path)
            .await
            .map_err(|e| format!("Failed to delete event file: {e}"))?;
        Ok(())
    }

    async fn create_todo(
        &self,
        uid: &str,
        todo: &aimcal_ical::VTodo<String>,
    ) -> Result<String, StoreError> {
        let path = self.file_path(uid);

        // Wrap todo in an ICalendar
        let calendar = ICalendar {
            components: vec![CalendarComponent::Todo(todo.clone())],
            ..Default::default()
        };

        // Write to file
        write_ics(&path, &calendar).await?;

        Ok(self.resource_id(uid))
    }

    async fn get_todo(&self, uid: &str) -> Result<aimcal_ical::VTodo<String>, StoreError> {
        let path = self.file_path(uid);
        let calendar = parse_ics(&path).await.map_err(into_store_error)?;

        // Extract the first todo component
        for component in calendar.components {
            if let CalendarComponent::Todo(todo) = component {
                return Ok(todo);
            }
        }

        Err(format!("Todo not found in file: {}", path.display()).into())
    }

    async fn update_todo(
        &self,
        uid: &str,
        patch: &TodoPatch,
    ) -> Result<aimcal_ical::VTodo<String>, StoreError> {
        let now = Zoned::now();

        // Try to get existing todo from file
        match self.get_todo(uid).await {
            Ok(mut todo) => {
                // File exists: apply patch and write back
                patch.resolve(&now).apply_to(&mut todo);

                let file_path = self.file_path(uid);
                let calendar = ICalendar {
                    components: vec![CalendarComponent::Todo(todo.clone())],
                    ..Default::default()
                };
                write_ics(&file_path, &calendar).await?;

                Ok(todo)
            }
            Err(_) if self.db.is_some() => {
                // File doesn't exist but we have DB: reconstruct from DB
                let db = self.db.as_ref().unwrap();
                let db_todo = db
                    .todos
                    .get(uid)
                    .await
                    .map_err(|e| StoreError::from(format!("{e}")))?
                    .ok_or_else(|| StoreError::from("Todo not found in database"))?;

                let mut todo = reconstruct_todo_from_db(&db_todo, &now);
                patch.resolve(&now).apply_to(&mut todo);

                // Write to file
                let file_path = self.file_path(uid);
                let calendar = ICalendar {
                    components: vec![CalendarComponent::Todo(todo.clone())],
                    ..Default::default()
                };
                write_ics(&file_path, &calendar).await?;

                // Update resource record and database
                db.resources
                    .insert(uid, &self.calendar_id, &self.resource_id(uid), None)
                    .await
                    .map_err(|e| StoreError::from(format!("{e}")))?;
                db.upsert_todo(uid, &todo, &self.calendar_id)
                    .await
                    .map_err(|e| StoreError::from(format!("{e}")))?;

                Ok(todo)
            }
            Err(e) => Err(e),
        }
    }

    async fn delete_todo(&self, uid: &str) -> Result<(), StoreError> {
        let file_path = self.file_path(uid);
        fs::remove_file(&file_path)
            .await
            .map_err(|e| format!("Failed to delete todo file: {e}"))?;
        Ok(())
    }

    async fn list_events(&self) -> Result<Vec<(String, aimcal_ical::VEvent<String>)>, StoreError> {
        let mut events = Vec::new();

        let mut entries = match fs::read_dir(&self.calendar_path).await {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Directory doesn't exist yet
                return Ok(events);
            }
            Err(e) => {
                return Err(format!("Failed to read calendar directory: {e}").into());
            }
        };

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            match path.extension() {
                Some(ext) if ext == "ics" => {
                    match parse_ics(&path).await.map_err(into_store_error) {
                        Ok(calendar) => {
                            for component in calendar.components {
                                if let CalendarComponent::Event(event) = component {
                                    let uid = event.uid.content.to_string();
                                    events.push((uid, event));
                                }
                            }
                        }
                        Err(e) => tracing::warn!(
                            path = %path.display(),
                            err = %e,
                            "Failed to parse ICS file"
                        ),
                    }
                }
                _ => {}
            }
        }

        Ok(events)
    }

    async fn list_todos(&self) -> Result<Vec<(String, aimcal_ical::VTodo<String>)>, StoreError> {
        let mut todos = Vec::new();

        let mut entries = match fs::read_dir(&self.calendar_path).await {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Directory doesn't exist yet
                return Ok(todos);
            }
            Err(e) => return Err(format!("Failed to read calendar directory: {e}").into()),
        };

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            match path.extension() {
                Some(ext) if ext == "ics" => {
                    match parse_ics(&path).await.map_err(into_store_error) {
                        Ok(calendar) => {
                            for component in calendar.components {
                                if let CalendarComponent::Todo(todo) = component {
                                    let uid = todo.uid.content.to_string();
                                    todos.push((uid, todo));
                                }
                            }
                        }
                        Err(e) => tracing::warn!(
                            path = %path.display(),
                            err = %e,
                            "Failed to parse ICS file"
                        ),
                    }
                }
                _ => {}
            }
        }

        Ok(todos)
    }

    async fn uid_exists(&self, uid: &str) -> Result<bool, StoreError> {
        let path = self.file_path(uid);
        Ok(fs::try_exists(&path).await?)
    }

    fn calendar_id(&self) -> &str {
        &self.calendar_id
    }

    async fn sync_cache(&self) -> Result<SyncResult, StoreError> {
        // For the local store, we sync from files on disk to the database
        match &self.db {
            Some(db) => self.sync_from_directory(db).await,
            // If no database is set, just return empty result
            // This happens when the store is created directly for testing
            None => Ok(SyncResult {
                created: 0,
                updated: 0,
                deleted: 0,
            }),
        }
    }
}

// TODO: support multiple calendars in one file
pub async fn parse_ics(path: &Path) -> Result<ICalendar<String>, Box<dyn Error>> {
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    let calendars = aimcal_ical::parse(&content)
        .map_err(|e| -> Box<dyn Error> { format!("Failed to parse calendar: {e:?}").into() })?;

    if calendars.is_empty() {
        return Err("No calendars found in file".into());
    }

    // Hybrid: parse borrowed, convert to owned for storage
    Ok(calendars.into_iter().next().unwrap().to_owned())
}

pub async fn write_ics(path: &Path, calendar: &ICalendar<String>) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create calendar directory: {e}"))?;
    }

    let ics_content = aimcal_ical::formatter::format(calendar)
        .map_err(|e| format!("Failed to format calendar: {e}"))?;

    fs::write(path, ics_content)
        .await
        .map_err(|e| format!("Failed to write calendar file: {e}"))
}

#[cfg(test)]
mod tests {
    use aimcal_ical::{DtEnd, DtStamp, DtStart, Summary, Uid};
    use jiff::civil::date;
    use jiff::tz::TimeZone;

    use super::*;

    use crate::store::Store;

    /// Helper function to create a test `VEvent`.
    fn create_test_vevent(uid: &str, summary: &str) -> aimcal_ical::VEvent<String> {
        let now = Zoned::now();
        let utc_now = now.with_time_zone(TimeZone::UTC);
        let dt_start = LooseDateTime::Local(
            date(2025, 1, 1)
                .at(10, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        let dt_end = LooseDateTime::Local(
            date(2025, 1, 1)
                .at(11, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        aimcal_ical::VEvent {
            uid: Uid::new(uid.to_string()),
            dt_stamp: DtStamp::new(utc_now.datetime()),
            dt_start: DtStart::new(dt_start),
            dt_end: Some(DtEnd::new(dt_end)),
            duration: None,
            summary: Some(Summary::new(summary.to_string())),
            description: None,
            location: None,
            geo: None,
            url: None,
            organizer: None,
            attendees: Vec::new(),
            last_modified: None,
            status: None,
            transparency: None,
            sequence: None,
            priority: None,
            classification: None,
            resources: None,
            categories: None,
            rrule: None,
            rdates: Vec::new(),
            ex_dates: Vec::new(),
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
            alarms: Vec::new(),
        }
    }

    /// Helper function to create a test `VTodo`.
    fn create_test_vtodo(uid: &str, summary: &str) -> aimcal_ical::VTodo<String> {
        let now = Zoned::now();
        let utc_now = now.with_time_zone(TimeZone::UTC);

        aimcal_ical::VTodo {
            uid: Uid::new(uid.to_string()),
            dt_stamp: DtStamp::new(utc_now.datetime()),
            dt_start: None,
            due: None,
            completed: None,
            duration: None,
            summary: Some(Summary::new(summary.to_string())),
            description: None,
            status: None,
            percent_complete: None,
            priority: None,
            location: None,
            geo: None,
            url: None,
            organizer: None,
            attendees: Vec::new(),
            last_modified: None,
            sequence: None,
            classification: None,
            resources: None,
            categories: None,
            rrule: None,
            rdates: Vec::new(),
            ex_dates: Vec::new(),
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
            alarms: Vec::new(),
        }
    }

    #[test]
    fn local_backend_new_creates_backend() {
        let calendar_path = PathBuf::from("/tmp/calendar");
        let backend = LocalStore::new(calendar_path.clone(), "default".to_string());

        assert_eq!(backend.calendar_path, calendar_path);
    }

    #[test]
    fn local_backend_file_path_constructs_correct_path() {
        let backend = LocalStore::new(PathBuf::from("/tmp/calendar"), "default".to_string());
        let path = backend.file_path("test-uid");

        assert_eq!(path, PathBuf::from("/tmp/calendar/test-uid.ics"));
    }

    #[test]
    fn local_backend_resource_id_constructs_file_url() {
        let backend = LocalStore::new(PathBuf::from("/tmp/calendar"), "default".to_string());
        let resource_id = backend.resource_id("test-uid");

        assert_eq!(resource_id, "file:///tmp/calendar/test-uid.ics");
    }

    #[test]
    fn local_backend_calendar_id_returns_default() {
        let backend = LocalStore::new(PathBuf::from("/tmp/calendar"), "default".to_string());
        assert_eq!(backend.calendar_id(), "default");
    }

    #[tokio::test]
    async fn local_backend_uid_exists_checks_file_existence() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        // Create a test file
        let test_file = backend.file_path("existing-uid");
        fs::write(&test_file, "test content")
            .await
            .expect("Failed to write test file");

        // Test exists
        assert!(backend.uid_exists("existing-uid").await.unwrap());

        // Test not exists
        assert!(!backend.uid_exists("non-existing-uid").await.unwrap());
    }

    #[tokio::test]
    async fn local_backend_create_event_writes_ics_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        let uid = "test-event-uid";
        let event = create_test_vevent(uid, "Test Event");

        let resource_id = backend.create_event(uid, &event).await.unwrap();

        // Verify resource_id
        assert_eq!(
            resource_id,
            format!("file://{}", backend.file_path(uid).display())
        );

        // Verify file exists
        let path = backend.file_path(uid);
        assert!(path.exists(), "ICS file should be created");

        // Verify content can be parsed
        let calendar = parse_ics(&path).await.unwrap();
        assert_eq!(calendar.components.len(), 1);

        if let CalendarComponent::Event(parsed_event) = calendar.components.first().unwrap() {
            assert_eq!(parsed_event.uid.content.to_string(), uid);
            assert_eq!(
                parsed_event.summary.as_ref().unwrap().content.to_string(),
                "Test Event"
            );
        } else {
            panic!("Expected Event component");
        }
    }

    #[tokio::test]
    async fn local_backend_create_todo_writes_ics_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        let uid = "test-todo-uid";
        let todo = create_test_vtodo(uid, "Test Todo");

        let resource_id = backend.create_todo(uid, &todo).await.unwrap();

        // Verify resource_id
        assert_eq!(
            resource_id,
            format!("file://{}", backend.file_path(uid).display())
        );

        // Verify file exists
        let path = backend.file_path(uid);
        assert!(path.exists(), "ICS file should be created");

        // Verify content can be parsed
        let calendar = parse_ics(&path).await.unwrap();
        assert_eq!(calendar.components.len(), 1);

        if let CalendarComponent::Todo(parsed_todo) = calendar.components.first().unwrap() {
            assert_eq!(parsed_todo.uid.content.to_string(), uid);
            assert_eq!(
                parsed_todo.summary.as_ref().unwrap().content.to_string(),
                "Test Todo"
            );
        } else {
            panic!("Expected Todo component");
        }
    }

    #[tokio::test]
    async fn local_backend_get_event_reads_ics_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        let uid = "test-event-get";
        let event = create_test_vevent(uid, "Get Test Event");

        // Create event file
        backend.create_event(uid, &event).await.unwrap();

        // Get event
        let retrieved = backend.get_event(uid).await.unwrap();

        assert_eq!(retrieved.uid.content.to_string(), uid);
        assert_eq!(
            retrieved.summary.as_ref().unwrap().content.to_string(),
            "Get Test Event"
        );
    }

    #[tokio::test]
    async fn local_backend_get_todo_reads_ics_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        let uid = "test-todo-get";
        let todo = create_test_vtodo(uid, "Get Test Todo");

        // Create todo file
        backend.create_todo(uid, &todo).await.unwrap();

        // Get todo
        let retrieved = backend.get_todo(uid).await.unwrap();

        assert_eq!(retrieved.uid.content.to_string(), uid);
        assert_eq!(
            retrieved.summary.as_ref().unwrap().content.to_string(),
            "Get Test Todo"
        );
    }

    #[tokio::test]
    async fn local_backend_update_event_modifies_ics_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        let uid = "test-event-update";
        let event = create_test_vevent(uid, "Original Summary");

        // Create event file
        backend.create_event(uid, &event).await.unwrap();

        // Update event
        let patch = EventPatch {
            summary: Some("Updated Summary".to_string()),
            ..Default::default()
        };

        let updated = backend.update_event(uid, &patch).await.unwrap();

        assert_eq!(
            updated.summary.as_ref().unwrap().content.to_string(),
            "Updated Summary"
        );

        // Verify file was updated
        let retrieved = backend.get_event(uid).await.unwrap();
        assert_eq!(
            retrieved.summary.as_ref().unwrap().content.to_string(),
            "Updated Summary"
        );
    }

    #[tokio::test]
    async fn local_backend_update_todo_modifies_ics_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        let uid = "test-todo-update";
        let todo = create_test_vtodo(uid, "Original Summary");

        // Create todo file
        backend.create_todo(uid, &todo).await.unwrap();

        // Update todo
        let patch = TodoPatch {
            summary: Some("Updated Summary".to_string()),
            ..Default::default()
        };

        let updated = backend.update_todo(uid, &patch).await.unwrap();

        assert_eq!(
            updated.summary.as_ref().unwrap().content.to_string(),
            "Updated Summary"
        );

        // Verify file was updated
        let retrieved = backend.get_todo(uid).await.unwrap();
        assert_eq!(
            retrieved.summary.as_ref().unwrap().content.to_string(),
            "Updated Summary"
        );
    }

    #[tokio::test]
    async fn local_backend_delete_event_removes_ics_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        let uid = "test-event-delete";
        let event = create_test_vevent(uid, "Delete Test Event");

        // Create event file
        backend.create_event(uid, &event).await.unwrap();
        let path = backend.file_path(uid);
        assert!(path.exists());

        // Delete event
        backend.delete_event(uid).await.unwrap();

        // Verify file was removed
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn local_backend_delete_todo_removes_ics_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        let uid = "test-todo-delete";
        let todo = create_test_vtodo(uid, "Delete Test Todo");

        // Create todo file
        backend.create_todo(uid, &todo).await.unwrap();
        let path = backend.file_path(uid);
        assert!(path.exists());

        // Delete todo
        backend.delete_todo(uid).await.unwrap();

        // Verify file was removed
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn local_backend_list_events_scans_directory() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        // Create multiple event files
        let first_event = create_test_vevent("event-1", "Event 1");
        let second_event = create_test_vevent("event-2", "Event 2");
        let first_todo = create_test_vtodo("todo-1", "Todo 1");

        backend.create_event("event-1", &first_event).await.unwrap();
        backend
            .create_event("event-2", &second_event)
            .await
            .unwrap();
        backend.create_todo("todo-1", &first_todo).await.unwrap();

        // List events
        let event_list = backend.list_events().await.unwrap();
        let mut events_sorted = event_list.clone();
        events_sorted.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(events_sorted.len(), 2);
        assert_eq!(events_sorted.first().unwrap().0, "event-1");
        assert_eq!(events_sorted.get(1).unwrap().0, "event-2");
    }

    #[tokio::test]
    async fn local_backend_list_todos_scans_directory() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(temp_dir.path().to_path_buf(), "default".to_string());

        // Create multiple todo files
        let first_todo = create_test_vtodo("todo-1", "Todo 1");
        let second_todo = create_test_vtodo("todo-2", "Todo 2");
        let first_event = create_test_vevent("event-1", "Event 1");

        backend.create_todo("todo-1", &first_todo).await.unwrap();
        backend.create_todo("todo-2", &second_todo).await.unwrap();
        backend.create_event("event-1", &first_event).await.unwrap();

        // List todos
        let todo_list = backend.list_todos().await.unwrap();
        let mut todos_sorted = todo_list.clone();
        todos_sorted.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(todos_sorted.len(), 2);
        assert_eq!(todos_sorted.first().unwrap().0, "todo-1");
        assert_eq!(todos_sorted.get(1).unwrap().0, "todo-2");
    }

    #[tokio::test]
    async fn local_backend_list_handles_nonexistent_directory() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let backend = LocalStore::new(
            temp_dir.path().join("nonexistent").clone(),
            "default".to_string(),
        );

        // List should return empty vec, not error
        let events = backend.list_events().await.unwrap();
        let todos = backend.list_todos().await.unwrap();

        assert!(events.is_empty());
        assert!(todos.is_empty());
    }
}
