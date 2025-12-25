// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions for semantic analysis.
//!
//! This module provides utility functions for converting `TypedComponent`
//! and `TypedProperty` into semantic types.

use crate::keyword::KW_VCALENDAR;
use crate::semantic::{DateTime, Duration, parse_icalendar};
use crate::typed::ValueDuration;
use crate::typed::{
    PropertyKind, TypedComponent, TypedParameter, TypedParameterKind, TypedProperty, Value,
};
use crate::{ICalendar, Uri};

/// Perform semantic analysis on typed components.
///
/// # Errors
///
/// Returns an error if:
/// - The root component structure is invalid (not exactly one VCALENDAR)
/// - The component is not a valid VCALENDAR
/// - Required properties are missing
/// - Property values are invalid
///
/// # Panics
///
/// Panics if `typed_components` has exactly one element but the iterator
/// yields `None`. This should never happen in practice as the length check
/// ensures there is exactly one element.
pub fn semantic_analysis(
    typed_components: Vec<TypedComponent<'_>>,
) -> Result<ICalendar, SemanticError> {
    // Expect exactly one VCALENDAR component at the root
    if typed_components.len() != 1 {
        return Err(SemanticError::InvalidStructure(format!(
            "Expected 1 root {KW_VCALENDAR} component, found {}",
            typed_components.len()
        )));
    }

    let root_component = typed_components.into_iter().next().unwrap();
    parse_icalendar(&root_component)
}

/// Extract the first property with the given name from a component
pub fn find_property<'src>(
    properties: &'src [TypedProperty<'src>],
    name: &str,
) -> Option<&'src TypedProperty<'src>> {
    properties.iter().find(|p| p.name == name)
}

/// Extract the first property with the given `PropertyKind` from a component
pub fn find_property_by_kind<'src>(
    properties: &'src [TypedProperty<'src>],
    kind: PropertyKind,
) -> Option<&'src TypedProperty<'src>> {
    properties.iter().find(|p| p.name == kind.as_str())
}

/// Extract all properties with the given `PropertyKind` from a component
pub fn find_properties<'src>(
    properties: &'src [TypedProperty<'src>],
    kind: PropertyKind,
) -> Vec<&'src TypedProperty<'src>> {
    properties
        .iter()
        .filter(|p| p.name == kind.as_str())
        .collect()
}

/// Extract a parameter value by kind from a property
pub fn find_parameter<'src>(
    parameters: &'src [TypedParameter<'src>],
    kind: TypedParameterKind,
) -> Option<&'src TypedParameter<'src>> {
    parameters.iter().find(|p| p.kind() == kind)
}

/// Get the first value from a property, or return an error
pub fn get_single_value<'src>(
    prop: &'src TypedProperty<'src>,
) -> Result<&'src Value<'src>, SemanticError> {
    match prop.values.first() {
        Some(value) => Ok(value),
        None => Err(SemanticError::InvalidStructure(format!(
            "Property '{}' has no values",
            prop.name
        ))),
    }
}

/// Convert a date value to semantic `DateTime`
pub fn value_to_date_time(value: &Value<'_>) -> Option<DateTime> {
    match value {
        Value::Date(date) => Some(DateTime {
            date: *date,
            time: None,
            tz_id: None,
            date_only: true,
        }),
        Value::DateTime(dt) => Some(DateTime {
            date: dt.date,
            time: Some(dt.time),
            tz_id: None,
            date_only: false,
        }),
        _ => None,
    }
}

/// Convert a duration value to semantic Duration
pub fn value_to_duration(value: &Value<'_>) -> Option<Duration> {
    match value {
        Value::Duration(vdur) => match vdur {
            ValueDuration::DateTime {
                positive,
                day,
                hour,
                minute,
                second,
            } => Some(Duration {
                positive: *positive,
                weeks: None,
                days: if *day > 0 { Some(*day) } else { None },
                hours: if *hour > 0 { Some(*hour) } else { None },
                minutes: if *minute > 0 { Some(*minute) } else { None },
                seconds: if *second > 0 { Some(*second) } else { None },
            }),
            ValueDuration::Week { positive, week } => Some(Duration {
                positive: *positive,
                weeks: Some(*week),
                days: None,
                hours: None,
                minutes: None,
                seconds: None,
            }),
        },
        _ => None,
    }
}

/// Convert text value to string
pub fn value_to_string(value: &Value<'_>) -> Option<String> {
    match value {
        Value::Text(text) => Some(text.resolve().to_string()),
        _ => None,
    }
}

/// Convert integer value to the requested type
pub fn value_to_int<T: TryFrom<i32>>(value: &Value<'_>) -> Option<T> {
    match value {
        Value::Integer(i) => T::try_from(*i).ok(),
        _ => None,
    }
}

/// Get the language parameter from a property's parameters
pub fn get_language(parameters: &[TypedParameter<'_>]) -> Option<String> {
    find_parameter(parameters, TypedParameterKind::Language).and_then(|p| match p {
        TypedParameter::Language { value, .. } => Some(value.resolve().to_string()),
        _ => None,
    })
}

/// Get the timezone identifier from a property's parameters
pub fn get_tzid(parameters: &[TypedParameter<'_>]) -> Option<String> {
    find_parameter(parameters, TypedParameterKind::TimeZoneIdentifier).and_then(|p| match p {
        TypedParameter::TimeZoneIdentifier { value, .. } => Some(value.resolve().to_string()),
        _ => None,
    })
}

/// Parse a calendar user address (URI) value
pub fn parse_cal_address(value: &Value<'_>) -> Option<Uri> {
    match value {
        Value::Text(text) => Some(Uri {
            uri: text.resolve().to_string(),
        }),
        _ => None,
    }
}

/// Error type for parsing operations
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
pub enum SemanticError {
    /// Missing required property
    #[error("Missing required property: {0}")]
    MissingProperty(String),

    /// Invalid property value
    #[error("Invalid value '{1}' for property: {0}")]
    InvalidValue(String, String),

    /// Invalid component structure
    #[error("Invalid component structure: {0}")]
    InvalidStructure(String),

    /// Unknown component type
    #[error("Unknown component type: {0}")]
    UnknownComponent(String),

    /// Unknown property
    #[error("Unknown property: {0}")]
    UnknownProperty(String),
}
