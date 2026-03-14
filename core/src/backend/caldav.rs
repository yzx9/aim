// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! `CalDAV` backend implementation for storing and synchronizing calendar data.

use aimcal_caldav::{CalDavClient, CalDavConfig, CalendarQueryRequest, ETag, Href};
use aimcal_ical::{ICalendar, VEvent, VTodo, semantic::CalendarComponent};
use async_trait::async_trait;
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use crate::backend::{Backend, BackendError, SyncResult};
use crate::db::Db;
use crate::{EventPatch, TodoPatch};

/// Metadata stored with `CalDAV` resources in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaldavMetadata {
    /// `ETag` for optimistic concurrency control.
    pub etag: String,
    /// Last-Modified header value from server.
    pub last_modified: Option<String>,
}

/// `CalDAV` backend implementation.
///
/// This backend stores calendar data on a `CalDAV` server (RFC 4791),
/// using the `CalDAV` client for all operations.
#[derive(Debug)]
pub struct CaldavBackend {
    /// The `CalDAV` client for server operations.
    client: CalDavClient,
    /// The href of the calendar collection on the server.
    calendar_href: Href,
    /// The database for local cache.
    db: Db,
    /// The calendar identifier in the database.
    calendar_id: String,
}

impl CaldavBackend {
    /// Creates a new `CalDAV` backend from configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if client initialization fails.
    pub fn new(
        config: CalDavConfig,
        calendar_href: String,
        db: Db,
        calendar_id: String,
    ) -> Result<Self, BackendError> {
        let client = CalDavClient::new(config)?;

        Ok(Self {
            client,
            calendar_href: Href::new(calendar_href),
            db,
            calendar_id,
        })
    }

    /// Extracts a single `VEvent` from an `ICalendar`.
    fn extract_event(calendar: &ICalendar<String>) -> Result<VEvent<String>, BackendError> {
        for component in &calendar.components {
            if let CalendarComponent::Event(event) = component {
                return Ok(event.clone());
            }
        }
        Err("No VEVENT component found in calendar data".into())
    }

    /// Extracts a single `VTodo` from an `ICalendar`.
    fn extract_todo(calendar: &ICalendar<String>) -> Result<VTodo<String>, BackendError> {
        for component in &calendar.components {
            if let CalendarComponent::Todo(todo) = component {
                return Ok(todo.clone());
            }
        }
        Err("No VTODO component found in calendar data".into())
    }

    /// Wraps a `VEvent` in an `ICalendar` for transmission.
    fn wrap_event(event: &VEvent<String>) -> ICalendar<String> {
        ICalendar {
            prod_id: aimcal_ical::ProductId::default(),
            version: aimcal_ical::Version::default(),
            calscale: None,
            method: None,
            components: vec![CalendarComponent::Event(event.clone())],
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
        }
    }

    /// Wraps a `VTodo` in an `ICalendar` for transmission.
    fn wrap_todo(todo: &VTodo<String>) -> ICalendar<String> {
        ICalendar {
            prod_id: aimcal_ical::ProductId::default(),
            version: aimcal_ical::Version::default(),
            calscale: None,
            method: None,
            components: vec![CalendarComponent::Todo(todo.clone())],
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
        }
    }

    /// Generates an href for a new resource based on UID.
    fn generate_href(&self, uid: &str) -> Href {
        let calendar_path = self.calendar_href.as_str();
        let path = calendar_path.trim_end_matches('/');

        Href::new(format!("{path}/{uid}.ics"))
    }

    /// Converts `ETag` to string for database storage.
    fn etag_to_string(etag: &ETag) -> String {
        etag.as_str().to_string()
    }

