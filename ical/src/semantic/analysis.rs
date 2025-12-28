// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions for semantic analysis.
//!
//! This module provides utility functions for converting `TypedComponent`
//! and `TypedProperty` into semantic types.

use crate::ICalendar;
use crate::keyword::KW_VCALENDAR;
use crate::typed::{PropertyKind, TypedComponent};

/// Perform semantic analysis on typed components.
///
/// # Errors
///
/// Returns a vector of errors if:
/// - The root component structure is invalid (not exactly one VCALENDAR)
/// - The component is not a valid VCALENDAR
/// - Required properties are missing
/// - Property values are invalid
#[allow(clippy::missing_panics_doc)]
pub fn semantic_analysis(
    typed_components: Vec<TypedComponent<'_>>,
) -> Result<ICalendar, Vec<SemanticError>> {
    // Expect exactly one VCALENDAR component at the root
    if typed_components.len() != 1 {
        return Err(vec![SemanticError::InvalidStructure(format!(
            "Expected 1 root {KW_VCALENDAR} component, found {}",
            typed_components.len()
        ))]);
    }

    let root_component = typed_components.into_iter().next().unwrap(); // SAFETY: length checked
    ICalendar::try_from(&root_component)
}

/// Error type for parsing operations
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
pub enum SemanticError {
    /// Missing required property
    #[error("Missing required property: {0}")]
    MissingProperty(PropertyKind),

    /// Invalid property value
    #[error("Invalid value '{1}' for property: {0}")]
    InvalidValue(PropertyKind, String),

    /// Duplicate property
    #[error("Duplicate {0} property")]
    DuplicateProperty(PropertyKind),

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
