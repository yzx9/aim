// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property and value types for iCalendar semantic components.

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use chumsky::{Parser, error::Rich, extra, input::Stream};

use crate::keyword::{KW_CLASS_CONFIDENTIAL, KW_CLASS_PRIVATE, KW_CLASS_PUBLIC};
use crate::semantic::{DateTime, SemanticError};
use crate::syntax::SpannedSegments;
use crate::typed::{
    AlarmTriggerRelationship, Encoding, PropertyKind, TypedParameter, TypedParameterKind,
    TypedProperty, Value, ValueDuration, ValueText, ValueType, values_float_semicolon,
};

/// Geographic position
#[derive(Debug, Clone, Copy)]
pub struct Geo {
    /// Latitude
    pub lat: f64,

    /// Longitude
    pub lon: f64,
}

impl TryFrom<TypedProperty<'_>> for Geo {
    type Error = SemanticError;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let value = prop.values.first().ok_or(SemanticError::MissingValue {
            property: PropertyKind::Geo,
        })?;

        let text = match value {
            Value::Text(t) => t.resolve().to_string(), // TODO: avoid allocation
            _ => {
                return Err(SemanticError::ExpectedType {
                    property: PropertyKind::Geo,
                    expected: ValueType::Text,
                });
            }
        };

        // Use the typed phase's float parser with semicolon separator
        let stream = Stream::from_iter(text.chars());
        let parser = values_float_semicolon::<_, extra::Err<Rich<char, _>>>();

        match parser.parse(stream).into_result() {
            Ok(result) => {
                let (Some(&lat), Some(&lon)) = (result.first(), result.get(1)) else {
                    return Err(SemanticError::InvalidValue {
                        property: PropertyKind::Geo,
                        value: format!(
                            "Expected exactly 2 float values (lat;long), got {}",
                            result.len()
                        ),
                    });
                };
                Ok(Geo { lat, lon })
            }
            Err(_) => Err(SemanticError::InvalidValue {
                property: PropertyKind::Geo,
                value: format!("Expected 'lat;long' format with semicolon separator, got {text}"),
            }),
        }
    }
}

/// Text with language and alternate representation information
#[derive(Debug, Clone)]
pub struct Text<'src> {
    /// The actual text content
    pub content: ValueText<'src>,

    /// Language code (optional)
    pub language: Option<SpannedSegments<'src>>,

    /// Alternate text representation URI (optional)
    ///
    /// Per RFC 5545, this parameter is not applicable to TZNAME and CATEGORIES
    /// properties, but may be present in other text properties like DESCRIPTION,
    /// SUMMARY, LOCATION, CONTACT, and RESOURCES.
    pub altrep: Option<SpannedSegments<'src>>,
}

impl<'src> TryFrom<TypedProperty<'src>> for Text<'src> {
    type Error = SemanticError;

    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
        let value = prop.values.first().ok_or(SemanticError::MissingValue {
            property: prop.kind,
        })?;

        let content = match value {
            Value::Text(text) => text.clone(),
            _ => {
                return Err(SemanticError::ExpectedType {
                    property: prop.kind,
                    expected: ValueType::Text,
                });
            }
        };

        // Extract language and altrep parameters
        let mut language = None;
        let mut altrep = None;

        for param in prop.parameters {
            match param.kind() {
                TypedParameterKind::Language => {
                    if let TypedParameter::Language { value, .. } = param {
                        language = Some(value);
                    }
                }
                TypedParameterKind::AlternateText => {
                    if let TypedParameter::AlternateText { value, .. } = param {
                        altrep = Some(value);
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            content,
            language,
            altrep,
        })
    }
}

/// Parse multi-valued text properties (CATEGORIES, RESOURCES)
///
/// This helper function parses properties that can have multiple text values
/// (like CATEGORIES or RESOURCES) and returns them as a Vec<Text>.
///
/// Note: Per RFC 5545, ALTREP is not applicable to CATEGORIES, so only the
/// language parameter is extracted.
pub fn parse_multi_text_property(prop: TypedProperty<'_>) -> Vec<Text<'_>> {
    // Get language parameter (shared by all values)
    let language = prop
        .parameters
        .iter()
        .find(|p| p.kind() == TypedParameterKind::Language)
        .and_then(|p| match p {
            TypedParameter::Language { value, .. } => Some(value.clone()),
            _ => None,
        });

    prop.values
        .into_iter()
        .filter_map(|v| match v {
            Value::Text(content) => Some(Text {
                content,
                language: language.clone(),
                altrep: None, // ALTREP not applicable to multi-valued text properties
            }),
            _ => None,
        })
        .collect()
}

/// Classification of calendar data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Classification {
    /// Public classification
    #[default]
    Public,

    /// Private classification
    Private,

    /// Confidential classification
    Confidential,
    // /// Custom classification
    // Custom(String),
}

