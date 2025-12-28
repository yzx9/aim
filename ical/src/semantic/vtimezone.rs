// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Timezone component (VTIMEZONE) for iCalendar semantic components.

use std::convert::TryFrom;

use crate::keyword::{KW_DAYLIGHT, KW_STANDARD, KW_VTIMEZONE};
use crate::semantic::property_util::{
    get_language, get_single_value, value_to_floating_date_time, value_to_string,
};
use crate::semantic::{SemanticError, Text, Uri};
use crate::typed::{PropertyKind, TypedComponent, Value, ValueDate, ValueType};
use crate::{DateTime, RecurrenceRule};

/// Timezone component (VTIMEZONE)
#[derive(Debug, Clone)]
pub struct VTimeZone {
    /// Timezone identifier
    pub tz_id: String,

    /// Last modification date/time
    pub last_modified: Option<DateTime>,

    /// Timezone URL
    pub tz_url: Option<Uri>,

    /// Standard time observances
    pub standard: Vec<TimeZoneObservance>,

    /// Daylight saving time observances
    pub daylight: Vec<TimeZoneObservance>,
}

/// Parse a `TypedComponent` into a `VTimeZone`
#[allow(clippy::too_many_lines)]
impl TryFrom<&TypedComponent<'_>> for VTimeZone {
    type Error = Vec<SemanticError>;

    fn try_from(comp: &TypedComponent<'_>) -> Result<Self, Self::Error> {
        if comp.name != KW_VTIMEZONE {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VTIMEZONE,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in &comp.properties {
            match prop.kind {
                PropertyKind::TzId if props.tz_id.is_none() => {
                    match get_single_value(prop).ok().and_then(value_to_string) {
                        Some(v) => props.tz_id = Some(v),
                        None => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::TzId,
                                expected: ValueType::Text,
                            });
                            props.tz_id = Some(String::new());
                        }
                    }
                }
                PropertyKind::TzId => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::TzId,
                    });
                }
                PropertyKind::LastModified if props.last_modified.is_none() => {
                    match get_single_value(prop)
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
                PropertyKind::LastModified => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::LastModified,
                    });
                }
                PropertyKind::TzUrl if props.tz_url.is_none() => match Uri::try_from(prop) {
                    Ok(v) => props.tz_url = Some(v),
                    Err(e) => errors.push(e),
                },
                PropertyKind::TzUrl => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::TzUrl,
                    });
                }
                // Ignore unknown properties
                _ => {}
            }
        }

        // Check required fields
        if props.tz_id.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::TzId,
            });
        }

        // Parse child components (STANDARD and DAYLIGHT observances)
        let mut standard = Vec::new();
        let mut daylight = Vec::new();

        for child in &comp.children {
            match child.name {
                KW_STANDARD => match parse_observance(child) {
                    Ok(v) => standard.push(v),
                    Err(e) => errors.extend(e),
                },
                KW_DAYLIGHT => match parse_observance(child) {
                    Ok(v) => daylight.push(v),
                    Err(e) => errors.extend(e),
                },
                _ => {
                    errors.push(SemanticError::UnknownComponent {
                        component: child.name.to_string(),
                    });
                }
            }
        }

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(VTimeZone {
            tz_id: props.tz_id.unwrap(), // SAFETY: checked above
            last_modified: props.last_modified,
            tz_url: props.tz_url,
            standard,
            daylight,
        })
    }
}

/// Timezone observance (standard or daylight)
#[derive(Debug, Clone)]
pub struct TimeZoneObservance {
    /// Start date/time for this observance
    pub dt_start: DateTime,

    /// Offset from UTC for this observance
    pub tz_offset_from: TimeZoneOffset,

    /// Offset from UTC for this observance
    pub tz_offset_to: TimeZoneOffset,

    /// Timezone names
    pub tz_name: Vec<Text>,

    /// Recurrence rule for this observance
    pub rrule: Option<RecurrenceRule>,
}

