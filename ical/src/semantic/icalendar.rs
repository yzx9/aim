// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! iCalendar container types.

use std::convert::TryFrom;

use crate::keyword::{
    KW_CALSCALE_GREGORIAN, KW_METHOD_ADD, KW_METHOD_CANCEL, KW_METHOD_COUNTER,
    KW_METHOD_DECLINECOUNTER, KW_METHOD_PUBLISH, KW_METHOD_REFRESH, KW_METHOD_REPLY,
    KW_METHOD_REQUEST, KW_VALARM, KW_VCALENDAR, KW_VERSION_2_0, KW_VEVENT, KW_VFREEBUSY,
    KW_VJOURNAL, KW_VTIMEZONE, KW_VTODO,
};
use crate::semantic::{SemanticError, VAlarm, VEvent, VFreeBusy, VJournal, VTimeZone, VTodo};
use crate::parameter::ValueType;
use crate::typed::{PropertyKind, TypedComponent, TypedProperty, Value};

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
    type Error = Vec<SemanticError>;

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
            match prop.kind {
                PropertyKind::ProdId => {
                    let value = match ProductId::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            Some(ProductId::default())
                        }
                    };

                    match props.prod_id {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::ProdId,
                        }),
                        None => props.prod_id = value,
                    }
                }
                PropertyKind::Version => {
                    let value = match Version::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            Some(Version::V2_0)
                        }
                    };

                    match props.version {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Version,
                        }),
                        None => props.version = value,
                    }
                }
                PropertyKind::CalScale => {
                    let value = match CalendarScale::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.calscale {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::CalScale,
                        }),
                        None => props.calscale = value,
                    }
                }
                PropertyKind::Method => {
                    let value = match Method::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.method {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Method,
                        }),
                        None => props.method = value,
                    }
                }
                // Ignore unknown properties
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
) -> Result<Vec<CalendarComponent<'_>>, Vec<SemanticError>> {
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
//     pub properties: Vec<TypedProperty<'src>>,
//
//     /// Nested components
//     pub children: Vec<CalendarComponent<'src>>,
// }

/// Product identifier that identifies the software that created the iCalendar data
#[derive(Debug, Clone, Default)]
pub struct ProductId {
    /// Company identifier
    pub company: String,

    /// Product identifier
    pub product: String,

    /// Language of the text (optional)
    pub language: Option<String>,
}

impl TryFrom<TypedProperty<'_>> for ProductId {
    type Error = Vec<SemanticError>;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                vec![SemanticError::UnexpectedType {
                    property: PropertyKind::ProdId,
                    expected: ValueType::Text,
                }]
            })?;

        // PRODID format: company//product//language
        // e.g., "-//Mozilla.org/NONSGML Mozilla Calendar V1.0//EN"
        let parts: Vec<_> = text.split("//").collect();
        if parts.len() >= 2 {
            Ok(ProductId {
                company: parts.first().map(|s| (*s).to_string()).unwrap_or_default(),
                product: parts.get(1).map(|s| (*s).to_string()).unwrap_or_default(),
                language: parts.get(2).map(|s| (*s).to_string()),
            })
        } else {
            // If not in the expected format, use the whole string as product
            Ok(ProductId {
                company: String::new(),
                product: text,
                language: None,
            })
        }
    }
}

/// iCalendar version specification
#[derive(Debug, Clone, Copy)]
pub enum Version {
    /// Version 2.0 (most common)
    V2_0,
}

impl TryFrom<TypedProperty<'_>> for Version {
    type Error = Vec<SemanticError>;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                vec![SemanticError::UnexpectedType {
                    property: PropertyKind::Version,
                    expected: ValueType::Text,
                }]
            })?;

        match text.as_str() {
            KW_VERSION_2_0 => Ok(Version::V2_0),
            _ => Err(vec![SemanticError::InvalidValue {
                property: PropertyKind::Version,
                value: format!("Unsupported iCalendar version: {text}"),
            }]),
        }
    }
}

/// Calendar scale specification
#[derive(Debug, Clone, Copy, Default)]
pub enum CalendarScale {
    /// Gregorian calendar
    #[default]
    Gregorian,
}

impl TryFrom<TypedProperty<'_>> for CalendarScale {
    type Error = Vec<SemanticError>;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                vec![SemanticError::UnexpectedType {
                    property: PropertyKind::CalScale,
                    expected: ValueType::Text,
                }]
            })?;

        match text.to_uppercase().as_str() {
            KW_CALSCALE_GREGORIAN => Ok(CalendarScale::Gregorian),
            _ => Err(vec![SemanticError::InvalidValue {
                property: PropertyKind::CalScale,
                value: format!("Unsupported calendar scale: {text}"),
            }]),
        }
    }
}

/// Method types for iCalendar objects
#[derive(Debug, Clone, Copy)]
pub enum Method {
    /// Publish an event
    Publish,

    /// Request an event
    Request,

    /// Reply to an event
    Reply,

    /// Add an event
    Add,

    /// Cancel an event
    Cancel,

    /// Refresh an event
    Refresh,

    /// Counter an event
    Counter,

    /// Decline counter
    DeclineCounter,
    // /// Custom method
    // Custom(String),
}

impl TryFrom<TypedProperty<'_>> for Method {
    type Error = Vec<SemanticError>;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                vec![SemanticError::UnexpectedType {
                    property: PropertyKind::Method,
                    expected: ValueType::Text,
                }]
            })?;

        match text.to_uppercase().as_str() {
            KW_METHOD_PUBLISH => Ok(Method::Publish),
            KW_METHOD_REQUEST => Ok(Method::Request),
            KW_METHOD_REPLY => Ok(Method::Reply),
            KW_METHOD_ADD => Ok(Method::Add),
            KW_METHOD_CANCEL => Ok(Method::Cancel),
            KW_METHOD_REFRESH => Ok(Method::Refresh),
            KW_METHOD_COUNTER => Ok(Method::Counter),
            KW_METHOD_DECLINECOUNTER => Ok(Method::DeclineCounter),
            _ => Err(vec![SemanticError::InvalidValue {
                property: PropertyKind::Method,
                value: format!("Unsupported method type: {text}"),
            }]),
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector {
    prod_id:  Option<ProductId>,
    version:  Option<Version>,
    calscale: Option<CalendarScale>,
    method:   Option<Method>,
}