impl FromStr for Classification {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_CLASS_PUBLIC => Ok(Self::Public),
            KW_CLASS_PRIVATE => Ok(Self::Private),
            KW_CLASS_CONFIDENTIAL => Ok(Self::Confidential),
            _ => Err(format!("Invalid classification: {s}")),
        }
    }
}

impl AsRef<str> for Classification {
    fn as_ref(&self) -> &str {
        match self {
            Self::Public => KW_CLASS_PUBLIC,
            Self::Private => KW_CLASS_PRIVATE,
            Self::Confidential => KW_CLASS_CONFIDENTIAL,
        }
    }
}

impl Display for Classification {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl TryFrom<TypedProperty<'_>> for Classification {
    type Error = SemanticError;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let value = prop.values.first().ok_or(SemanticError::MissingValue {
            property: PropertyKind::Class,
        })?;

        let text = match value {
            Value::Text(t) => t.resolve().to_string(),
            _ => {
                return Err(SemanticError::ExpectedType {
                    property: PropertyKind::Class,
                    expected: ValueType::Text,
                });
            }
        };

        text.parse().map_err(|e| SemanticError::InvalidValue {
            property: PropertyKind::Class,
            value: e,
        })
    }
}

/// Organizer information
#[derive(Debug, Clone)]
pub struct Organizer<'src> {
    /// Calendar user address (mailto: or other URI)
    pub cal_address: ValueText<'src>, // TODO: parse mailto:

    /// Common name (optional)
    pub cn: Option<SpannedSegments<'src>>,

    /// Directory entry reference (optional)
    pub dir: Option<SpannedSegments<'src>>,

    /// Sent by (optional)
    pub sent_by: Option<SpannedSegments<'src>>,

    /// Language (optional)
    pub language: Option<SpannedSegments<'src>>,
}

impl<'src> TryFrom<TypedProperty<'src>> for Organizer<'src> {
    type Error = SemanticError;

    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
        // Collect all optional parameters in a single pass
        let mut cn = None;
        let mut dir = None;
        let mut sent_by = None;
        let mut language = None;

        for param in prop.parameters {
            match param.kind() {
                TypedParameterKind::CommonName => {
                    if let TypedParameter::CommonName { value, .. } = param {
                        cn = Some(value);
                    }
                }
                TypedParameterKind::Directory => {
                    if let TypedParameter::Directory { value, .. } = param {
                        dir = Some(value);
                    }
                }
                TypedParameterKind::SendBy => {
                    if let TypedParameter::SendBy { value, .. } = param {
                        sent_by = Some(value);
                    }
                }
                TypedParameterKind::Language => {
                    if let TypedParameter::Language { value, .. } = param {
                        language = Some(value);
                    }
                }
                // Ignore unknown parameters
                _ => {}
            }
        }

        let cal_address = take_single_value_text(prop.kind, prop.values)?;

        Ok(Organizer {
            cal_address,
            cn,
            dir,
            sent_by,
            language,
        })
    }
}

/// Attachment information
#[derive(Debug, Clone)]
pub struct Attachment<'src> {
    /// URI or binary data
    pub value: AttachmentValue<'src>,

    /// Format type (optional)
    pub fmt_type: Option<SpannedSegments<'src>>,

    /// Encoding (optional)
    pub encoding: Option<Encoding>,
}

/// Attachment value (URI or binary)
#[derive(Debug, Clone)]
pub enum AttachmentValue<'src> {
    /// URI reference
    Uri(ValueText<'src>),

    /// Binary data
    Binary(SpannedSegments<'src>),
}

impl<'src> TryFrom<TypedProperty<'src>> for Attachment<'src> {
    type Error = SemanticError;

    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
        // Collect all optional parameters in a single pass
        let mut fmt_type = None;
        let mut encoding = None;

        for param in &prop.parameters {
            match param.kind() {
                TypedParameterKind::FormatType => {
                    if let TypedParameter::FormatType { value, .. } = param {
                        fmt_type = Some(value.clone());
                    }
                }
                TypedParameterKind::Encoding => {
                    if let TypedParameter::Encoding { value, .. } = param {
                        encoding = Some(*value);
                    }
                }
                // Ignore unknown parameters
                _ => {}
            }
        }

        let value = take_single_value(prop.kind, prop.values)?;
        match value {
            Value::Text(uri) => Ok(Attachment {
                value: AttachmentValue::Uri(uri),
                fmt_type,
                encoding,
            }),
            Value::Binary(data) => Ok(Attachment {
                value: AttachmentValue::Binary(data),
                fmt_type,
                encoding,
            }),
            _ => Err(SemanticError::InvalidValue {
                property: PropertyKind::Attach,
                value: "Expected URI or binary value".to_string(),
            }),
        }
    }
}

