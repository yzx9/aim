// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Timezone component (VTIMEZONE) for iCalendar semantic components.

use crate::keyword::{KW_DAYLIGHT, KW_STANDARD, KW_VTIMEZONE};
use crate::property::{
    DtStart, LastModified, Property, PropertyKind, RRule, TzId, TzName, TzOffsetFrom, TzOffsetTo,
    TzUrl, XNameProperty,
};
use crate::semantic::SemanticError;
use crate::string_storage::{SpannedSegments, StringStorage};
use crate::typed::TypedComponent;

/// Timezone component (VTIMEZONE)
#[derive(Debug, Clone)]
pub struct VTimeZone<S: StringStorage> {
    /// Timezone identifier
    pub tz_id: TzId<S>,
    /// Last modification date/time
    pub last_modified: Option<LastModified<S>>,
    /// Timezone URL
    pub tz_url: Option<TzUrl<S>>,
    /// Standard time observances
    pub standard: Vec<TimeZoneObservance<S>>,
    /// Daylight saving time observances
    pub daylight: Vec<TimeZoneObservance<S>>,
    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<XNameProperty<S>>,
    /// Unrecognized / Non-standard properties (preserved for round-trip)
    pub retained_properties: Vec<Property<S>>,
}

/// Parse a `TypedComponent` into a `VTimeZone`
impl<'src> TryFrom<TypedComponent<'src>> for VTimeZone<SpannedSegments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        if !comp.name.eq_str_ignore_ascii_case(KW_VTIMEZONE) {
            errors.push(SemanticError::ExpectedComponent {
                expected: KW_VTIMEZONE,
                got: comp.name,
                span: comp.span,
            });
        }

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop {
                Property::TzId(tz_id) => match props.tz_id {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::TzId,
                        span: tz_id.span,
                    }),
                    None => props.tz_id = Some(tz_id),
                },
                Property::LastModified(dt) => match props.last_modified {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::LastModified,
                        span: dt.span,
                    }),
                    None => props.last_modified = Some(dt),
                },
                Property::TzUrl(tz_url) => match props.tz_url {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::TzUrl,
                        span: tz_url.span,
                    }),
                    None => props.tz_url = Some(tz_url),
                },
                // Preserve unknown properties for round-trip
                Property::XName(prop) => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                prop => {
                    // Preserve other properties not used by VTimeZone for round-trip
                    props.unrecognized_properties.push(prop);
                }
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
            if child.name.eq_str_ignore_ascii_case(KW_STANDARD) {
                match child.try_into() {
                    Ok(v) => standard.push(v),
                    Err(e) => errors.extend(e),
                }
            } else if child.name.eq_str_ignore_ascii_case(KW_DAYLIGHT) {
                match child.try_into() {
                    Ok(v) => daylight.push(v),
                    Err(e) => errors.extend(e),
                }
            } else {
                errors.push(SemanticError::UnknownComponent {
                    span: child.span,
                    component: child.name.to_owned(),
                });

                // still attempt to parse to collect errors
                match TimeZoneObservance::try_from(child) {
                    Ok(_) => {}
                    Err(e) => errors.extend(e),
                }
            }
        }

        if errors.is_empty() {
            Ok(VTimeZone {
                tz_id: props.tz_id.unwrap(), // SAFETY: checked above
                last_modified: props.last_modified,
                tz_url: props.tz_url,
                standard,
                daylight,
                x_properties: props.x_properties,
                retained_properties: props.unrecognized_properties,
            })
        } else {
            Err(errors)
        }
    }
}

impl VTimeZone<SpannedSegments<'_>> {
    /// Convert borrowed data to owned data
    pub fn to_owned(&self) -> VTimeZone<String> {
        VTimeZone {
            tz_id: self.tz_id.to_owned(),
            last_modified: self.last_modified.as_ref().map(LastModified::to_owned),
            tz_url: self.tz_url.as_ref().map(TzUrl::to_owned),
            standard: self
                .standard
                .iter()
                .map(TimeZoneObservance::to_owned)
                .collect(),
            daylight: self
                .daylight
                .iter()
                .map(TimeZoneObservance::to_owned)
                .collect(),
            x_properties: self
                .x_properties
                .iter()
                .map(XNameProperty::to_owned)
                .collect(),
            retained_properties: self
                .retained_properties
                .iter()
                .map(Property::to_owned)
                .collect(),
        }
    }
}

