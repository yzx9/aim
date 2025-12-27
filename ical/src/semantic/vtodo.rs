// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! To-do component (VTODO) for iCalendar semantic components.

use crate::RecurrenceRule;
use crate::keyword::{KW_VALARM, KW_VTODO};
use crate::semantic::SemanticError;
use crate::semantic::analysis::{
    get_language, get_single_value, get_tzid, parse_attendee_property,
    parse_classification_property, parse_geo_property, parse_multi_text_property,
    parse_organizer_property, value_to_any_date_time, value_to_date_time,
    value_to_date_time_with_tz, value_to_int, value_to_string,
};
use crate::semantic::property::{
    Attendee, Classification, DateTime, Geo, Organizer, Period, Text, Uri,
};
use crate::semantic::valarm::{VAlarm, parse_valarm};
use crate::typed::ValueDuration;
use crate::typed::{PropertyKind, TypedComponent, TypedProperty, Value, ValueDate};

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

/// To-do status
#[derive(Debug, Clone, Copy)]
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

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'a> {
    uid:        Option<&'a TypedProperty<'a>>,
    dt_stamp:   Option<&'a TypedProperty<'a>>,
    dt_start:   Option<&'a TypedProperty<'a>>,
    due:        Option<&'a TypedProperty<'a>>,
    completed:  Option<&'a TypedProperty<'a>>,
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
    sequence:   Option<&'a TypedProperty<'a>>,
    priority:   Option<&'a TypedProperty<'a>>,
    percent_complete: Option<&'a TypedProperty<'a>>,
    classification: Option<&'a TypedProperty<'a>>,
    resources:  Option<&'a TypedProperty<'a>>,
    categories: Option<&'a TypedProperty<'a>>,
    rrule:      Option<&'a TypedProperty<'a>>,
    ex_dates:   Vec<&'a TypedProperty<'a>>,
}

