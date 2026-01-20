// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! `CalDAV` client for calendar operations.

use std::io::Cursor;
use std::sync::Arc;

use aimcal_ical::{ICalendar, TodoStatusValue, formatter, parse};
use jiff::Zoned;
use jiff::civil::DateTime;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use reqwest::Method;

use crate::config::CalDavConfig;
use crate::error::CalDavError;
use crate::http::HttpClient;
use crate::request::{
    CalendarMultiGetRequest, CalendarQueryRequest, FreeBusyQueryRequest, Prop, PropFindRequest,
};
use crate::response::MultiStatusResponse;
use crate::todo_helper::{get_todo_status, is_completed_todo, is_pending_todo};
use crate::todo_overlap::todo_overlaps_time_range;
use crate::types::{CalendarCollection, CalendarResource, ETag, Href};
use crate::xml::ns;

/// `CalDAV` client for accessing and managing calendars on `CalDAV` servers.
///
/// # Example
///
/// ```ignore
/// use aimcal_caldav::{CalDavClient, CalDavConfig, AuthMethod};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = CalDavConfig {
///     base_url: "https://caldav.example.com".to_string(),
///     calendar_home: "/dav/calendars/user/".to_string(),
///     auth: AuthMethod::Basic {
///         username: "user".to_string(),
///         password: "pass".to_string(),
///     },
///     ..Default::default()
/// };
///
/// let client = CalDavClient::new(config).await?;
/// let calendars = client.list_calendars().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CalDavClient {
    http: Arc<HttpClient>,
    config: CalDavConfig,
}

impl CalDavClient {
    /// Creates a new `CalDAV` client.
    ///
    /// # Errors
    ///
    /// Returns an error if HTTP client initialization fails.
    pub fn new(config: CalDavConfig) -> Result<Self, CalDavError> {
        let http = HttpClient::new(config.clone())?;
        Ok(Self {
            http: Arc::new(http),
            config,
        })
    }

    /// Discovers `CalDAV` support and calendar home set.
    ///
    /// # Errors
    ///
    /// Returns an error if the server doesn't support `CalDAV` or discovery fails.
    pub async fn discover(&self) -> Result<DiscoverResult, CalDavError> {
        // Check for CalDAV support
        let url = self.full_url(&self.config.calendar_home);
        let resp = self
            .http
            .execute(self.http.build_request(Method::OPTIONS, &url))
            .await?;

        let dav_header = resp
            .headers()
            .get("DAV")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let supports_calendars = dav_header.contains("calendar-access");

        // Find calendar home set
        let mut propfind = PropFindRequest::new();
        propfind.add_property(Prop::CalendarHomeSet);

        let xml_body = propfind.build()?;
        let resp = self
            .http
            .execute(
                self.http
                    .build_request(
                        Method::from_bytes(b"PROPFIND")
                            .map_err(|e| CalDavError::Http(format!("Invalid method: {e}")))?,
                        &url,
                    )
                    .header("Content-Type", "application/xml; charset=utf-8")
                    .body(xml_body),
            )
            .await?;

        let xml = resp.text().await?;
        let multistatus = MultiStatusResponse::from_xml(&xml)?;

        let calendar_home = multistatus
            .responses
            .iter()
            .find_map(|r| {
                r.prop_stats
                    .iter()
                    .find(|p| p.status.contains("200"))
                    .and_then(|p| p.props.calendar_home_set.clone())
            })
            .unwrap_or_else(|| Href::new(self.config.calendar_home.clone()));

        Ok(DiscoverResult {
            supports_calendars,
            calendar_home,
        })
    }

