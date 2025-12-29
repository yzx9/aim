// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Journal entry component (VJOURNAL) for iCalendar semantic components.

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::RecurrenceRule;
use crate::keyword::{
    KW_JOURNAL_STATUS_CANCELLED, KW_JOURNAL_STATUS_DRAFT, KW_JOURNAL_STATUS_FINAL, KW_VJOURNAL,
};
use crate::semantic::property_util::{
    get_language, get_single_value, parse_multi_text_property, value_to_floating_date_time,
    value_to_string,
};
use crate::semantic::{
    Attendee, Classification, DateTime, Organizer, Period, SemanticError, Text, Uri,
};
use crate::typed::{PropertyKind, TypedComponent, TypedProperty, Value, ValueDate, ValueType};

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

/// Parse a `TypedComponent` into a `VJournal`
#[allow(clippy::too_many_lines)]
impl TryFrom<TypedComponent<'_>> for VJournal {
    type Error = Vec<SemanticError>;

    fn try_from(comp: TypedComponent<'_>) -> Result<Self, Self::Error> {
        if comp.name != KW_VJOURNAL {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VJOURNAL,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop.kind {
                PropertyKind::Uid => {
                    if props.uid.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Uid,
                        });
                        continue;
                    }
                    match get_single_value(&prop).ok().and_then(value_to_string) {
                        Some(v) => props.uid = Some(v),
                        None => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Uid,
                                expected: ValueType::Text,
                            });
                            props.uid = Some(String::new());
                        }
                    }
                }
                PropertyKind::DtStamp => {
                    if props.dt_stamp.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtStamp,
                        });
                        continue;
                    }
                    match get_single_value(&prop)
                        .ok()
                        .and_then(value_to_floating_date_time)
                    {
                        Some(v) => props.dt_stamp = Some(v),
                        None => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::DtStamp,
                                expected: ValueType::DateTime,
                            });
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
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtStart,
                        });
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
                PropertyKind::Summary => {
                    if props.summary.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Summary,
                        });
                        continue;
                    }
                    match get_single_value(&prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(v) => {
                                props.summary = Some(Text {
                                    content: v,
                                    language: get_language(&prop.parameters),
                                });
                            }
                            None => {
                                errors.push(SemanticError::ExpectedType {
                                    property: PropertyKind::Summary,
                                    expected: ValueType::Text,
                                });
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Description => {
                    // VJOURNAL allows multiple DESCRIPTION properties
                    match get_single_value(&prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(v) => props.descriptions.push(Text {
                                content: v,
                                language: get_language(&prop.parameters),
                            }),
                            None => {
                                errors.push(SemanticError::ExpectedType {
                                    property: PropertyKind::Description,
                                    expected: ValueType::Text,
                                });
                            }
                        },
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
                    match get_single_value(&prop)
                        .ok()
                        .and_then(value_to_floating_date_time)
                    {
                        Some(v) => props.last_modified = Some(v),
                        None => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::LastModified,
                                expected: ValueType::DateTime,
                            });
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
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Status,
                        });
                        continue;
                    }
                    match get_single_value(&prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(text) => match text.parse() {
                                Ok(v) => props.status = Some(v),
                                Err(e) => errors.push(SemanticError::InvalidValue {
                                    property: PropertyKind::Status,
                                    value: e,
                                }),
                            },
                            None => {
                                errors.push(SemanticError::ExpectedType {
                                    property: PropertyKind::Status,
                                    expected: ValueType::Text,
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
                    match get_single_value(&prop) {
                        Ok(Value::Text(_)) => {}
                        Ok(_) => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::RRule,
                                expected: ValueType::Text,
                            });
                        }
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::ExDate => {
                    props.ex_dates.push(prop);
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

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(VJournal {
            uid: props.uid.unwrap(),           // SAFETY: checked above
            dt_stamp: props.dt_stamp.unwrap(), // SAFETY: checked above
            dt_start: props.dt_start.unwrap(), // SAFETY: checked above
            summary: props.summary,
            descriptions: props.descriptions,
            organizer: props.organizer,
            attendees,
            last_modified: props.last_modified,
            status: props.status,
            classification: props.classification,
            categories: props.categories.unwrap_or_default(),
            rrule: props.rrule,
            rdate: vec![], // TODO: implement RDATE parsing
            ex_date,
            url: props.url,
        })
    }
}

/// Journal status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalStatus {
    /// Journal entry is draft
    Draft,

    /// Journal entry is final
    Final,

    /// Journal entry is cancelled
    Cancelled,
}

impl FromStr for JournalStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_JOURNAL_STATUS_DRAFT => Ok(Self::Draft),
            KW_JOURNAL_STATUS_FINAL => Ok(Self::Final),
            KW_JOURNAL_STATUS_CANCELLED => Ok(Self::Cancelled),
            _ => Err(format!("Invalid journal status: {s}")),
        }
    }
}

impl Display for JournalStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => KW_JOURNAL_STATUS_DRAFT.fmt(f),
            Self::Final => KW_JOURNAL_STATUS_FINAL.fmt(f),
            Self::Cancelled => KW_JOURNAL_STATUS_CANCELLED.fmt(f),
        }
    }
}

impl AsRef<str> for JournalStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::Draft => KW_JOURNAL_STATUS_DRAFT,
            Self::Final => KW_JOURNAL_STATUS_FINAL,
            Self::Cancelled => KW_JOURNAL_STATUS_CANCELLED,
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
    summary:        Option<Text>,
    descriptions:   Vec<Text>,
    organizer:      Option<Organizer>,
    attendees:      Vec<TypedProperty<'a>>,
    last_modified:  Option<DateTime>,
    status:         Option<JournalStatus>,
    classification: Option<Classification>,
    categories:     Option<Vec<Text>>,
    rrule:          Option<RecurrenceRule>,
    ex_dates:       Vec<TypedProperty<'a>>,
    url:            Option<Uri>,
}
