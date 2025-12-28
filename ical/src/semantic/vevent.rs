// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event component (VEVENT) for iCalendar semantic components.

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::RecurrenceRule;
use crate::keyword::{
    KW_EVENT_STATUS_CANCELLED, KW_EVENT_STATUS_CONFIRMED, KW_EVENT_STATUS_TENTATIVE,
    KW_TRANSP_OPAQUE, KW_TRANSP_TRANSPARENT, KW_VALARM, KW_VEVENT,
};
use crate::semantic::property_util::{
    get_language, get_single_value, get_tzid, parse_multi_text_property,
    value_to_floating_date_time, value_to_int, value_to_string,
};
use crate::semantic::{
    Attendee, Classification, DateTime, Geo, Organizer, Period, SemanticError, Text, Uri, VAlarm,
};
use crate::typed::{PropertyKind, TypedComponent, TypedProperty, Value, ValueDate, ValueDuration};

/// Event component (VEVENT)
#[derive(Debug, Clone)]
pub struct VEvent {
    /// Unique identifier for the event
    pub uid: String,

    /// Date/time the event was created
    pub dt_stamp: DateTime,

    /// Date/time the event starts
    pub dt_start: DateTime,

    /// Date/time the event ends
    pub dt_end: Option<DateTime>,

    /// Duration of the event (alternative to `dt_end`)
    pub duration: Option<ValueDuration>,

    /// Summary/title of the event
    pub summary: Option<Text>,

    /// Description of the event
    pub description: Option<Text>,

    /// Location of the event
    pub location: Option<Text>,

    /// Geographic position
    pub geo: Option<Geo>,

    /// URL associated with the event
    pub url: Option<Uri>,

    /// Organizer of the event
    pub organizer: Option<Organizer>,

    /// Attendees of the event
    pub attendees: Vec<Attendee>,

    /// Last modification date/time
    pub last_modified: Option<DateTime>,

    /// Status of the event
    pub status: Option<EventStatus>,

    /// Time transparency
    pub transparency: Option<TimeTransparency>,

    /// Sequence number for revisions
    pub sequence: Option<u32>,

    /// Priority (1-9, 1 is highest)
    pub priority: Option<u8>,

    /// Classification
    pub classification: Option<Classification>,

    /// Resources
    pub resources: Vec<Text>,

    /// Categories
    pub categories: Vec<Text>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period>,

    /// Exception dates
    pub ex_date: Vec<DateTime>,

    /// Timezone identifier
    pub tz_id: Option<String>,

    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
    /// Sub-components (like alarms)
    pub alarms: Vec<VAlarm>,
}

