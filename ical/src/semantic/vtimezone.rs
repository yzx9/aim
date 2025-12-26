// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Timezone component (VTIMEZONE) for iCalendar semantic components.

use crate::RecurrenceRule;
use crate::keyword::{KW_DAYLIGHT, KW_STANDARD, KW_VTIMEZONE};
use crate::semantic::SemanticError;
use crate::semantic::analysis::{
    get_language, get_single_value, value_to_date_time, value_to_string,
};
use crate::semantic::properties::{Text, TimeZoneOffset, Uri};
use crate::typed::{PropertyKind, TypedComponent, TypedProperty, Value, ValueDate};

/// Timezone component (VTIMEZONE)
#[derive(Debug, Clone)]
pub struct VTimeZone {
    /// Timezone identifier
    pub tz_id: String,

    /// Last modification date/time
    pub last_modified: Option<crate::semantic::DateTime>,

    /// Timezone URL
    pub tz_url: Option<Uri>,

    /// Standard time observances
    pub standard: Vec<TimeZoneObservance>,

    /// Daylight saving time observances
    pub daylight: Vec<TimeZoneObservance>,
}

/// Timezone observance (standard or daylight)
#[derive(Debug, Clone)]
pub struct TimeZoneObservance {
    /// Start date/time for this observance
    pub dt_start: crate::semantic::DateTime,

    /// Offset from UTC for this observance
    pub tz_offset_from: TimeZoneOffset,

    /// Offset from UTC for this observance
    pub tz_offset_to: TimeZoneOffset,

    /// Timezone names
    pub tz_name: Vec<Text>,

    /// Recurrence rule for this observance
    pub rrule: Option<RecurrenceRule>,
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'a> {
    tz_id:      Option<&'a TypedProperty<'a>>,
    last_modified: Option<&'a TypedProperty<'a>>,
    tz_url:     Option<&'a TypedProperty<'a>>,
}

/// Parse a `TypedComponent` into a `VTimeZone`
#[allow(clippy::too_many_lines)]
pub fn parse_vtimezone(comp: &TypedComponent) -> Result<VTimeZone, Vec<SemanticError>> {
    if comp.name != KW_VTIMEZONE {
        return Err(vec![SemanticError::InvalidStructure(format!(
            "Expected VTIMEZONE component, got '{}'",
            comp.name
        ))]);
    }

    let mut errors = Vec::new();

    // Collect all properties in a single pass
    let mut props = PropertyCollector::default();
    for prop in &comp.properties {
        match prop.name {
            name if name == PropertyKind::TzId.as_str() => {
                if props.tz_id.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::TzId.as_str()
                    )));
                } else {
                    props.tz_id = Some(prop);
                }
            }
            name if name == PropertyKind::LastModified.as_str() => {
                if props.last_modified.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::LastModified.as_str()
                    )));
                } else {
                    props.last_modified = Some(prop);
                }
            }
            name if name == PropertyKind::TzUrl.as_str() => {
                if props.tz_url.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::TzUrl.as_str()
                    )));
                } else {
                    props.tz_url = Some(prop);
                }
            }
            // Ignore unknown properties
            _ => {}
        }
    }

    // TZID is required
    let tz_id = match props.tz_id {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::TzId.as_str().to_string(),
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
                PropertyKind::TzId.as_str().to_string(),
            ));
            String::new()
        }
    };

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
                    crate::semantic::DateTime::Date {
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
                crate::semantic::DateTime::Date {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                }
            }
        });

    // TZURL is optional
    let tz_url = props.tz_url.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_string(value) {
            Some(v) => Uri { uri: v },
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::TzUrl.as_str().to_string(),
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

    // Parse STANDARD and DAYLIGHT sub-components
    let mut standard = Vec::new();
    let mut daylight = Vec::new();

    for child in &comp.children {
        match child.name {
            name if name == KW_STANDARD => match parse_observance(child) {
                Ok(obs) => standard.push(obs),
                Err(e) => errors.extend(e),
            },
            name if name == KW_DAYLIGHT => match parse_observance(child) {
                Ok(obs) => daylight.push(obs),
                Err(e) => errors.extend(e),
            },
            // Ignore unknown sub-components
            _ => {}
        }
    }

    // If we have errors, return them all
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(VTimeZone {
        tz_id,
        last_modified,
        tz_url,
        standard,
        daylight,
    })
}

