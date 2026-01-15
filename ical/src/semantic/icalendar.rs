// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! iCalendar container types.

use std::convert::TryFrom;

use crate::keyword::{
    KW_VALARM, KW_VCALENDAR, KW_VEVENT, KW_VFREEBUSY, KW_VJOURNAL, KW_VTIMEZONE, KW_VTODO,
};
use crate::property::{
    CalendarScale, Method, ProductId, Property, PropertyKind, Version, XNameProperty,
};
use crate::semantic::{
    CustomComponent, SemanticError, VAlarm, VEvent, VFreeBusy, VJournal, VTimeZone, VTodo,
};
use crate::string_storage::{Segments, StringStorage};
use crate::typed::TypedComponent;

/// Main iCalendar object that contains components and properties
#[derive(Debug, Clone)]
pub struct ICalendar<S: StringStorage> {
    /// Product identifier that generated the iCalendar data
    pub prod_id: ProductId<S>,
    /// Version of iCalendar specification
    pub version: Version<S>,
    /// Calendar scale (usually GREGORIAN)
    pub calscale: Option<CalendarScale<S>>,
    /// Method for the iCalendar object (e.g., PUBLISH, REQUEST)
    pub method: Option<Method<S>>,
    /// All calendar components (events, todos, journals, etc.)
    pub components: Vec<CalendarComponent<S>>,
    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<XNameProperty<S>>,
    /// Unrecognized / Non-standard properties (preserved for round-trip)
    pub retained_properties: Vec<Property<S>>,
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
impl<'src> TryFrom<TypedComponent<'src>> for ICalendar<Segments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        if !comp.name.eq_str_ignore_ascii_case(KW_VCALENDAR) {
            errors.push(SemanticError::ExpectedComponent {
                expected: KW_VCALENDAR,
                got: comp.name,
                span: comp.span,
            });
        }

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop {
                Property::ProdId(prod_id) => match props.prod_id {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::ProdId,
                        span: prod_id.span,
                    }),
                    None => props.prod_id = Some(prod_id),
                },
                Property::Version(version) => match props.version {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Version,
                        span: version.span,
                    }),
                    None => props.version = Some(version),
                },
                Property::CalScale(calscale) => match props.calscale {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::CalScale,
                        span: calscale.span,
                    }),
                    None => props.calscale = Some(calscale),
                },
                Property::Method(method) => match props.method {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Method,
                        span: method.span,
                    }),
                    None => props.method = Some(method),
                },
                // Preserve unknown properties for round-trip
                Property::XName(prop) => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                prop => {
                    // Preserve other properties not used by ICalendar for round-trip
                    props.unrecognized_properties.push(prop);
                }
            }
        }

        // Check required fields and use defaults if missing
        if props.prod_id.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::ProdId,
                span: comp.span,
            });
        }

        if props.version.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Version,
                span: comp.span,
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

        if errors.is_empty() {
            Ok(ICalendar {
                prod_id: props.prod_id.unwrap(), // SAFETY: checked above
                version: props.version.unwrap(), // SAFETY: checked above
                calscale: props.calscale,
                method: props.method,
                components,
                x_properties: props.x_properties,
                retained_properties: props.unrecognized_properties,
            })
        } else {
            Err(errors)
        }
    }
}