    /// Creates a new calendar collection.
    ///
    /// # Errors
    ///
    /// Returns an error if MKCALENDAR fails.
    pub async fn mkcalendar(
        &self,
        href: &Href,
        display_name: &str,
        description: Option<&str>,
    ) -> Result<(), CalDavError> {
        let url = self.full_url(href.as_str());

        // Build MKCALENDAR request body
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        // <C:mkcalendar xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
        let mut mkcalendar = BytesStart::new("C:mkcalendar");
        mkcalendar.push_attribute(("xmlns:D", ns::DAV));
        mkcalendar.push_attribute(("xmlns:C", ns::CALDAV));
        writer.write_event(Event::Start(mkcalendar))?;

        // <D:set>
        writer.write_event(Event::Start(BytesStart::new("D:set")))?;

        // <D:prop>
        writer.write_event(Event::Start(BytesStart::new("D:prop")))?;

        // <D:displayname>
        writer.write_event(Event::Start(BytesStart::new("D:displayname")))?;
        writer.write_event(Event::Text(BytesText::new(display_name)))?;
        writer.write_event(Event::End(BytesEnd::new("D:displayname")))?;

        // <C:calendar-description>
        if let Some(desc) = description {
            writer.write_event(Event::Start(BytesStart::new("C:calendar-description")))?;
            writer.write_event(Event::Text(BytesText::new(desc)))?;
            writer.write_event(Event::End(BytesEnd::new("C:calendar-description")))?;
        }

        // </D:prop>
        writer.write_event(Event::End(BytesEnd::new("D:prop")))?;

        // </D:set>
        writer.write_event(Event::End(BytesEnd::new("D:set")))?;

        // </C:mkcalendar>
        writer.write_event(Event::End(BytesEnd::new("C:mkcalendar")))?;

        let body = String::from_utf8(writer.into_inner().into_inner())
            .map_err(|e| CalDavError::Xml(format!("UTF-8 error: {e}")))?;

        let _ = self
            .http
            .execute(
                self.http
                    .build_request(
                        Method::from_bytes(b"MKCALENDAR")
                            .map_err(|e| CalDavError::Http(format!("Invalid method: {e}")))?,
                        &url,
                    )
                    .header("Content-Type", "application/xml; charset=utf-8")
                    .body(body),
            )
            .await?;

        Ok(())
    }

    /// Gets a single calendar object by href.
    ///
    /// # Errors
    ///
    /// Returns an error if the object doesn't exist or parsing fails.
    pub async fn get_event(&self, href: &Href) -> Result<CalendarResource, CalDavError> {
        let url = self.full_url(href.as_str());
        let resp = self
            .http
            .execute(self.http.build_request(Method::GET, &url))
            .await?;

        let etag = HttpClient::extract_etag(&resp)?;
        let ical_data = resp.text().await?;

        let calendars = parse(&ical_data).map_err(|e| CalDavError::Ical(format!("{e:?}")))?;

        let data = calendars
            .into_iter()
            .next()
            .ok_or_else(|| CalDavError::InvalidResponse("No calendar data found".to_string()))?;

        Ok(CalendarResource::new(href.clone(), etag, data.to_owned()))
    }

    /// Gets a single todo by href.
    ///
    /// This is a convenience alias for [`get_event`] that works with VTODO components.
    ///
    /// # Errors
    ///
    /// Returns an error if the object doesn't exist or parsing fails.
    pub async fn get_todo(&self, href: &Href) -> Result<CalendarResource, CalDavError> {
        self.get_event(href).await
    }

    /// Creates a new calendar object.
    ///
    /// # Errors
    ///
    /// Returns an error if creation fails.
    pub async fn create_event(
        &self,
        href: &Href,
        calendar: &ICalendar<String>,
    ) -> Result<ETag, CalDavError> {
        let url = self.full_url(href.as_str());
        let ical_data = formatter::format(calendar)
            .map_err(|e| CalDavError::Ical(format!("Formatter error: {e}")))?;

        let resp = self
            .http
            .execute(
                self.http
                    .build_request(Method::PUT, &url)
                    .header("Content-Type", "text/calendar; charset=utf-8")
                    .body(ical_data),
            )
            .await?;

        HttpClient::extract_etag(&resp)
    }

    /// Creates a new todo.
    ///
    /// This is a convenience alias for [`create_event`] that works with VTODO components.
    ///
    /// # Errors
    ///
    /// Returns an error if creation fails.
    pub async fn create_todo(
        &self,
        href: &Href,
        calendar: &ICalendar<String>,
    ) -> Result<ETag, CalDavError> {
        self.create_event(href, calendar).await
    }

    /// Updates an existing calendar object.
    ///
    /// # Errors
    ///
    /// Returns an error if update fails or `ETag` mismatch.
    pub async fn update_event(
        &self,
        href: &Href,
        etag: &ETag,
        calendar: &ICalendar<String>,
    ) -> Result<ETag, CalDavError> {
        let url = self.full_url(href.as_str());
        let ical_data = formatter::format(calendar)
            .map_err(|e| CalDavError::Ical(format!("Formatter error: {e}")))?;

        let resp = self
            .http
            .execute(HttpClient::if_match(
                self.http
                    .build_request(Method::PUT, &url)
                    .header("Content-Type", "text/calendar; charset=utf-8")
                    .body(ical_data),
                etag,
            ))
            .await?;

        HttpClient::extract_etag(&resp)
    }

    /// Updates an existing todo.
    ///
    /// This is a convenience alias for [`update_event`] that works with VTODO components.
    ///
    /// # Errors
    ///
    /// Returns an error if update fails or `ETag` mismatch.
    pub async fn update_todo(
        &self,
        href: &Href,
        etag: &ETag,
        calendar: &ICalendar<String>,
    ) -> Result<ETag, CalDavError> {
        self.update_event(href, etag, calendar).await
    }

