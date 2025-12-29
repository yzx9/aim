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
use crate::typed::{
    AlarmTriggerRelationship, Encoding, PropertyKind, TypedParameter, TypedParameterKind,
    TypedProperty, Value, ValueDuration, ValueType, values_float_semicolon,
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
            Value::Text(t) => t.resolve().to_string(),
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

/// URI representation
#[derive(Debug, Clone)]
pub struct Uri {
    /// The URI string
    pub uri: String,
}

impl TryFrom<&Value<'_>> for Uri {
    type Error = SemanticError;

    fn try_from(value: &Value<'_>) -> Result<Self, Self::Error> {
        match value {
            Value::Text(text) => Ok(Uri {
                uri: text.resolve().to_string(),
            }),
            _ => Err(SemanticError::InvalidValue {
                property: PropertyKind::Url,
                value: format!("Expected text value, got {value:?}"),
            }),
        }
    }
}

impl TryFrom<TypedProperty<'_>> for Uri {
    type Error = SemanticError;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let value = prop.values.first().ok_or(SemanticError::MissingValue {
            property: PropertyKind::Url,
        })?;

        Uri::try_from(value).map_err(|_| SemanticError::ExpectedType {
            property: prop.kind,
            expected: ValueType::Text,
        })
    }
}

/// Text with language and encoding information
#[derive(Debug, Clone)]
pub struct Text {
    /// The actual text content
    pub content: String,

    /// Language code (optional)
    pub language: Option<String>,
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

impl Display for Classification {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => KW_CLASS_PUBLIC.fmt(f),
            Self::Private => KW_CLASS_PRIVATE.fmt(f),
            Self::Confidential => KW_CLASS_CONFIDENTIAL.fmt(f),
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
pub struct Organizer {
    /// Calendar user address (mailto: or other URI)
    pub cal_address: Uri,

    /// Common name (optional)
    pub cn: Option<String>,

    /// Directory entry reference (optional)
    pub dir: Option<Uri>,

    /// Sent by (optional)
    pub sent_by: Option<Uri>,

    /// Language (optional)
    pub language: Option<String>,
}

impl TryFrom<TypedProperty<'_>> for Organizer {
    type Error = SemanticError;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let value = prop.values.first().ok_or(SemanticError::MissingValue {
            property: PropertyKind::Organizer,
        })?;

        let cal_address = Uri::try_from(value).map_err(|_| SemanticError::InvalidValue {
            property: PropertyKind::Organizer,
            value: "Expected calendar user address".to_string(),
        })?;

        // Collect all optional parameters in a single pass
        let mut cn = None;
        let mut dir = None;
        let mut sent_by = None;
        let mut language = None;

        for param in &prop.parameters {
            match param.kind() {
                TypedParameterKind::CommonName => {
                    if let TypedParameter::CommonName { value, .. } = param {
                        cn = Some(value.resolve().to_string());
                    }
                }
                TypedParameterKind::Directory => {
                    if let TypedParameter::Directory { value, .. } = param {
                        dir = Some(Uri {
                            uri: value.resolve().to_string(),
                        });
                    }
                }
                TypedParameterKind::SendBy => {
                    if let TypedParameter::SendBy { value, .. } = param {
                        sent_by = Some(Uri {
                            uri: value.resolve().to_string(),
                        });
                    }
                }
                TypedParameterKind::Language => {
                    if let TypedParameter::Language { value, .. } = param {
                        language = Some(value.resolve().to_string());
                    }
                }
                // Ignore unknown parameters
                _ => {}
            }
        }

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
pub struct Attachment {
    /// URI or binary data
    pub value: AttachmentValue,

    /// Format type (optional)
    pub fmt_type: Option<String>,

    /// Encoding (optional)
    pub encoding: Option<Encoding>,
}

/// Attachment value (URI or binary)
#[derive(Debug, Clone)]
pub enum AttachmentValue {
    /// URI reference
    Uri(Uri),

    /// Binary data
    Binary(Vec<u8>),
}

impl TryFrom<TypedProperty<'_>> for Attachment {
    type Error = SemanticError;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let value = prop.values.first().ok_or(SemanticError::MissingValue {
            property: PropertyKind::Attach,
        })?;

        // Collect all optional parameters in a single pass
        let mut fmt_type = None;
        let mut encoding = None;

        for param in &prop.parameters {
            match param.kind() {
                TypedParameterKind::FormatType => {
                    if let TypedParameter::FormatType { value, .. } = param {
                        fmt_type = Some(value.resolve().to_string());
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

        match value {
            Value::Text(uri) => Ok(Attachment {
                value: AttachmentValue::Uri(Uri {
                    uri: uri.resolve().to_string(),
                }),
                fmt_type,
                encoding,
            }),
            Value::Binary(data) => {
                // Convert SpannedSegments to String, then to Vec<u8>
                let data_str = data.resolve().to_string();
                Ok(Attachment {
                    value: AttachmentValue::Binary(data_str.into_bytes()),
                    fmt_type,
                    encoding,
                })
            }
            _ => Err(SemanticError::InvalidValue {
                property: PropertyKind::Attach,
                value: "Expected URI or binary value".to_string(),
            }),
        }
    }
}

/// Trigger for alarms
#[derive(Debug, Clone)]
pub struct Trigger {
    /// When to trigger (relative or absolute)
    pub value: TriggerValue,

    /// Related parameter for relative triggers
    pub related: Option<AlarmTriggerRelationship>,
}

/// Trigger value (relative duration or absolute date/time)
#[derive(Debug, Clone)]
pub enum TriggerValue {
    /// Relative duration before/after the event
    Duration(ValueDuration),

    /// Absolute date/time
    DateTime(DateTime),
}

impl TryFrom<TypedProperty<'_>> for Trigger {
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
