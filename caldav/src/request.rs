// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Request builders for `CalDAV` operations.

use std::io::Cursor;

use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

use crate::error::CalDavError;
use crate::xml::ns;

/// PROPFIND request builder.
#[derive(Debug)]
pub struct PropFindRequest {
    props: Vec<Prop>,
}

/// Properties to request in PROPFIND.
#[derive(Debug, Clone, Copy)]
pub enum Prop {
    /// Display name.
    DisplayName,
    /// Resource type.
    ResourceType,
    /// `ETag`.
    GetETag,
    /// Calendar data.
    CalendarData,
    /// Calendar home set.
    CalendarHomeSet,
    /// Supported calendar components.
    SupportedCalendarComponents,
    /// Calendar description.
    CalendarDescription,
    /// Calendar timezone.
    CalendarTimezone,
}

impl Prop {
    const fn name(self) -> &'static str {
        match self {
            Self::DisplayName => "displayname",
            Self::ResourceType => "resourcetype",
            Self::GetETag => "getetag",
            Self::CalendarData => "calendar-data",
            Self::CalendarHomeSet => "calendar-home-set",
            Self::SupportedCalendarComponents => "supported-calendar-component-set",
            Self::CalendarDescription => "calendar-description",
            Self::CalendarTimezone => "calendar-timezone",
        }
    }

    const fn namespace(self) -> Option<&'static str> {
        match self {
            Self::DisplayName | Self::ResourceType | Self::GetETag => None,
            Self::CalendarData
            | Self::CalendarHomeSet
            | Self::SupportedCalendarComponents
            | Self::CalendarDescription
            | Self::CalendarTimezone => Some(ns::CALDAV),
        }
    }
}

impl PropFindRequest {
    /// Creates a new PROPFIND request.
    #[must_use]
    pub fn new() -> Self {
        Self { props: Vec::new() }
    }

    /// Adds a property to the request.
    pub fn add_property(&mut self, prop: Prop) -> &mut Self {
        self.props.push(prop);
        self
    }

    /// Builds the XML body for the PROPFIND request.
    ///
    /// # Errors
    ///
    /// Returns an error if XML building fails.
    pub fn build(&self) -> Result<String, CalDavError> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        // <D:propfind xmlns:D="DAV:">
        let mut propfind = BytesStart::new("D:propfind");
        propfind.push_attribute(("xmlns:D", ns::DAV));
        if self.props.iter().any(|p| p.namespace().is_some()) {
            propfind.push_attribute(("xmlns:C", ns::CALDAV));
        }
        writer.write_event(Event::Start(propfind))?;

        // <D:prop>
        writer.write_event(Event::Start(BytesStart::new("D:prop")))?;

        // Properties
        for prop in &self.props {
            let name = prop.name();

            if prop.namespace().is_some() {
                let elem = BytesStart::new(format!("C:{name}"));
                // Namespace already declared on propfind
                writer.write_event(Event::Start(elem))?;
                writer.write_event(Event::End(BytesEnd::new(format!("C:{name}"))))?;
            } else {
                writer.write_event(Event::Start(BytesStart::new(format!("D:{name}"))))?;
                writer.write_event(Event::End(BytesEnd::new(format!("D:{name}"))))?;
            }
        }

        // </D:prop>
        writer.write_event(Event::End(BytesEnd::new("D:prop")))?;

        // </D:propfind>
        writer.write_event(Event::End(BytesEnd::new("D:propfind")))?;

        let bytes = writer.into_inner().into_inner();
        String::from_utf8(bytes).map_err(|e| CalDavError::Xml(format!("UTF-8 error: {e}")))
    }
}

impl Default for PropFindRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Calendar query request builder.
#[derive(Debug)]
pub struct CalendarQueryRequest {
    time_range: Option<TimeRange>,
    #[expect(dead_code)]
    text_match: Option<TextMatch>,
    component: Option<String>,
}

/// Time range filter for calendar queries.
#[derive(Debug, Clone)]
pub struct TimeRange {
    /// Start date/time.
    pub start: String,
    /// End date/time.
    pub end: Option<String>,
}

/// Text match filter for calendar queries.
#[derive(Debug, Clone)]
pub struct TextMatch {
    /// Text to search for.
    pub text: String,
    /// Collation to use.
    pub collation: Option<String>,
    /// Whether to negate the match.
    pub negate: bool,
}