/// Timezone observance (standard or daylight)
#[derive(Debug, Clone)]
pub struct TimeZoneObservance<S: StringStorage> {
    /// Start date/time for this observance
    pub dt_start: DtStart<S>,
    /// Offset from UTC for this observance
    pub tz_offset_from: TzOffsetFrom<S>,
    /// Offset from UTC for this observance
    pub tz_offset_to: TzOffsetTo<S>,
    /// Timezone names
    pub tz_names: Vec<TzName<S>>,
    /// Recurrence rule for this observance
    pub rrule: Option<RRule<S>>,
    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<XNameProperty<S>>,
    /// Unrecognized / Non-standard properties (preserved for round-trip)
    pub retained_properties: Vec<Property<S>>,
}

impl<'src> TryFrom<TypedComponent<'src>> for TimeZoneObservance<SpannedSegments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    /// Parse a timezone observance (STANDARD or DAYLIGHT) component
    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Vec<SemanticError<'src>>> {
        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = ObservanceCollector::default();
        for prop in comp.properties {
            match prop {
                Property::DtStart(dt) => match props.dt_start {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: dt.span,
                        property: PropertyKind::DtStart,
                    }),
                    None => props.dt_start = Some(dt),
                },
                Property::TzOffsetFrom(offset) => match props.tz_offset_from {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: offset.span,
                        property: PropertyKind::TzOffsetFrom,
                    }),
                    None => props.tz_offset_from = Some(offset),
                },
                Property::TzOffsetTo(offset) => match props.tz_offset_to {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: offset.span,
                        property: PropertyKind::TzOffsetTo,
                    }),
                    None => props.tz_offset_to = Some(offset),
                },
                // TZNAME can appear multiple times
                Property::TzName(tz_name) => props.tz_name.push(tz_name),
                Property::RRule(rrule) => match props.rrule {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: rrule.span,
                        property: PropertyKind::RRule,
                    }),
                    None => props.rrule = Some(rrule),
                },
                // Preserve unknown properties for round-trip
                Property::XName(prop) => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                prop => {
                    // Preserve other properties not used by TimeZoneObservance for round-trip
                    props.unrecognized_properties.push(prop);
                }
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
            tz_names: props.tz_name,
            rrule: props.rrule,
            x_properties: props.x_properties,
            retained_properties: props.unrecognized_properties,
        })
    }
}

impl TimeZoneObservance<SpannedSegments<'_>> {
    /// Convert borrowed data to owned data
    pub fn to_owned(&self) -> TimeZoneObservance<String> {
        TimeZoneObservance {
            dt_start: self.dt_start.to_owned(),
            tz_offset_from: self.tz_offset_from.to_owned(),
            tz_offset_to: self.tz_offset_to.to_owned(),
            tz_names: self.tz_names.iter().map(TzName::to_owned).collect(),
            rrule: self.rrule.as_ref().map(RRule::to_owned),
            x_properties: self
                .x_properties
                .iter()
                .map(XNameProperty::to_owned)
                .collect(),
            retained_properties: self
                .retained_properties
                .iter()
                .map(Property::to_owned)
                .collect(),
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<S: StringStorage> {
    tz_id:            Option<TzId<S>>,
    last_modified:    Option<LastModified<S>>,
    tz_url:           Option<TzUrl<S>>,
    x_properties:     Vec<XNameProperty<S>>,
    unrecognized_properties: Vec<Property<S>>,
}

/// Helper struct to collect observance properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct ObservanceCollector<S: StringStorage> {
    dt_start:       Option<DtStart<S>>,
    tz_offset_from: Option<TzOffsetFrom<S>>,
    tz_offset_to:   Option<TzOffsetTo<S>>,
    tz_name:        Vec<TzName<S>>,
    rrule:          Option<RRule<S>>,
    x_properties:   Vec<XNameProperty<S>>,
    unrecognized_properties: Vec<Property<S>>,
}