    /// Deletes a calendar object.
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails.
    pub async fn delete_event(&self, href: &Href, etag: &ETag) -> Result<(), CalDavError> {
        let url = self.full_url(href.as_str());

        self.http
            .execute(HttpClient::if_match(
                self.http.build_request(Method::DELETE, &url),
                etag,
            ))
            .await?;

        Ok(())
    }

    /// Deletes a todo.
    ///
    /// This is a convenience alias for [`delete_event`] that works with VTODO components.
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails.
    pub async fn delete_todo(&self, href: &Href, etag: &ETag) -> Result<(), CalDavError> {
        self.delete_event(href, etag).await
    }

    /// Queries calendar objects with filters.
    ///
    /// # Errors
    ///
    /// Returns an error if query fails.
    pub async fn query(
        &self,
        calendar_href: &Href,
        request: &CalendarQueryRequest,
    ) -> Result<Vec<CalendarResource>, CalDavError> {
        let url = self.full_url(calendar_href.as_str());
        let xml_body = request.build()?;

        let resp = self
            .http
            .execute(
                self.http
                    .build_request(
                        Method::from_bytes(b"REPORT")
                            .map_err(|e| CalDavError::Http(format!("Invalid method: {e}")))?,
                        &url,
                    )
                    .header("Content-Type", "application/xml; charset=utf-8")
                    .body(xml_body),
            )
            .await?;

        let xml = resp.text().await?;
        let multistatus = MultiStatusResponse::from_xml(&xml)?;
        multistatus.into_resources()
    }

    /// Retrieves multiple calendar objects by href.
    ///
    /// # Errors
    ///
    /// Returns an error if multiget fails.
    pub async fn multiget(&self, hrefs: &[Href]) -> Result<Vec<CalendarResource>, CalDavError> {
        if hrefs.is_empty() {
            return Ok(Vec::new());
        }

        let url = self.full_url(&self.config.calendar_home);

        let mut multiget = CalendarMultiGetRequest::new();
        for href in hrefs {
            multiget.add_href(href.as_str().to_string());
        }

        let xml_body = multiget.build()?;

        let resp = self
            .http
            .execute(
                self.http
                    .build_request(
                        Method::from_bytes(b"REPORT")
                            .map_err(|e| CalDavError::Http(format!("Invalid method: {e}")))?,
                        &url,
                    )
                    .header("Content-Type", "application/xml; charset=utf-8")
                    .body(xml_body),
            )
            .await?;

        let xml = resp.text().await?;
        let multistatus = MultiStatusResponse::from_xml(&xml)?;
        multistatus.into_resources()
    }

    /// Gets all pending todos from a calendar.
    ///
    /// Pending todos are those without a COMPLETED property and with status
    /// other than "COMPLETED" or "CANCELLED".
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn get_pending_todos(
        &self,
        calendar_href: &Href,
    ) -> Result<Vec<CalendarResource>, CalDavError> {
        let request = CalendarQueryRequest::new().component("VTODO".to_string());

        let resources = self.query(calendar_href, &request).await?;

        Ok(resources
            .into_iter()
            .filter(|resource| is_pending_todo(&resource.data))
            .collect())
    }

    /// Gets all completed todos from a calendar.
    ///
    /// Completed todos have a COMPLETED property or status "COMPLETED".
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn get_completed_todos(
        &self,
        calendar_href: &Href,
    ) -> Result<Vec<CalendarResource>, CalDavError> {
        let request = CalendarQueryRequest::new().component("VTODO".to_string());

        let resources = self.query(calendar_href, &request).await?;

        Ok(resources
            .into_iter()
            .filter(|resource| is_completed_todo(&resource.data))
            .collect())
    }

    /// Queries todos with optional time range and status filters.
    ///
    /// # Arguments
    ///
    /// * `calendar_href` - The calendar collection href
    /// * `start` - Optional start of time range (inclusive, UTC format)
    /// * `end` - Optional end of time range (exclusive, UTC format)
    /// * `statuses` - Optional list of status values to filter by
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn query_todos(
        &self,
        calendar_href: &Href,
        start: Option<&str>,
        end: Option<&str>,
        statuses: Option<&[TodoStatusValue]>,
    ) -> Result<Vec<CalendarResource>, CalDavError> {
        let request = if let Some(start) = start {
            CalendarQueryRequest::new()
                .component("VTODO".to_string())
                .time_range(start.to_string(), end.map(str::to_string))
        } else {
            CalendarQueryRequest::new().component("VTODO".to_string())
        };

        let resources = self.query(calendar_href, &request).await?;

        let mut filtered = resources;

        if let Some(statuses) = statuses {
            filtered.retain(|resource| {
                get_todo_status(&resource.data)
                    .is_some_and(|status| statuses.contains(&status))
            });
        }

        Ok(filtered)
    }

