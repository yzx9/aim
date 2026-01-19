// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Custom and unrecognized component types for x-comp and iana-comp components

use crate::property::Property;
use crate::semantic::icalendar::parse_component_children;
use crate::semantic::{CalendarComponent, SemanticError};
use crate::string_storage::{Segments, StringStorage};
use crate::typed::TypedComponent;

/// X-component (x-comp)
///
/// Per RFC 5545 Section 3.6:
/// ```txt
/// x-comp     = "BEGIN" ":" x-name CRLF
///               1*contentline
///               "END" ":" x-name CRLF
/// ```
///
/// Custom experimental components MUST have names starting with "X-" or "x-".
/// Applications MUST ignore x-comp values they don't recognize.
///
/// X-components can contain any child components (standard or custom),
/// as the RFC 5545 definition allows any content lines, including BEGIN/END blocks.
#[derive(Debug, Clone)]
pub struct XComponent<S: StringStorage> {
    /// Component name (e.g., "X-CUSTOM", "X-VENDOR-SPECIAL")
    /// Guaranteed to start with "X-" or "x-"
    pub name: String,
    /// Properties in this component
    pub properties: Vec<Property<S>>,
    /// Nested child components (can be any component type)
    pub children: Vec<CalendarComponent<S>>,
    /// Span of the entire component
    pub span: S::Span,
}

impl XComponent<Segments<'_>> {
    /// Convert borrowed data to owned data
    #[must_use]
    pub fn to_owned(&self) -> XComponent<String> {
        XComponent {
            name: self.name.clone(),
            properties: self.properties.iter().map(Property::to_owned).collect(),
            children: self
                .children
                .iter()
                .map(CalendarComponent::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<'src> TryFrom<TypedComponent<'src>> for XComponent<Segments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    /// Parse an X-component with all its children
    fn try_from(
        comp: TypedComponent<'_>,
    ) -> Result<XComponent<Segments<'_>>, Vec<SemanticError<'_>>> {
        let mut errors = Vec::new();

        // Parse child components recursively
        let span = comp.span();
        let children = match parse_component_children(comp.children) {
            Ok(v) => v,
            Err(e) => {
                errors.extend(e);
                Vec::new()
            }
        };
        if errors.is_empty() {
            Ok(XComponent {
                name: comp.name.to_owned(),
                properties: comp.properties,
                children,
                span,
            })
        } else {
            Err(errors)
        }
    }
}

/// Unrecognized component (iana-comp)
///
/// Per RFC 5545 Section 3.6:
/// ```txt
/// iana-comp  = "BEGIN" ":" iana-token CRLF
///               1*contentline
///               "END" ":" iana-token CRLF
/// ```
///
/// Components registered with IANA or other unrecognized components
/// that don't start with "X-". Applications MUST ignore iana-comp
/// values they don't recognize.
///
/// Unrecognized components can contain any child components (standard or custom),
/// as the RFC 5545 definition allows any content lines, including BEGIN/END blocks.
#[derive(Debug, Clone)]
pub struct UnrecognizedComponent<S: StringStorage> {
    /// Component name (e.g., "V-SOME-IANA-COMP")
    /// Does not start with "X-" or "x-"
    pub name: String,
    /// Properties in this component
    pub properties: Vec<Property<S>>,
    /// Nested child components (can be any component type)
    pub children: Vec<CalendarComponent<S>>,
    /// Span of the entire component
    pub span: S::Span,
}

impl UnrecognizedComponent<Segments<'_>> {
    /// Convert borrowed data to owned data
    #[must_use]
    pub fn to_owned(&self) -> UnrecognizedComponent<String> {
        UnrecognizedComponent {
            name: self.name.clone(),
            properties: self.properties.iter().map(Property::to_owned).collect(),
            children: self
                .children
                .iter()
                .map(CalendarComponent::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<'src> TryFrom<TypedComponent<'src>> for UnrecognizedComponent<Segments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    /// Parse an unrecognized component with all its children
    fn try_from(
        comp: TypedComponent<'_>,
    ) -> Result<UnrecognizedComponent<Segments<'_>>, Vec<SemanticError<'_>>> {
        let mut errors = Vec::new();

        // Parse child components recursively
        let span = comp.span();
        let children = match parse_component_children(comp.children) {
            Ok(v) => v,
            Err(e) => {
                errors.extend(e);
                Vec::new()
            }
        };
        if errors.is_empty() {
            Ok(UnrecognizedComponent {
                name: comp.name.to_owned(),
                properties: comp.properties,
                children,
                span,
            })
        } else {
            Err(errors)
        }
    }
}
