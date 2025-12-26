// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! iCalendar container types.

use crate::keyword::{
    KW_CALSCALE, KW_CALSCALE_GREGORIAN, KW_METHOD, KW_METHOD_ADD, KW_METHOD_CANCEL,
    KW_METHOD_COUNTER, KW_METHOD_DECLINECOUNTER, KW_METHOD_PUBLISH, KW_METHOD_REFRESH,
    KW_METHOD_REPLY, KW_METHOD_REQUEST, KW_PRODID, KW_VALARM, KW_VCALENDAR, KW_VERSION,
    KW_VERSION_2_0, KW_VEVENT, KW_VFREEBUSY, KW_VJOURNAL, KW_VTIMEZONE, KW_VTODO,
};
use crate::semantic::analysis::{find_property, get_single_value, value_to_string};
use crate::semantic::properties::ProductId;
use crate::semantic::valarm::parse_valarm;
use crate::semantic::vevent::parse_vevent;
use crate::semantic::vfreebusy::parse_vfreebusy;
use crate::semantic::vjournal::parse_vjournal;
use crate::semantic::vtimezone::parse_vtimezone;
use crate::semantic::vtodo::parse_vtodo;
use crate::typed::{TypedComponent, TypedProperty};
use crate::{SemanticError, VAlarm, VEvent, VFreeBusy, VJournal, VTimeZone, VTodo};

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
pub fn parse_icalendar(comp: &TypedComponent<'_>) -> Result<ICalendar, Vec<SemanticError>> {
    let mut errors = Vec::new();

    if comp.name != KW_VCALENDAR {
        return Err(vec![SemanticError::InvalidStructure(format!(
            "Expected VCALENDAR component, got '{}'",
            comp.name
        ))]);
    }

    // PRODID is required
    let prod_id = match find_property(&comp.properties, KW_PRODID) {
        Some(prop) => match parse_product_id(prop) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                ProductId::default()
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(KW_PRODID.to_string()));
            ProductId::default()
        }
    };

    // VERSION is required (should be "2.0")
    let version = match find_property(&comp.properties, KW_VERSION) {
        Some(prop) => match parse_version(prop) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                VersionType::V2_0
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(KW_VERSION.to_string()));
            VersionType::V2_0
        }
    };

    // CALSCALE is optional
    let calscale = match find_property(&comp.properties, KW_CALSCALE) {
        Some(prop) => match parse_calscale(prop) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => None,
    };

    // METHOD is optional
    let method = match find_property(&comp.properties, KW_METHOD) {
        Some(prop) => match parse_method(prop) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => None,
    };

    // Parse child components
    let (components, child_errors) = parse_component_children(&comp.children);
    errors.extend(child_errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(ICalendar {
        prod_id,
        version,
        calscale,
        method,
        components,
    })
}

/// Parse PRODID property into `ProductId`
fn parse_product_id(prop: &TypedProperty<'_>) -> Result<ProductId, SemanticError> {
    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or_else(|| {
        SemanticError::InvalidValue(KW_PRODID.to_string(), "Expected text value".to_string())
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

/// Parse VERSION property into `VersionType`
fn parse_version(prop: &TypedProperty<'_>) -> Result<VersionType, SemanticError> {
    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or(SemanticError::InvalidValue(
        KW_VERSION.to_string(),
        "Expected text value".to_string(),
    ))?;

    match text.as_str() {
        KW_VERSION_2_0 => Ok(VersionType::V2_0),
        _ => Err(SemanticError::InvalidValue(
            KW_VERSION.to_string(),
            format!("Unsupported iCalendar version: {text}"),
        )),
    }
}

/// Parse CALSCALE property into `CalendarScaleType`
fn parse_calscale(prop: &TypedProperty<'_>) -> Result<CalendarScaleType, SemanticError> {
    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or_else(|| {
        SemanticError::InvalidValue(KW_CALSCALE.to_string(), "Expected text value".to_string())
    })?;

    match text.to_uppercase().as_str() {
        KW_CALSCALE_GREGORIAN => Ok(CalendarScaleType::Gregorian),
        _ => Err(SemanticError::InvalidValue(
            KW_CALSCALE.to_string(),
            format!("Unsupported calendar scale: {text}"),
        )),
    }
}

/// Parse METHOD property into `MethodType`
fn parse_method(prop: &TypedProperty<'_>) -> Result<MethodType, SemanticError> {
    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or_else(|| {
        SemanticError::InvalidValue(KW_METHOD.to_string(), "Expected text value".to_string())
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
            KW_METHOD.to_string(),
            format!("Unsupported method type: {text}"),
        )),
    }
}

/// Parse component children into `CalendarComponent` enum
///
/// Returns the components and any errors encountered during parsing.
/// Components with errors are skipped, allowing all valid components
/// to be collected while reporting all errors.
fn parse_component_children(
    children: &[TypedComponent<'_>],
) -> (Vec<CalendarComponent>, Vec<SemanticError>) {
    let mut components = Vec::new();
    let mut errors = Vec::new();

    for child in children {
        match child.name {
            KW_VEVENT => match parse_vevent(child.clone()) {
                Ok(v) => components.push(CalendarComponent::Event(v)),
                Err(e) => errors.extend(e),
            },
            KW_VTODO => match parse_vtodo(child.clone()) {
                Ok(v) => components.push(CalendarComponent::Todo(v)),
                Err(e) => errors.extend(e),
            },
            KW_VJOURNAL => match parse_vjournal(child.clone()) {
                Ok(v) => components.push(CalendarComponent::VJournal(v)),
                Err(e) => errors.extend(e),
            },
            KW_VFREEBUSY => match parse_vfreebusy(child.clone()) {
                Ok(v) => components.push(CalendarComponent::VFreeBusy(v)),
                Err(e) => errors.extend(e),
            },
            KW_VTIMEZONE => match parse_vtimezone(child.clone()) {
                Ok(v) => components.push(CalendarComponent::VTimeZone(v)),
                Err(e) => errors.extend(e),
            },
            KW_VALARM => match parse_valarm(child.clone()) {
                Ok(v) => components.push(CalendarComponent::VAlarm(v)),
                Err(e) => errors.extend(e),
            },
            _ => errors.push(SemanticError::UnknownComponent(child.name.to_string())),
        }
    }

    (components, errors)
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

/// iCalendar version specification
#[derive(Debug, Clone, Copy)]
pub enum VersionType {
    /// Version 2.0 (most common)
    V2_0,
}

/// Calendar scale specification
#[derive(Debug, Clone, Copy)]
pub enum CalendarScaleType {
    /// Gregorian calendar
    Gregorian,
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