/// Parse a timezone observance (STANDARD or DAYLIGHT) component
#[allow(clippy::too_many_lines)]
fn parse_observance(comp: &TypedComponent) -> Result<TimeZoneObservance, Vec<SemanticError>> {
    let mut errors = Vec::new();

    // Collect all properties in a single pass
    let mut props = ObservanceCollector::default();
    for prop in &comp.properties {
        match prop.kind {
            PropertyKind::DtStart if props.dt_start.is_none() => {
                match get_single_value(prop)
                    .ok()
                    .and_then(value_to_floating_date_time)
                {
                    Some(v) => props.dt_start = Some(v),
                    None => {
                        errors.push(SemanticError::InvalidValue {
                            property: PropertyKind::DtStart,
                            value: "Expected date-time value".to_string(),
                        });
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
            PropertyKind::DtStart => {
                errors.push(SemanticError::DuplicateProperty {
                    property: PropertyKind::DtStart,
                });
            }
            PropertyKind::TzOffsetFrom if props.tz_offset_from.is_none() => {
                match get_single_value(prop) {
                    Ok(value) => match TimeZoneOffset::try_from(value) {
                        Ok(v) => props.tz_offset_from = Some(v),
                        Err(e) => {
                            errors.push(e);
                            props.tz_offset_from = Some(TimeZoneOffset {
                                positive: true,
                                hours: 0,
                                minutes: 0,
                            });
                        }
                    },
                    Err(e) => {
                        errors.push(e);
                        props.tz_offset_from = Some(TimeZoneOffset {
                            positive: true,
                            hours: 0,
                            minutes: 0,
                        });
                    }
                }
            }
            PropertyKind::TzOffsetFrom => {
                errors.push(SemanticError::DuplicateProperty {
                    property: PropertyKind::TzOffsetFrom,
                });
            }
            PropertyKind::TzOffsetTo if props.tz_offset_to.is_none() => {
                match get_single_value(prop) {
                    Ok(value) => match TimeZoneOffset::try_from(value) {
                        Ok(v) => props.tz_offset_to = Some(v),
                        Err(e) => {
                            errors.push(e);
                            props.tz_offset_to = Some(TimeZoneOffset {
                                positive: true,
                                hours: 0,
                                minutes: 0,
                            });
                        }
                    },
                    Err(e) => {
                        errors.push(e);
                        props.tz_offset_to = Some(TimeZoneOffset {
                            positive: true,
                            hours: 0,
                            minutes: 0,
                        });
                    }
                }
            }
            PropertyKind::TzOffsetTo => {
                errors.push(SemanticError::DuplicateProperty {
                    property: PropertyKind::TzOffsetTo,
                });
            }
            PropertyKind::TzName => {
                // TZNAME can appear multiple times
                match get_single_value(prop) {
                    Ok(value) => match value_to_string(value) {
                        Some(v) => props.tz_name.push(Text {
                            content: v,
                            language: get_language(&prop.parameters),
                        }),
                        None => {
                            errors.push(SemanticError::InvalidValue {
                                property: PropertyKind::TzName,
                                value: "Expected text value".to_string(),
                            });
                        }
                    },
                    Err(e) => errors.push(e),
                }
            }
            PropertyKind::RRule if props.rrule.is_none() => {
                // TODO: Parse RRULE from text format
                match get_single_value(prop) {
                    Ok(Value::Text(_)) => {}
                    Ok(_) => {
                        errors.push(SemanticError::InvalidValue {
                            property: PropertyKind::RRule,
                            value: "Expected text value".to_string(),
                        });
                    }
                    Err(e) => errors.push(e),
                }
            }
            PropertyKind::RRule => {
                errors.push(SemanticError::DuplicateProperty {
                    property: PropertyKind::RRule,
                });
            }
            // Ignore unknown properties
            _ => {}
        }
    }

    // Check required fields
    if props.dt_start.is_none() {
        errors.push(SemanticError::MissingProperty {
            property: PropertyKind::DtStart,
        });
    }
    if props.tz_offset_from.is_none() {
        errors.push(SemanticError::MissingProperty {
            property: PropertyKind::TzOffsetFrom,
        });
    }
    if props.tz_offset_to.is_none() {
        errors.push(SemanticError::MissingProperty {
            property: PropertyKind::TzOffsetTo,
        });
    }

    // Return all errors if any occurred
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(TimeZoneObservance {
        dt_start: props.dt_start.unwrap(), // SAFETY: checked above
        tz_offset_from: props.tz_offset_from.unwrap(), // SAFETY: checked above
        tz_offset_to: props.tz_offset_to.unwrap(), // SAFETY: checked above
        tz_name: props.tz_name,
        rrule: props.rrule,
    })
}

/// Timezone offset
#[derive(Debug, Clone, Copy)]
pub struct TimeZoneOffset {
    /// Whether the offset is positive
    pub positive: bool,

    /// Hours
    pub hours: u8,

    /// Minutes
    pub minutes: u8,
}

impl TimeZoneOffset {
    /// Try to convert from a Value with `PropertyKind` context
    ///
    /// # Errors
    ///
    /// Returns `Err` if the value is not a `UtcOffset`
    pub fn try_from_value(value: &Value<'_>, kind: PropertyKind) -> Result<Self, SemanticError> {
        match value {
            Value::UtcOffset(offset) => Ok(TimeZoneOffset {
                positive: offset.positive,
                #[allow(clippy::cast_sign_loss)]
                hours: offset.hour as u8,
                #[allow(clippy::cast_sign_loss)]
                minutes: offset.minute as u8,
            }),
            _ => Err(SemanticError::InvalidValue {
                property: kind,
                value: format!("Expected UTC offset value, got {value:?}"),
            }),
        }
    }
}

impl TryFrom<&Value<'_>> for TimeZoneOffset {
    type Error = SemanticError;

    fn try_from(value: &Value<'_>) -> Result<Self, Self::Error> {
        match value {
            Value::UtcOffset(offset) => Ok(TimeZoneOffset {
                positive: offset.positive,
                #[allow(clippy::cast_sign_loss)]
                hours: offset.hour as u8,
                #[allow(clippy::cast_sign_loss)]
                minutes: offset.minute as u8,
            }),
            _ => Err(SemanticError::InvalidValue {
                property: PropertyKind::TzOffsetFrom, // Default fallback
                value: format!("Expected UTC offset value, got {value:?}"),
            }),
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[derive(Debug, Default)]
struct PropertyCollector {
    tz_id: Option<String>,
    last_modified: Option<DateTime>,
    tz_url: Option<Uri>,
}

/// Helper struct to collect observance properties during single-pass iteration
#[derive(Debug, Default)]
struct ObservanceCollector {
    dt_start: Option<DateTime>,
    tz_offset_from: Option<TimeZoneOffset>,
    tz_offset_to: Option<TimeZoneOffset>,
    tz_name: Vec<Text>,
    rrule: Option<RecurrenceRule>,
}
