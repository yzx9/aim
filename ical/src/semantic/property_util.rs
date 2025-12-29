// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property parser functions for iCalendar semantic components.
//!
//! This module provides helper functions for parsing `TypedProperty`
//! values into semantic property types.

use crate::semantic::{DateTime, SemanticError, Text};
use crate::typed::{TypedParameter, TypedParameterKind, TypedProperty, Value};

/// Convert a date-time value to semantic `DateTime` (floating)
pub fn value_to_floating_date_time(value: &Value<'_>) -> Option<DateTime> {
    match value {
        Value::DateTime(dt) => Some(DateTime::Floating {
            date: dt.date,
            time: dt.time,
        }),
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

/// Parse multi-valued text properties (CATEGORIES, RESOURCES)
///
/// This helper function parses properties that can have multiple text values
/// (like CATEGORIES or RESOURCES) and returns them as a Vec<Text>.
pub fn parse_multi_text_property(prop: TypedProperty<'_>) -> Vec<Text> {
    let language = get_language(&prop.parameters);
    prop.values
        .into_iter()
        .filter_map(|v| {
            value_to_string(&v).map(|s| Text {
                content: s,
                language: language.clone(),
            })
        })
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
        None => Err(SemanticError::MissingValue {
            property: prop.kind,
        }),
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