/// Parse a `TypedComponent` into a `VTodo`
#[allow(clippy::too_many_lines)]
pub fn parse_vtodo(comp: TypedComponent) -> Result<VTodo, Vec<SemanticError>> {
    if comp.name != KW_VTODO {
        return Err(vec![SemanticError::InvalidStructure(format!(
            "Expected VTODO component, got '{}'",
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
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Uid.as_str()
                    )));
                } else {
                    props.uid = Some(prop);
                }
            }
            PropertyKind::DtStamp => {
                if props.dt_stamp.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::DtStamp.as_str()
                    )));
                } else {
                    props.dt_stamp = Some(prop);
                }
            }
            PropertyKind::DtStart => {
                if props.dt_start.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::DtStart.as_str()
                    )));
                } else {
                    props.dt_start = Some(prop);
                }
            }
            PropertyKind::Due => {
                if props.due.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Due.as_str()
                    )));
                } else {
                    props.due = Some(prop);
                }
            }
            PropertyKind::Completed => {
                if props.completed.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Completed.as_str()
                    )));
                } else {
                    props.completed = Some(prop);
                }
            }
            PropertyKind::Duration => {
                if props.duration.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Duration.as_str()
                    )));
                } else {
                    props.duration = Some(prop);
                }
            }
            PropertyKind::Summary => {
                if props.summary.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Summary.as_str()
                    )));
                } else {
                    props.summary = Some(prop);
                }
            }
            PropertyKind::Description => {
                if props.description.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Description.as_str()
                    )));
                } else {
                    props.description = Some(prop);
                }
            }
            PropertyKind::Location => {
                if props.location.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Location.as_str()
                    )));
                } else {
                    props.location = Some(prop);
                }
            }
            PropertyKind::Geo => {
                if props.geo.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Geo.as_str()
                    )));
                } else {
                    props.geo = Some(prop);
                }
            }
            PropertyKind::Url => {
                if props.url.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Url.as_str()
                    )));
                } else {
                    props.url = Some(prop);
                }
            }
            PropertyKind::Organizer => {
                if props.organizer.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Organizer.as_str()
                    )));
                } else {
                    props.organizer = Some(prop);
                }
            }
            PropertyKind::Attendee => {
                props.attendees.push(prop);
            }
            PropertyKind::LastModified => {
                if props.last_modified.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::LastModified.as_str()
                    )));
                } else {
                    props.last_modified = Some(prop);
                }
            }
            PropertyKind::Status => {
                if props.status.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Status.as_str()
                    )));
                } else {
                    props.status = Some(prop);
                }
            }
            PropertyKind::Sequence => {
                if props.sequence.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Sequence.as_str()
                    )));
                } else {
                    props.sequence = Some(prop);
                }
            }
            PropertyKind::Priority => {
                if props.priority.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Priority.as_str()
                    )));
                } else {
                    props.priority = Some(prop);
                }
            }
            PropertyKind::PercentComplete => {
                if props.percent_complete.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::PercentComplete.as_str()
                    )));
                } else {
                    props.percent_complete = Some(prop);
                }
            }
            PropertyKind::Class => {
                if props.classification.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Class.as_str()
                    )));
                } else {
                    props.classification = Some(prop);
                }
            }
            PropertyKind::Resources => {
                if props.resources.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Resources.as_str()
                    )));
                } else {
                    props.resources = Some(prop);
                }
            }
            PropertyKind::Categories => {
                if props.categories.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Categories.as_str()
                    )));
                } else {
                    props.categories = Some(prop);
                }
            }
            PropertyKind::RRule => {
                if props.rrule.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::RRule.as_str()
                    )));
                } else {
                    props.rrule = Some(prop);
                }
            }
            PropertyKind::ExDate => {
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
                    DateTime::Date {
                        date: ValueDate {
                            year: 0,
                            month: 1,
                            day: 1,
                        },
                    }
                }
            },
            Err(e) => {
                errors.push(e);
                DateTime::Date {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                }
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::DtStamp.as_str().to_string(),
            ));
            DateTime::Date {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
            }
        }
    };

    // DTSTART is optional
    let dt_start = props.dt_start.map(|prop| match get_single_value(prop) {
        Ok(value) => {
            let tz_id = get_tzid(&prop.parameters);
            let result = match &tz_id {
                Some(id) => value_to_date_time_with_tz(value, id.clone()),
                None => value_to_any_date_time(value),
            };
            match result {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::DtStart.as_str().to_string(),
                        "Expected date-time value".to_string(),
                    ));
                    DateTime::Date {
                        date: ValueDate {
                            year: 0,
                            month: 1,
                            day: 1,
                        },
                    }
                }
            }
        }
        Err(e) => {
            errors.push(e);
            DateTime::Date {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
            }
        }
    });

    // DUE is optional
    let due = props.due.map(|prop| match get_single_value(prop) {
        Ok(value) => {
            let tz_id = get_tzid(&prop.parameters);
            let result = match &tz_id {
                Some(id) => value_to_date_time_with_tz(value, id.clone()),
                None => value_to_any_date_time(value),
            };
            match result {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Due.as_str().to_string(),
                        "Expected date-time value".to_string(),
                    ));
                    DateTime::Date {
                        date: ValueDate {
                            year: 0,
                            month: 1,
                            day: 1,
                        },
                    }
                }
            }
        }
        Err(e) => {
            errors.push(e);
            DateTime::Date {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
            }
        }
    });

    // COMPLETED is optional
    let completed = props.completed.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_date_time(value) {
            Some(v) => v,
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Completed.as_str().to_string(),
                    "Expected date-time value".to_string(),
                ));
                DateTime::Date {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                }
            }
        },
        Err(e) => {
            errors.push(e);
            DateTime::Date {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
            }
        }
    });

    // DURATION is optional (alternative to DUE)
    let duration = props.duration.map(|prop| match get_single_value(prop) {
        Ok(Value::Duration(v)) => *v,
        Ok(_) => {
            errors.push(SemanticError::InvalidValue(
                PropertyKind::Duration.as_str().to_string(),
                "Expected duration value".to_string(),
            ));
            ValueDuration::DateTime {
                positive: true,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
            }
        }
        Err(e) => {
            errors.push(e);
            ValueDuration::DateTime {
                positive: true,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
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
    let geo = props.geo.map(|prop| match parse_geo_property(prop) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            Geo { lat: 0.0, lon: 0.0 }
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
        Some(prop) => match parse_organizer_property(prop) {
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
        .filter_map(|prop| match parse_attendee_property(prop) {
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
                    DateTime::Date {
                        date: ValueDate {
                            year: 0,
                            month: 1,
                            day: 1,
                        },
                    }
                }
            },
            Err(e) => {
                errors.push(e);
                DateTime::Date {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                }
            }
        });

    // STATUS is optional
    let status = match props.status {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(text) => match text.to_uppercase().as_str() {
                    "NEEDS-ACTION" => Some(TodoStatus::NeedsAction),
                    "COMPLETED" => Some(TodoStatus::Completed),
                    "IN-PROCESS" => Some(TodoStatus::InProcess),
                    "CANCELLED" => Some(TodoStatus::Cancelled),
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

    // PERCENT-COMPLETE is optional
    let percent_complete = props
        .percent_complete
        .map(|prop| match get_single_value(prop) {
            Ok(value) => match value_to_int::<u8>(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::PercentComplete.as_str().to_string(),
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
        Some(prop) => match parse_classification_property(prop) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => None,
    };

    // RESOURCES can appear multiple times (comma-separated values)
    let resources = props.resources.map(parse_multi_text_property);

    // CATEGORIES can appear multiple times (comma-separated values)
    let categories = props.categories.map(parse_multi_text_property);

    // RRULE is optional
    let rrule = match props.rrule {
        Some(prop) => match get_single_value(prop) {
            Ok(Value::Text(_text)) => None, // TODO: Parse RRULE from text format
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
                Some(parse_valarm(&child))
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

    Ok(VTodo {
        uid,
        dt_stamp,
        dt_start,
        due,
        completed,
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
        sequence,
        priority,
        percent_complete,
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
