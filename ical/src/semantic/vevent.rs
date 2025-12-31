// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event component (VEVENT) for iCalendar semantic components.

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::keyword::{
    KW_EVENT_STATUS_CANCELLED, KW_EVENT_STATUS_CONFIRMED, KW_EVENT_STATUS_TENTATIVE,
    KW_TRANSP_OPAQUE, KW_TRANSP_TRANSPARENT, KW_VALARM, KW_VEVENT,
};
use crate::semantic::property_common::{
    parse_multi_text_property, take_single_floating_date_time, take_single_int, take_single_text,
    take_single_value, take_single_value_string, value_to_floating_date_time,
};
use crate::semantic::{
    Attendee, Classification, DateTime, Geo, Organizer, Period, SemanticError, Text, VAlarm,
};
use crate::typed::{
    PropertyKind, TypedComponent, Value, ValueType,
};
use crate::value::{RecurrenceRule, ValueDate, ValueDuration, ValueText};

/// Event component (VEVENT)
#[derive(Debug, Clone)]
pub struct VEvent<'src> {
    /// Unique identifier for the event
    pub uid: ValueText<'src>,

    /// Date/time the event was created
    pub dt_stamp: DateTime<'src>,

    /// Date/time the event starts
    pub dt_start: DateTime<'src>,

    /// Date/time the event ends
    pub dt_end: Option<DateTime<'src>>,

    /// Duration of the event (alternative to `dt_end`)
    pub duration: Option<ValueDuration>,

    /// Summary/title of the event
    pub summary: Option<Text<'src>>,

    /// Description of the event
    pub description: Option<Text<'src>>,

    /// Location of the event
    pub location: Option<Text<'src>>,

    /// Geographic position
    pub geo: Option<Geo>,

    /// URL associated with the event
    pub url: Option<ValueText<'src>>,

    /// Organizer of the event
    pub organizer: Option<Organizer<'src>>,

    /// Attendees of the event
    pub attendees: Vec<Attendee<'src>>,

    /// Last modification date/time
    pub last_modified: Option<DateTime<'src>>,

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
    pub resources: Vec<Text<'src>>,

    /// Categories
    pub categories: Vec<Text<'src>>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period<'src>>,

    /// Exception dates
    pub ex_date: Vec<DateTime<'src>>,

    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
    /// Sub-components (like alarms)
    pub alarms: Vec<VAlarm<'src>>,
}

