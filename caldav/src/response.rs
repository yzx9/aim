// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Response parsers for WebDAV/CalDAV operations.

use aimcal_ical::parse;
use quick_xml::events::Event;

use crate::error::CalDavError;
use crate::types::{CalendarCollection, CalendarResource, ETag, Href};

/// `WebDAV` multistatus response.
#[derive(Debug, Clone)]
pub struct MultiStatusResponse {
    /// The response items.
    pub responses: Vec<ResponseItem>,
}

/// Individual response in multistatus.
#[derive(Debug, Clone)]
pub struct ResponseItem {
    pub href: Href,
    pub prop_stats: Vec<PropStat>,
    pub status: Option<String>,
}

/// Property stat with status and value.
#[derive(Debug, Clone)]
pub struct PropStat {
    pub props: Properties,
    pub status: String,
}

/// WebDAV/CalDAV properties.
#[derive(Debug, Clone, Default)]
pub struct Properties {
    pub display_name: Option<String>,
    pub resource_type: Option<Vec<String>>,
    pub get_etag: Option<ETag>,
    pub calendar_data: Option<String>,
    pub calendar_home_set: Option<Href>,
    pub supported_calendar_components: Option<Vec<String>>,
    pub calendar_description: Option<String>,
    pub calendar_timezone: Option<String>,
    pub is_calendar: bool,
    pub is_collection: bool,
}

