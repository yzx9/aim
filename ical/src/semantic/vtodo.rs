// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! To-do component (VTODO) for iCalendar semantic components.

use std::convert::TryFrom;

use crate::keyword::{KW_VALARM, KW_VTODO};
use crate::property::{TodoStatus, parse_multi_text_property};
use crate::semantic::property_util::{
    take_single_floating_date_time, take_single_int, take_single_text, take_single_value,
    take_single_value_string, value_to_floating_date_time,
};
use crate::semantic::{
    Attendee, Classification, DateTime, Geo, Organizer, Period, SemanticError, Text, VAlarm,
};
use crate::typed::{PropertyKind, TypedComponent, Value, ValueType};
use crate::value::{RecurrenceRule, ValueDate, ValueDuration, ValueText};

/// To-do component (VTODO)
#[derive(Debug, Clone)]
pub struct VTodo<'src> {
    /// Unique identifier for the todo
    pub uid: ValueText<'src>,

    /// Date/time the todo was created
    pub dt_stamp: DateTime<'src>,

    /// Date/time to start the todo
    pub dt_start: Option<DateTime<'src>>,

    /// Date/time the todo is due
    pub due: Option<DateTime<'src>>,

    /// Completion date/time
    pub completed: Option<DateTime<'src>>,

    /// Duration of the todo
    pub duration: Option<ValueDuration>,

    /// Summary/title of the todo
    pub summary: Option<Text<'src>>,

    /// Description of the todo
    pub description: Option<Text<'src>>,

    /// Location of the todo
    pub location: Option<Text<'src>>,

    /// Geographic position
    pub geo: Option<Geo>,

    /// URL associated with the todo
    pub url: Option<ValueText<'src>>,

    /// Organizer of the todo
    pub organizer: Option<Organizer<'src>>,

    /// Attendees of the todo
    pub attendees: Vec<Attendee<'src>>,

    /// Last modification date/time
    pub last_modified: Option<DateTime<'src>>,

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

/// Parse a `TypedComponent` into a `VTodo`
#[allow(clippy::too_many_lines)]
impl<'src> TryFrom<TypedComponent<'src>> for VTodo<'src> {
    type Error = Vec<SemanticError>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VTODO {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VTODO,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop.kind {
                PropertyKind::Uid => {
                    let value = match take_single_text(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            Some(ValueText::default())
                        }
                    };

                    match props.uid {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Uid,
                        }),
                        None => props.uid = value,
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
                PropertyKind::Due => {
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

                    match props.due {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Due,
                        }),
                        None => props.due = value,
                    }
                }
                PropertyKind::Completed => {
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

                    match props.completed {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Completed,
                        }),
                        None => props.completed = value,
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
                        Ok(v) => Some(v),
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
                        Ok(v) => Some(v),
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
                        Ok(v) => Some(v),
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
                PropertyKind::PercentComplete => {
                    let value = match take_single_int(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            None
                        }
                    };

                    match props.percent_complete {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::PercentComplete,
                        }),
                        None => props.percent_complete = value,
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
                    let _value = match take_single_value(prop.kind, prop.values) {
                        Ok(Value::Text(_)) => Some(()),
                        Ok(_) => {
                            errors.push(SemanticError::UnexpectedType {
                                property: PropertyKind::RRule,
                                expected: ValueType::Text,
                            });
                            None
                        }
                        Err(e) => {
                            errors.push(e);
                            None
                        }
                    };

                    // Don't set props.rrule as it's not implemented yet
                    if props.rrule.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::RRule,
                        });
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
            attendees: props.attendees,
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
            ex_date: props.ex_dates,
            alarms,
        })
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    uid:            Option<ValueText<'src>>,
    dt_stamp:       Option<DateTime<'src>>,
    dt_start:       Option<DateTime<'src>>,
    due:            Option<DateTime<'src>>,
    completed:      Option<DateTime<'src>>,
    duration:       Option<ValueDuration>,
    summary:        Option<Text<'src>>,
    description:    Option<Text<'src>>,
    location:       Option<Text<'src>>,
    geo:            Option<Geo>,
    url:            Option<ValueText<'src>>,
    organizer:      Option<Organizer<'src>>,
    attendees:      Vec<Attendee<'src>>,
    last_modified:  Option<DateTime<'src>>,
    status:         Option<TodoStatus>,
    sequence:       Option<u32>,
    priority:       Option<u8>,
    percent_complete: Option<u8>,
    classification: Option<Classification>,
    resources:      Option<Vec<Text<'src>>>,
    categories:     Option<Vec<Text<'src>>>,
    rrule:          Option<RecurrenceRule>,
    ex_dates:       Vec<DateTime<'src>>,
}
