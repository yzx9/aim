// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::ops::Deref;

use aimcal_ical::ICalendar;

/// Calendar resource href (path).
///
/// A `Href` represents the path to a calendar resource on a `CalDAV` server,
/// such as `/calendars/user/event1.ics`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Href(String);

impl Href {
    /// Creates a new `Href` from a string.
    #[must_use]
    pub const fn new(href: String) -> Self {
        Self(href)
    }

    /// Returns the inner string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Href {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Href {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Href {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for Href {
    fn from(href: String) -> Self {
        Self(href)
    }
}

impl From<&str> for Href {
    fn from(href: &str) -> Self {
        Self(href.to_string())
    }
}

/// Entity tag for change detection.
///
/// An `ETag` represents an entity tag returned by the `CalDAV` server,
/// used for optimistic concurrency control and change detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ETag(String);

impl ETag {
    /// Creates a new `ETag` from a string.
    #[must_use]
    pub const fn new(etag: String) -> Self {
        Self(etag)
    }

    /// Returns the inner string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for ETag {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ETag {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ETag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for ETag {
    fn from(etag: String) -> Self {
        Self(etag)
    }
}

impl From<&str> for ETag {
    fn from(etag: &str) -> Self {
        Self(etag.to_string())
    }
}

/// A calendar object resource.
///
/// Represents a calendar object (event, todo, etc.) stored on a `CalDAV` server,
/// including its href, `ETag`, and iCalendar data.
#[derive(Debug, Clone)]
pub struct CalendarResource {
    /// The href of the resource.
    pub href: Href,
    /// The entity tag of the resource.
    pub etag: ETag,
    /// The iCalendar data.
    pub data: ICalendar<String>,
}

impl CalendarResource {
    /// Creates a new `CalendarResource`.
    #[must_use]
    pub const fn new(href: Href, etag: ETag, data: ICalendar<String>) -> Self {
        Self { href, etag, data }
    }
}

/// Calendar collection metadata.
///
/// Represents a calendar collection on a `CalDAV` server, containing
/// metadata about the calendar.
#[derive(Debug, Clone)]
pub struct CalendarCollection {
    /// The href of the calendar collection.
    pub href: Href,
    /// The display name of the calendar.
    pub display_name: Option<String>,
    /// The description of the calendar.
    pub description: Option<String>,
    /// Supported component types (VEVENT, VTODO, etc.).
    pub supported_components: Vec<String>,
    /// The collection tag (`CTag`) for change detection.
    pub ctag: Option<ETag>,
}

impl CalendarCollection {
    /// Creates a new `CalendarCollection`.
    #[must_use]
    pub fn new(href: Href) -> Self {
        Self {
            href,
            display_name: None,
            description: None,
            supported_components: Vec::new(),
            ctag: None,
        }
    }
}

/// Server capabilities discovered from the CalDAV server.
///
/// Represents the features and operations supported by the server,
/// as discovered via the DAV header and PROPFIND operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct ServerCapabilities {
    /// Whether the server supports CalDAV (calendar-access).
    pub supports_calendars: bool,
    /// Whether the server supports the MKCALENDAR method.
    pub supports_mkcalendar: bool,
    /// Whether the server supports calendar-query REPORT.
    pub supports_calendar_query: bool,
    /// Whether the server supports calendar-multiget REPORT.
    pub supports_calendar_multiget: bool,
    /// Whether the server supports free-busy-query REPORT.
    pub supports_free_busy: bool,
}

impl ServerCapabilities {
    /// Creates a new `ServerCapabilities` with all features unsupported.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            supports_calendars: false,
            supports_mkcalendar: false,
            supports_calendar_query: false,
            supports_calendar_multiget: false,
            supports_free_busy: false,
        }
    }

    /// Creates capabilities from a DAV header value.
    ///
    /// Parses the DAV header (e.g., "1, 2, calendar-access, extended-mkcol")
    /// and sets the appropriate capability flags.
    #[must_use]
    pub fn from_dav_header(dav_header: &str) -> Self {
        let mut caps = Self::new();
        let header = dav_header.to_lowercase();

        // RFC 4791: calendar-access indicates CalDAV support
        caps.supports_calendars = header.contains("calendar-access");

        // RFC 4791: extended-mkcol indicates MKCALENDAR support
        // (also implied by calendar-access on most servers)
        caps.supports_mkcalendar = header.contains("extended-mkcol") || caps.supports_calendars;

        // RFC 4791: calendar-access implies support for these REPORTs
        caps.supports_calendar_query = caps.supports_calendars;
        caps.supports_calendar_multiget = caps.supports_calendars;
        caps.supports_free_busy = caps.supports_calendars;

        caps
    }

    /// Checks if the server supports calendar-query REPORT.
    #[must_use]
    pub const fn can_query(&self) -> bool {
        self.supports_calendars && self.supports_calendar_query
    }

    /// Checks if the server supports calendar-multiget REPORT.
    #[must_use]
    pub const fn can_multiget(&self) -> bool {
        self.supports_calendars && self.supports_calendar_multiget
    }

    /// Checks if the server supports free-busy-query REPORT.
    #[must_use]
    pub const fn can_free_busy(&self) -> bool {
        self.supports_calendars && self.supports_free_busy
    }

    /// Checks if the server supports MKCALENDAR.
    #[must_use]
    pub const fn can_mkcalendar(&self) -> bool {
        self.supports_calendars && self.supports_mkcalendar
    }
}