impl MultiStatusResponse {
    /// Parses multistatus response from XML.
    ///
    /// # Errors
    ///
    /// Returns an error if XML parsing fails.
    #[expect(clippy::too_many_lines)]
    pub fn from_xml(xml: &str) -> Result<Self, CalDavError> {
        let mut reader = quick_xml::Reader::from_str(xml);
        // Configure reader to trim text and check namespaces
        reader.config_mut().trim_text(true);
        reader.config_mut().check_end_names = true;

        let mut responses = Vec::new();
        let mut current_response: Option<ResponseItem> = None;
        let mut current_prop_stats: Vec<PropStat> = Vec::new();
        let mut current_props: Properties = Properties::default();
        let mut in_prop = false;
        let mut in_response = false;
        let mut in_propstat = false;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(ref e) if e.name().local_name().into_inner() == b"multistatus" => break,
                Event::Eof => break,

                Event::Start(ref e) => {
                    match e.name().local_name().into_inner() {
                        b"response" => {
                            in_response = true;
                            current_response = Some(ResponseItem {
                                href: Href::new(String::new()),
                                prop_stats: Vec::new(),
                                status: None,
                            });
                        }
                        b"href" if in_response => {
                            if let Event::Text(text) = reader.read_event_into(&mut buf)? {
                                let href = text.unescape()?.to_string();
                                if let Some(ref mut resp) = current_response {
                                    resp.href = Href::new(href);
                                }
                            }
                        }
                        b"propstat" if in_response => {
                            in_propstat = true;
                            current_props = Properties::default();
                        }

                        b"prop" if in_propstat => in_prop = true,
                        b"prop" => in_prop = true,

                        b"displayname" if in_prop => {
                            if let Event::Text(text) = reader.read_event_into(&mut buf)? {
                                current_props.display_name = Some(text.unescape()?.to_string());
                            }
                        }
                        b"resourcetype" if in_prop => {
                            current_props.is_collection = true;
                            // Look for calendar or collection
                            loop {
                                match reader.read_event_into(&mut buf)? {
                                    Event::End(ref e)
                                        if e.name().local_name().into_inner()
                                            == b"resourcetype" =>
                                    {
                                        break;
                                    }
                                    Event::Start(ref e) | Event::Empty(ref e) => {
                                        if e.name().local_name().into_inner() == b"calendar" {
                                            current_props.is_calendar = true;
                                        }
                                    }
                                    Event::Eof => {
                                        return Err(CalDavError::Xml("Unexpected EOF".to_string()));
                                    }
                                    _ => {}
                                }
                            }
                        }
                        b"getetag" if in_prop => {
                            if let Event::Text(text) = reader.read_event_into(&mut buf)? {
                                current_props.get_etag =
                                    Some(ETag::new(text.unescape()?.to_string()));
                            }
                        }
                        b"calendar-data" if in_prop => {
                            if let Event::Text(text) = reader.read_event_into(&mut buf)? {
                                current_props.calendar_data = Some(text.unescape()?.to_string());
                            }
                        }
                        b"calendar-home-set" if in_prop => {
                            // Read href content
                            loop {
                                match reader.read_event_into(&mut buf)? {
                                    Event::End(ref e)
                                        if e.name().local_name().into_inner()
                                            == b"calendar-home-set" =>
                                    {
                                        break;
                                    }
                                    Event::Start(ref e)
                                        if e.name().local_name().into_inner() == b"href" =>
                                    {
                                        if let Event::Text(text) =
                                            reader.read_event_into(&mut buf)?
                                        {
                                            current_props.calendar_home_set =
                                                Some(Href::new(text.unescape()?.to_string()));
                                        }
                                    }
                                    Event::Eof => {
                                        return Err(CalDavError::Xml("Unexpected EOF".to_string()));
                                    }
                                    _ => {}
                                }
                            }
                        }
                        b"supported-calendar-component-set" if in_prop => {
                            let mut components = Vec::new();
                            loop {
                                match reader.read_event_into(&mut buf)? {
                                    Event::End(ref e)
                                        if e.name().local_name().into_inner()
                                            == b"supported-calendar-component-set" =>
                                    {
                                        break;
                                    }
                                    Event::Start(ref e) | Event::Empty(ref e)
                                        if e.name().local_name().into_inner() == b"comp" =>
                                    {
                                        if let Ok(Some(name_attr)) = e.try_get_attribute("name") {
                                            let name = std::str::from_utf8(&name_attr.value)
                                                .map_err(|e| {
                                                    CalDavError::Xml(format!("UTF-8 error: {e}"))
                                                })?
                                                .to_string();
                                            components.push(name);
                                        }
                                    }
                                    Event::Eof => {
                                        return Err(CalDavError::Xml("Unexpected EOF".to_string()));
                                    }
                                    _ => {}
                                }
                            }
                            current_props.supported_calendar_components = Some(components);
                        }
                        b"calendar-description" if in_prop => {
                            if let Event::Text(text) = reader.read_event_into(&mut buf)? {
                                current_props.calendar_description =
                                    Some(text.unescape()?.to_string());
                            }
                        }
                        b"calendar-timezone" if in_prop => {
                            if let Event::Text(text) = reader.read_event_into(&mut buf)? {
                                current_props.calendar_timezone =
                                    Some(text.unescape()?.to_string());
                            }
                        }
                        b"status" if in_propstat => {
                            if let Event::Text(text) = reader.read_event_into(&mut buf)? {
                                let status = text.unescape()?.to_string();
                                current_prop_stats.push(PropStat {
                                    props: current_props.clone(),
                                    status,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                Event::End(ref e) => match e.name().local_name().into_inner() {
                    b"response" if in_response => {
                        in_response = false;
                        if let Some(mut resp) = current_response.take() {
                            resp.prop_stats.clone_from(&current_prop_stats);
                            current_prop_stats.clear();
                            responses.push(resp);
                        }
                    }
                    b"propstat" if in_propstat => {
                        in_propstat = false;
                    }
                    b"prop" => {
                        in_prop = false;
                    }
                    _ => {}
                },
                _ => {}
            }
            buf.clear();
        }

        Ok(Self { responses })
    }

    /// Converts multistatus response to calendar resources.
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails.
    pub fn into_resources(self) -> Result<Vec<CalendarResource>, CalDavError> {
        let mut resources = Vec::new();

        for response in self.responses {
            // Find successful propstat (status starts with "HTTP/1.1 200" or "HTTP/1.1 207")
            for prop_stat in &response.prop_stats {
                if prop_stat.status.contains("200") || prop_stat.status.contains("207") {
                    // Check if we have calendar data
                    if let Some(data) = &prop_stat.props.calendar_data {
                        // Parse iCalendar data
                        let calendars = parse(data)
                            .map_err(|e| CalDavError::Ical(format!("Parse error: {e:?}")))?;

                        for calendar in calendars {
                            let owned = calendar.to_owned();
                            resources.push(CalendarResource::new(
                                response.href.clone(),
                                prop_stat
                                    .props
                                    .get_etag
                                    .clone()
                                    .unwrap_or_else(|| ETag::new(String::new())),
                                owned,
                            ));
                        }
                    }
                }
            }
        }

        Ok(resources)
    }

    /// Converts multistatus response to calendar collections.
    #[must_use]
    pub fn into_collections(self) -> Vec<CalendarCollection> {
        let mut collections = Vec::new();

        for response in self.responses {
            for prop_stat in &response.prop_stats {
                if prop_stat.status.contains("200") || prop_stat.status.contains("207") {
                    // Only include if it's a calendar collection
                    if prop_stat.props.is_calendar && prop_stat.props.is_collection {
                        let mut collection = CalendarCollection::new(response.href.clone());
                        collection
                            .display_name
                            .clone_from(&prop_stat.props.display_name);
                        collection
                            .description
                            .clone_from(&prop_stat.props.calendar_description);
                        collection.supported_components = prop_stat
                            .props
                            .supported_calendar_components
                            .clone()
                            .unwrap_or_default();
                        collection.ctag.clone_from(&prop_stat.props.get_etag);
                        collections.push(collection);
                    }
                }
            }
        }

        collections
    }
}