/// Parse component children into `CalendarComponent` enum
///
/// # Errors
///
/// Returns a vector of errors if no components could be parsed successfully.
/// Individual component parsing errors are collected and included in the result.
pub(crate) fn parse_component_children(
    children: Vec<TypedComponent<'_>>,
) -> Result<Vec<CalendarComponent<Segments<'_>>>, Vec<SemanticError<'_>>> {
    let mut components = Vec::with_capacity(children.len());
    let mut errors = Vec::new();

    for child in children {
        // Use if-else chain since `Segments` doesn't match directly against &str
        let component = if child.name.eq_str_ignore_ascii_case(KW_VEVENT) {
            match child.try_into() {
                Ok(v) => CalendarComponent::Event(v),
                Err(e) => {
                    errors.extend(e);
                    return Ok(components);
                }
            }
        } else if child.name.eq_str_ignore_ascii_case(KW_VTODO) {
            match child.try_into() {
                Ok(v) => CalendarComponent::Todo(v),
                Err(e) => {
                    errors.extend(e);
                    return Ok(components);
                }
            }
        } else if child.name.eq_str_ignore_ascii_case(KW_VJOURNAL) {
            match child.try_into() {
                Ok(v) => CalendarComponent::VJournal(v),
                Err(e) => {
                    errors.extend(e);
                    return Ok(components);
                }
            }
        } else if child.name.eq_str_ignore_ascii_case(KW_VFREEBUSY) {
            match child.try_into() {
                Ok(v) => CalendarComponent::VFreeBusy(v),
                Err(e) => {
                    errors.extend(e);
                    return Ok(components);
                }
            }
        } else if child.name.eq_str_ignore_ascii_case(KW_VTIMEZONE) {
            match child.try_into() {
                Ok(v) => CalendarComponent::VTimeZone(v),
                Err(e) => {
                    errors.extend(e);
                    return Ok(components);
                }
            }
        } else if child.name.eq_str_ignore_ascii_case(KW_VALARM) {
            match child.try_into() {
                Ok(v) => CalendarComponent::VAlarm(v),
                Err(e) => {
                    errors.extend(e);
                    return Ok(components);
                }
            }
        } else {
            // Parse custom component with all its children (recursively)
            match CustomComponent::try_from(child) {
                Ok(custom) => CalendarComponent::Custom(custom),
                Err(e) => {
                    errors.extend(e);
                    return Ok(components);
                }
            }
        };

        components.push(component);
    }

    // Return error only if no components were parsed successfully
    if components.is_empty() && !errors.is_empty() {
        return Err(errors);
    }

    Ok(components)
}

/// Calendar components that can appear in an iCalendar object
#[derive(Debug, Clone)]
pub enum CalendarComponent<S: StringStorage> {
    /// Event component
    Event(VEvent<S>),

    /// To-do component
    Todo(VTodo<S>),

    /// Journal entry component
    VJournal(VJournal<S>),

    /// Free/busy time component
    VFreeBusy(VFreeBusy<S>),

    /// Timezone definition component
    VTimeZone(VTimeZone<S>),

    /// Alarm component
    VAlarm(VAlarm<S>),

    /// Custom component (x-comp or iana-comp)
    Custom(CustomComponent<S>),
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<S: StringStorage> {
    prod_id:        Option<ProductId<S>>,
    version:        Option<Version<S>>,
    calscale:       Option<CalendarScale<S>>,
    method:         Option<Method<S>>,
    x_properties:   Vec<XNameProperty<S>>,
    unrecognized_properties: Vec<Property<S>>,
}

impl ICalendar<Segments<'_>> {
    /// Convert borrowed data to owned data
    #[must_use]
    pub fn to_owned(&self) -> ICalendar<String> {
        ICalendar {
            prod_id: self.prod_id.to_owned(),
            version: self.version.to_owned(),
            calscale: self.calscale.as_ref().map(CalendarScale::to_owned),
            method: self.method.as_ref().map(Method::to_owned),
            components: self
                .components
                .iter()
                .map(CalendarComponent::to_owned)
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

impl CalendarComponent<Segments<'_>> {
    /// Convert borrowed data to owned data
    #[must_use]
    pub fn to_owned(&self) -> CalendarComponent<String> {
        match self {
            Self::Event(v) => CalendarComponent::Event(v.to_owned()),
            Self::Todo(v) => CalendarComponent::Todo(v.to_owned()),
            Self::VJournal(v) => CalendarComponent::VJournal(v.to_owned()),
            Self::VFreeBusy(v) => CalendarComponent::VFreeBusy(v.to_owned()),
            Self::VTimeZone(v) => CalendarComponent::VTimeZone(v.to_owned()),
            Self::VAlarm(v) => CalendarComponent::VAlarm(v.to_owned()),
            Self::Custom(v) => CalendarComponent::Custom(v.to_owned()),
        }
    }
}