    /// Gets todos within a date range using RFC 4791 §9.9 VTODO overlap logic.
    ///
    /// This method queries the server for todos in the time range, then applies
    /// RFC 4791 §9.9 VTODO-specific overlap logic on the client side.
    ///
    /// # Arguments
    ///
    /// * `calendar_href` - The calendar collection href
    /// * `start` - Start of time range in UTC format (e.g., "20260101T000000Z")
    /// * `end` - End of time range in UTC format (e.g., "20260131T235959Z")
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn get_todos_by_date_range(
        &self,
        calendar_href: &Href,
        start: &str,
        end: &str,
    ) -> Result<Vec<CalendarResource>, CalDavError> {
        let request = CalendarQueryRequest::new()
            .component("VTODO".to_string())
            .time_range(start.to_string(), Some(end.to_string()));

        let resources = self.query(calendar_href, &request).await?;

        let start_dt = Self::parse_utc_datetime(start)?;
        let end_dt = Self::parse_utc_datetime(end)?;

        Ok(resources
            .into_iter()
            .filter(|resource| todo_overlaps_time_range(&resource.data, &start_dt, &end_dt))
            .collect())
    }

    /// Gets free/busy information.
    ///
    /// # Errors
    ///
    /// Returns an error if free-busy query fails.
    pub async fn free_busy(
        &self,
        calendar_href: &Href,
        start: &str,
        end: &str,
    ) -> Result<FreeBusyData, CalDavError> {
        let url = self.full_url(calendar_href.as_str());

        let request = FreeBusyQueryRequest::new(start.to_string(), end.to_string());
        let xml_body = request.build()?;

        let resp = self
            .http
            .execute(
                self.http
                    .build_request(
                        Method::from_bytes(b"REPORT")
                            .map_err(|e| CalDavError::Http(format!("Invalid method: {e}")))?,
                        &url,
                    )
                    .header("Content-Type", "application/xml; charset=utf-8")
                    .body(xml_body),
            )
            .await?;

        // Parse free-busy response
        let xml = resp.text().await?;

        // For now, just return the raw XML as string data
        // TODO: Parse free-busy data properly
        Ok(FreeBusyData {
            raw_data: Some(xml),
        })
    }

    /// Gets list of calendar collections.
    ///
    /// # Errors
    ///
    /// Returns an error if PROPFIND fails.
    pub async fn list_calendars(&self) -> Result<Vec<CalendarCollection>, CalDavError> {
        let url = self.full_url(&self.config.calendar_home);

        let mut propfind = PropFindRequest::new();
        propfind.add_property(Prop::DisplayName);
        propfind.add_property(Prop::ResourceType);
        propfind.add_property(Prop::CalendarDescription);
        propfind.add_property(Prop::SupportedCalendarComponents);

        let xml_body = propfind.build()?;
        let resp = self
            .http
            .execute(
                self.http
                    .build_request(
                        Method::from_bytes(b"PROPFIND")
                            .map_err(|e| CalDavError::Http(format!("Invalid method: {e}")))?,
                        &url,
                    )
                    .header("Content-Type", "application/xml; charset=utf-8")
                    .body(xml_body)
                    .header("Depth", "1"),
            )
            .await?;

        let xml = resp.text().await?;
        let multistatus = MultiStatusResponse::from_xml(&xml)?;
        Ok(multistatus.into_collections())
    }

    /// Builds full URL from href.
    fn full_url(&self, href: &str) -> String {
        format!("{}{}", self.config.base_url.trim_end_matches('/'), href)
    }

    /// Parses a UTC datetime string from `CalDAV` format.
    ///
    /// # Errors
    ///
    /// Returns an error if the datetime string is invalid.
    fn parse_utc_datetime(s: &str) -> Result<Zoned, CalDavError> {
        // CalDAV uses format like "20260101T000000Z"
        DateTime::strptime(s, "%Y%m%dT%H%M%SZ")
            .map_err(|e| CalDavError::InvalidResponse(format!("Invalid datetime format: {e}")))?
            .to_zoned(jiff::tz::TimeZone::UTC)
            .map_err(|e| CalDavError::InvalidResponse(format!("Datetime conversion error: {e}")))
    }
}

/// Result of `CalDAV` server discovery.
#[derive(Debug, Clone)]
pub struct DiscoverResult {
    /// Whether the server supports `CalDAV`.
    pub supports_calendars: bool,
    /// The calendar home set href.
    pub calendar_home: Href,
}

/// Free/busy data.
#[derive(Debug, Clone, Default)]
pub struct FreeBusyData {
    /// Raw free/busy data from server.
    pub raw_data: Option<String>,
}
