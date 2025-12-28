// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Free/busy time component (VFREEBUSY) for iCalendar semantic components.

use std::convert::TryFrom;

use crate::keyword::KW_VFREEBUSY;
use crate::semantic::property_util::{
    find_parameter, get_language, get_single_value, value_to_floating_date_time, value_to_string,
};
use crate::semantic::{DateTime, Organizer, Period, SemanticError, Text, Uri};
use crate::typed::parameter_type::{FreeBusyType, ValueType};
use crate::typed::{
    PropertyKind, TypedComponent, TypedParameter, TypedParameterKind, TypedProperty, Value,
    ValueDate, ValueDuration,
};

/// Free/busy time component (VFREEBUSY)
#[derive(Debug, Clone)]
pub struct VFreeBusy {
    /// Unique identifier for the free/busy info
    pub uid: String,

    /// Date/time the free/busy info was created
    pub dt_stamp: DateTime,

    /// Start of the free/busy period
    pub dt_start: DateTime,

    /// End of the free/busy period
    pub dt_end: Option<DateTime>,

    /// Duration of the free/busy period
    pub duration: Option<ValueDuration>,

    /// Organizer of the free/busy info
    pub organizer: Organizer,

    /// Contact information
    pub contact: Option<Text>,

    /// URL for additional free/busy info
    pub url: Option<Uri>,

    /// Busy periods
    pub busy: Vec<Period>,

    /// Free periods
    pub free: Vec<Period>,

    /// Busy-tentative periods
    pub busy_tentative: Vec<Period>,

    /// Unavailable periods
    pub busy_unavailable: Vec<Period>,
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'a> {
    uid:        Option<String>,
    dt_stamp:   Option<DateTime>,
    dt_start:   Option<DateTime>,
    dt_end:     Option<DateTime>,
    duration:   Option<ValueDuration>,
    organizer:  Option<Organizer>,
    contact:    Option<Text>,
    url:        Option<Uri>,
    freebusy:   Vec<&'a TypedProperty<'a>>,
}

/// Parse a `TypedComponent` into a `VFreeBusy`
#[allow(clippy::too_many_lines)]
impl TryFrom<&TypedComponent<'_>> for VFreeBusy {
    type Error = Vec<SemanticError>;

    fn try_from(comp: &TypedComponent<'_>) -> Result<Self, Self::Error> {
        if comp.name != KW_VFREEBUSY {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VFREEBUSY,
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
                    match get_single_value(prop).ok().and_then(value_to_string) {
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
                    match get_single_value(prop)
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
                PropertyKind::DtEnd => {
                    if props.dt_end.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtEnd,
                        });
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
                PropertyKind::Organizer => {
                    if props.organizer.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Organizer,
                        });
                        continue;
                    }
                    match Organizer::try_from(prop) {
                        Ok(v) => props.organizer = Some(v),
                        Err(e) => {
                            errors.push(e);
                            props.organizer = Some(Organizer {
                                cal_address: Uri { uri: String::new() },
                                cn: None,
                                dir: None,
                                sent_by: None,
                                language: None,
                            });
                        }
                    }
                }
                PropertyKind::Contact => {
                    if props.contact.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Contact,
                        });
                        continue;
                    }
                    match get_single_value(prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(v) => {
                                props.contact = Some(Text {
                                    content: v,
                                    language: get_language(&prop.parameters),
                                });
                            }
                            None => {
                                errors.push(SemanticError::ExpectedType {
                                    property: PropertyKind::Contact,
                                    expected: ValueType::Text,
                                });
                            }
                        },
                        Err(e) => errors.push(e),
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
                PropertyKind::FreeBusy => {
                    props.freebusy.push(prop);
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
        if props.organizer.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Organizer,
            });
        }

        // Parse FREEBUSY properties
        let (mut busy, mut free, mut busy_tentative, mut busy_unavailable) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());

        for prop in props.freebusy {
            // Get the FBTYPE parameter (default is BUSY)
            let fb_type = find_parameter(&prop.parameters, TypedParameterKind::FreeBusyType)
                .and_then(|p| match p {
                    TypedParameter::FreeBusyType { value, .. } => Some(*value),
                    _ => None,
                });

            // Parse all period values
            for value in &prop.values {
                if let Ok(period) = Period::try_from(value) {
                    match fb_type.unwrap_or(FreeBusyType::Busy) {
                        FreeBusyType::Free => free.push(period),
                        FreeBusyType::Busy => busy.push(period),
                        FreeBusyType::BusyTentative => busy_tentative.push(period),
                        FreeBusyType::BusyUnavailable => busy_unavailable.push(period),
                    }
                } else {
                    errors.push(SemanticError::ExpectedType {
                        property: PropertyKind::FreeBusy,
                        expected: ValueType::Period,
                    });
                }
            }
        }

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(VFreeBusy {
            uid: props.uid.unwrap(),           // SAFETY: checked above
            dt_stamp: props.dt_stamp.unwrap(), // SAFETY: checked above
            dt_start: props.dt_start.unwrap(), // SAFETY: checked above
            dt_end: props.dt_end,
            duration: props.duration,
            organizer: props.organizer.unwrap(), // SAFETY: checked above
            contact: props.contact,
            url: props.url,
            busy,
            free,
            busy_tentative,
            busy_unavailable,
        })
    }
}