/// Helper struct to collect observance properties during single-pass iteration
#[derive(Debug, Default)]
struct ObservanceCollector<'a> {
    dt_start: Option<&'a TypedProperty<'a>>,
    tz_offset_from: Option<&'a TypedProperty<'a>>,
    tz_offset_to: Option<&'a TypedProperty<'a>>,
    tz_name: Vec<Text>,
    rrule: Option<&'a TypedProperty<'a>>,
}

/// Parse a timezone observance (STANDARD or DAYLIGHT) component
#[allow(clippy::too_many_lines)]
fn parse_observance(comp: &TypedComponent) -> Result<TimeZoneObservance, Vec<SemanticError>> {
    let mut errors = Vec::new();

    // Collect all properties in a single pass
    let mut props = ObservanceCollector::default();
    for prop in &comp.properties {
        match prop.name {
            name if name == PropertyKind::DtStart.as_str() => {
                if props.dt_start.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::DtStart.as_str()
                    )));
                } else {
                    props.dt_start = Some(prop);
                }
            }
            name if name == PropertyKind::TzOffsetFrom.as_str() => {
                if props.tz_offset_from.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::TzOffsetFrom.as_str()
                    )));
                } else {
                    props.tz_offset_from = Some(prop);
                }
            }
            name if name == PropertyKind::TzOffsetTo.as_str() => {
                if props.tz_offset_to.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::TzOffsetTo.as_str()
                    )));
                } else {
                    props.tz_offset_to = Some(prop);
                }
            }
            name if name == PropertyKind::TzName.as_str() => {
                // TZNAME can appear multiple times
                match get_single_value(prop) {
                    Ok(value) => match value_to_string(value) {
                        Some(v) => props.tz_name.push(Text {
                            content: v,
                            language: get_language(&prop.parameters),
                        }),
                        None => {
                            errors.push(SemanticError::InvalidValue(
                                PropertyKind::TzName.as_str().to_string(),
                                "Expected text value".to_string(),
                            ));
                        }
                    },
                    Err(e) => errors.push(e),
                }
            }
            name if name == PropertyKind::RRule.as_str() => {
                if props.rrule.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::RRule.as_str()
                    )));
                } else {
                    props.rrule = Some(prop);
                }
            }
            // Ignore unknown properties
            _ => {}
        }
    }

    // Check required properties
    let dt_start = match props.dt_start {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_date_time(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::DtStart.as_str().to_string(),
                        "Expected date-time value".to_string(),
                    ));
                    crate::semantic::DateTime::Date {
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
                crate::semantic::DateTime::Date {
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
            crate::semantic::DateTime::Date {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
            }
        }
    };

    let tz_offset_from = match props.tz_offset_from {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_offset(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::TzOffsetFrom.as_str().to_string(),
                        "Expected UTC offset value".to_string(),
                    ));
                    TimeZoneOffset {
                        positive: true,
                        hours: 0,
                        minutes: 0,
                    }
                }
            },
            Err(e) => {
                errors.push(e);
                TimeZoneOffset {
                    positive: true,
                    hours: 0,
                    minutes: 0,
                }
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::TzOffsetFrom.as_str().to_string(),
            ));
            TimeZoneOffset {
                positive: true,
                hours: 0,
                minutes: 0,
            }
        }
    };

    let tz_offset_to = match props.tz_offset_to {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_offset(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::TzOffsetTo.as_str().to_string(),
                        "Expected UTC offset value".to_string(),
                    ));
                    TimeZoneOffset {
                        positive: true,
                        hours: 0,
                        minutes: 0,
                    }
                }
            },
            Err(e) => {
                errors.push(e);
                TimeZoneOffset {
                    positive: true,
                    hours: 0,
                    minutes: 0,
                }
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::TzOffsetTo.as_str().to_string(),
            ));
            TimeZoneOffset {
                positive: true,
                hours: 0,
                minutes: 0,
            }
        }
    };

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

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(TimeZoneObservance {
        dt_start,
        tz_offset_from,
        tz_offset_to,
        tz_name: props.tz_name,
        rrule,
    })
}

/// Convert a Value to a `TimeZoneOffset`
fn value_to_offset(value: &Value<'_>) -> Option<TimeZoneOffset> {
    match value {
        Value::UtcOffset(offset) => Some(TimeZoneOffset {
            positive: offset.positive,
            #[allow(clippy::cast_sign_loss)]
            hours: offset.hour as u8,
            #[allow(clippy::cast_sign_loss)]
            minutes: offset.minute as u8,
        }),
        _ => None,
    }
}
