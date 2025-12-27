// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Journal entry component (VJOURNAL) for iCalendar semantic components.

use crate::RecurrenceRule;
use crate::keyword::KW_VJOURNAL;
use crate::semantic::SemanticError;
use crate::semantic::analysis::{
    get_language, get_single_value, get_tzid, parse_attendee_property,
    parse_classification_property, parse_multi_text_property, parse_organizer_property,
    value_to_any_date_time, value_to_date_time, value_to_date_time_with_tz, value_to_string,
};
use crate::semantic::properties::{
    Attendee, Classification, DateTime, Organizer, Period, Text, Uri,
};
use crate::typed::{PropertyKind, TypedComponent, TypedProperty, Value, ValueDate};

/// Journal entry component (VJOURNAL)
#[derive(Debug, Clone)]
pub struct VJournal {
    /// Unique identifier for the journal entry
    pub uid: String,

    /// Date/time the journal entry was created
    pub dt_stamp: DateTime,

    /// Date/time of the journal entry
    pub dt_start: DateTime,

    /// Summary/title of the journal entry
    pub summary: Option<Text>,

    /// Description of the journal entry (can appear multiple times)
    pub descriptions: Vec<Text>,

    /// Organizer of the journal entry
    pub organizer: Option<Organizer>,

    /// Attendees of the journal entry
    pub attendees: Vec<Attendee>,

    /// Last modification date/time
    pub last_modified: Option<DateTime>,

    /// Status of the journal entry
    pub status: Option<JournalStatus>,

    /// Classification
    pub classification: Option<Classification>,

    /// Categories
    pub categories: Vec<Text>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period>,

    /// Exception dates
    pub ex_date: Vec<DateTime>,

    /// URL associated with the journal entry
    pub url: Option<Uri>,
    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
}

/// Journal status
#[derive(Debug, Clone, Copy)]
pub enum JournalStatus {
    /// Journal entry is draft
    Draft,

    /// Journal entry is final
    Final,

    /// Journal entry is cancelled
    Cancelled,
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'a> {
    uid:        Option<&'a TypedProperty<'a>>,
    dt_stamp:   Option<&'a TypedProperty<'a>>,
    dt_start:   Option<&'a TypedProperty<'a>>,
    summary:    Option<&'a TypedProperty<'a>>,
    descriptions: Vec<&'a TypedProperty<'a>>,
    organizer:  Option<&'a TypedProperty<'a>>,
    attendees:  Vec<&'a TypedProperty<'a>>,
    last_modified: Option<&'a TypedProperty<'a>>,
    status:     Option<&'a TypedProperty<'a>>,
    classification: Option<&'a TypedProperty<'a>>,
    categories: Option<&'a TypedProperty<'a>>,
    rrule:      Option<&'a TypedProperty<'a>>,
    ex_dates:   Vec<&'a TypedProperty<'a>>,
    url:        Option<&'a TypedProperty<'a>>,
}

/// Parse a `TypedComponent` into a `VJournal`
#[allow(clippy::too_many_lines)]
pub fn parse_vjournal(comp: &TypedComponent) -> Result<VJournal, Vec<SemanticError>> {
    if comp.name != KW_VJOURNAL {
        return Err(vec![SemanticError::InvalidStructure(format!(
            "Expected VJOURNAL component, got '{}'",
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
                // VJOURNAL allows multiple DESCRIPTION properties
                props.descriptions.push(prop);
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

    // DTSTART is required
    let dt_start = match props.dt_start {
        Some(prop) => match get_single_value(prop) {
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
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::DtStart.as_str().to_string(),
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

    // DESCRIPTION can appear multiple times
    let descriptions = props
        .descriptions
        .into_iter()
        .filter_map(|prop| match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(v) => Some(Text {
                    content: v,
                    language: get_language(&prop.parameters),
                }),
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Description.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    None
                }
            },
            Err(e) => {
                errors.push(e);
                None
            }
        })
        .collect();

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
                    "DRAFT" => Some(JournalStatus::Draft),
                    "FINAL" => Some(JournalStatus::Final),
                    "CANCELLED" => Some(JournalStatus::Cancelled),
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

    // CATEGORIES can appear multiple times (comma-separated values)
    let categories = props.categories.map(parse_multi_text_property);

    // RRULE is optional
    let rrule = match props.rrule {
        Some(prop) => match get_single_value(prop) {
            Ok(Value::Text(_text)) => {
                // TODO: Parse RRULE from text format
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

    // If we have errors, return them all
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(VJournal {
        uid,
        dt_stamp,
        dt_start,
        summary,
        descriptions,
        organizer,
        attendees,
        last_modified,
        status,
        classification,
        categories: categories.unwrap_or_default(),
        rrule,
        rdate,
        ex_date,
        url,
    })
}
