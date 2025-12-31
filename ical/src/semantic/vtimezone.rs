// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Timezone component (VTIMEZONE) for iCalendar semantic components.

use std::convert::TryFrom;

use crate::keyword::{KW_DAYLIGHT, KW_STANDARD, KW_VTIMEZONE};
use crate::property::TimeZoneOffset;
use crate::semantic::property_util::{
    take_single_floating_date_time, take_single_text, take_single_value,
};
use crate::semantic::{DateTime, SemanticError, Text};
use crate::typed::{PropertyKind, TypedComponent, Value};
use crate::value::{RecurrenceRule, ValueDate, ValueText};

/// Timezone component (VTIMEZONE)
#[derive(Debug, Clone)]
pub struct VTimeZone<'src> {
    /// Timezone identifier
    pub tz_id: ValueText<'src>,

    /// Last modification date/time
    pub last_modified: Option<DateTime<'src>>,

    /// Timezone URL
    pub tz_url: Option<ValueText<'src>>,

    /// Standard time observances
    pub standard: Vec<TimeZoneObservance<'src>>,

    /// Daylight saving time observances
    pub daylight: Vec<TimeZoneObservance<'src>>,
}

/// Parse a `TypedComponent` into a `VTimeZone`
#[allow(clippy::too_many_lines)]
impl<'src> TryFrom<TypedComponent<'src>> for VTimeZone<'src> {
    type Error = Vec<SemanticError>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VTIMEZONE {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VTIMEZONE,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop.kind {
                PropertyKind::TzId => {
                    if props.tz_id.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::TzId,
                        });
                        continue;
                    }

                    props.tz_id = match take_single_text(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            Some(ValueText::default())
                        }
                    }
                }
                PropertyKind::LastModified => {
                    if props.last_modified.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::LastModified,
                        });
                        continue;
                    }

                    props.last_modified =
                        match take_single_floating_date_time(prop.kind, prop.values) {
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
                PropertyKind::TzUrl => {
                    if props.tz_url.is_some() {
                        errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::TzUrl,
                        });
                        continue;
                    }

                    props.tz_url = match take_single_text(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            Some(ValueText::default())
                        }
                    }
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

        for child in comp.children {
            match child.name {
                KW_STANDARD => match child.try_into() {
                    Ok(v) => standard.push(v),
                    Err(e) => errors.extend(e),
                },
                KW_DAYLIGHT => match child.try_into() {
                    Ok(v) => daylight.push(v),
                    Err(e) => errors.extend(e),
                },
                _ => errors.push(SemanticError::UnknownComponent {
                    component: child.name.to_string(),
                }),
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
pub struct TimeZoneObservance<'src> {
    /// Start date/time for this observance
    pub dt_start: DateTime<'src>,

    /// Offset from UTC for this observance
    pub tz_offset_from: TimeZoneOffset,

    /// Offset from UTC for this observance
    pub tz_offset_to: TimeZoneOffset,

    /// Timezone names
    pub tz_name: Vec<Text<'src>>,

    /// Recurrence rule for this observance
    pub rrule: Option<RecurrenceRule>,
}

impl<'src> TryFrom<TypedComponent<'src>> for TimeZoneObservance<'src> {
    type Error = Vec<SemanticError>;

    /// Parse a timezone observance (STANDARD or DAYLIGHT) component
    #[allow(clippy::too_many_lines)]
    fn try_from(
        comp: TypedComponent<'src>,
    ) -> Result<TimeZoneObservance<'src>, Vec<SemanticError>> {
        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = ObservanceCollector::default();
        for prop in comp.properties {
            match prop.kind {
                PropertyKind::DtStart => {
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

                    match props.dt_start {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::DtStart,
                        }),
                        None => props.dt_start = value,
                    }
                }
                PropertyKind::TzOffsetFrom => {
                    let value = match take_single_value(prop.kind, prop.values) {
                        Ok(value) => match value.try_into() {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push(e);
                                Some(TimeZoneOffset {
                                    positive: true,
                                    hours: 0,
                                    minutes: 0,
                                })
                            }
                        },
                        Err(e) => {
                            errors.push(e);
                            Some(TimeZoneOffset {
                                positive: true,
                                hours: 0,
                                minutes: 0,
                            })
                        }
                    };

                    match props.tz_offset_from {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::TzOffsetFrom,
                        }),
                        None => props.tz_offset_from = value,
                    }
                }
                PropertyKind::TzOffsetTo => {
                    let value = match take_single_value(prop.kind, prop.values) {
                        Ok(value) => match TimeZoneOffset::try_from(value) {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push(e);
                                Some(TimeZoneOffset {
                                    positive: true,
                                    hours: 0,
                                    minutes: 0,
                                })
                            }
                        },
                        Err(e) => {
                            errors.push(e);
                            Some(TimeZoneOffset {
                                positive: true,
                                hours: 0,
                                minutes: 0,
                            })
                        }
                    };

                    match props.tz_offset_to {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::TzOffsetTo,
                        }),
                        None => props.tz_offset_to = value,
                    }
                }
                PropertyKind::TzName => match Text::try_from(prop) {
                    // TZNAME can appear multiple times
                    Ok(text) => props.tz_name.push(text),
                    Err(e) => errors.extend(e),
                },
                PropertyKind::RRule => {
                    match take_single_value(prop.kind, prop.values) {
                        Ok(Value::Text(_)) => {
                            // TODO: Parse RRULE
                            let value = None;

                            match props.rrule {
                                Some(_) => errors.push(SemanticError::DuplicateProperty {
                                    property: PropertyKind::RRule,
                                }),
                                None => props.rrule = value,
                            }
                        }
                        Ok(_) => errors.push(SemanticError::InvalidValue {
                            property: PropertyKind::RRule,
                            value: "Expected text value".to_string(),
                        }),
                        Err(e) => errors.push(e),
                    }
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
}

/// Helper struct to collect properties during single-pass iteration
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    tz_id: Option<ValueText<'src>>,
    last_modified: Option<DateTime<'src>>,
    tz_url: Option<ValueText<'src>>,
}

/// Helper struct to collect observance properties during single-pass iteration
#[derive(Debug, Default)]
struct ObservanceCollector<'src> {
    dt_start: Option<DateTime<'src>>,
    tz_offset_from: Option<TimeZoneOffset>,
    tz_offset_to: Option<TimeZoneOffset>,
    tz_name: Vec<Text<'src>>,
    rrule: Option<RecurrenceRule>,
}
