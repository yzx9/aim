// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Custom component for x-comp and iana-comp components

use crate::property::Property;
use crate::semantic::icalendar::parse_component_children;
use crate::semantic::{CalendarComponent, SemanticError};
use crate::string_storage::{Segments, StringStorage};
use crate::typed::TypedComponent;

/// Custom component (x-comp or iana-comp)
///
/// Per RFC 5545 Section 3.6:
/// ```txt
/// x-comp     = "BEGIN" ":" x-name CRLF
///               1*contentline
///               "END" ":" x-name CRLF
///
/// iana-comp  = "BEGIN" ":" iana-token CRLF
///               1*contentline
///               "END" ":" iana-token CRLF
/// ```
///
/// Applications MUST ignore x-comp and iana-comp values they don't recognize.
///
/// Custom components can contain any child components (standard or custom),
/// as the RFC 5545 definition allows any content lines, including BEGIN/END blocks.
#[derive(Debug, Clone)]
pub struct CustomComponent<S: StringStorage> {
    /// Component name (e.g., "X-CUSTOM", "X-VENDOR-SPECIAL", "V-SOME-IANA-COMP")
    pub name: String,
    /// Properties in this component
    pub properties: Vec<Property<S>>,
    /// Nested child components (can be any component type)
    pub children: Vec<CalendarComponent<S>>,
    /// Span of the entire component
    pub span: S::Span,
}

impl CustomComponent<Segments<'_>> {
    /// Convert borrowed data to owned data
    #[must_use]
    pub fn to_owned(&self) -> CustomComponent<String> {
        CustomComponent {
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

impl<'src> TryFrom<TypedComponent<'src>> for CustomComponent<Segments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    /// Parse a custom component with all its children
    fn try_from(
        comp: TypedComponent<'_>,
    ) -> Result<CustomComponent<Segments<'_>>, Vec<SemanticError<'_>>> {
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
            Ok(CustomComponent {
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
