// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Journal entry component (VJOURNAL) for iCalendar semantic components.

use std::convert::TryFrom;

use crate::keyword::KW_VJOURNAL;
use crate::property::JournalStatus;
use crate::property::parse_multi_text_property;
use crate::semantic::property_util::{
    take_single_floating_date_time, take_single_text, take_single_value, take_single_value_string,
    value_to_floating_date_time,
};
use crate::semantic::{Attendee, Classification, DateTime, Organizer, Period, SemanticError, Text};
use crate::typed::{PropertyKind, TypedComponent, Value, ValueType};
use crate::value::{RecurrenceRule, ValueDate, ValueText};

/// Journal entry component (VJOURNAL)
#[derive(Debug, Clone)]
pub struct VJournal<'src> {
    /// Unique identifier for the journal entry
    pub uid: ValueText<'src>,

    /// Date/time the journal entry was created
    pub dt_stamp: DateTime<'src>,

    /// Date/time of the journal entry
    pub dt_start: DateTime<'src>,

    /// Summary/title of the journal entry
    pub summary: Option<Text<'src>>,

    /// Description of the journal entry (can appear multiple times)
    pub descriptions: Vec<Text<'src>>,

    /// Organizer of the journal entry
    pub organizer: Option<Organizer<'src>>,

    /// Attendees of the journal entry
    pub attendees: Vec<Attendee<'src>>,

    /// Last modification date/time
    pub last_modified: Option<DateTime<'src>>,

    /// Status of the journal entry
    pub status: Option<JournalStatus>,

    /// Classification
    pub classification: Option<Classification>,

    /// Categories
    pub categories: Vec<Text<'src>>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period<'src>>,

    /// Exception dates
    pub ex_date: Vec<DateTime<'src>>,

    /// URL associated with the journal entry
    pub url: Option<ValueText<'src>>,
    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
}

/// Parse a `TypedComponent` into a `VJournal`
#[allow(clippy::too_many_lines)]
impl<'src> TryFrom<TypedComponent<'src>> for VJournal<'src> {
    type Error = Vec<SemanticError>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
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
                    // VJOURNAL allows multiple DESCRIPTION properties
                    match Text::try_from(prop) {
                        Ok(text) => props.descriptions.push(text),
                        Err(e) => errors.extend(e),
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
                    let has_duplicate = match take_single_value(prop.kind, prop.values) {
                        Ok(Value::Text(_)) => props.rrule.is_some(),
                        Ok(_) => {
                            errors.push(SemanticError::UnexpectedType {
                                property: PropertyKind::RRule,
                                expected: ValueType::Text,
                            });
                            props.rrule.is_some()
                        }
                        Err(e) => {
                            errors.push(e);
                            props.rrule.is_some()
                        }
                    };

                    if has_duplicate {
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
            attendees: props.attendees,
            last_modified: props.last_modified,
            status: props.status,
            classification: props.classification,
            categories: props.categories.unwrap_or_default(),
            rrule: props.rrule,
            rdate: vec![], // TODO: implement RDATE parsing
            ex_date: props.ex_dates,
            url: props.url,
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
    summary:        Option<Text<'src>>,
    descriptions:   Vec<Text<'src>>,
    organizer:      Option<Organizer<'src>>,
    attendees:      Vec<Attendee<'src>>,
    last_modified:  Option<DateTime<'src>>,
    status:         Option<JournalStatus>,
    classification: Option<Classification>,
    categories:     Option<Vec<Text<'src>>>,
    rrule:          Option<RecurrenceRule>,
    ex_dates:       Vec<DateTime<'src>>,
    url:            Option<ValueText<'src>>,
}