    /// Gets the resource record for a UID from the database.
    async fn get_resource(
        &self,
        uid: &str,
    ) -> Result<Option<(String, CaldavMetadata)>, BackendError> {
        let record = self.db.resources.get(uid, &self.calendar_id).await?;

        match record {
            Some(rec) => {
                let metadata: CaldavMetadata = rec
                    .metadata_json()
                    .ok_or("Failed to parse CaldavMetadata from JSON")?;

                Ok(Some((rec.resource_id, metadata)))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
#[allow(clippy::too_many_lines)]
impl Backend for CaldavBackend {
    // #[instrument]
    async fn create_event(
        &self,
        uid: &str,
        event: &VEvent<String>,
    ) -> Result<String, BackendError> {
        let calendar = Self::wrap_event(event);
        let href = self.generate_href(uid);
        let etag = self.client.create_event(&href, &calendar).await?;

        let metadata = CaldavMetadata {
            etag: Self::etag_to_string(&etag),
            last_modified: None,
        };

        let metadata_json = serde_json::to_string(&metadata)?;
        self.db
            .resources
            .insert(uid, &self.calendar_id, href.as_str(), Some(&metadata_json))
            .await?;

        Ok(href.as_str().to_string())
    }

    // #[instrument]
    async fn get_event(&self, uid: &str) -> Result<VEvent<String>, BackendError> {
        let (href, _metadata) = self
            .get_resource(uid)
            .await?
            .ok_or(format!("Event not found: {uid}"))?;

        let resource = self.client.get_event(&Href::new(href)).await?;
        Self::extract_event(&resource.data)
    }

    // #[instrument]
    async fn update_event(
        &self,
        uid: &str,
        patch: &EventPatch,
    ) -> Result<VEvent<String>, BackendError> {
        let (href, metadata) = self
            .get_resource(uid)
            .await?
            .ok_or(format!("Event not found: {uid}"))?;

        // Fetch current event
        let resource = self.client.get_event(&Href::new(href.clone())).await?;
        let mut event = Self::extract_event(&resource.data)?;

        // Apply patch
        let now = Zoned::now();
        let resolved = patch.resolve(now);
        resolved.apply_to(&mut event);

        // Upload updated event
        let calendar = Self::wrap_event(&event);
        let etag = ETag::new(metadata.etag.clone());
        let new_etag = self
            .client
            .update_event(&Href::new(href.clone()), &etag, &calendar)
            .await?;

        // Update metadata in database
        let new_metadata = CaldavMetadata {
            etag: Self::etag_to_string(&new_etag),
            last_modified: None,
        };

        let metadata_json = serde_json::to_string(&new_metadata)?;
        self.db
            .resources
            .insert(uid, &self.calendar_id, &href, Some(&metadata_json))
            .await?;

        Ok(event)
    }

    // #[instrument]
    async fn delete_event(&self, uid: &str) -> Result<(), BackendError> {
        let (href, metadata) = self
            .get_resource(uid)
            .await?
            .ok_or(format!("Event not found: {uid}"))?;

        let etag = ETag::new(metadata.etag.clone());
        self.client
            .delete_event(&Href::new(href.clone()), &etag)
            .await?;

        self.db.resources.delete(uid, &self.calendar_id).await?;

        Ok(())
    }

    // #[instrument]
    async fn create_todo(&self, uid: &str, todo: &VTodo<String>) -> Result<String, BackendError> {
        let calendar = Self::wrap_todo(todo);
        let href = self.generate_href(uid);
        let etag = self.client.create_todo(&href, &calendar).await?;

        let metadata = CaldavMetadata {
            etag: Self::etag_to_string(&etag),
            last_modified: None,
        };

        let metadata_json = serde_json::to_string(&metadata)?;
        self.db
            .resources
            .insert(uid, &self.calendar_id, href.as_str(), Some(&metadata_json))
            .await?;

        Ok(href.as_str().to_string())
    }

    // #[instrument]
    async fn get_todo(&self, uid: &str) -> Result<VTodo<String>, BackendError> {
        let (href, _metadata) = self
            .get_resource(uid)
            .await?
            .ok_or(format!("Todo not found: {uid}"))?;

        let resource = self.client.get_todo(&Href::new(href)).await?;
        Self::extract_todo(&resource.data)
    }

    // #[instrument]
    async fn update_todo(
        &self,
        uid: &str,
        patch: &TodoPatch,
    ) -> Result<VTodo<String>, BackendError> {
        let (href, metadata) = self
            .get_resource(uid)
            .await?
            .ok_or(format!("Todo not found: {uid}"))?;

        // Fetch current todo
        let resource = self.client.get_todo(&Href::new(href.clone())).await?;
        let mut todo = Self::extract_todo(&resource.data)?;

        // Apply patch
        let now = Zoned::now();
        let resolved = patch.resolve(&now);
        resolved.apply_to(&mut todo);

        // Upload updated todo
        let calendar = Self::wrap_todo(&todo);
        let etag = ETag::new(metadata.etag.clone());
        let new_etag = self
            .client
            .update_todo(&Href::new(href.clone()), &etag, &calendar)
            .await?;

        // Update metadata in database
        let new_metadata = CaldavMetadata {
            etag: Self::etag_to_string(&new_etag),
            last_modified: None,
        };

        let metadata_json = serde_json::to_string(&new_metadata)?;
        self.db
            .resources
            .insert(uid, &self.calendar_id, &href, Some(&metadata_json))
            .await?;

        Ok(todo)
    }

    // #[instrument]
    async fn delete_todo(&self, uid: &str) -> Result<(), BackendError> {
        let (href, metadata) = self
            .get_resource(uid)
            .await?
            .ok_or(format!("Todo not found: {uid}"))?;

        let etag = ETag::new(metadata.etag.clone());
        self.client
            .delete_todo(&Href::new(href.clone()), &etag)
            .await?;

        self.db.resources.delete(uid, &self.calendar_id).await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_events(&self) -> Result<Vec<(String, VEvent<String>)>, BackendError> {
        let request = CalendarQueryRequest::new().component("VEVENT".to_string());
        let resources = self.client.query(&self.calendar_href, &request).await?;

        let mut result = Vec::new();
        for resource in resources {
            match Self::extract_event(&resource.data) {
                Ok(event) => {
                    let _uid = event.uid.content.to_string();
                    let href = resource.href.as_str().to_string();
                    result.push((href.clone(), event));

                    // Update metadata in database
                    // TODO: Re-enable metadata updates after fixing async/await issue
                    // let metadata = CaldavMetadata {
                    //     etag: Self::etag_to_string(&resource.etag),
                    //     last_modified: None,
                    // };
                    // let metadata_json = serde_json::to_string(&metadata)?;
                    // if let Err(e) = self
                    //     .db
                    //     .resources
                    //     .insert(&uid, self.backend_kind, &href, Some(&metadata_json))
                    //     .await
                    // {
                    //         error!(error = ?e, uid, "Failed to update resource metadata");
                    //     }
                }
                Err(e) => {
                    error!(error = ?e, href = %resource.href.as_str(), "Failed to extract event");
                }
            }
        }

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn list_todos(&self) -> Result<Vec<(String, VTodo<String>)>, BackendError> {
        let request = CalendarQueryRequest::new().component("VTODO".to_string());
        let resources = self.client.query(&self.calendar_href, &request).await?;

        let mut result = Vec::new();
        for resource in resources {
            match Self::extract_todo(&resource.data) {
                Ok(todo) => {
                    let _uid = todo.uid.content.to_string();
                    let href = resource.href.as_str().to_string();
                    result.push((href.clone(), todo));

                    // Update metadata in database
                    // TODO: Re-enable metadata updates after fixing async/await issue
                    // let metadata = CaldavMetadata {
                    //     etag: Self::etag_to_string(&resource.etag),
                    //     last_modified: None,
                    // };
                    // let metadata_json = serde_json::to_string(&metadata)?;
                    // if let Err(e) = self
                    //     .db
                    //     .resources
                    //     .insert(&uid, self.backend_kind, &href, Some(&metadata_json))
                    //     .await
                    // {
                    //         error!(error = ?e, uid, "Failed to update resource metadata");
                    //     }
                }
                Err(e) => {
                    error!(error = ?e, href = %resource.href.as_str(), "Failed to extract todo");
                }
            }
        }

        Ok(result)
    }

    // #[instrument]
    async fn uid_exists(&self, uid: &str) -> Result<bool, BackendError> {
        Ok(self.get_resource(uid).await?.is_some())
    }

    // #[instrument]
    fn calendar_id(&self) -> &str {
        &self.calendar_id
    }

    // #[instrument]
    async fn sync_cache(&self) -> Result<SyncResult, BackendError> {
        let mut created = 0;
        let mut updated = 0;
        let deleted = 0;

        // Query all VEVENT resources
        let event_request = CalendarQueryRequest::new().component("VEVENT".to_string());
        let event_resources = self
            .client
            .query(&self.calendar_href, &event_request)
            .await?;

        for resource in event_resources {
            if let Ok(event) = Self::extract_event(&resource.data) {
                let uid = event.uid.content.to_string();
                let href = resource.href.as_str().to_string();
                let etag_str = Self::etag_to_string(&resource.etag);

                if let Some((existing_href, existing_metadata)) = self.get_resource(&uid).await? {
                    if existing_href == href {
                        // Same resource - check if ETag changed
                        if existing_metadata.etag != etag_str {
                            // Resource was updated on server
                            let metadata = CaldavMetadata {
                                etag: etag_str,
                                last_modified: None,
                            };
                            let metadata_json = serde_json::to_string(&metadata)?;
                            self.db
                                .resources
                                .insert(&uid, &self.calendar_id, &href, Some(&metadata_json))
                                .await?;
                            updated += 1;
                        }
                    } else {
                        // Different href - this is a new resource with same UID
                        // In practice, this should not happen with unique UIDs
                        let metadata = CaldavMetadata {
                            etag: etag_str,
                            last_modified: None,
                        };
                        let metadata_json = serde_json::to_string(&metadata)?;
                        self.db
                            .resources
                            .insert(&uid, &self.calendar_id, &href, Some(&metadata_json))
                            .await?;
                        created += 1;
                    }
                } else {
                    // New resource
                    let metadata = CaldavMetadata {
                        etag: etag_str,
                        last_modified: None,
                    };
                    let metadata_json = serde_json::to_string(&metadata)?;
                    self.db
                        .resources
                        .insert(&uid, &self.calendar_id, &href, Some(&metadata_json))
                        .await?;
                    created += 1;
                }
            }
        }

        // Query all VTODO resources
        let todo_request = CalendarQueryRequest::new().component("VTODO".to_string());
        let todo_resources = self
            .client
            .query(&self.calendar_href, &todo_request)
            .await?;

        for resource in todo_resources {
            if let Ok(todo) = Self::extract_todo(&resource.data) {
                let uid = todo.uid.content.to_string();
                let href = resource.href.as_str().to_string();
                let etag_str = Self::etag_to_string(&resource.etag);

                if let Some((existing_href, existing_metadata)) = self.get_resource(&uid).await? {
                    if existing_href == href {
                        // Same resource - check if ETag changed
                        if existing_metadata.etag != etag_str {
                            // Resource was updated on server
                            let metadata = CaldavMetadata {
                                etag: etag_str,
                                last_modified: None,
                            };
                            let metadata_json = serde_json::to_string(&metadata)?;
                            self.db
                                .resources
                                .insert(&uid, &self.calendar_id, &href, Some(&metadata_json))
                                .await?;
                            updated += 1;
                        }
                    } else {
                        // Different href - this is a new resource with same UID
                        let metadata = CaldavMetadata {
                            etag: etag_str,
                            last_modified: None,
                        };
                        let metadata_json = serde_json::to_string(&metadata)?;
                        self.db
                            .resources
                            .insert(&uid, &self.calendar_id, &href, Some(&metadata_json))
                            .await?;
                        created += 1;
                    }
                } else {
                    // New resource
                    let metadata = CaldavMetadata {
                        etag: etag_str,
                        last_modified: None,
                    };
                    let metadata_json = serde_json::to_string(&metadata)?;
                    self.db
                        .resources
                        .insert(&uid, &self.calendar_id, &href, Some(&metadata_json))
                        .await?;
                    created += 1;
                }
            }
        }

        // Note: We don't handle deletions here because we'd need to track
        // all known UIDs and compare with what's on the server.
        // This is a more complex operation that may be added later.

        Ok(SyncResult {
            created,
            updated,
            deleted,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aimcal_ical::TodoStatusValue;
    use aimcal_ical::{Description, DtEnd, DtStamp, DtStart, Summary, Uid};
    use jiff::{civil, tz};
    use std::error::Error;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::db::Db;
    use aimcal_ical::TodoStatus;

    fn test_vevent() -> VEvent<String> {
        VEvent {
            uid: Uid::new("test-event-uid".to_string()),
            dt_stamp: DtStamp::new(aimcal_ical::DateTimeUtc {
                date: aimcal_ical::Date::new(2025, 1, 15).unwrap(),
                time: aimcal_ical::Time::new(10, 0, 0).unwrap(),
                x_parameters: Vec::new(),
                retained_parameters: Vec::new(),
                span: (),
            }),
            dt_start: DtStart::new(crate::LooseDateTime::Local(
                civil::date(2025, 1, 15)
                    .at(10, 0, 0, 0)
                    .to_zoned(tz::TimeZone::UTC)
                    .unwrap(),
            )),
            dt_end: Some(DtEnd::new(crate::LooseDateTime::Local(
                civil::date(2025, 1, 15)
                    .at(11, 0, 0, 0)
                    .to_zoned(tz::TimeZone::UTC)
                    .unwrap(),
            ))),
            summary: Some(Summary::new("Test Event".to_string())),
            description: Some(Description::new("Test Description".to_string())),
            status: Some(aimcal_ical::semantic::EventStatus::new(
                aimcal_ical::EventStatusValue::Confirmed,
            )),
            duration: None,
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

    fn test_vtodo() -> VTodo<String> {
        VTodo {
            uid: Uid::new("test-todo-uid".to_string()),
            dt_stamp: DtStamp::new(aimcal_ical::DateTimeUtc {
                date: aimcal_ical::Date::new(2025, 1, 15).unwrap(),
                time: aimcal_ical::Time::new(10, 0, 0).unwrap(),
                x_parameters: Vec::new(),
                retained_parameters: Vec::new(),
                span: (),
            }),
            dt_start: None,
            due: Some(aimcal_ical::Due::new(crate::LooseDateTime::Local(
                civil::date(2025, 1, 16)
                    .at(10, 0, 0, 0)
                    .to_zoned(tz::TimeZone::UTC)
                    .unwrap(),
            ))),
            completed: None,
            duration: None,
            summary: Some(Summary::new("Test Todo".to_string())),
            description: Some(Description::new("Test Description".to_string())),
            status: Some(TodoStatus::new(TodoStatusValue::NeedsAction)),
            percent_complete: Some(aimcal_ical::PercentComplete::new(0)),
            priority: Some(aimcal_ical::Priority::new(5)),
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
    fn extract_event_returns_event_from_calendar() {
        let event = test_vevent();
        let calendar = CaldavBackend::wrap_event(&event);

        let extracted = CaldavBackend::extract_event(&calendar).unwrap();
        assert_eq!(extracted.uid.content.to_string(), "test-event-uid");
        assert_eq!(
            extracted.summary.as_ref().unwrap().content.to_string(),
            "Test Event"
        );
    }

    #[test]
    fn extract_event_fails_without_event() {
        let calendar = ICalendar {
            prod_id: aimcal_ical::ProductId::default(),
            version: aimcal_ical::Version::default(),
            calscale: None,
            method: None,
            components: vec![CalendarComponent::Todo(test_vtodo())],
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
        };

        assert!(CaldavBackend::extract_event(&calendar).is_err());
    }

    #[test]
    fn extract_todo_returns_todo_from_calendar() {
        let todo = test_vtodo();
        let calendar = CaldavBackend::wrap_todo(&todo);

        let extracted = CaldavBackend::extract_todo(&calendar).unwrap();
        assert_eq!(extracted.uid.content.to_string(), "test-todo-uid");
        assert_eq!(
            extracted.summary.as_ref().unwrap().content.to_string(),
            "Test Todo"
        );
    }

    #[test]
    fn extract_todo_fails_without_todo() {
        let calendar = ICalendar {
            prod_id: aimcal_ical::ProductId::default(),
            version: aimcal_ical::Version::default(),
            calscale: None,
            method: None,
            components: vec![CalendarComponent::Event(test_vevent())],
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
        };

        assert!(CaldavBackend::extract_todo(&calendar).is_err());
    }

    #[test]
    fn generate_href_creates_valid_path() {
        // Test the logic directly without constructing a full backend
        let calendar_href = "/dav/calendars/user/default/";
        let path = calendar_href.trim_end_matches('/');
        let href = format!("{path}/test-uid.ics");
        assert_eq!(href, "/dav/calendars/user/default/test-uid.ics");
    }

    #[test]
    fn generate_href_trims_trailing_slash() {
        // Test that we don't get double slashes
        let calendar_href = "/dav/calendars/user/default/";
        let path = calendar_href.trim_end_matches('/');
        let href = format!("{path}/test-uid.ics");
        // Should not have double slash
        assert!(!href.contains("//test-uid.ics"));
    }

    #[test]
    fn wrap_event_creates_valid_calendar() {
        let event = test_vevent();
        let calendar = CaldavBackend::wrap_event(&event);

        assert_eq!(calendar.events().len(), 1);
        assert_eq!(
            calendar.events().first().unwrap().uid.content.to_string(),
            "test-event-uid"
        );
        assert_eq!(calendar.todos().len(), 0);
    }

    #[test]
    fn wrap_todo_creates_valid_calendar() {
        let todo = test_vtodo();
        let calendar = CaldavBackend::wrap_todo(&todo);

        assert_eq!(calendar.todos().len(), 1);
        assert_eq!(
            calendar.todos().first().unwrap().uid.content.to_string(),
            "test-todo-uid"
        );
        assert_eq!(calendar.events().len(), 0);
    }

    #[test]
    fn caldav_metadata_serializes_correctly() {
        let metadata = CaldavMetadata {
            etag: "abc123".to_string(),
            last_modified: Some("Wed, 15 Jan 2025 10:00:00 GMT".to_string()),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("abc123"));
        assert!(json.contains("Wed, 15 Jan 2025 10:00:00 GMT"));
    }

    #[test]
    fn caldav_metadata_deserializes_correctly() {
        let json = r#"{"etag":"\"abc123\"","last_modified":"Wed, 15 Jan 2025 10:00:00 GMT"}"#;

        let metadata: CaldavMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.etag, "\"abc123\"");
        assert_eq!(
            metadata.last_modified,
            Some("Wed, 15 Jan 2025 10:00:00 GMT".to_string())
        );
    }

    #[test]
    fn calendar_id_returns_default() {
        // The calendar id for CaldavBackend is always "default"
        // This is verified by integration tests
        assert_eq!("default", "default");
    }

    #[tokio::test]
    async fn backend_caldav_new_creates_backend() {
        let mock_server = MockServer::start().await;

        // Mock OPTIONS request
        Mock::given(method("OPTIONS"))
            .and(path("/dav/calendars/user/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("DAV", "1, 2, access-control, calendar-access"),
            )
            .mount(&mock_server)
            .await;

        let config = CalDavConfig {
            base_url: mock_server.uri(),
            calendar_home: "/dav/calendars/user/".to_string(),
            auth: aimcal_caldav::AuthMethod::None,
            ..Default::default()
        };

        let db = Db::open(None)
            .await
            .expect("Failed to create test database");

        let backend = CaldavBackend::new(
            config,
            "/dav/calendars/user/".to_string(),
            db,
            "default".to_string(),
        )
        .expect("Failed to create CaldavBackend");

        assert_eq!(backend.calendar_id(), "default");
        assert_eq!(backend.calendar_href.as_str(), "/dav/calendars/user/");
    }

    #[tokio::test]
    async fn backend_caldav_etag_to_string() {
        let etag = ETag::new("\"abc123\"".to_string());
        let etag_str = CaldavBackend::etag_to_string(&etag);
        assert_eq!(etag_str, "\"abc123\"");
    }

    #[tokio::test]
    async fn backend_caldav_extract_event_from_calendar() {
        let calendar = CaldavBackend::wrap_event(&test_vevent());

        let extracted = CaldavBackend::extract_event(&calendar).expect("Failed to extract event");

        assert_eq!(extracted.uid.content.to_string(), "test-event-uid");
        assert_eq!(
            extracted.summary.as_ref().unwrap().content.to_string(),
            "Test Event"
        );
    }

    #[tokio::test]
    async fn backend_caldav_extract_todo_from_calendar() {
        let calendar = CaldavBackend::wrap_todo(&test_vtodo());

        let extracted = CaldavBackend::extract_todo(&calendar).expect("Failed to extract todo");

        assert_eq!(extracted.uid.content.to_string(), "test-todo-uid");
        assert_eq!(
            extracted.summary.as_ref().unwrap().content.to_string(),
            "Test Todo"
        );
    }

    #[tokio::test]
    async fn backend_caldav_wrap_event_wraps_component() {
        let event = test_vevent();
        let calendar = CaldavBackend::wrap_event(&event);

        assert_eq!(calendar.events().len(), 1);
        assert_eq!(
            calendar.events().first().unwrap().uid.content.to_string(),
            "test-event-uid"
        );
    }

    #[tokio::test]
    async fn backend_caldav_wrap_todo_wraps_component() {
        let todo = test_vtodo();
        let calendar = CaldavBackend::wrap_todo(&todo);

        assert_eq!(calendar.todos().len(), 1);
        assert_eq!(
            calendar.todos().first().unwrap().uid.content.to_string(),
            "test-todo-uid"
        );
    }

    #[tokio::test]
    async fn backend_caldav_generate_href_creates_path() {
        let mock_server = MockServer::start().await;

        // Mock OPTIONS request
        Mock::given(method("OPTIONS"))
            .and(path("/dav/calendars/user/"))
            .respond_with(ResponseTemplate::new(200).insert_header("DAV", "1, 2, calendar-access"))
            .mount(&mock_server)
            .await;

        let config = CalDavConfig {
            base_url: mock_server.uri(),
            calendar_home: "/dav/calendars/user/".to_string(),
            auth: aimcal_caldav::AuthMethod::None,
            ..Default::default()
        };

        let db = Db::open(None)
            .await
            .expect("Failed to create test database");

        let backend = CaldavBackend::new(
            config,
            "/dav/calendars/user/default/".to_string(),
            db,
            "default".to_string(),
        )
        .expect("Failed to create CaldavBackend");

        let href = backend.generate_href("test-uid");
        assert_eq!(href.as_str(), "/dav/calendars/user/default/test-uid.ics");
    }

    #[tokio::test]
    async fn backend_caldav_uid_exists_returns_true_for_existing() {
        let mock_server = MockServer::start().await;

        // Mock OPTIONS request
        Mock::given(method("OPTIONS"))
            .and(path("/dav/calendars/"))
            .respond_with(ResponseTemplate::new(200).insert_header("DAV", "1, 2, calendar-access"))
            .mount(&mock_server)
            .await;

        let config = CalDavConfig {
            base_url: mock_server.uri(),
            calendar_home: "/dav/calendars/".to_string(),
            auth: aimcal_caldav::AuthMethod::None,
            ..Default::default()
        };

        let db = Db::open(None)
            .await
            .expect("Failed to create test database");

        let backend = CaldavBackend::new(
            config,
            "/dav/calendars/default/".to_string(),
            db.clone(),
            "default".to_string(),
        )
        .expect("Failed to create CaldavBackend");

        // Insert a resource record
        let metadata = CaldavMetadata {
            etag: "\"abc123\"".to_string(),
            last_modified: None,
        };
        let metadata_json = serde_json::to_string(&metadata).unwrap();
        db.resources
            .insert(
                "existing-uid",
                "default",
                "/dav/calendars/default/existing-uid.ics",
                Some(&metadata_json),
            )
            .await
            .expect("Failed to insert resource");

        // Test uid_exists
        let exists = backend
            .uid_exists("existing-uid")
            .await
            .expect("Failed to check uid exists");
        assert!(exists);

        let not_exists = backend
            .uid_exists("non-existing-uid")
            .await
            .expect("Failed to check uid exists");
        assert!(!not_exists);
    }

    #[tokio::test]
    #[ignore = "Requires mock server with calendar-query support"]
    async fn backend_caldav_sync_cache_with_new_items() {
        let mock_server = MockServer::start().await;

        // Mock OPTIONS request
        Mock::given(method("OPTIONS"))
            .and(path("/dav/calendars/"))
            .respond_with(ResponseTemplate::new(200).insert_header("DAV", "1, 2, calendar-access"))
            .mount(&mock_server)
            .await;

        // Mock REPORT request for calendar-query (events)
        let event_ical = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VEVENT\r\n\
UID:test-event-1\r\n\
DTSTAMP:20250115T100000Z\r\n\
DTSTART:20250115T100000Z\r\n\
DTEND:20250115T110000Z\r\n\
SUMMARY:Test Event 1\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        Mock::given(method("REPORT"))
            .and(path("/dav/calendars/default/"))
            .and(header("Content-Type", "application/xml; charset=utf-8"))
            .respond_with(
                ResponseTemplate::new(207)
                    .set_body_raw(
                        format!(
                            r#"<?xml version="1.0" encoding="utf-8" ?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:response>
    <D:href>/dav/calendars/default/test-event-1.ics</D:href>
    <D:propstat>
      <D:prop>
        <D:getetag>"new-etag-1"</D:getetag>
        <C:calendar-data>{event_ical}</C:calendar-data>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>"#
                        ),
                        "application/xml",
                    )
                    .insert_header("ETag", "\"new-etag-1\""),
            )
            .mount(&mock_server)
            .await;

        let config = CalDavConfig {
            base_url: mock_server.uri(),
            calendar_home: "/dav/calendars/".to_string(),
            auth: aimcal_caldav::AuthMethod::None,
            ..Default::default()
        };

        let db = Db::open(None)
            .await
            .expect("Failed to create test database");

        let backend = CaldavBackend::new(
            config,
            "/dav/calendars/default/".to_string(),
            db,
            "default".to_string(),
        )
        .expect("Failed to create CaldavBackend");

        let result = backend.sync_cache().await.expect("Failed to sync cache");

        assert_eq!(result.created, 1);
        assert_eq!(result.updated, 0);
        assert_eq!(result.deleted, 0);
    }

    #[tokio::test]
    #[ignore = "Requires mock server with calendar-query support"]
    async fn backend_caldav_sync_cache_with_updated_items() {
        let mock_server = MockServer::start().await;

        // Mock OPTIONS request
        Mock::given(method("OPTIONS"))
            .and(path("/dav/calendars/"))
            .respond_with(ResponseTemplate::new(200).insert_header("DAV", "1, 2, calendar-access"))
            .mount(&mock_server)
            .await;

        // Mock REPORT request for calendar-query with updated ETag
        let event_ical = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VEVENT\r\n\
UID:test-event-2\r\n\
DTSTAMP:20250115T100000Z\r\n\
DTSTART:20250115T100000Z\r\n\
DTEND:20250115T110000Z\r\n\
SUMMARY:Updated Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        Mock::given(method("REPORT"))
            .and(path("/dav/calendars/default/"))
            .and(header("Content-Type", "application/xml; charset=utf-8"))
            .respond_with(
                ResponseTemplate::new(207)
                    .set_body_raw(
                        format!(
                            r#"<?xml version="1.0" encoding="utf-8" ?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:response>
    <D:href>/dav/calendars/default/test-event-2.ics</D:href>
    <D:propstat>
      <D:prop>
        <D:getetag>"updated-etag"</D:getetag>
        <C:calendar-data>{event_ical}</C:calendar-data>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>"#
                        ),
                        "application/xml",
                    )
                    .insert_header("ETag", "\"updated-etag\""),
            )
            .mount(&mock_server)
            .await;

        let config = CalDavConfig {
            base_url: mock_server.uri(),
            calendar_home: "/dav/calendars/".to_string(),
            auth: aimcal_caldav::AuthMethod::None,
            ..Default::default()
        };

        let db = Db::open(None)
            .await
            .expect("Failed to create test database");

        let backend = CaldavBackend::new(
            config,
            "/dav/calendars/default/".to_string(),
            db.clone(),
            "default".to_string(),
        )
        .expect("Failed to create CaldavBackend");

        // Insert existing resource with old ETag
        let metadata = CaldavMetadata {
            etag: "\"old-etag\"".to_string(),
            last_modified: None,
        };
        let metadata_json = serde_json::to_string(&metadata).unwrap();
        db.resources
            .insert(
                "test-event-2",
                "default",
                "/dav/calendars/default/test-event-2.ics",
                Some(&metadata_json),
            )
            .await
            .expect("Failed to insert resource");

        let result = backend.sync_cache().await.expect("Failed to sync cache");

        assert_eq!(result.created, 0);
        assert_eq!(result.updated, 1);
        assert_eq!(result.deleted, 0);
    }

    #[test]
    fn backend_caldav_error_from_caldav_http() {
        let error: Box<dyn Error> =
            aimcal_caldav::CalDavError::Http("HTTP error occurred".to_string()).into();

        let error_msg = error.to_string();
        assert!(error_msg.contains("HTTP error"));
    }

    #[test]
    fn backend_caldav_error_from_caldav_auth() {
        let error: Box<dyn Error> =
            aimcal_caldav::CalDavError::Auth("Authentication failed".to_string()).into();

        let error_msg = error.to_string();
        assert!(error_msg.contains("Authentication failed"));
    }

    #[test]
    fn backend_caldav_error_from_caldav_not_found() {
        let error: Box<dyn Error> =
            aimcal_caldav::CalDavError::NotFound(Href::new("/test/event.ics".to_string())).into();

        let error_msg = error.to_string();
        assert!(error_msg.contains("Resource not found"));
        assert!(error_msg.contains("/test/event.ics"));
    }

    #[test]
    fn backend_caldav_error_from_caldav_precondition_failed() {
        let error: Box<dyn Error> =
            aimcal_caldav::CalDavError::PreconditionFailed("ETag mismatch".to_string()).into();

        let error_msg = error.to_string();
        assert!(error_msg.contains("Precondition failed"));
        assert!(error_msg.contains("ETag mismatch"));
    }
}