/// Parse a `TypedComponent` into a `VEvent`
#[allow(clippy::too_many_lines)]
impl<'src> TryFrom<TypedComponent<'src>> for VEvent<'src> {
    type Error = Vec<SemanticError>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VEVENT {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VEVENT,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop.kind {
                PropertyKind::Uid => {
                    let uid = take_single_text(prop.kind, prop.values).unwrap_or_else(|e| {
                        errors.push(e);
                        ValueText::default()
                    });

                    match props.uid {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Uid,
                        }),
                        None => props.uid = Some(uid),
                    }
                }
                PropertyKind::DtStamp => {
                    let value = match take_single_floating_date_time(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            })
                        }
                    };

                    match props.dt_stamp {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtStamp,
                        }),
                        None => props.dt_stamp = value,
                    }
                }
                PropertyKind::DtStart => {
                    let value = match DateTime::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            })
                        }
                    };

                    match props.dt_start {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtStart,
                        }),
                        None => props.dt_start = value,
                    }
                }
                PropertyKind::DtEnd => {
                    let value = match DateTime::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            })
                        }
                    };

                    match props.dt_end {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtEnd,
                        }),
                        None => props.dt_end = value,
                    }
                }
                PropertyKind::Duration => {
                    let value = match take_single_value(prop.kind, prop.values) {
                        Ok(Value::Duration(v)) => Some(v),
                        _ => {
                            errors.push(SemanticError::UnexpectedType {
                                property: PropertyKind::Duration,
                                expected: ValueType::Duration,
                            });
                            Some(ValueDuration::DateTime {
                                positive: true,
                                day: 0,
                                hour: 0,
                                minute: 0,
                                second: 0,
                            })
                        }
                    };

                    match props.duration {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Duration,
                        }),
                        None => props.duration = value,
                    }
                }
                PropertyKind::Summary => {
                    let value = match Text::try_from(prop) {
                        Ok(text) => Some(text),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.summary {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Summary,
                        }),
                        None => props.summary = value,
                    }
                }
                PropertyKind::Description => {
                    let value = match Text::try_from(prop) {
                        Ok(text) => Some(text),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.description {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Description,
                        }),
                        None => props.description = value,
                    }
                }
                PropertyKind::Location => {
                    let value = match Text::try_from(prop) {
                        Ok(text) => Some(text),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.location {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Location,
                        }),
                        None => props.location = value,
                    }
                }
                PropertyKind::Geo => {
                    let value = match Geo::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            Some(Geo { lat: 0.0, lon: 0.0 })
                        }
                    };

                    match props.geo {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Geo,
                        }),
                        None => props.geo = value,
                    }
                }
                PropertyKind::Url => {
                    let value = match take_single_text(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            None
                        }
                    };

                    match props.url {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Url,
                        }),
                        None => props.url = value,
                    }
                }
                PropertyKind::Organizer => {
                    let value = match Organizer::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.organizer {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Organizer,
                        }),
                        None => props.organizer = value,
                    }
                }
                PropertyKind::Attendee => match Attendee::try_from(prop) {
                    Ok(v) => props.attendees.push(v),
                    Err(e) => errors.extend(e),
                },
                PropertyKind::LastModified => {
                    let value = match take_single_floating_date_time(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            })
                        }
                    };

                    match props.last_modified {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::LastModified,
                        }),
                        None => props.last_modified = value,
                    }
                }
                PropertyKind::Status => {
                    let value = match take_single_value_string(prop.kind, prop.values) {
                        Ok(text) => match text.parse() {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push(SemanticError::InvalidValue {
                                    property: PropertyKind::Status,
                                    value: e,
                                });
                                None
                            }
                        },
                        Err(e) => {
                            errors.push(e);
                            None
                        }
                    };

                    match props.status {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Status,
                        }),
                        None => props.status = value,
                    }
                }
                PropertyKind::Transp => {
                    let value = match take_single_value_string(prop.kind, prop.values) {
                        Ok(text) => match text.parse() {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push(SemanticError::InvalidValue {
                                    property: PropertyKind::Transp,
                                    value: e,
                                });
                                None
                            }
                        },
                        Err(e) => {
                            errors.push(e);
                            None
                        }
                    };

                    match props.transparency {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Transp,
                        }),
                        None => props.transparency = value,
                    }
                }
                PropertyKind::Sequence => {
                    let value = match take_single_int(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            None
                        }
                    };

                    match props.sequence {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Sequence,
                        }),
                        None => props.sequence = value,
                    }
                }
                PropertyKind::Priority => {
                    let value = match take_single_int(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            None
                        }
                    };

                    match props.priority {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Priority,
                        }),
                        None => props.priority = value,
                    }
                }
                PropertyKind::Class => {
                    let value = match Classification::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.classification {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Class,
                        }),
                        None => props.classification = value,
                    }
                }
                PropertyKind::Resources => {
                    let value = Some(parse_multi_text_property(prop));

                    match props.resources {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Resources,
                        }),
                        None => props.resources = value,
                    }
                }
                PropertyKind::Categories => {
                    let value = Some(parse_multi_text_property(prop));

                    match props.categories {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Categories,
                        }),
                        None => props.categories = value,
                    }
                }
                PropertyKind::RRule => {
                    // TODO: Parse RRULE from text format
                    match take_single_value(prop.kind, prop.values) {
                        Ok(Value::Text(_)) => {}
                        Ok(_) => {
                            errors.push(SemanticError::UnexpectedType {
                                property: PropertyKind::RRule,
                                expected: ValueType::Text,
                            });
                        }
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::ExDate => {
                    for value in prop.values {
                        if let Some(dt) = value_to_floating_date_time(&value) {
                            props.ex_dates.push(dt);
                        } else {
                            errors.push(SemanticError::UnexpectedType {
                                property: PropertyKind::ExDate,
                                expected: ValueType::DateTime,
                            });
                        }
                    }
                }
                // Ignore unknown properties
                _ => {}
            }
        }

        // Check required fields
        if props.uid.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Uid,
            });
        }
        if props.dt_stamp.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::DtStamp,
            });
        }
        if props.dt_start.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::DtStart,
            });
        }

        // Parse sub-components (alarms)
        let alarms = comp
            .children
            .into_iter()
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
            attendees: props.attendees,
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
            ex_date: props.ex_dates,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeTransparency {
    /// Event blocks time
    #[default]
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
struct PropertyCollector<'src> {
    uid:            Option<ValueText<'src>>,
    dt_stamp:       Option<DateTime<'src>>,
    dt_start:       Option<DateTime<'src>>,
    dt_end:         Option<DateTime<'src>>,
    duration:       Option<ValueDuration>,
    summary:        Option<Text<'src>>,
    description:    Option<Text<'src>>,
    location:       Option<Text<'src>>,
    geo:            Option<Geo>,
    url:            Option<ValueText<'src>>,
    organizer:      Option<Organizer<'src>>,
    attendees:      Vec<Attendee<'src>>,
    last_modified:  Option<DateTime<'src>>,
    status:         Option<EventStatus>,
    transparency:   Option<TimeTransparency>,
    sequence:       Option<u32>,
    priority:       Option<u8>,
    classification: Option<Classification>,
    resources:      Option<Vec<Text<'src>>>,
    categories:     Option<Vec<Text<'src>>>,
    rrule:          Option<RecurrenceRule>,
    ex_dates:       Vec<DateTime<'src>>,
}