impl CalendarQueryRequest {
    /// Creates a new calendar query request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            time_range: None,
            text_match: None,
            component: None,
        }
    }

    /// Sets the time range filter.
    #[must_use]
    pub fn time_range(mut self, start: String, end: Option<String>) -> Self {
        self.time_range = Some(TimeRange { start, end });
        self
    }

    /// Sets the component filter (VEVENT, VTODO, etc.).
    #[must_use]
    pub fn component(mut self, component: String) -> Self {
        self.component = Some(component);
        self
    }

    /// Builds the XML body for the calendar query request.
    ///
    /// # Errors
    ///
    /// Returns an error if XML building fails.
    pub fn build(&self) -> Result<String, CalDavError> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        // <C:calendar-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
        let mut calendar_query = BytesStart::new("C:calendar-query");
        calendar_query.push_attribute(("xmlns:D", ns::DAV));
        calendar_query.push_attribute(("xmlns:C", ns::CALDAV));
        writer.write_event(Event::Start(calendar_query))?;

        // <D:prop>
        writer.write_event(Event::Start(BytesStart::new("D:prop")))?;
        writer.write_event(Event::Start(BytesStart::new("D:getetag")))?;
        writer.write_event(Event::End(BytesEnd::new("D:getetag")))?;
        writer.write_event(Event::Start(BytesStart::new("C:calendar-data")))?;
        writer.write_event(Event::End(BytesEnd::new("C:calendar-data")))?;
        writer.write_event(Event::End(BytesEnd::new("D:prop")))?;

        // <C:filter>
        writer.write_event(Event::Start(BytesStart::new("C:filter")))?;

        // <C:comp-filter name="VCALENDAR">
        let mut comp_filter = BytesStart::new("C:comp-filter");
        comp_filter.push_attribute(("name", "VCALENDAR"));
        writer.write_event(Event::Start(comp_filter))?;

        // Component filter (VEVENT, VTODO, etc.)
        if let Some(component) = &self.component {
            let mut comp_filter_inner = BytesStart::new("C:comp-filter");
            comp_filter_inner.push_attribute(("name", component.as_str()));
            writer.write_event(Event::Start(comp_filter_inner))?;

            // Time range filter
            if let Some(tr) = &self.time_range {
                let mut time_range = BytesStart::new("C:time-range");
                time_range.push_attribute(("start", tr.start.as_str()));
                if let Some(end) = &tr.end {
                    time_range.push_attribute(("end", end.as_str()));
                }
                writer.write_event(Event::Empty(time_range))?;
            }

            writer.write_event(Event::End(BytesEnd::new("C:comp-filter")))?;
        }

        // </C:comp-filter>
        writer.write_event(Event::End(BytesEnd::new("C:comp-filter")))?;

        // </C:filter>
        writer.write_event(Event::End(BytesEnd::new("C:filter")))?;

        // </C:calendar-query>
        writer.write_event(Event::End(BytesEnd::new("C:calendar-query")))?;

        let bytes = writer.into_inner().into_inner();
        String::from_utf8(bytes).map_err(|e| CalDavError::Xml(format!("UTF-8 error: {e}")))
    }
}

impl Default for CalendarQueryRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Calendar multiget request builder.
#[derive(Debug)]
pub struct CalendarMultiGetRequest {
    hrefs: Vec<String>,
}

impl CalendarMultiGetRequest {
    /// Creates a new calendar multiget request.
    #[must_use]
    pub fn new() -> Self {
        Self { hrefs: Vec::new() }
    }

    /// Adds an href to the request.
    pub fn add_href(&mut self, href: String) -> &mut Self {
        self.hrefs.push(href);
        self
    }

    /// Builds the XML body for the calendar multiget request.
    ///
    /// # Errors
    ///
    /// Returns an error if XML building fails.
    pub fn build(&self) -> Result<String, CalDavError> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        // <C:calendar-multiget xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
        let mut multiget = BytesStart::new("C:calendar-multiget");
        multiget.push_attribute(("xmlns:D", ns::DAV));
        multiget.push_attribute(("xmlns:C", ns::CALDAV));
        writer.write_event(Event::Start(multiget))?;

        // <D:prop>
        writer.write_event(Event::Start(BytesStart::new("D:prop")))?;
        writer.write_event(Event::Start(BytesStart::new("D:getetag")))?;
        writer.write_event(Event::End(BytesEnd::new("D:getetag")))?;
        writer.write_event(Event::Start(BytesStart::new("C:calendar-data")))?;
        writer.write_event(Event::End(BytesEnd::new("C:calendar-data")))?;
        writer.write_event(Event::End(BytesEnd::new("D:prop")))?;

        // <D:href> for each href
        for href in &self.hrefs {
            writer.write_event(Event::Start(BytesStart::new("D:href")))?;
            writer.write_event(Event::Text(BytesText::new(href.as_str())))?;
            writer.write_event(Event::End(BytesEnd::new("D:href")))?;
        }

        // </C:calendar-multiget>
        writer.write_event(Event::End(BytesEnd::new("C:calendar-multiget")))?;

        let bytes = writer.into_inner().into_inner();
        String::from_utf8(bytes).map_err(|e| CalDavError::Xml(format!("UTF-8 error: {e}")))
    }
}

impl Default for CalendarMultiGetRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Free/busy query request builder.
#[derive(Debug)]
pub struct FreeBusyQueryRequest {
    start: String,
    end: String,
}

impl FreeBusyQueryRequest {
    /// Creates a new free/busy query request.
    #[must_use]
    pub fn new(start: String, end: String) -> Self {
        Self { start, end }
    }

    /// Builds the XML body for the free/busy query request.
    ///
    /// # Errors
    ///
    /// Returns an error if XML building fails.
    pub fn build(&self) -> Result<String, CalDavError> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        // <C:free-busy-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
        let mut free_busy = BytesStart::new("C:free-busy-query");
        free_busy.push_attribute(("xmlns:D", ns::DAV));
        free_busy.push_attribute(("xmlns:C", ns::CALDAV));
        writer.write_event(Event::Start(free_busy))?;

        // <C:time-range start="..." end="..."/>
        let mut time_range = BytesStart::new("C:time-range");
        time_range.push_attribute(("start", self.start.as_str()));
        time_range.push_attribute(("end", self.end.as_str()));
        writer.write_event(Event::Empty(time_range))?;

        // </C:free-busy-query>
        writer.write_event(Event::End(BytesEnd::new("C:free-busy-query")))?;

        let bytes = writer.into_inner().into_inner();
        String::from_utf8(bytes).map_err(|e| CalDavError::Xml(format!("UTF-8 error: {e}")))
    }
}
