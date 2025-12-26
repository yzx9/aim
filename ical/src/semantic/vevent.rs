// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event component (VEVENT) for iCalendar semantic components.

use chumsky::Parser;
use chumsky::error::Rich;
use chumsky::extra::Err as ChumskyErr;
use chumsky::input::Stream;

use crate::RecurrenceRule;
use crate::keyword::{KW_VALARM, KW_VEVENT};
use crate::semantic::SemanticError;
use crate::semantic::analysis::{
    find_parameter, get_language, get_single_value, get_tzid, parse_cal_address,
    value_to_date_time, value_to_duration, value_to_int, value_to_string,
};
use crate::semantic::enums::{Classification, Period};
use crate::semantic::properties::{Attendee, DateTime, Duration, Geo, Organizer, Text, Uri};
use crate::semantic::valarm::{VAlarm, parse_valarm};
use crate::typed::parameter_types::{CalendarUserType, ParticipationRole, ParticipationStatus};
use crate::typed::{
    PropertyKind, TypedComponent, TypedParameter, TypedParameterKind, TypedProperty, Value,
    ValueDate, values_float_semicolon,
};

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
    pub duration: Option<Duration>,

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

/// Event status
#[derive(Debug, Clone, Copy)]
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

/// Time transparency for events
#[derive(Debug, Clone, Copy)]
pub enum TimeTransparency {
    /// Event blocks time
    Opaque,

    /// Event does not block time
    Transparent,
    // /// Custom transparency
    // Custom(String),
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'a> {
    uid:        Option<&'a TypedProperty<'a>>,
    dt_stamp:   Option<&'a TypedProperty<'a>>,
    dt_start:   Option<&'a TypedProperty<'a>>,
    dt_end:     Option<&'a TypedProperty<'a>>,
    duration:   Option<&'a TypedProperty<'a>>,
    summary:    Option<&'a TypedProperty<'a>>,
    description: Option<&'a TypedProperty<'a>>,
    location:   Option<&'a TypedProperty<'a>>,
    geo:        Option<&'a TypedProperty<'a>>,
    url:        Option<&'a TypedProperty<'a>>,
    organizer:  Option<&'a TypedProperty<'a>>,
    attendees:  Vec<&'a TypedProperty<'a>>,
    last_modified: Option<&'a TypedProperty<'a>>,
    status:     Option<&'a TypedProperty<'a>>,
    transparency: Option<&'a TypedProperty<'a>>,
    sequence:   Option<&'a TypedProperty<'a>>,
    priority:   Option<&'a TypedProperty<'a>>,
    classification: Option<&'a TypedProperty<'a>>,
    resources:  Option<&'a TypedProperty<'a>>,
    categories: Option<&'a TypedProperty<'a>>,
    rrule:      Option<&'a TypedProperty<'a>>,
    ex_dates:   Vec<&'a TypedProperty<'a>>,
}

