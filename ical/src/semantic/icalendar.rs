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
use crate::typed::PropertyKind;
use crate::typed::{TypedComponent, TypedProperty, Value};

/// Main iCalendar object that contains components and properties
#[derive(Debug, Clone)]
pub struct ICalendar {
    /// Product identifier that generated the iCalendar data
    pub prod_id: ProductId,

    /// Version of iCalendar specification
    pub version: VersionType,

    /// Calendar scale (usually GREGORIAN)
    pub calscale: Option<CalendarScaleType>,

    /// Method for the iCalendar object (e.g., PUBLISH, REQUEST)
    pub method: Option<MethodType>,

    /// All calendar components (events, todos, journals, etc.)
    pub components: Vec<CalendarComponent>,
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
impl TryFrom<&TypedComponent<'_>> for ICalendar {
    type Error = Vec<SemanticError>;

    fn try_from(comp: &TypedComponent<'_>) -> Result<Self, Self::Error> {
        if comp.name != KW_VCALENDAR {
            return Err(vec![SemanticError::InvalidStructure(format!(
                "Expected VCALENDAR component, got '{}'",
                comp.name
            ))]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in &comp.properties {
            match prop.kind {
                PropertyKind::ProdId => {
                    if props.prod_id.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::ProdId));
                        continue;
                    }

                    match ProductId::try_from(prop) {
                        Ok(v) => props.prod_id = Some(v),
                        Err(e) => {
                            errors.push(e);
                            props.prod_id = Some(ProductId::default());
                        }
                    }
                }
                PropertyKind::Version => {
                    if props.version.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Version));
                        continue;
                    }

                    match VersionType::try_from(prop) {
                        Ok(v) => props.version = Some(v),
                        Err(e) => {
                            errors.push(e);
                            props.version = Some(VersionType::V2_0);
                        }
                    }
                }
                PropertyKind::CalScale => {
                    if props.calscale.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::CalScale));
                        continue;
                    }

                    match CalendarScaleType::try_from(prop) {
                        Ok(v) => props.calscale = Some(v),
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Method => {
                    if props.method.is_some() {
                        errors.push(SemanticError::DuplicateProperty(PropertyKind::Method));
                        continue;
                    }

                    match MethodType::try_from(prop) {
                        Ok(v) => props.method = Some(v),
                        Err(e) => errors.push(e),
                    }
                }
                // Ignore unknown properties
                _ => {}
            }
        }

        // Check required fields and use defaults if missing
        if props.prod_id.is_none() {
            errors.push(SemanticError::MissingProperty(PropertyKind::ProdId));
        }

        if props.version.is_none() {
            errors.push(SemanticError::MissingProperty(PropertyKind::Version));
        }

        // Parse child components
        let components = match parse_component_children(&comp.children) {
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
    children: &[TypedComponent<'_>],
) -> Result<Vec<CalendarComponent>, Vec<SemanticError>> {
    let mut components = Vec::with_capacity(children.len());
    let mut errors = Vec::new();

    for child in children {
        match child.name {
            KW_VEVENT => match VEvent::try_from(child) {
                Ok(v) => components.push(CalendarComponent::Event(v)),
                Err(e) => errors.extend(e),
            },
            KW_VTODO => match VTodo::try_from(child) {
                Ok(v) => components.push(CalendarComponent::Todo(v)),
                Err(e) => errors.extend(e),
            },
            KW_VJOURNAL => match VJournal::try_from(child) {
                Ok(v) => components.push(CalendarComponent::VJournal(v)),
                Err(e) => errors.extend(e),
            },
            KW_VFREEBUSY => match VFreeBusy::try_from(child) {
                Ok(v) => components.push(CalendarComponent::VFreeBusy(v)),
                Err(e) => errors.extend(e),
            },
            KW_VTIMEZONE => match VTimeZone::try_from(child) {
                Ok(v) => components.push(CalendarComponent::VTimeZone(v)),
                Err(e) => errors.extend(e),
            },
            KW_VALARM => match VAlarm::try_from(child) {
                Ok(v) => components.push(CalendarComponent::VAlarm(v)),
                Err(e) => errors.extend(e),
            },
            _ => errors.push(SemanticError::UnknownComponent(child.name.to_string())),
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
pub enum CalendarComponent {
    /// Event component
    Event(VEvent),

    /// To-do component
    Todo(VTodo),

    /// Journal entry component
    VJournal(VJournal),

    /// Free/busy time component
    VFreeBusy(VFreeBusy),

    /// Timezone definition component
    VTimeZone(VTimeZone),

    /// Alarm component
    VAlarm(VAlarm),
    // /// Custom component
    // Custom(String, CustomComponent),
}

// /// Custom component for unknown component types
// #[derive(Debug, Clone)]
// pub struct CustomComponent {
//     /// Component name
//     pub name: String,
//
//     /// Properties
//     pub properties: HashMap<String, Vec<String>>,
//
//     /// Nested components
//     pub children: Vec<CalendarComponent>,
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

impl TryFrom<&TypedProperty<'_>> for ProductId {
    type Error = SemanticError;

    fn try_from(prop: &TypedProperty<'_>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                SemanticError::InvalidValue(PropertyKind::ProdId, "Expected text value".to_string())
            })?;

        // PRODID format: company//product//language
        // e.g., "-//Mozilla.org/NONSGML Mozilla Calendar V1.0//EN"
        let parts: Vec<&str> = text.split("//").collect();
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
pub enum VersionType {
    /// Version 2.0 (most common)
    V2_0,
}

impl TryFrom<&TypedProperty<'_>> for VersionType {
    type Error = SemanticError;

    fn try_from(prop: &TypedProperty<'_>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::Version,
                    "Expected text value".to_string(),
                )
            })?;

        match text.as_str() {
            KW_VERSION_2_0 => Ok(VersionType::V2_0),
            _ => Err(SemanticError::InvalidValue(
                PropertyKind::Version,
                format!("Unsupported iCalendar version: {text}"),
            )),
        }
    }
}

