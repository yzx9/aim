// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Free/busy time component (VFREEBUSY) for iCalendar semantic components.

use std::convert::TryFrom;

use crate::keyword::KW_VFREEBUSY;
use crate::parameter::{FreeBusyType, ValueType};
use crate::semantic::property_util::{
    take_single_floating_date_time, take_single_text, take_single_value,
};
use crate::semantic::{DateTime, Organizer, Period, SemanticError, Text};
use crate::typed::{PropertyKind, TypedComponent, TypedParameter, TypedParameterKind, Value};
use crate::value::{ValueDate, ValueDuration, ValueText};

/// Free/busy time component (VFREEBUSY)
#[derive(Debug, Clone)]
pub struct VFreeBusy<'src> {
    /// Unique identifier for the free/busy info
    pub uid: ValueText<'src>,

    /// Date/time the free/busy info was created
    pub dt_stamp: DateTime<'src>,

    /// Start of the free/busy period
    pub dt_start: DateTime<'src>,

    /// End of the free/busy period
    pub dt_end: Option<DateTime<'src>>,

    /// Duration of the free/busy period
    pub duration: Option<ValueDuration>,

    /// Organizer of the free/busy info
    pub organizer: Organizer<'src>,

    /// Contact information
    pub contact: Option<Text<'src>>,

    /// URL for additional free/busy info
    pub url: Option<ValueText<'src>>,

    /// Busy periods
    pub busy: Vec<Period<'src>>,

    /// Free periods
    pub free: Vec<Period<'src>>,

    /// Busy-tentative periods
    pub busy_tentative: Vec<Period<'src>>,

    /// Unavailable periods
    pub busy_unavailable: Vec<Period<'src>>,
}

/// Parse a `TypedComponent` into a `VFreeBusy`
#[allow(clippy::too_many_lines)]
impl<'src> TryFrom<TypedComponent<'src>> for VFreeBusy<'src> {
    type Error = Vec<SemanticError>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VFREEBUSY {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VFREEBUSY,
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
                PropertyKind::Organizer => {
                    let value = match Organizer::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            Some(Organizer {
                                cal_address: ValueText::default(),
                                cn: None,
                                dir: None,
                                sent_by: None,
                                language: None,
                            })
                        }
                    };

                    match props.organizer {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Organizer,
                        }),
                        None => props.organizer = value,
                    }
                }
                PropertyKind::Contact => {
                    let value = match Text::try_from(prop) {
                        Ok(text) => Some(text),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.contact {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Contact,
                        }),
                        None => props.contact = value,
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
                PropertyKind::FreeBusy => {
                    // Get the FBTYPE parameter
                    let mut fb_type = None;
                    for param in prop.parameters {
                        #[allow(clippy::single_match)]
                        match param {
                            TypedParameter::FreeBusyType { value, .. } => match fb_type {
                                Some(_) => errors.push(SemanticError::DuplicateParameter {
                                    parameter: TypedParameterKind::FreeBusyType,
                                }),
                                None => fb_type = Some(value),
                            },
                            _ => {}
                        }
                    }

                    // Parse all period values
                    for value in &prop.values {
                        match Period::try_from(value) {
                            Ok(period) => match fb_type.unwrap_or_default() {
                                FreeBusyType::Free => props.free.push(period),
                                FreeBusyType::Busy => props.busy.push(period),
                                FreeBusyType::BusyTentative => props.busy_tentative.push(period),
                                FreeBusyType::BusyUnavailable => {
                                    props.busy_unavailable.push(period);
                                }
                            },
                            Err(e) => errors.push(e),
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
        if props.organizer.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Organizer,
            });
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
            busy: props.busy,
            free: props.free,
            busy_tentative: props.busy_tentative,
            busy_unavailable: props.busy_unavailable,
        })
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    uid:              Option<ValueText<'src>>,
    dt_stamp:         Option<DateTime<'src>>,
    dt_start:         Option<DateTime<'src>>,
    dt_end:           Option<DateTime<'src>>,
    duration:         Option<ValueDuration>,
    organizer:        Option<Organizer<'src>>,
    contact:          Option<Text<'src>>,
    url:              Option<ValueText<'src>>,
    busy:             Vec<Period<'src>>,
    free:             Vec<Period<'src>>,
    busy_tentative:   Vec<Period<'src>>,
    busy_unavailable: Vec<Period<'src>>,
}
