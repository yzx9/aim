// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions for semantic analysis.
//!
//! This module provides utility functions for converting `TypedComponent`
//! and `TypedProperty` into semantic types.

use crate::ICalendar;
use crate::keyword::KW_VCALENDAR;
use crate::typed::{PropertyKind, TypedComponent, ValueType};

/// Perform semantic analysis on typed components.
///
/// # Errors
///
/// Returns a vector of errors if:
/// - No VCALENDAR components are found
/// - Any components failed to parse
#[allow(clippy::missing_panics_doc)]
pub fn semantic_analysis(
    typed_components: Vec<TypedComponent<'_>>,
) -> Result<Vec<ICalendar>, Vec<SemanticError>> {
    // Return error only if no calendars
    if typed_components.is_empty() {
        return Err(vec![SemanticError::ConstraintViolation {
            message: format!("No {KW_VCALENDAR} components found"),
        }]);
    }

    let mut calendars = Vec::with_capacity(typed_components.len());
    let mut all_errors = Vec::new();

    for component in typed_components {
        match ICalendar::try_from(component) {
            Ok(calendar) => calendars.push(calendar),
            Err(errors) => all_errors.extend(errors),
        }
    }

    if all_errors.is_empty() {
        Ok(calendars)
    } else {
        Err(all_errors)
    }
}

/// Error type for parsing operations
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
pub enum SemanticError {
    /// Missing required property
    #[error("Missing required property: {property}")]
    MissingProperty {
        /// The property that is missing
        property: PropertyKind,
    },

    /// Property has no values
    #[error("Property '{property}' has no values")]
    MissingValue {
        /// The property that has no values
        property: PropertyKind,
    },

    /// Invalid property value
    #[error("Invalid value '{value}' for property: {property}")]
    InvalidValue {
        /// The property that has the invalid value
        property: PropertyKind,
        /// The invalid value description
        value: String,
    },

    /// Expected a different value type
    #[error("Expected {expected} value for property: {property}")]
    ExpectedType {
        /// The property that has the wrong type
        property: PropertyKind,
        /// The expected value type
        expected: ValueType,
    },

    /// Duplicate property
    #[error("Duplicate {property} property")]
    DuplicateProperty {
        /// The property that is duplicated
        property: PropertyKind,
    },

    /// Business rule constraint violation
    #[error("Constraint violation: {message}")]
    ConstraintViolation {
        /// Error message describing the constraint violation
        message: String,
    },

    /// Expected a different component type
    #[error("Expected {expected} component, got '{got}'")]
    ExpectedComponent {
        /// The expected component name
        expected: &'static str,
        /// The actual component name that was found
        got: String,
    },

    /// Unknown component type
    #[error("Unknown component type: {component}")]
    UnknownComponent {
        /// The unknown component name
        component: String,
    },

    /// Unknown property
    #[error("Unknown property: {property}")]
    UnknownProperty {
        /// The unknown property name
        property: String,
    },
}