/// Calendar scale specification
#[derive(Debug, Clone, Copy)]
pub enum CalendarScaleType {
    /// Gregorian calendar
    Gregorian,
}

impl TryFrom<&TypedProperty<'_>> for CalendarScaleType {
    type Error = SemanticError;

    fn try_from(prop: &TypedProperty<'_>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::CalScale,
                    "Expected text value".to_string(),
                )
            })?;

        match text.to_uppercase().as_str() {
            KW_CALSCALE_GREGORIAN => Ok(CalendarScaleType::Gregorian),
            _ => Err(SemanticError::InvalidValue(
                PropertyKind::CalScale,
                format!("Unsupported calendar scale: {text}"),
            )),
        }
    }
}

/// Method types for iCalendar objects
#[derive(Debug, Clone, Copy)]
pub enum MethodType {
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

impl TryFrom<&TypedProperty<'_>> for MethodType {
    type Error = SemanticError;

    fn try_from(prop: &TypedProperty<'_>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                SemanticError::InvalidValue(PropertyKind::Method, "Expected text value".to_string())
            })?;

        match text.to_uppercase().as_str() {
            KW_METHOD_PUBLISH => Ok(MethodType::Publish),
            KW_METHOD_REQUEST => Ok(MethodType::Request),
            KW_METHOD_REPLY => Ok(MethodType::Reply),
            KW_METHOD_ADD => Ok(MethodType::Add),
            KW_METHOD_CANCEL => Ok(MethodType::Cancel),
            KW_METHOD_REFRESH => Ok(MethodType::Refresh),
            KW_METHOD_COUNTER => Ok(MethodType::Counter),
            KW_METHOD_DECLINECOUNTER => Ok(MethodType::DeclineCounter),
            _ => Err(SemanticError::InvalidValue(
                PropertyKind::Method,
                format!("Unsupported method type: {text}"),
            )),
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector {
    prod_id:  Option<ProductId>,
    version:  Option<VersionType>,
    calscale: Option<CalendarScaleType>,
    method:   Option<MethodType>,
}
