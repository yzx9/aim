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
