// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions for semantic analysis.
//!
//! This module provides utility functions for converting `TypedComponent`
//! and `TypedProperty` into semantic types.

use crate::keyword::KW_VCALENDAR;
use crate::semantic::{Attendee, Classification, DateTime, Geo, Organizer, parse_icalendar};
use crate::typed::{
    CalendarUserType, ParticipationRole, ParticipationStatus, TypedComponent, TypedParameter,
    TypedParameterKind, TypedProperty, Value,
};
use crate::{ICalendar, Uri};
use chumsky::Parser;
use chumsky::error::Rich;

/// Perform semantic analysis on typed components.
///
/// # Errors
///
/// Returns a vector of errors if:
/// - The root component structure is invalid (not exactly one VCALENDAR)
/// - The component is not a valid VCALENDAR
/// - Required properties are missing
/// - Property values are invalid
///
/// # Panics
///
/// Panics if `typed_components` has exactly one element but the iterator
/// yields `None`. This should never happen in practice as the length check
/// ensures there is exactly one element.
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

    let root_component = typed_components.into_iter().next().unwrap();
    parse_icalendar(&root_component)
}

/// Extract the first property with the given name from a component
pub fn find_property<'src>(
    properties: &'src [TypedProperty<'src>],
    name: &str,
) -> Option<&'src TypedProperty<'src>> {
    properties.iter().find(|p| p.name == name)
}

/// Extract a parameter value by kind from a property
pub fn find_parameter<'src>(
    parameters: &'src [TypedParameter<'src>],
    kind: TypedParameterKind,
) -> Option<&'src TypedParameter<'src>> {
    parameters.iter().find(|p| p.kind() == kind)
}

/// Get the first value from a property, or return an error
pub fn get_single_value<'src>(
    prop: &'src TypedProperty<'src>,
) -> Result<&'src Value<'src>, SemanticError> {
    match prop.values.first() {
        Some(value) => Ok(value),
        None => Err(SemanticError::InvalidStructure(format!(
            "Property '{}' has no values",
            prop.name
        ))),
    }
}

/// Convert a date value to semantic `DateTime::Date`
#[allow(dead_code)]
pub fn value_to_date(value: &Value<'_>) -> Option<DateTime> {
    match value {
        Value::Date(date) => Some(DateTime::Date { date: *date }),
        _ => None,
    }
}

/// Convert a date-time value to semantic `DateTime` (floating)
pub fn value_to_date_time(value: &Value<'_>) -> Option<DateTime> {
    match value {
        Value::DateTime(dt) => Some(DateTime::Floating {
            date: dt.date,
            time: dt.time,
        }),
        _ => None,
    }
}

/// Convert a date-time value to semantic `DateTime` with timezone
pub fn value_to_date_time_with_tz(value: &Value<'_>, tz_id: String) -> Option<DateTime> {
    match value {
        Value::DateTime(dt) => {
            // Check if the timezone is UTC (RFC 5545: TZID of "UTC" or empty with trailing Z)
            let is_utc = tz_id.eq_ignore_ascii_case("UTC")
                || tz_id.eq_ignore_ascii_case("GMT")
                || tz_id == "Z"
                || tz_id.eq_ignore_ascii_case("UTC0")
                || tz_id.eq_ignore_ascii_case("UTCF");

            if is_utc {
                Some(DateTime::Utc {
                    date: dt.date,
                    time: dt.time,
                })
            } else {
                Some(DateTime::Zoned {
                    date: dt.date,
                    time: dt.time,
                    tz_id,
                })
            }
        }
        _ => None,
    }
}

/// Convert a date or date-time value to semantic `DateTime`
/// Returns `Date` for date values, `Floating` or `Utc` for date-time values
pub fn value_to_any_date_time(value: &Value<'_>) -> Option<DateTime> {
    match value {
        Value::Date(date) => Some(DateTime::Date { date: *date }),
        Value::DateTime(dt) => {
            // Check if this is a UTC time (indicated by time.utc flag)
            if dt.time.utc {
                Some(DateTime::Utc {
                    date: dt.date,
                    time: dt.time,
                })
            } else {
                Some(DateTime::Floating {
                    date: dt.date,
                    time: dt.time,
                })
            }
        }
        _ => None,
    }
}

/// Convert text value to string
pub fn value_to_string(value: &Value<'_>) -> Option<String> {
    match value {
        Value::Text(text) => Some(text.resolve().to_string()),
        _ => None,
    }
}

/// Convert integer value to the requested type
pub fn value_to_int<T: TryFrom<i32>>(value: &Value<'_>) -> Option<T> {
    match value {
        Value::Integer(i) => T::try_from(*i).ok(),
        _ => None,
    }
}

