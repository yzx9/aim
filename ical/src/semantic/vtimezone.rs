// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Timezone component (VTIMEZONE) for iCalendar semantic components.

use crate::keyword::{KW_DAYLIGHT, KW_STANDARD, KW_VTIMEZONE};
use crate::property::{
    DateTime, LastModified, Property, PropertyKind, Text, TzOffsetFrom, TzOffsetTo,
};
use crate::semantic::SemanticError;
use crate::typed::TypedComponent;
use crate::value::RecurrenceRule;

/// Timezone component (VTIMEZONE)
#[derive(Debug, Clone)]
pub struct VTimeZone<'src> {
    /// Timezone identifier
    pub tz_id: Text<'src>,

    /// Last modification date/time
    pub last_modified: Option<LastModified<'src>>,

    /// Timezone URL
    pub tz_url: Option<Text<'src>>,

    /// Standard time observances
    pub standard: Vec<TimeZoneObservance<'src>>,

    /// Daylight saving time observances
    pub daylight: Vec<TimeZoneObservance<'src>>,

    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<Property<'src>>,

    /// Unknown IANA properties (preserved for round-trip)
    pub unrecognized_properties: Vec<Property<'src>>,
}

/// Parse a `TypedComponent` into a `VTimeZone`
impl<'src> TryFrom<TypedComponent<'src>> for VTimeZone<'src> {
    type Error = Vec<SemanticError<'src>>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VTIMEZONE {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VTIMEZONE,
                got: comp.name,
                span: comp.span,
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            // TODO: Use property span instead of component span for DuplicateProperty
            match prop {
                Property::TzId(tz_id) => match props.tz_id {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::TzId,
                        span: comp.span,
                    }),
                    None => props.tz_id = Some(tz_id.0.clone()),
                },
                Property::LastModified(dt) => match props.last_modified {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::LastModified,
                        span: comp.span,
                    }),
                    None => props.last_modified = Some(dt),
                },
                Property::TzUrl(tz_url) => match props.tz_url {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::TzUrl,
                        span: comp.span,
                    }),
                    None => props.tz_url = Some(tz_url.0.clone()),
                },
                // Preserve unknown properties for round-trip
                prop @ Property::XName { .. } => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                // Ignore other properties not used by VTimeZone
                _ => {}
            }
        }

        // Check required fields
        if props.tz_id.is_none() {
            errors.push(SemanticError::MissingProperty {
                span: comp.span,
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
                _ => {
                    errors.push(SemanticError::UnknownComponent {
                        span: child.span,
                        component: child.name.to_string(),
                    });

                    // still attempt to parse to collect errors
                    match TimeZoneObservance::try_from(child) {
                        Ok(_) => {}
                        Err(e) => errors.extend(e),
                    }
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
            x_properties: props.x_properties,
            unrecognized_properties: props.unrecognized_properties,
        })
    }
}

/// Timezone observance (standard or daylight)
#[derive(Debug, Clone)]
pub struct TimeZoneObservance<'src> {
    /// Start date/time for this observance
    pub dt_start: DateTime<'src>,

    /// Offset from UTC for this observance
    pub tz_offset_from: TzOffsetFrom<'src>,

    /// Offset from UTC for this observance
    pub tz_offset_to: TzOffsetTo<'src>,

    /// Timezone names
    pub tz_name: Vec<Text<'src>>,

    /// Recurrence rule for this observance
    pub rrule: Option<RecurrenceRule>,
}

impl<'src> TryFrom<TypedComponent<'src>> for TimeZoneObservance<'src> {
    type Error = Vec<SemanticError<'src>>;

    /// Parse a timezone observance (STANDARD or DAYLIGHT) component
    fn try_from(
        comp: TypedComponent<'src>,
    ) -> Result<TimeZoneObservance<'src>, Vec<SemanticError<'src>>> {
        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = ObservanceCollector::default();
        for prop in comp.properties {
            // TODO: Use property span instead of component span for DuplicateProperty
            match prop {
                Property::DtStart(dt) => match props.dt_start {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: comp.span,
                        property: PropertyKind::DtStart,
                    }),
                    None => props.dt_start = Some(dt.0.clone()),
                },
                Property::TzOffsetFrom(offset) => match props.tz_offset_from {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: comp.span,
                        property: PropertyKind::TzOffsetFrom,
                    }),
                    None => props.tz_offset_from = Some(offset),
                },
                Property::TzOffsetTo(offset) => match props.tz_offset_to {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: comp.span,
                        property: PropertyKind::TzOffsetTo,
                    }),
                    None => props.tz_offset_to = Some(offset),
                },
                Property::TzName(tz_name) => {
                    // TZNAME can appear multiple times
                    props.tz_name.push(tz_name.0.clone());
                }
                Property::RRule(rrule) => match props.rrule {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: comp.span,
                        property: PropertyKind::RRule,
                    }),
                    None => props.rrule = Some(rrule),
                },
                // Ignore other properties not used by TimeZoneObservance
                _ => {}
            }
        }

        // Check required fields
        if props.dt_start.is_none() {
            errors.push(SemanticError::MissingProperty {
                span: comp.span,
                property: PropertyKind::DtStart,
            });
        }
        if props.tz_offset_from.is_none() {
            errors.push(SemanticError::MissingProperty {
                span: comp.span,
                property: PropertyKind::TzOffsetFrom,
            });
        }
        if props.tz_offset_to.is_none() {
            errors.push(SemanticError::MissingProperty {
                span: comp.span,
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
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    tz_id:            Option<Text<'src>>,
    last_modified:    Option<LastModified<'src>>,
    tz_url:           Option<Text<'src>>,
    x_properties:     Vec<Property<'src>>,
    unrecognized_properties: Vec<Property<'src>>,
}

/// Helper struct to collect observance properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct ObservanceCollector<'src> {
    dt_start:       Option<DateTime<'src>>,
    tz_offset_from: Option<TzOffsetFrom<'src>>,
    tz_offset_to:   Option<TzOffsetTo<'src>>,
    tz_name:        Vec<Text<'src>>,
    rrule:          Option<RecurrenceRule>,
}