/// Parse a `TypedComponent` into a `VEvent`
#[allow(clippy::too_many_lines)]
impl TryFrom<&TypedComponent<'_>> for VEvent {
    type Error = Vec<SemanticError>;

    fn try_from(comp: &TypedComponent<'_>) -> Result<Self, Self::Error> {
        if comp.name != KW_VEVENT {
            return Err(vec![SemanticError::InvalidStructure(format!(
                "Expected VEVENT component, got '{}'",
                comp.name
            ))]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in &comp.properties {
            match prop.kind {
                PropertyKind::Uid => {
                    if props.uid.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Uid));
                        continue;
                    }
                    match get_single_value(prop).ok().and_then(value_to_string) {
                        Some(v) => props.uid = Some(v),
                        None => {
                            errors.push(SemanticError::InvalidValue(
                                PropertyKind::Uid,
                                "Expected text value".to_string(),
                            ));
                            props.uid = Some(String::new());
                        }
                    }
                }
                PropertyKind::DtStamp => {
                    if props.dt_stamp.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::DtStamp));
                        continue;
                    }
                    match get_single_value(prop)
                        .ok()
                        .and_then(value_to_floating_date_time)
                    {
                        Some(v) => props.dt_stamp = Some(v),
                        None => {
                            errors.push(SemanticError::InvalidValue(
                                PropertyKind::DtStamp,
                                "Expected date-time value".to_string(),
                            ));
                            props.dt_stamp = Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            });
                        }
                    }
                }
                PropertyKind::DtStart => {
                    if props.dt_start.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::DtStart));
                        continue;
                    }
                    match DateTime::try_from(prop) {
                        Ok(v) => props.dt_start = Some(v),
                        Err(e) => {
                            errors.push(e);
                            props.dt_start = Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            });
                        }
                    }
                }
                PropertyKind::DtEnd => {
                    if props.dt_end.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::DtEnd));
                        continue;
                    }
                    match DateTime::try_from(prop) {
                        Ok(v) => props.dt_end = Some(v),
                        Err(e) => {
                            errors.push(e);
                            props.dt_end = Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            });
                        }
                    }
                }
                PropertyKind::Duration => {
                    if props.duration.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Duration));
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(Value::Duration(v)) => props.duration = Some(*v),
                        _ => {
                            errors.push(SemanticError::InvalidValue(
                                PropertyKind::Duration,
                                "Expected duration value".to_string(),
                            ));
                            props.duration = Some(ValueDuration::DateTime {
                                positive: true,
                                day: 0,
                                hour: 0,
                                minute: 0,
                                second: 0,
                            });
                        }
                    }
                }
                PropertyKind::Summary => {
                    if props.summary.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Summary));
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(v) => {
                                props.summary = Some(Text {
                                    content: v,
                                    language: get_language(&prop.parameters),
                                });
                            }
                            None => {
                                errors.push(SemanticError::InvalidValue(
                                    PropertyKind::Summary,
                                    "Expected text value".to_string(),
                                ));
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Description => {
                    if props.description.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Description));
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(v) => {
                                props.description = Some(Text {
                                    content: v,
                                    language: get_language(&prop.parameters),
                                });
                            }
                            None => {
                                errors.push(SemanticError::InvalidValue(
                                    PropertyKind::Description,
                                    "Expected text value".to_string(),
                                ));
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Location => {
                    if props.location.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Location));
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(v) => {
                                props.location = Some(Text {
                                    content: v,
                                    language: get_language(&prop.parameters),
                                });
                            }
                            None => {
                                errors.push(SemanticError::InvalidValue(
                                    PropertyKind::Location,
                                    "Expected text value".to_string(),
                                ));
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Geo => {
                    if props.geo.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Geo));
                        continue;
                    }
                    match Geo::try_from(prop) {
                        Ok(v) => props.geo = Some(v),
                        Err(e) => {
                            errors.push(e);
                            props.geo = Some(Geo { lat: 0.0, lon: 0.0 });
                        }
                    }
                }
                PropertyKind::Url => {
                    if props.url.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Url));
                        continue;
                    }
                    match Uri::try_from(prop) {
                        Ok(v) => props.url = Some(v),
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Organizer => {
                    if props.organizer.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Organizer));
                        continue;
                    }
                    match Organizer::try_from(prop) {
                        Ok(v) => props.organizer = Some(v),
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Attendee => {
                    props.attendees.push(prop);
                }
                PropertyKind::LastModified => {
                    if props.last_modified.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::LastModified));
                        continue;
                    }
                    match get_single_value(prop)
                        .ok()
                        .and_then(value_to_floating_date_time)
                    {
                        Some(v) => props.last_modified = Some(v),
                        None => {
                            errors.push(SemanticError::InvalidValue(
                                PropertyKind::LastModified,
                                "Expected date-time value".to_string(),
                            ));
                            props.last_modified = Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            });
                        }
                    }
                }
                PropertyKind::Status => {
                    if props.status.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Status));
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(text) => match text.parse() {
                                Ok(v) => props.status = Some(v),
                                Err(e) => errors
                                    .push(SemanticError::InvalidValue(PropertyKind::Status, e)),
                            },
                            None => {
                                errors.push(SemanticError::InvalidValue(
                                    PropertyKind::Status,
                                    "Expected text value".to_string(),
                                ));
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Transp => {
                    if props.transparency.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Transp));
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(text) => match text.parse() {
                                Ok(v) => props.transparency = Some(v),
                                Err(e) => errors
                                    .push(SemanticError::InvalidValue(PropertyKind::Transp, e)),
                            },
                            None => {
                                errors.push(SemanticError::InvalidValue(
                                    PropertyKind::Transp,
                                    "Expected text value".to_string(),
                                ));
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Sequence => {
                    if props.sequence.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Sequence));
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_int::<u32>(value) {
                            Some(v) => props.sequence = Some(v),
                            None => {
                                errors.push(SemanticError::InvalidValue(
                                    PropertyKind::Sequence,
                                    "Expected integer value".to_string(),
                                ));
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Priority => {
                    if props.priority.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Priority));
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_int::<u8>(value) {
                            Some(v) => props.priority = Some(v),
                            None => {
                                errors.push(SemanticError::InvalidValue(
                                    PropertyKind::Priority,
                                    "Expected integer value".to_string(),
                                ));
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Class => {
                    if props.classification.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Class));
                        continue;
                    }
                    match Classification::try_from(prop) {
                        Ok(v) => props.classification = Some(v),
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Resources => {
                    if props.resources.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Resources));
                        continue;
                    }
                    props.resources = Some(parse_multi_text_property(prop));
                }
                PropertyKind::Categories => {
                    if props.categories.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Categories));
                        continue;
                    }
                    props.categories = Some(parse_multi_text_property(prop));
                }
                PropertyKind::RRule => {
                    if props.rrule.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::RRule));
                        continue;
                    }
                    // TODO: Parse RRULE from text format
                    match get_single_value(prop) {
                        Ok(Value::Text(_)) => {}
                        Ok(_) => {
                            errors.push(SemanticError::InvalidValue(
                                PropertyKind::RRule,
                                "Expected text value".to_string(),
                            ));
                        }
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::ExDate => {
                    props.ex_dates.push(prop);
                }
                // Ignore unknown properties
                _ => {}
            }
        }

        // Check required fields
        if props.uid.is_none() {
            errors.push(SemanticError::MissingProperty(PropertyKind::Uid));
        }
        if props.dt_stamp.is_none() {
            errors.push(SemanticError::MissingProperty(PropertyKind::DtStamp));
        }
        if props.dt_start.is_none() {
            errors.push(SemanticError::MissingProperty(PropertyKind::DtStart));
        }

        // Parse multi-value properties
        let attendees = props
            .attendees
            .into_iter()
            .filter_map(|prop| match Attendee::try_from(prop) {
                Ok(v) => Some(v),
                Err(e) => {
                    errors.push(e);
                    None
                }
            })
            .collect();

        let ex_date = props
            .ex_dates
            .into_iter()
            .flat_map(|p| {
                p.values
                    .iter()
                    .filter_map(value_to_floating_date_time)
                    .collect::<Vec<_>>()
            })
            .collect();

        // Parse sub-components (alarms)
        let alarms = comp
            .children
            .iter()
            .filter_map(|child| {
                if child.name == KW_VALARM {
                    Some(VAlarm::try_from(child))
                } else {
                    None
                }
            })
            .filter_map(|result| match result {
                Ok(v) => Some(v),
                Err(e) => {
                    errors.extend(e);
                    None
                }
            })
            .collect();

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        // Get tz_id from dt_start property
        let tz_id = comp
            .properties
            .iter()
            .find(|p| p.kind == PropertyKind::DtStart)
            .and_then(|p| get_tzid(&p.parameters));

        Ok(VEvent {
            uid: props.uid.unwrap(),           // SAFETY: checked above
            dt_stamp: props.dt_stamp.unwrap(), // SAFETY: checked above
            dt_start: props.dt_start.unwrap(), // SAFETY: checked above
            dt_end: props.dt_end,
            duration: props.duration,
            summary: props.summary,
            description: props.description,
            location: props.location,
            geo: props.geo,
            url: props.url,
            organizer: props.organizer,
            attendees,
            last_modified: props.last_modified,
            status: props.status,
            transparency: props.transparency,
            sequence: props.sequence,
            priority: props.priority,
            classification: props.classification,
            resources: props.resources.unwrap_or_default(),
            categories: props.categories.unwrap_or_default(),
            rrule: props.rrule,
            rdate: vec![], // TODO: implement RDATE parsing
            ex_date,
            tz_id,
            alarms,
        })
    }
}