/// Trigger for alarms
#[derive(Debug, Clone)]
pub struct Trigger<'src> {
    /// When to trigger (relative or absolute)
    pub value: TriggerValue<'src>,

    /// Related parameter for relative triggers
    pub related: Option<AlarmTriggerRelationship>,
}

/// Trigger value (relative duration or absolute date/time)
#[derive(Debug, Clone)]
pub enum TriggerValue<'src> {
    /// Relative duration before/after the event
    Duration(ValueDuration),

    /// Absolute date/time
    DateTime(DateTime<'src>),
}

impl<'src> TryFrom<TypedProperty<'src>> for Trigger<'src> {
    type Error = SemanticError;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        // Collect the RELATED parameter (optional, default is START)
        let mut related = None;

        for param in &prop.parameters {
            if param.kind() == TypedParameterKind::AlarmTriggerRelationship
                && let TypedParameter::AlarmTriggerRelationship { value, .. } = param
            {
                related = Some(*value);
            }
            // Ignore unknown parameters
        }

        let value = prop.values.first().ok_or(SemanticError::MissingValue {
            property: PropertyKind::Trigger,
        })?;

        match value {
            Value::Duration(dur) => Ok(Trigger {
                value: TriggerValue::Duration(*dur),
                related: Some(related.unwrap_or(AlarmTriggerRelationship::Start)),
            }),
            Value::DateTime(dt) => Ok(Trigger {
                value: TriggerValue::DateTime(DateTime::Floating {
                    date: dt.date,
                    time: dt.time,
                }),
                related: None,
            }),
            _ => Err(SemanticError::InvalidValue {
                property: PropertyKind::Trigger,
                value: "Expected duration or date-time value".to_string(),
            }),
        }
    }
}

/// Get the first value from a property, or return an error
pub fn take_single_value(
    kind: PropertyKind,
    mut values: Vec<Value<'_>>,
) -> Result<Value<'_>, SemanticError> {
    let len = values.len();
    if len > 1 {
        // TODO: better error reporting
        return Err(SemanticError::ConstraintViolation {
            message: format!("Property {kind:?} expected to have a single value, but has {len}",),
        });
    }

    match values.pop() {
        Some(value) => Ok(value),
        None => Err(SemanticError::MissingValue { property: kind }),
    }
}

pub fn take_single_value_text(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<ValueText<'_>, SemanticError> {
    match take_single_value(kind, values) {
        Ok(Value::Text(text)) => Ok(text),
        Ok(_) => Err(SemanticError::ExpectedType {
            property: PropertyKind::Url,
            expected: ValueType::Text,
        }),
        Err(e) => Err(e),
    }
}

/// Get a single floating date-time value from a property
pub fn take_single_value_floating_date_time(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<DateTime<'_>, SemanticError> {
    match take_single_value(kind, values) {
        Ok(Value::DateTime(dt)) => Ok(DateTime::Floating {
            date: dt.date,
            time: dt.time,
        }),
        Ok(_) => Err(SemanticError::ExpectedType {
            property: kind,
            expected: ValueType::DateTime,
        }),
        Err(e) => Err(e),
    }
}

/// Get a single string value from a property
pub fn take_single_value_string(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<String, SemanticError> {
    match take_single_value(kind, values) {
        Ok(Value::Text(v)) => Ok(v.resolve().to_string()),
        Ok(_) => Err(SemanticError::ExpectedType {
            property: kind,
            expected: ValueType::Text,
        }),
        Err(e) => Err(e),
    }
}

/// Get a single integer value from a property
pub fn take_single_value_int<T: TryFrom<i32>>(
    kind: PropertyKind,
    values: Vec<Value<'_>>,
) -> Result<T, SemanticError> {
    match take_single_value(kind, values) {
        Ok(value) => match value {
            Value::Integer(i) => T::try_from(i).map_err(|_| SemanticError::ExpectedType {
                property: kind,
                expected: ValueType::Integer,
            }),
            _ => Err(SemanticError::ExpectedType {
                property: kind,
                expected: ValueType::Integer,
            }),
        },
        Err(e) => Err(e),
    }
}

/// Convert a date-time value to semantic `DateTime` (floating)
pub fn value_to_floating_date_time<'src>(value: &Value<'src>) -> Option<DateTime<'src>> {
    match value {
        Value::DateTime(dt) => Some(DateTime::Floating {
            date: dt.date,
            time: dt.time,
        }),
        _ => None,
    }
}
