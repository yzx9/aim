// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! iCalendar container types.

use std::convert::TryFrom;

use crate::keyword::{
    KW_VALARM, KW_VCALENDAR, KW_VEVENT, KW_VFREEBUSY, KW_VJOURNAL, KW_VTIMEZONE, KW_VTODO,
};
use crate::property::{CalendarScale, Method, ProductId, Property, PropertyKind, Version};
use crate::semantic::{SemanticError, VAlarm, VEvent, VFreeBusy, VJournal, VTimeZone, VTodo};
use crate::typed::TypedComponent;

/// Main iCalendar object that contains components and properties
#[derive(Debug, Clone)]
pub struct ICalendar<'src> {
    /// Product identifier that generated the iCalendar data
    pub prod_id: ProductId,

    /// Version of iCalendar specification
    pub version: Version,

    /// Calendar scale (usually GREGORIAN)
    pub calscale: Option<CalendarScale>,

    /// Method for the iCalendar object (e.g., PUBLISH, REQUEST)
    pub method: Option<Method>,

    /// All calendar components (events, todos, journals, etc.)
    pub components: Vec<CalendarComponent<'src>>,

    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<Property<'src>>,

    /// Unknown IANA properties (preserved for round-trip)
    pub unrecognized_properties: Vec<Property<'src>>,
}

/// Parse a `TypedComponent` into typed `ICalendar`
///
/// # Errors
///
/// Returns a vector of errors if:
/// - The component name is not VCALENDAR
/// - Required properties (PRODID, VERSION) are missing
/// - Property values are invalid or malformed
/// - Child components cannot be parsed
impl<'src> TryFrom<TypedComponent<'src>> for ICalendar<'src> {
    type Error = Vec<SemanticError<'src>>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VCALENDAR {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VCALENDAR,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop {
                Property::ProdId(value) => match props.prod_id {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::ProdId,
                    }),
                    None => props.prod_id = Some(value),
                },
                Property::Version(value) => match props.version {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Version,
                    }),
                    None => props.version = Some(value),
                },
                Property::CalScale(value) => match props.calscale {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::CalScale,
                    }),
                    None => props.calscale = Some(value),
                },
                Property::Method(value) => match props.method {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Method,
                    }),
                    None => props.method = Some(value),
                },
                // Preserve unknown properties for round-trip
                prop @ Property::XName { .. } => {
                    props.x_properties.push(prop);
                }
                prop @ Property::Unrecognized { .. } => {
                    props.unrecognized_properties.push(prop);
                }
                // Ignore other properties not used by ICalendar
                _ => {}
            }
        }

        // Check required fields and use defaults if missing
        if props.prod_id.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::ProdId,
            });
        }

        if props.version.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Version,
            });
        }

        // Parse child components
        let components = match parse_component_children(comp.children) {
            Ok(v) => v,
            Err(e) => {
                errors.extend(e);
                Vec::new()
            }
        };

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ICalendar {
            prod_id: props.prod_id.unwrap(), // SAFETY: checked above
            version: props.version.unwrap(), // SAFETY: checked above
            calscale: props.calscale,
            method: props.method,
            components,
            x_properties: props.x_properties,
            unrecognized_properties: props.unrecognized_properties,
        })
    }
}

/// Parse component children into `CalendarComponent` enum
///
/// # Errors
///
/// Returns a vector of errors if no components could be parsed successfully.
/// Individual component parsing errors are collected and included in the result.
fn parse_component_children(
    children: Vec<TypedComponent<'_>>,
) -> Result<Vec<CalendarComponent<'_>>, Vec<SemanticError<'_>>> {
    let mut components = Vec::with_capacity(children.len());
    let mut errors = Vec::new();

    for child in children {
        match child.name {
            KW_VEVENT => match child.try_into() {
                Ok(v) => components.push(CalendarComponent::Event(v)),
                Err(e) => errors.extend(e),
            },
            KW_VTODO => match child.try_into() {
                Ok(v) => components.push(CalendarComponent::Todo(v)),
                Err(e) => errors.extend(e),
            },
            KW_VJOURNAL => match child.try_into() {
                Ok(v) => components.push(CalendarComponent::VJournal(v)),
                Err(e) => errors.extend(e),
            },
            KW_VFREEBUSY => match child.try_into() {
                Ok(v) => components.push(CalendarComponent::VFreeBusy(v)),
                Err(e) => errors.extend(e),
            },
            KW_VTIMEZONE => match child.try_into() {
                Ok(v) => components.push(CalendarComponent::VTimeZone(v)),
                Err(e) => errors.extend(e),
            },
            KW_VALARM => match child.try_into() {
                Ok(v) => components.push(CalendarComponent::VAlarm(v)),
                Err(e) => errors.extend(e),
            },
            _ => errors.push(SemanticError::UnknownComponent {
                component: child.name.to_string(),
            }),
        }
    }

    // Return error only if no components were parsed successfully
    if components.is_empty() && !errors.is_empty() {
        return Err(errors);
    }

    Ok(components)
}

/// Calendar components that can appear in an iCalendar object
#[derive(Debug, Clone)]
pub enum CalendarComponent<'src> {
    /// Event component
    Event(VEvent<'src>),

    /// To-do component
    Todo(VTodo<'src>),

    /// Journal entry component
    VJournal(VJournal<'src>),

    /// Free/busy time component
    VFreeBusy(VFreeBusy<'src>),

    /// Timezone definition component
    VTimeZone(VTimeZone<'src>),

    /// Alarm component
    VAlarm(VAlarm<'src>),
    // /// Custom component
    // Custom(CustomComponent),
}

// /// Custom component for unknown component types
// #[derive(Debug, Clone)]
// pub struct CustomComponent<'src> {
//     /// Component name
//     pub name: SpannedSegments<'src>,
//
//     /// Properties
//     pub properties: Vec<Property<'src>>,
//
//     /// Nested components
//     pub children: Vec<CalendarComponent<'src>>,
// }

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    prod_id:            Option<ProductId>,
    version:            Option<Version>,
    calscale:           Option<CalendarScale>,
    method:             Option<Method>,
    x_properties:       Vec<Property<'src>>,
    unrecognized_properties: Vec<Property<'src>>,
}