/// Event status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStatus {
    /// Event is tentative
    Tentative,

    /// Event is confirmed
    Confirmed,

    /// Event is cancelled
    Cancelled,
    // /// Custom status
    // Custom(String),
}

impl FromStr for EventStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_EVENT_STATUS_TENTATIVE => Ok(Self::Tentative),
            KW_EVENT_STATUS_CONFIRMED => Ok(Self::Confirmed),
            KW_EVENT_STATUS_CANCELLED => Ok(Self::Cancelled),
            _ => Err(format!("Invalid event status: {s}")),
        }
    }
}

impl Display for EventStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tentative => KW_EVENT_STATUS_TENTATIVE.fmt(f),
            Self::Confirmed => KW_EVENT_STATUS_CONFIRMED.fmt(f),
            Self::Cancelled => KW_EVENT_STATUS_CANCELLED.fmt(f),
        }
    }
}

impl AsRef<str> for EventStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::Tentative => KW_EVENT_STATUS_TENTATIVE,
            Self::Confirmed => KW_EVENT_STATUS_CONFIRMED,
            Self::Cancelled => KW_EVENT_STATUS_CANCELLED,
        }
    }
}

/// Time transparency for events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeTransparency {
    /// Event blocks time
    Opaque,

    /// Event does not block time
    Transparent,
    // /// Custom transparency
    // Custom(String),
}

impl FromStr for TimeTransparency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_TRANSP_OPAQUE => Ok(Self::Opaque),
            KW_TRANSP_TRANSPARENT => Ok(Self::Transparent),
            _ => Err(format!("Invalid time transparency: {s}")),
        }
    }
}

impl Display for TimeTransparency {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Opaque => KW_TRANSP_OPAQUE.fmt(f),
            Self::Transparent => KW_TRANSP_TRANSPARENT.fmt(f),
        }
    }
}

impl AsRef<str> for TimeTransparency {
    fn as_ref(&self) -> &str {
        match self {
            Self::Opaque => KW_TRANSP_OPAQUE,
            Self::Transparent => KW_TRANSP_TRANSPARENT,
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'a> {
    uid:            Option<String>,
    dt_stamp:       Option<DateTime>,
    dt_start:       Option<DateTime>,
    dt_end:         Option<DateTime>,
    duration:       Option<ValueDuration>,
    summary:        Option<Text>,
    description:    Option<Text>,
    location:       Option<Text>,
    geo:            Option<Geo>,
    url:            Option<Uri>,
    organizer:      Option<Organizer>,
    attendees:      Vec<&'a TypedProperty<'a>>,
    last_modified:  Option<DateTime>,
    status:         Option<EventStatus>,
    transparency:   Option<TimeTransparency>,
    sequence:       Option<u32>,
    priority:       Option<u8>,
    classification: Option<Classification>,
    resources:      Option<Vec<Text>>,
    categories:     Option<Vec<Text>>,
    rrule:          Option<RecurrenceRule>,
    ex_dates:       Vec<&'a TypedProperty<'a>>,
}
