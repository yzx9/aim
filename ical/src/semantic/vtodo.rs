// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! To-do component (VTODO) for iCalendar semantic components.

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::RecurrenceRule;
use crate::keyword::{
    KW_TODO_STATUS_CANCELLED, KW_TODO_STATUS_COMPLETED, KW_TODO_STATUS_IN_PROCESS,
    KW_TODO_STATUS_NEEDS_ACTION, KW_VALARM, KW_VTODO,
};
use crate::semantic::property_util::{
    get_language, get_single_value, get_tzid, parse_multi_text_property,
    value_to_floating_date_time, value_to_int, value_to_string,
};
use crate::semantic::{
    Attendee, Classification, DateTime, Geo, Organizer, Period, SemanticError, Text, Uri, VAlarm,
};
use crate::typed::{
    PropertyKind, TypedComponent, TypedProperty, Value, ValueDate, ValueDuration, ValueType,
};

/// To-do component (VTODO)
#[derive(Debug, Clone)]
pub struct VTodo {
    /// Unique identifier for the todo
    pub uid: String,

    /// Date/time the todo was created
    pub dt_stamp: DateTime,

    /// Date/time to start the todo
    pub dt_start: Option<DateTime>,

    /// Date/time the todo is due
    pub due: Option<DateTime>,

    /// Completion date/time
    pub completed: Option<DateTime>,

    /// Duration of the todo
    pub duration: Option<ValueDuration>,

    /// Summary/title of the todo
    pub summary: Option<Text>,

    /// Description of the todo
    pub description: Option<Text>,

    /// Location of the todo
    pub location: Option<Text>,

    /// Geographic position
    pub geo: Option<Geo>,

    /// URL associated with the todo
    pub url: Option<Uri>,

    /// Organizer of the todo
    pub organizer: Option<Organizer>,

    /// Attendees of the todo
    pub attendees: Vec<Attendee>,

    /// Last modification date/time
    pub last_modified: Option<DateTime>,

    /// Status of the todo
    pub status: Option<TodoStatus>,

    /// Sequence number for revisions
    pub sequence: Option<u32>,

    /// Priority (1-9, 1 is highest)
    pub priority: Option<u8>,

    /// Percentage complete (0-100)
    pub percent_complete: Option<u8>,

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

/// Parse a `TypedComponent` into a `VTodo`
#[allow(clippy::too_many_lines)]
impl TryFrom<&TypedComponent<'_>> for VTodo {
    type Error = Vec<SemanticError>;

