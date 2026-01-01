// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Descriptive Component Properties (RFC 5545 Section 3.8.1)
//!
//! This module contains property types for the "Descriptive Component Properties"
//! section of RFC 5545, including:
//! - 3.8.1.1 Attachment
//! - 3.8.1.3 Classification
//! - 3.8.1.6 Geographic Position
//! - Text helpers for properties like Description, Summary, Location, Contact

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use chumsky::{Parser, error::Rich, extra, input::Stream};

use crate::keyword::{KW_CLASS_CONFIDENTIAL, KW_CLASS_PRIVATE, KW_CLASS_PUBLIC};
use crate::parameter::{Encoding, Parameter};
use crate::property::PropertyKind;
use crate::property::util::{take_single_string, take_single_text, take_single_value};
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueText, values_float_semicolon};

/// Geographic position (RFC 5545 Section 3.8.1.6)
#[derive(Debug, Clone, Copy)]
pub struct Geo {
    /// Latitude
    pub lat: f64,

    /// Longitude
    pub lon: f64,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Geo {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let text = take_single_string(prop.kind, prop.values).map_err(|e| vec![e])?;

        // Use the typed phase's float parser with semicolon separator
        let stream = Stream::from_iter(text.chars());
        let parser = values_float_semicolon::<_, extra::Err<Rich<char, _>>>();

        match parser.parse(stream).into_result() {
            Ok(result) => match (result.first(), result.get(1)) {
                (Some(&lat), Some(&lon)) => Ok(Geo { lat, lon }),
                (_, _) => Err(vec![TypedError::PropertyInvalidValue {
                    property: PropertyKind::Geo,
                    value: format!(
                        "Expected exactly 2 float values (lat;long), got {}",
                        result.len()
                    ),
                    span: prop.span,
                }]),
            },
            Err(_) => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Geo,
                value: format!("Expected 'lat;long' format with semicolon separator, got {text}"),
                span: prop.span,
            }]),
        }
    }
}

/// Text with language and alternate representation information
///
/// This is a helper type used by many text properties like:
/// - 3.8.1.5: `Description`
/// - 3.8.1.12: `Summary`
/// - 3.8.1.7: `Location`
/// - 3.8.4.2: `Contact`
/// - 3.8.3.2: `TzName`
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

impl<'src> TryFrom<ParsedProperty<'src>> for Text<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        let content = match take_single_text(prop.kind, prop.values) {
            Ok(text) => text,
            Err(e) => return Err(vec![e]),
        };

        // Extract language and altrep parameters
        let mut language = None;
        let mut altrep = None;

        for param in prop.parameters {
            let kind_name = param.kind().name();
            let param_span = param.span();

            match param {
                Parameter::Language { value, .. } => match language {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => language = Some(value),
                },
                Parameter::AlternateText { value, .. } => match altrep {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => altrep = Some(value),
                },
                _ => {}
            }
        }

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            content,
            language,
            altrep,
        })
    }
}

/// Multi-valued text properties (CATEGORIES, RESOURCES)
///
/// This type represents properties that can have multiple text values,
/// such as CATEGORIES or RESOURCES.
///
/// Note: Per RFC 5545, ALTREP is not applicable to CATEGORIES and RESOURCES,
/// so only the language parameter is extracted.
#[derive(Debug, Clone)]
pub struct Texts<'src> {
    /// List of text values
    pub values: Vec<Text<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Texts<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        // Get language parameter (shared by all values)
        let language = prop
            .parameters
            .into_iter()
            .find(|p| matches!(p, Parameter::Language { .. }))
            .and_then(|p| match p {
                Parameter::Language { value, .. } => Some(value),
                _ => None,
            });

        let values = prop
            .values
            .into_iter()
            .filter_map(|v| match v {
                Value::Text(content) => Some(Text {
                    content,
                    language: language.clone(),
                    altrep: None, // ALTREP not applicable to multi-valued text properties
                }),
                _ => None,
            })
            .collect();

        Ok(Self { values })
    }
}

/// Classification of calendar data (RFC 5545 Section 3.8.1.3)
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

impl<'src> TryFrom<ParsedProperty<'src>> for Classification {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let text = take_single_string(PropertyKind::Class, prop.values).map_err(|e| vec![e])?;
        text.parse().map_err(|e| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Class,
                value: e,
                span: prop.span,
            }]
        })
    }
}

/// Organizer information (RFC 5545 Section 3.8.4.3)
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

impl<'src> TryFrom<ParsedProperty<'src>> for Organizer<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        // Collect all optional parameters in a single pass
        let mut cn = None;
        let mut dir = None;
        let mut sent_by = None;
        let mut language = None;

        for param in prop.parameters {
            let kind_name = param.kind().name();
            let param_span = param.span();

            match param {
                Parameter::CommonName { value, .. } => match cn {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => cn = Some(value),
                },
                Parameter::Directory { value, .. } => match dir {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => dir = Some(value),
                },
                Parameter::SendBy { value, .. } => match sent_by {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => sent_by = Some(value),
                },
                Parameter::Language { value, .. } => match language {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => language = Some(value),
                },
                // Ignore unknown parameters
                _ => {}
            }
        }

        // Get cal_address value
        let cal_address = match take_single_text(prop.kind, prop.values) {
            Ok(text) => text,
            Err(e) => {
                errors.push(e);
                return Err(errors);
            }
        };

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
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

/// Attachment information (RFC 5545 Section 3.8.1.1)
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

impl<'src> TryFrom<ParsedProperty<'src>> for Attachment<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        // Collect all optional parameters in a single pass
        let mut fmt_type = None;
        let mut encoding = None;

        for param in prop.parameters {
            let kind_name = param.kind().name();
            let param_span = param.span();

            match param {
                Parameter::FormatType { value, .. } => match fmt_type {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => fmt_type = Some(value),
                },
                Parameter::Encoding { value, .. } => match encoding {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => encoding = Some(value),
                },
                // Ignore unknown parameters
                _ => {}
            }
        }

        // Get value
        let value = match take_single_value(prop.kind, prop.values) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                return Err(errors);
            }
        };

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        match value {
            Value::Text(uri) => Ok(Attachment {
                value: AttachmentValue::Uri(uri.clone()),
                fmt_type,
                encoding,
            }),
            Value::Binary(data) => Ok(Attachment {
                value: AttachmentValue::Binary(data.clone()),
                fmt_type,
                encoding,
            }),
            _ => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Attach,
                value: "Expected URI or binary value".to_string(),
                span: prop.span,
            }]),
        }
    }
}
