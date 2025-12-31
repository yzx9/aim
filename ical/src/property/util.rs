// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions for property parsing in semantic components.
//!
//! This module provides utility functions for extracting and converting
//! property values from typed properties to semantic types.

use std::convert::TryFrom;

use crate::parameter::ValueType;
use crate::property::DateTime;
use crate::semantic::SemanticError;
use crate::typed::{PropertyKind, Value};
use crate::value::ValueText;

/// Get the first value from a property, or return an error
///
/// # Errors
/// Returns `SemanticError::ConstraintViolation` if there are multiple values
pub fn take_single_value(
    kind: PropertyKind,
    mut values: Vec<Value<'_>>,
) -> Result<Value<'_>, SemanticError> {
    let len = values.len();
    if len > 1 {
        // TODO: better error reporting
        return Err(SemanticError::ConstraintViolation {
            message: format!("Property {kind:?} expected to have a single value, but has {len}",),
        });
    }

    match values.pop() {
        Some(value) => Ok(value),
        None => Err(SemanticError::MissingValue { property: kind }),
    }
}

/// Get a single text value from a property
///
/// # Errors
/// Returns `SemanticError::UnexpectedType` if the value is not text
pub fn take_single_text(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<ValueText<'_>, SemanticError> {
    match take_single_value(kind, values) {
        Ok(Value::Text(text)) => Ok(text),
        Ok(_) => Err(SemanticError::UnexpectedType {
            property: PropertyKind::Url,
            expected: ValueType::Text,
        }),
        Err(e) => Err(e),
    }
}

/// Get a single floating date-time value from a property
///
/// # Errors
/// Returns `SemanticError::UnexpectedType` if the value is not date-time
pub fn take_single_floating_date_time(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<DateTime<'_>, SemanticError> {
    match take_single_value(kind, values) {
        Ok(Value::DateTime(dt)) => Ok(DateTime::Floating {
            date: dt.date,
            time: dt.time.into(),
        }),
        Ok(_) => Err(SemanticError::UnexpectedType {
            property: kind,
            expected: ValueType::DateTime,
        }),
        Err(e) => Err(e),
    }
}

/// Get a single string value from a property
///
/// # Errors
/// Returns `SemanticError::UnexpectedType` if the value is not text
pub fn take_single_value_string(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<String, SemanticError> {
    match take_single_value(kind, values) {
        Ok(Value::Text(v)) => Ok(v.resolve().to_string()),
        Ok(_) => Err(SemanticError::UnexpectedType {
            property: kind,
            expected: ValueType::Text,
        }),
        Err(e) => Err(e),
    }
}

/// Get a single integer value from a property
///
/// # Errors
/// Returns `SemanticError::UnexpectedType` if the value is not an integer
pub fn take_single_int<T: TryFrom<i32>>(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<T, SemanticError> {
    match take_single_value(kind, values) {
        Ok(value) => match value {
            Value::Integer(i) => T::try_from(i).map_err(|_| SemanticError::UnexpectedType {
                property: kind,
                expected: ValueType::Integer,
            }),
            _ => Err(SemanticError::UnexpectedType {
                property: kind,
                expected: ValueType::Integer,
            }),
        },
        Err(e) => Err(e),
    }
}

/// Convert a date-time value to semantic `DateTime` (floating)
#[must_use]
pub fn value_to_floating_date_time<'src>(value: &Value<'src>) -> Option<DateTime<'src>> {
    match value {
        Value::DateTime(dt) => Some(DateTime::Floating {
            date: dt.date,
            time: dt.time.into(),
        }),
        _ => None,
    }
}