/// Get the language parameter from a property's parameters
pub fn get_language(parameters: &[TypedParameter<'_>]) -> Option<String> {
    find_parameter(parameters, TypedParameterKind::Language).and_then(|p| match p {
        TypedParameter::Language { value, .. } => Some(value.resolve().to_string()),
        _ => None,
    })
}

/// Get the timezone identifier from a property's parameters
pub fn get_tzid(parameters: &[TypedParameter<'_>]) -> Option<String> {
    find_parameter(parameters, TypedParameterKind::TimeZoneIdentifier).and_then(|p| match p {
        TypedParameter::TimeZoneIdentifier { value, .. } => Some(value.resolve().to_string()),
        _ => None,
    })
}

/// Parse a GEO property value
///
/// The GEO property represents a geographic position with latitude and longitude
/// values separated by a semicolon (e.g., "37.386013;-122.082932").
///
/// # Errors
///
/// Returns an error if:
/// - The property value is not a text value
/// - The value cannot be parsed as two floats separated by a semicolon
/// - The value doesn't contain exactly 2 floats
pub fn parse_geo_property(prop: &TypedProperty<'_>) -> Result<Geo, SemanticError> {
    use chumsky::extra::Err;
    use chumsky::input::Stream;

    use crate::typed::values_float_semicolon;

    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or_else(|| {
        SemanticError::InvalidValue(
            crate::typed::PropertyKind::Geo.as_str().to_string(),
            "Expected text value".to_string(),
        )
    })?;

    // Use the typed phase's float parser with semicolon separator
    let stream = Stream::from_iter(text.chars());
    let parser = values_float_semicolon::<_, Err<Rich<char, _>>>();

    match parser.parse(stream).into_result() {
        Ok(result) => {
            let (Some(&lat), Some(&lon)) = (result.first(), result.get(1)) else {
                return Err(SemanticError::InvalidValue(
                    crate::typed::PropertyKind::Geo.as_str().to_string(),
                    format!(
                        "Expected exactly 2 float values (lat;long), got {}",
                        result.len()
                    ),
                ));
            };
            Ok(Geo { lat, lon })
        }
        Err(_) => Err(SemanticError::InvalidValue(
            crate::typed::PropertyKind::Geo.as_str().to_string(),
            format!("Expected 'lat;long' format with semicolon separator, got {text}"),
        )),
    }
}

/// Parse a CLASSIFICATION property value
///
/// The CLASSIFICATION property specifies the classification of a calendar component
/// (e.g., PUBLIC, PRIVATE, CONFIDENTIAL).
///
/// # Errors
///
/// Returns an error if:
/// - The property value is not a text value
/// - The value is not a valid classification (PUBLIC, PRIVATE, or CONFIDENTIAL)
pub fn parse_classification_property(
    prop: &TypedProperty<'_>,
) -> Result<Classification, SemanticError> {
    let value = get_single_value(prop)?;
    let text = value_to_string(value).ok_or_else(|| {
        SemanticError::InvalidValue(
            crate::typed::PropertyKind::Class.as_str().to_string(),
            "Expected text value".to_string(),
        )
    })?;

    match text.to_uppercase().as_str() {
        "PUBLIC" => Ok(Classification::Public),
        "PRIVATE" => Ok(Classification::Private),
        "CONFIDENTIAL" => Ok(Classification::Confidential),
        _ => Err(SemanticError::InvalidValue(
            crate::typed::PropertyKind::Class.as_str().to_string(),
            format!("Invalid classification: {text}"),
        )),
    }
}

/// Parse an ORGANIZER property into an Organizer
///
/// The ORGANIZER property represents the organizer of a calendar component.
///
/// # Errors
///
/// Returns an error if:
/// - The property value is not a valid calendar user address
pub fn parse_organizer_property(prop: &TypedProperty<'_>) -> Result<Organizer, SemanticError> {
    let cal_address = parse_cal_address(get_single_value(prop)?).ok_or_else(|| {
        SemanticError::InvalidValue(
            crate::typed::PropertyKind::Organizer.as_str().to_string(),
            "Expected calendar user address".to_string(),
        )
    })?;

    // Extract CN parameter
    let cn =
        find_parameter(&prop.parameters, TypedParameterKind::CommonName).and_then(|p| match p {
            TypedParameter::CommonName { value, .. } => Some(value.resolve().to_string()),
            _ => None,
        });

    // Extract DIR parameter
    let dir =
        find_parameter(&prop.parameters, TypedParameterKind::Directory).and_then(|p| match p {
            TypedParameter::Directory { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract SENT-BY parameter
    let sent_by =
        find_parameter(&prop.parameters, TypedParameterKind::SendBy).and_then(|p| match p {
            TypedParameter::SendBy { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract LANGUAGE parameter
    let language = get_language(&prop.parameters);

    Ok(Organizer {
        cal_address,
        cn,
        dir,
        sent_by,
        language,
    })
}

/// Parse an ATTENDEE property into an Attendee
///
/// The ATTENDEE property represents a participant in a calendar component.
///
/// # Errors
///
/// Returns an error if:
/// - The property value is not a valid calendar user address
pub fn parse_attendee_property(prop: &TypedProperty<'_>) -> Result<Attendee, SemanticError> {
    let cal_address = parse_cal_address(get_single_value(prop)?).ok_or_else(|| {
        SemanticError::InvalidValue(
            crate::typed::PropertyKind::Attendee.as_str().to_string(),
            "Expected calendar user address".to_string(),
        )
    })?;

    // Extract CN parameter
    let cn =
        find_parameter(&prop.parameters, TypedParameterKind::CommonName).and_then(|p| match p {
            TypedParameter::CommonName { value, .. } => Some(value.resolve().to_string()),
            _ => None,
        });

    // Extract ROLE parameter (default: REQ-PARTICIPANT)
    let role = find_parameter(&prop.parameters, TypedParameterKind::ParticipationRole)
        .and_then(|p| match p {
            TypedParameter::ParticipationRole { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap_or(ParticipationRole::ReqParticipant);

    // Extract PARTSTAT parameter (default: NEEDS-ACTION)
    let part_stat = find_parameter(&prop.parameters, TypedParameterKind::ParticipationStatus)
        .and_then(|p| match p {
            TypedParameter::ParticipationStatus { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap_or(ParticipationStatus::NeedsAction);

    // Extract RSVP parameter
    let rsvp = find_parameter(&prop.parameters, TypedParameterKind::RsvpExpectation).and_then(
        |p| match p {
            TypedParameter::RsvpExpectation { value, .. } => Some(*value),
            _ => None,
        },
    );

    // Extract CUTYPE parameter (default: INDIVIDUAL)
    let cutype = find_parameter(&prop.parameters, TypedParameterKind::CalendarUserType)
        .and_then(|p| match p {
            TypedParameter::CalendarUserType { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap_or(CalendarUserType::Individual);

    // Extract MEMBER parameter
    let member = find_parameter(&prop.parameters, TypedParameterKind::GroupOrListMembership)
        .and_then(|p| match p {
            TypedParameter::GroupOrListMembership { values, .. } => values.first().map(|v| Uri {
                uri: v.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract DELEGATED-TO parameter
    let delegated_to =
        find_parameter(&prop.parameters, TypedParameterKind::Delegatees).and_then(|p| match p {
            TypedParameter::Delegatees { values, .. } => values.first().map(|v| Uri {
                uri: v.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract DELEGATED-FROM parameter
    let delegated_from =
        find_parameter(&prop.parameters, TypedParameterKind::Delegators).and_then(|p| match p {
            TypedParameter::Delegators { values, .. } => values.first().map(|v| Uri {
                uri: v.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract DIR parameter
    let dir =
        find_parameter(&prop.parameters, TypedParameterKind::Directory).and_then(|p| match p {
            TypedParameter::Directory { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract SENT-BY parameter
    let sent_by =
        find_parameter(&prop.parameters, TypedParameterKind::SendBy).and_then(|p| match p {
            TypedParameter::SendBy { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract LANGUAGE parameter
    let language = get_language(&prop.parameters);

    Ok(Attendee {
        cal_address,
        cn,
        role,
        part_stat,
        rsvp,
        cutype,
        member,
        delegated_to,
        delegated_from,
        dir,
        sent_by,
        language,
    })
}

/// Parse multi-valued text properties (CATEGORIES, RESOURCES)
///
/// This helper function parses properties that can have multiple text values
/// (like CATEGORIES or RESOURCES) and returns them as a Vec<Text>.
///
/// # Arguments
///
/// * `prop` - The property containing multiple text values
///
/// # Returns
///
/// A vector of Text values with their associated language information
pub fn parse_multi_text_property(prop: &TypedProperty<'_>) -> Vec<crate::semantic::Text> {
    use crate::semantic::Text;

    prop.values
        .iter()
        .filter_map(|v| {
            value_to_string(v).map(|s| Text {
                content: s,
                language: get_language(&prop.parameters),
            })
        })
        .collect()
}

/// Parse a calendar user address (URI) value
pub fn parse_cal_address(value: &Value<'_>) -> Option<Uri> {
    match value {
        Value::Text(text) => Some(Uri {
            uri: text.resolve().to_string(),
        }),
        _ => None,
    }
}

/// Error type for parsing operations
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
pub enum SemanticError {
    /// Missing required property
    #[error("Missing required property: {0}")]
    MissingProperty(String),

    /// Invalid property value
    #[error("Invalid value '{1}' for property: {0}")]
    InvalidValue(String, String),

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