    fn try_from(comp: &TypedComponent<'_>) -> Result<Self, Self::Error> {
        if comp.name != KW_VTODO {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VTODO,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in &comp.properties {
            match prop.kind {
                PropertyKind::Uid => {
                    if props.uid.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Uid,
                        });
                        continue;
                    }

                    props.uid = get_single_value(prop)
                        .ok()
                        .and_then(value_to_string)
                        .or_else(|| {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Uid,
                                expected: ValueType::Text,
                            });
                            Some(String::new())
                        });
                }
                PropertyKind::DtStamp => {
                    if props.dt_stamp.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtStamp,
                        });
                        continue;
                    }

                    props.dt_stamp = get_single_value(prop)
                        .ok()
                        .and_then(value_to_floating_date_time)
                        .or_else(|| {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::DtStamp,
                                expected: ValueType::DateTime,
                            });

                            Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            })
                        });
                }
                PropertyKind::DtStart => {
                    if props.dt_start.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtStart,
                        });
                        continue;
                    }

                    props.dt_start = match DateTime::try_from(prop) {
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
                    }
                }
                PropertyKind::Due => {
                    if props.due.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Due,
                        });
                        continue;
                    }
                    match DateTime::try_from(prop) {
                        Ok(v) => props.due = Some(v),
                        Err(e) => {
                            errors.push(e);
                            props.due = Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            });
                        }
                    }
                }
                PropertyKind::Completed => {
                    if props.completed.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Completed,
                        });
                        continue;
                    }
                    match get_single_value(prop)
                        .ok()
                        .and_then(value_to_floating_date_time)
                    {
                        Some(v) => props.completed = Some(v),
                        None => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Completed,
                                expected: ValueType::DateTime,
                            });
                            props.completed = Some(DateTime::Date {
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
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Duration,
                        });
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(Value::Duration(v)) => props.duration = Some(*v),
                        _ => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Duration,
                                expected: ValueType::Duration,
                            });
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
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Summary,
                        });
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
                            None => errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Summary,
                                expected: ValueType::Text,
                            }),
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Description => {
                    if props.description.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Description,
                        });
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
                            None => errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Description,
                                expected: ValueType::Text,
                            }),
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Location => {
                    if props.location.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Location,
                        });
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
                            None => errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Location,
                                expected: ValueType::Text,
                            }),
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Geo => {
                    if props.geo.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Geo,
                        });
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
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Url,
                        });
                        continue;
                    }
                    match Uri::try_from(prop) {
                        Ok(v) => props.url = Some(v),
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Organizer => {
                    if props.organizer.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Organizer,
                        });
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
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::LastModified,
                        });
                        continue;
                    }

                    props.last_modified = get_single_value(prop)
                        .ok()
                        .and_then(value_to_floating_date_time)
                        .or_else(|| {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::LastModified,
                                expected: ValueType::DateTime,
                            });
                            Some(DateTime::Date {
                                date: ValueDate {
                                    year: 0,
                                    month: 1,
                                    day: 1,
                                },
                            })
                        });
                }
                PropertyKind::Status => {
                    if props.status.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Status,
                        });
                        continue;
                    }

                    match get_single_value(prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(text) => match text.parse() {
                                Ok(v) => props.status = Some(v),
                                Err(e) => errors.push(SemanticError::InvalidValue {
                                    property: PropertyKind::Status,
                                    value: e,
                                }),
                            },
                            None => errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Status,
                                expected: ValueType::Text,
                            }),
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Sequence => {
                    if props.sequence.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Sequence,
                        });
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_int::<u32>(value) {
                            Some(v) => props.sequence = Some(v),
                            None => {
                                errors.push(SemanticError::ExpectedType {
                                    property: PropertyKind::Sequence,
                                    expected: ValueType::Integer,
                                });
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Priority => {
                    if props.priority.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Priority,
                        });
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_int::<u8>(value) {
                            Some(v) => props.priority = Some(v),
                            None => {
                                errors.push(SemanticError::ExpectedType {
                                    property: PropertyKind::Priority,
                                    expected: ValueType::Integer,
                                });
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::PercentComplete => {
                    if props.percent_complete.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::PercentComplete,
                        });
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_int::<u8>(value) {
                            Some(v) => props.percent_complete = Some(v),
                            None => {
                                errors.push(SemanticError::ExpectedType {
                                    property: PropertyKind::PercentComplete,
                                    expected: ValueType::Integer,
                                });
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Class => {
                    if props.classification.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Class,
                        });
                        continue;
                    }
                    match Classification::try_from(prop) {
                        Ok(v) => props.classification = Some(v),
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Resources => {
                    if props.resources.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Resources,
                        });
                        continue;
                    }
                    props.resources = Some(parse_multi_text_property(prop));
                }
                PropertyKind::Categories => {
                    if props.categories.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Categories,
                        });
                        continue;
                    }
                    props.categories = Some(parse_multi_text_property(prop));
                }
                PropertyKind::RRule => {
                    if props.rrule.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::RRule,
                        });
                        continue;
                    }
                    // TODO: Parse RRULE from text format
                    match get_single_value(prop) {
                        Ok(Value::Text(_)) => {}
                        Ok(_) => errors.push(SemanticError::ExpectedType {
                            property: PropertyKind::RRule,
                            expected: ValueType::Text,
                        }),
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
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Uid,
            });
        }
        if props.dt_stamp.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::DtStamp,
            });
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

        Ok(VTodo {
            uid: props.uid.unwrap(),           // SAFETY: checked above
            dt_stamp: props.dt_stamp.unwrap(), // SAFETY: checked above
            dt_start: props.dt_start,
            due: props.due,
            completed: props.completed,
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
            sequence: props.sequence,
            priority: props.priority,
            percent_complete: props.percent_complete,
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

/// To-do status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoStatus {
    /// To-do needs action
    NeedsAction,

    /// To-do is completed
    Completed,

    /// To-do is in process
    InProcess,

    /// To-do is cancelled
    Cancelled,
    // /// Custom status
    // Custom(String),
}

impl FromStr for TodoStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_TODO_STATUS_NEEDS_ACTION => Ok(Self::NeedsAction),
            KW_TODO_STATUS_COMPLETED => Ok(Self::Completed),
            KW_TODO_STATUS_IN_PROCESS => Ok(Self::InProcess),
            KW_TODO_STATUS_CANCELLED => Ok(Self::Cancelled),
            _ => Err(format!("Invalid todo status: {s}")),
        }
    }
}

impl Display for TodoStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeedsAction => KW_TODO_STATUS_NEEDS_ACTION.fmt(f),
            Self::Completed => KW_TODO_STATUS_COMPLETED.fmt(f),
            Self::InProcess => KW_TODO_STATUS_IN_PROCESS.fmt(f),
            Self::Cancelled => KW_TODO_STATUS_CANCELLED.fmt(f),
        }
    }
}

impl AsRef<str> for TodoStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::NeedsAction => KW_TODO_STATUS_NEEDS_ACTION,
            Self::Completed => KW_TODO_STATUS_COMPLETED,
            Self::InProcess => KW_TODO_STATUS_IN_PROCESS,
            Self::Cancelled => KW_TODO_STATUS_CANCELLED,
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
    due:            Option<DateTime>,
    completed:      Option<DateTime>,
    duration:       Option<ValueDuration>,
    summary:        Option<Text>,
    description:    Option<Text>,
    location:       Option<Text>,
    geo:            Option<Geo>,
    url:            Option<Uri>,
    organizer:      Option<Organizer>,
    attendees:      Vec<&'a TypedProperty<'a>>,
    last_modified:  Option<DateTime>,
    status:         Option<TodoStatus>,
    sequence:       Option<u32>,
    priority:       Option<u8>,
    percent_complete: Option<u8>,
    classification: Option<Classification>,
    resources:      Option<Vec<Text>>,
    categories:     Option<Vec<Text>>,
    rrule:          Option<RecurrenceRule>,
    ex_dates:       Vec<&'a TypedProperty<'a>>,
}
