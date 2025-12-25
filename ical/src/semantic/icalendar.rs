// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! iCalendar container types.

use crate::SemanticError;
use crate::keyword::{
    KW_CALSCALE, KW_METHOD, KW_VALARM, KW_VEVENT, KW_VFREEBUSY, KW_VJOURNAL, KW_VTIMEZONE, KW_VTODO,
};
use crate::semantic::analysis::{get_single_value, value_to_string};
use crate::semantic::properties::ProductId;
use crate::semantic::valarm::parse_valarm;
use crate::semantic::vevent::parse_vevent;
use crate::semantic::vfreebusy::parse_vfreebusy;
use crate::semantic::vjournal::parse_vjournal;
use crate::semantic::vtimezone::parse_vtimezone;
use crate::semantic::vtodo::parse_vtodo;
use crate::semantic::{valarm, vevent, vfreebusy, vjournal, vtimezone, vtodo};
use crate::typed::{TypedComponent, TypedProperty};

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
/// Returns an error if:
/// - The component name is not VCALENDAR
/// - Required properties (PRODID, VERSION) are missing
/// - Property values are invalid or malformed
/// - Child components cannot be parsed
pub fn parse_icalendar(comp: &TypedComponent<'_>) -> Result<ICalendar, SemanticError> {
    use crate::keyword::{KW_CALSCALE, KW_METHOD, KW_PRODID, KW_VCALENDAR, KW_VERSION};
    use crate::semantic::analysis::find_property;

    if comp.name != KW_VCALENDAR {
        return Err(SemanticError::InvalidStructure(format!(
            "Expected VCALENDAR component, got '{}'",
            comp.name
        )));
    }

    // PRODID is required
    let prod_id = find_property(&comp.properties, KW_PRODID)
        .map(parse_product_id)
        .ok_or(SemanticError::MissingProperty(KW_PRODID.to_string()))??;

    // VERSION is required (should be "2.0")
    let version = find_property(&comp.properties, KW_VERSION)
        .map(parse_version)
        .ok_or(SemanticError::MissingProperty(KW_VERSION.to_string()))??;

    // CALSCALE is optional
    let calscale = find_property(&comp.properties, KW_CALSCALE)
        .map(parse_calscale)
        .transpose()?;

    // METHOD is optional
    let method = find_property(&comp.properties, KW_METHOD)
        .map(parse_method)
        .transpose()?;

    // Parse child components
    let components = parse_component_children(&comp.children)?;

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
    use crate::keyword::KW_PRODID;
    use crate::semantic::analysis::{get_single_value, value_to_string};

    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or(SemanticError::InvalidValue(
        KW_PRODID.to_string(),
        "Expected text value".to_string(),
    ))?;

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
    use crate::keyword::KW_VERSION;
    use crate::semantic::analysis::{get_single_value, value_to_string};

    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or(SemanticError::InvalidValue(
        KW_VERSION.to_string(),
        "Expected text value".to_string(),
    ))?;

    match text.as_str() {
        "2.0" => Ok(VersionType::V2_0),
        _ => Err(SemanticError::InvalidValue(
            KW_VERSION.to_string(),
            format!("Unsupported iCalendar version: {text}"),
        )),
    }
}

/// Parse CALSCALE property into `CalendarScaleType`
fn parse_calscale(prop: &TypedProperty<'_>) -> Result<CalendarScaleType, SemanticError> {
    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or(SemanticError::InvalidValue(
        KW_CALSCALE.to_string(),
        "Expected text value".to_string(),
    ))?;

    match text.to_uppercase().as_str() {
        "GREGORIAN" => Ok(CalendarScaleType::Gregorian),
        _ => Err(SemanticError::InvalidValue(
            KW_CALSCALE.to_string(),
            format!("Unsupported calendar scale: {text}"),
        )),
    }
}

/// Parse METHOD property into `MethodType`
fn parse_method(prop: &TypedProperty<'_>) -> Result<MethodType, SemanticError> {
    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or(SemanticError::InvalidValue(
        KW_METHOD.to_string(),
        "Expected text value".to_string(),
    ))?;

    match text.to_uppercase().as_str() {
        "PUBLISH" => Ok(MethodType::Publish),
        "REQUEST" => Ok(MethodType::Request),
        "REPLY" => Ok(MethodType::Reply),
        "ADD" => Ok(MethodType::Add),
        "CANCEL" => Ok(MethodType::Cancel),
        "REFRESH" => Ok(MethodType::Refresh),
        "COUNTER" => Ok(MethodType::Counter),
        "DECLINECOUNTER" => Ok(MethodType::DeclineCounter),
        _ => Err(SemanticError::InvalidValue(
            KW_METHOD.to_string(),
            format!("Unsupported method type: {text}"),
        )),
    }
}

/// Parse component children into `CalendarComponent` enum
fn parse_component_children(
    children: &[TypedComponent<'_>],
) -> Result<Vec<CalendarComponent>, SemanticError> {
    let mut components = Vec::new();
    for child in children {
        match child.name {
            KW_VEVENT => components.push(CalendarComponent::Event(parse_vevent(child.clone())?)),
            KW_VTODO => components.push(CalendarComponent::Todo(parse_vtodo(child.clone())?)),
            KW_VJOURNAL => {
                components.push(CalendarComponent::VJournal(parse_vjournal(child.clone())?));
            }
            KW_VFREEBUSY => components.push(CalendarComponent::VFreeBusy(parse_vfreebusy(
                child.clone(),
            )?)),
            KW_VTIMEZONE => components.push(CalendarComponent::VTimeZone(parse_vtimezone(
                child.clone(),
            )?)),
            KW_VALARM => components.push(CalendarComponent::VAlarm(parse_valarm(child.clone())?)),
            _ => return Err(SemanticError::UnknownComponent(child.name.to_string())),
        }
    }
    Ok(components)
}

/// Calendar components that can appear in an iCalendar object
#[derive(Debug, Clone)]
pub enum CalendarComponent {
    /// Event component
    Event(vevent::VEvent),

    /// To-do component
    Todo(vtodo::VTodo),

    /// Journal entry component
    VJournal(vjournal::VJournal),

    /// Free/busy time component
    VFreeBusy(vfreebusy::VFreeBusy),

    /// Timezone definition component
    VTimeZone(vtimezone::VTimeZone),

    /// Alarm component
    VAlarm(valarm::VAlarm),
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
