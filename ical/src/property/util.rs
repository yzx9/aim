// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions for property parsing in semantic components.
//!
//! This module provides utility functions for extracting and converting
//! property values from typed properties to semantic types.

use crate::parameter::ValueKind;
use crate::property::PropertyKind;
use crate::typed::TypedError;
use crate::value::{Value, ValueText};

/// Get the first value from a property, or return an error
///
/// # Errors
/// Returns `SemanticError::ConstraintViolation` if there are multiple values
pub fn take_single_value(
    kind: PropertyKind,
    mut values: Vec<Value<'_>>,
) -> Result<Value<'_>, TypedError<'_>> {
    let len = values.len();
    if len > 1 {
        return Err(TypedError::PropertyInvalidValueCount {
            property: kind,
            expected: 1,
            found: len,
            span: 0..0, // TODO: improve span reporting
        });
    }

    match values.pop() {
        Some(value) => Ok(value),
        None => Err(TypedError::PropertyMissingValue {
            property: kind,
            span: 0..0, // TODO: improve span reporting
        }),
    }
}

/// Get a single text value from a property
///
/// # Errors
/// Returns `SemanticError::UnexpectedType` if the value is not text
pub fn take_single_text(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<ValueText<'_>, TypedError<'_>> {
    match take_single_value(kind, values) {
        Ok(Value::Text(text)) => Ok(text),
        Ok(v) => Err(TypedError::PropertyUnexpectedValue {
            property: kind,
            expected: ValueKind::Text,
            found: v.kind(),
            span: 0..0, // TODO: improve span reporting
        }),
        Err(e) => Err(e),
    }
}

/// Get a single string value from a property
///
/// # Errors
/// Returns `SemanticError::UnexpectedType` if the value is not text
pub fn take_single_string(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<String, TypedError<'_>> {
    match take_single_value(kind, values) {
        Ok(Value::Text(v)) => Ok(v.resolve().to_string()), // TODO: avoid allocation
        Ok(v) => Err(TypedError::PropertyUnexpectedValue {
            property: kind,
            expected: ValueKind::Text,
            found: v.kind(),
            span: 0..0, // TODO: improve span reporting
        }),
        Err(e) => Err(e),
    }
}