/// Parse a `TypedComponent` into a `VEvent`
#[allow(clippy::too_many_lines)]
pub fn parse_vevent(comp: TypedComponent) -> Result<VEvent, Vec<SemanticError>> {
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
        match prop.name {
            name if name == PropertyKind::Uid.as_str() => {
                if props.uid.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Uid.as_str()
                    )));
                } else {
                    props.uid = Some(prop);
                }
            }
            name if name == PropertyKind::DtStamp.as_str() => {
                if props.dt_stamp.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::DtStamp.as_str()
                    )));
                } else {
                    props.dt_stamp = Some(prop);
                }
            }
            name if name == PropertyKind::DtStart.as_str() => {
                if props.dt_start.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::DtStart.as_str()
                    )));
                } else {
                    props.dt_start = Some(prop);
                }
            }
            name if name == PropertyKind::DtEnd.as_str() => {
                if props.dt_end.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::DtEnd.as_str()
                    )));
                } else {
                    props.dt_end = Some(prop);
                }
            }
            name if name == PropertyKind::Duration.as_str() => {
                if props.duration.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Duration.as_str()
                    )));
                } else {
                    props.duration = Some(prop);
                }
            }
            name if name == PropertyKind::Summary.as_str() => {
                if props.summary.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Summary.as_str()
                    )));
                } else {
                    props.summary = Some(prop);
                }
            }
            name if name == PropertyKind::Description.as_str() => {
                if props.description.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Description.as_str()
                    )));
                } else {
                    props.description = Some(prop);
                }
            }
            name if name == PropertyKind::Location.as_str() => {
                if props.location.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Location.as_str()
                    )));
                } else {
                    props.location = Some(prop);
                }
            }
            name if name == PropertyKind::Geo.as_str() => {
                if props.geo.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Geo.as_str()
                    )));
                } else {
                    props.geo = Some(prop);
                }
            }
            name if name == PropertyKind::Url.as_str() => {
                if props.url.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Url.as_str()
                    )));
                } else {
                    props.url = Some(prop);
                }
            }
            name if name == PropertyKind::Organizer.as_str() => {
                if props.organizer.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Organizer.as_str()
                    )));
                } else {
                    props.organizer = Some(prop);
                }
            }
            name if name == PropertyKind::Attendee.as_str() => {
                props.attendees.push(prop);
            }
            name if name == PropertyKind::LastModified.as_str() => {
                if props.last_modified.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::LastModified.as_str()
                    )));
                } else {
                    props.last_modified = Some(prop);
                }
            }
            name if name == PropertyKind::Status.as_str() => {
                if props.status.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Status.as_str()
                    )));
                } else {
                    props.status = Some(prop);
                }
            }
            name if name == PropertyKind::Transp.as_str() => {
                if props.transparency.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Transp.as_str()
                    )));
                } else {
                    props.transparency = Some(prop);
                }
            }
            name if name == PropertyKind::Sequence.as_str() => {
                if props.sequence.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Sequence.as_str()
                    )));
                } else {
                    props.sequence = Some(prop);
                }
            }
            name if name == PropertyKind::Priority.as_str() => {
                if props.priority.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Priority.as_str()
                    )));
                } else {
                    props.priority = Some(prop);
                }
            }
            name if name == PropertyKind::Class.as_str() => {
                if props.classification.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Class.as_str()
                    )));
                } else {
                    props.classification = Some(prop);
                }
            }
            name if name == PropertyKind::Resources.as_str() => {
                if props.resources.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Resources.as_str()
                    )));
                } else {
                    props.resources = Some(prop);
                }
            }
            name if name == PropertyKind::Categories.as_str() => {
                if props.categories.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Categories.as_str()
                    )));
                } else {
                    props.categories = Some(prop);
                }
            }
            name if name == PropertyKind::RRule.as_str() => {
                if props.rrule.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::RRule.as_str()
                    )));
                } else {
                    props.rrule = Some(prop);
                }
            }
            name if name == PropertyKind::ExDate.as_str() => {
                props.ex_dates.push(prop);
            }
            // Ignore unknown properties
            _ => {}
        }
    }

    // UID is required
    let uid = match props.uid {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Uid.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    String::new()
                }
            },
            Err(e) => {
                errors.push(e);
                String::new()
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::Uid.as_str().to_string(),
            ));
            String::new()
        }
    };

    // DTSTAMP is required
    let dt_stamp = match props.dt_stamp {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_date_time(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::DtStamp.as_str().to_string(),
                        "Expected date-time value".to_string(),
                    ));
                    // Return a dummy value to continue parsing
                    DateTime {
                        date: ValueDate {
                            year: 0,
                            month: 1,
                            day: 1,
                        },
                        time: None,
                        tz_id: None,
                        date_only: true,
                    }
                }
            },
            Err(e) => {
                errors.push(e);
                DateTime {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                    time: None,
                    tz_id: None,
                    date_only: true,
                }
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::DtStamp.as_str().to_string(),
            ));
            DateTime {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
                time: None,
                tz_id: None,
                date_only: true,
            }
        }
    };

    // DTSTART is required
    let dt_start = match props.dt_start {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_date_time(value) {
                Some(mut v) => {
                    // Add timezone if specified
                    if let Some(tz_id) = get_tzid(&prop.parameters) {
                        v.tz_id = Some(tz_id);
                    }
                    v
                }
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::DtStart.as_str().to_string(),
                        "Expected date-time value".to_string(),
                    ));
                    DateTime {
                        date: ValueDate {
                            year: 0,
                            month: 1,
                            day: 1,
                        },
                        time: None,
                        tz_id: None,
                        date_only: true,
                    }
                }
            },
            Err(e) => {
                errors.push(e);
                DateTime {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                    time: None,
                    tz_id: None,
                    date_only: true,
                }
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::DtStart.as_str().to_string(),
            ));
            DateTime {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
                time: None,
                tz_id: None,
                date_only: true,
            }
        }
    };

    // DTEND is optional
    let dt_end = props.dt_end.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_date_time(value) {
            Some(mut v) => {
                if let Some(tz_id) = get_tzid(&prop.parameters) {
                    v.tz_id = Some(tz_id);
                }
                v
            }
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::DtEnd.as_str().to_string(),
                    "Expected date-time value".to_string(),
                ));
                DateTime {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                    time: None,
                    tz_id: None,
                    date_only: true,
                }
            }
        },
        Err(e) => {
            errors.push(e);
            DateTime {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
                time: None,
                tz_id: None,
                date_only: true,
            }
        }
    });

    // DURATION is optional (alternative to DTEND)
    let duration = props.duration.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_duration(value) {
            Some(v) => v,
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Duration.as_str().to_string(),
                    "Expected duration value".to_string(),
                ));
                Duration {
                    positive: true,
                    weeks: None,
                    days: None,
                    hours: None,
                    minutes: None,
                    seconds: None,
                }
            }
        },
        Err(e) => {
            errors.push(e);
            Duration {
                positive: true,
                weeks: None,
                days: None,
                hours: None,
                minutes: None,
                seconds: None,
            }
        }
    });

    // SUMMARY is optional
    let summary = props.summary.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_string(value) {
            Some(v) => {
                let language = get_language(&prop.parameters);
                Text {
                    content: v,
                    language,
                }
            }
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Summary.as_str().to_string(),
                    "Expected text value".to_string(),
                ));
                Text {
                    content: String::new(),
                    language: None,
                }
            }
        },
        Err(e) => {
            errors.push(e);
            Text {
                content: String::new(),
                language: None,
            }
        }
    });

    // DESCRIPTION is optional
    let description = props.description.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_string(value) {
            Some(v) => {
                let language = get_language(&prop.parameters);
                Text {
                    content: v,
                    language,
                }
            }
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Description.as_str().to_string(),
                    "Expected text value".to_string(),
                ));
                Text {
                    content: String::new(),
                    language: None,
                }
            }
        },
        Err(e) => {
            errors.push(e);
            Text {
                content: String::new(),
                language: None,
            }
        }
    });

    // LOCATION is optional
    let location = props.location.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_string(value) {
            Some(v) => {
                let language = get_language(&prop.parameters);
                Text {
                    content: v,
                    language,
                }
            }
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Location.as_str().to_string(),
                    "Expected text value".to_string(),
                ));
                Text {
                    content: String::new(),
                    language: None,
                }
            }
        },
        Err(e) => {
            errors.push(e);
            Text {
                content: String::new(),
                language: None,
            }
        }
    });

    // GEO is optional (semicolon-separated lat;long)
    let geo = props.geo.map(|prop| {
        match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(text) => {
                    // Use the typed phase's float parser with semicolon separator
                    let stream = Stream::from_iter(text.chars());
                    let parser = values_float_semicolon::<_, ChumskyErr<Rich<char, _>>>();
                    match parser.parse(stream).into_result() {
                        Ok(result) => {
                            let (Some(&lat), Some(&lon)) = (result.first(), result.get(1)) else {
                                errors.push(SemanticError::InvalidValue(
                                    PropertyKind::Geo.as_str().to_string(),
                                    format!(
                                        "Expected exactly 2 float values (lat;long), got {}",
                                        result.len()
                                    ),
                                ));
                                return Geo { lat: 0.0, lon: 0.0 };
                            };
                            Geo { lat, lon }
                        }
                        Err(_) => {
                            errors.push(SemanticError::InvalidValue(
                                PropertyKind::Geo.as_str().to_string(),
                                format!(
                                    "Expected 'lat;long' format with semicolon separator, got {text}"
                                ),
                            ));
                            Geo { lat: 0.0, lon: 0.0 }
                        }
                    }
                }
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Geo.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    Geo { lat: 0.0, lon: 0.0 }
                }
            },
            Err(e) => {
                errors.push(e);
                Geo { lat: 0.0, lon: 0.0 }
            }
        }
    });

    // URL is optional
    let url = props.url.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_string(value) {
            Some(v) => Uri { uri: v },
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Url.as_str().to_string(),
                    "Expected URI value".to_string(),
                ));
                Uri { uri: String::new() }
            }
        },
        Err(e) => {
            errors.push(e);
            Uri { uri: String::new() }
        }
    });

    // ORGANIZER is optional
    let organizer = match props.organizer {
        Some(prop) => match parse_organizer(prop) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => None,
    };

    // ATTENDEE can appear multiple times
    let attendees = props
        .attendees
        .into_iter()
        .filter_map(|prop| match parse_attendee(prop) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(e);
                None
            }
        })
        .collect();

    // LAST-MODIFIED is optional
    let last_modified = props
        .last_modified
        .map(|prop| match get_single_value(prop) {
            Ok(value) => match value_to_date_time(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::LastModified.as_str().to_string(),
                        "Expected date-time value".to_string(),
                    ));
                    DateTime {
                        date: ValueDate {
                            year: 0,
                            month: 1,
                            day: 1,
                        },
                        time: None,
                        tz_id: None,
                        date_only: true,
                    }
                }
            },
            Err(e) => {
                errors.push(e);
                DateTime {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                    time: None,
                    tz_id: None,
                    date_only: true,
                }
            }
        });

    // STATUS is optional
    let status = match props.status {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(text) => match text.to_uppercase().as_str() {
                    "TENTATIVE" => Some(EventStatus::Tentative),
                    "CONFIRMED" => Some(EventStatus::Confirmed),
                    "CANCELLED" => Some(EventStatus::Cancelled),
                    _ => {
                        errors.push(SemanticError::InvalidValue(
                            PropertyKind::Status.as_str().to_string(),
                            format!("Invalid status: {text}"),
                        ));
                        None
                    }
                },
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Status.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    None
                }
            },
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => None,
    };

    // TRANSP is optional
    let transparency = match props.transparency {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(text) => match text.to_uppercase().as_str() {
                    "OPAQUE" => Some(TimeTransparency::Opaque),
                    "TRANSPARENT" => Some(TimeTransparency::Transparent),
                    _ => {
                        errors.push(SemanticError::InvalidValue(
                            PropertyKind::Transp.as_str().to_string(),
                            format!("Invalid transparency: {text}"),
                        ));
                        None
                    }
                },
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Transp.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    None
                }
            },
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => None,
    };

    // SEQUENCE is optional
    let sequence = props.sequence.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_int::<u32>(value) {
            Some(v) => v,
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Sequence.as_str().to_string(),
                    "Expected integer value".to_string(),
                ));
                0
            }
        },
        Err(e) => {
            errors.push(e);
            0
        }
    });

    // PRIORITY is optional
    let priority = props.priority.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_int::<u8>(value) {
            Some(v) => v,
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Priority.as_str().to_string(),
                    "Expected integer value".to_string(),
                ));
                0
            }
        },
        Err(e) => {
            errors.push(e);
            0
        }
    });

    // CLASS is optional
    let classification = match props.classification {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(text) => match text.to_uppercase().as_str() {
                    "PUBLIC" => Some(Classification::Public),
                    "PRIVATE" => Some(Classification::Private),
                    "CONFIDENTIAL" => Some(Classification::Confidential),
                    _ => {
                        errors.push(SemanticError::InvalidValue(
                            PropertyKind::Class.as_str().to_string(),
                            format!("Invalid classification: {text}"),
                        ));
                        None
                    }
                },
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Class.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    None
                }
            },
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => None,
    };

    // RESOURCES can appear multiple times (comma-separated values)
    let resources = props.resources.map(|p| {
        p.values
            .iter()
            .filter_map(|v| {
                value_to_string(v).map(|s| Text {
                    content: s,
                    language: get_language(&p.parameters),
                })
            })
            .collect()
    });

    // CATEGORIES can appear multiple times (comma-separated values)
    let categories = props.categories.map(|p| {
        p.values
            .iter()
            .filter_map(|v| {
                value_to_string(v).map(|s| Text {
                    content: s,
                    language: get_language(&p.parameters),
                })
            })
            .collect()
    });

    // RRULE is optional
    let rrule = match props.rrule {
        Some(prop) => match get_single_value(prop) {
            Ok(Value::Text(_text)) => {
                // TODO: Parse RRULE from text format
                // For now, skip RRULE parsing
                None
            }
            Ok(_) => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::RRule.as_str().to_string(),
                    "Expected text value".to_string(),
                ));
                None
            }
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => None,
    };

    // RDATE is optional (periods)
    let rdate = vec![]; // TODO: implement RDATE parsing

    // EXDATE is optional
    let ex_date = props
        .ex_dates
        .into_iter()
        .flat_map(|p| {
            p.values
                .iter()
                .filter_map(|v| value_to_date_time(v))
                .collect::<Vec<_>>()
        })
        .collect();

    // Parse sub-components (alarms)
    let alarms = comp
        .children
        .into_iter()
        .filter_map(|child| {
            if child.name == KW_VALARM {
                Some(parse_valarm(child))
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

    // If we have errors, return them all
    if !errors.is_empty() {
        return Err(errors);
    }

    // Get tz_id from dt_start parameters
    let tz_id = props.dt_start.and_then(|p| get_tzid(&p.parameters));

    Ok(VEvent {
        uid,
        dt_stamp,
        dt_start,
        dt_end,
        duration,
        summary,
        description,
        location,
        geo,
        url,
        organizer,
        attendees,
        last_modified,
        status,
        transparency,
        sequence,
        priority,
        classification,
        resources: resources.unwrap_or_default(),
        categories: categories.unwrap_or_default(),
        rrule,
        rdate,
        ex_date,
        tz_id,
        alarms,
    })
}

/// Parse an ORGANIZER property into an Organizer
fn parse_organizer(prop: &TypedProperty<'_>) -> Result<Organizer, SemanticError> {
    let cal_address = parse_cal_address(get_single_value(prop)?).ok_or_else(|| {
        SemanticError::InvalidValue(
            PropertyKind::Organizer.as_str().to_string(),
            "Expected calendar user address".to_string(),
        )
    })?;

    // Extract CN parameter
    let cn =
        find_parameter(&prop.parameters, TypedParameterKind::CommonName).and_then(|p| match p {
            TypedParameter::CommonName { value, .. } => Some(value.resolve().to_string()),
            _ => None,
        });

    // Extract DIR parameter
    let dir =
        find_parameter(&prop.parameters, TypedParameterKind::Directory).and_then(|p| match p {
            TypedParameter::Directory { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract SENT-BY parameter
    let sent_by =
        find_parameter(&prop.parameters, TypedParameterKind::SendBy).and_then(|p| match p {
            TypedParameter::SendBy { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract LANGUAGE parameter
    let language = get_language(&prop.parameters);

    Ok(Organizer {
        cal_address,
        cn,
        dir,
        sent_by,
        language,
    })
}

/// Parse an ATTENDEE property into an Attendee
fn parse_attendee(prop: &TypedProperty<'_>) -> Result<Attendee, SemanticError> {
    let cal_address = parse_cal_address(get_single_value(prop)?).ok_or_else(|| {
        SemanticError::InvalidValue(
            PropertyKind::Attendee.as_str().to_string(),
            "Expected calendar user address".to_string(),
        )
    })?;

    // Extract CN parameter
    let cn =
        find_parameter(&prop.parameters, TypedParameterKind::CommonName).and_then(|p| match p {
            TypedParameter::CommonName { value, .. } => Some(value.resolve().to_string()),
            _ => None,
        });

    // Extract ROLE parameter (default: REQ-PARTICIPANT)
    let role = find_parameter(&prop.parameters, TypedParameterKind::ParticipationRole)
        .and_then(|p| match p {
            TypedParameter::ParticipationRole { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap_or(ParticipationRole::ReqParticipant);

    // Extract PARTSTAT parameter (default: NEEDS-ACTION)
    let part_stat = find_parameter(&prop.parameters, TypedParameterKind::ParticipationStatus)
        .and_then(|p| match p {
            TypedParameter::ParticipationStatus { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap_or(ParticipationStatus::NeedsAction);

    // Extract RSVP parameter
    let rsvp = find_parameter(&prop.parameters, TypedParameterKind::RsvpExpectation).and_then(
        |p| match p {
            TypedParameter::RsvpExpectation { value, .. } => Some(*value),
            _ => None,
        },
    );

    // Extract CUTYPE parameter (default: INDIVIDUAL)
    let cutype = find_parameter(&prop.parameters, TypedParameterKind::CalendarUserType)
        .and_then(|p| match p {
            TypedParameter::CalendarUserType { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap_or(CalendarUserType::Individual);

    // Extract MEMBER parameter
    let member = find_parameter(&prop.parameters, TypedParameterKind::GroupOrListMembership)
        .and_then(|p| match p {
            TypedParameter::GroupOrListMembership { values, .. } => values.first().map(|v| Uri {
                uri: v.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract DELEGATED-TO parameter
    let delegated_to =
        find_parameter(&prop.parameters, TypedParameterKind::Delegatees).and_then(|p| match p {
            TypedParameter::Delegatees { values, .. } => values.first().map(|v| Uri {
                uri: v.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract DELEGATED-FROM parameter
    let delegated_from =
        find_parameter(&prop.parameters, TypedParameterKind::Delegators).and_then(|p| match p {
            TypedParameter::Delegators { values, .. } => values.first().map(|v| Uri {
                uri: v.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract DIR parameter
    let dir =
        find_parameter(&prop.parameters, TypedParameterKind::Directory).and_then(|p| match p {
            TypedParameter::Directory { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract SENT-BY parameter
    let sent_by =
        find_parameter(&prop.parameters, TypedParameterKind::SendBy).and_then(|p| match p {
            TypedParameter::SendBy { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract LANGUAGE parameter
    let language = get_language(&prop.parameters);

    Ok(Attendee {
        cal_address,
        cn,
        role,
        part_stat,
        rsvp,
        cutype,
        member,
        delegated_to,
        delegated_from,
        dir,
        sent_by,
        language,
    })
}
