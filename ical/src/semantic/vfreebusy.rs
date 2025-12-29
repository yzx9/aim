// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Free/busy time component (VFREEBUSY) for iCalendar semantic components.

use std::convert::TryFrom;

use crate::keyword::KW_VFREEBUSY;
use crate::semantic::property_common::{
    take_single_value, take_single_value_floating_date_time, take_single_value_text,
};
use crate::semantic::{DateTime, Organizer, Period, SemanticError, Text};
use crate::typed::parameter_type::{FreeBusyType, ValueType};
use crate::typed::{
    PropertyKind, TypedComponent, TypedParameter, TypedParameterKind, Value, ValueDate,
    ValueDuration, ValueText,
};

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
                    if props.uid.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Uid,
                        });
                        continue;
                    }
                    props.uid = match take_single_value_text(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            Some(ValueText::default())
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

                    props.dt_stamp =
                        match take_single_value_floating_date_time(prop.kind, prop.values) {
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
                PropertyKind::DtEnd => {
                    if props.dt_end.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtEnd,
                        });
                        continue;
                    }

                    props.dt_end = match DateTime::try_from(prop) {
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
                PropertyKind::Duration => {
                    if props.duration.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Duration,
                        });
                        continue;
                    }
                    props.duration = match take_single_value(prop.kind, prop.values) {
                        Ok(Value::Duration(v)) => Some(v),
                        _ => {
                            errors.push(SemanticError::ExpectedType {
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
                    }
                }
                PropertyKind::Organizer => {
                    if props.organizer.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Organizer,
                        });
                        continue;
                    }

                    props.organizer = match Organizer::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            Some(Organizer {
                                cal_address: ValueText::default(),
                                cn: None,
                                dir: None,
                                sent_by: None,
                                language: None,
                            })
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
                    match Text::try_from(prop) {
                        Ok(text) => props.contact = Some(text),
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

                    match take_single_value_text(prop.kind, prop.values) {
                        Ok(v) => props.url = Some(v),
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::FreeBusy => {
                    // Get the FBTYPE parameter
                    let mut fb_type = None;
                    for param in &prop.parameters {
                        if matches!(param.kind(), TypedParameterKind::FreeBusyType) {
                            if let TypedParameter::FreeBusyType { value, .. } = param {
                                fb_type = Some(*value);
                            }
                            break;
                        }
                    }

                    // Parse all period values
                    for value in &prop.values {
                        if let Ok(period) = Period::try_from(value) {
                            match fb_type.unwrap_or_default() {
                                FreeBusyType::Free => props.free.push(period),
                                FreeBusyType::Busy => props.busy.push(period),
                                FreeBusyType::BusyTentative => props.busy_tentative.push(period),
                                FreeBusyType::BusyUnavailable => {
                                    props.busy_unavailable.push(period);
                                }
                            }
                        } else {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::FreeBusy,
                                expected: ValueType::Period,
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
