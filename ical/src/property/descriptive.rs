// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Classification and Status Properties (RFC 5545 Section 3.8.1)
//!
//! - 3.8.1.1: `Attachment` - Attached documents or resources
//! - 3.8.1.2: `Categories` - Categories or tags
//! - 3.8.1.3: `Classification` - Access classification (PUBLIC, PRIVATE, CONFIDENTIAL)
//! - 3.8.1.4: `Comment` - Non-processing comments
//! - 3.8.1.5: `Description` - Detailed description
//! - 3.8.1.6: `Geo` - Geographic position (latitude/longitude)
//! - 3.8.1.7: `Location` - Venue location
//! - 3.8.1.8: `PercentComplete` - Percent complete for todos (0-100)
//! - 3.8.1.9: `Priority` - Priority level (0-9, undefined = 0)
//! - 3.8.1.10: `Resources` - Resources
//! - 3.8.1.11: `Status` - Component status (TENTATIVE, CONFIRMED, CANCELLED, etc.)
//! - 3.8.1.12: `Summary` - Summary/subject

use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use chumsky::{Parser, error::Rich, extra, input::Stream};

use crate::keyword::{
    KW_CLASS_CONFIDENTIAL, KW_CLASS_PRIVATE, KW_CLASS_PUBLIC, KW_STATUS_CANCELLED,
    KW_STATUS_COMPLETED, KW_STATUS_CONFIRMED, KW_STATUS_DRAFT, KW_STATUS_FINAL,
    KW_STATUS_IN_PROCESS, KW_STATUS_NEEDS_ACTION, KW_STATUS_TENTATIVE,
};
use crate::parameter::{Encoding, Parameter, ValueKind};
use crate::property::PropertyKind;
use crate::property::util::{Text, Texts, take_single_string, take_single_value};
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueText, values_float_semicolon};

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

impl Attachment<'_> {
    /// Get the property kind for `Attachment`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Attach
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Attachment<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

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
}

impl Classification {
    /// Get the property kind for `Classification`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Class
    }
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

impl fmt::Display for Classification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Classification {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let text = take_single_string(Self::kind(), prop.values).map_err(|e| vec![e])?;
        text.parse().map_err(|e| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Class,
                value: e,
                span: prop.span,
            }]
        })
    }
}

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.4)
    Comment<'src>: Text<'src> => Comment
);

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.5)
    Description<'src>: Text<'src> => Description
);

/// Geographic position (RFC 5545 Section 3.8.1.6)
#[derive(Debug, Clone, Copy)]
pub struct Geo {
    /// Latitude
    pub lat: f64,

    /// Longitude
    pub lon: f64,
}

impl Geo {
    /// Get the property kind for `Geo`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Geo
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Geo {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let text = take_single_string(Self::kind(), prop.values).map_err(|e| vec![e])?;

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

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.7)
    Location<'src>: Text<'src> => Location
);

/// Event/To-do/Journal status (RFC 5545 Section 3.8.1.11)
///
/// This enum represents the status of calendar components such as events,
/// to-dos, and journal entries. Each variant corresponds to a specific status
/// defined in the iCalendar specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Event is tentative
    Tentative,

    /// Event is confirmed
    Confirmed,

    /// To-do needs action
    NeedsAction,

    /// To-do is completed
    Completed,

    /// To-do is in process
    InProcess,

    /// Journal entry is draft
    Draft,

    /// Journal entry is final
    Final,

    /// Event/To-do/Journal is cancelled
    Cancelled,
}

impl Status {
    /// Get the property kind for `Status`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Status
    }
}

impl FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_STATUS_TENTATIVE => Ok(Self::Tentative),
            KW_STATUS_CONFIRMED => Ok(Self::Confirmed),
            KW_STATUS_NEEDS_ACTION => Ok(Self::NeedsAction),
            KW_STATUS_COMPLETED => Ok(Self::Completed),
            KW_STATUS_IN_PROCESS => Ok(Self::InProcess),
            KW_STATUS_DRAFT => Ok(Self::Draft),
            KW_STATUS_FINAL => Ok(Self::Final),
            KW_STATUS_CANCELLED => Ok(Self::Cancelled),
            _ => Err(format!("Invalid status: {s}")),
        }
    }
}

impl AsRef<str> for Status {
    fn as_ref(&self) -> &str {
        match self {
            Self::Tentative => KW_STATUS_TENTATIVE,
            Self::Confirmed => KW_STATUS_CONFIRMED,
            Self::NeedsAction => KW_STATUS_NEEDS_ACTION,
            Self::Completed => KW_STATUS_COMPLETED,
            Self::InProcess => KW_STATUS_IN_PROCESS,
            Self::Draft => KW_STATUS_DRAFT,
            Self::Final => KW_STATUS_FINAL,
            Self::Cancelled => KW_STATUS_CANCELLED,
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Status {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let text = take_single_string(Self::kind(), prop.values).map_err(|e| vec![e])?;
        text.parse().map_err(|e| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Status,
                value: e,
                span: prop.span,
            }]
        })
    }
}

/// Percent Complete (RFC 5545 Section 3.8.1.8)
///
/// This property defines the percent complete for a todo.
/// Value must be between 0 and 100.
#[derive(Debug, Clone, Copy)]
pub struct PercentComplete {
    /// Percent complete (0-100)
    pub value: u8,
}

impl PercentComplete {
    /// Get the property kind for `PercentComplete`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::PercentComplete
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for PercentComplete {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        match take_single_value(Self::kind(), prop.values) {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok(Value::Integer(i)) if (0..=100).contains(&i) => Ok(Self { value: i as u8 }),
            Ok(Value::Integer(_)) => Err(vec![TypedError::PropertyInvalidValue {
                property: prop.kind,
                value: "Percent complete must be 0-100".to_string(),
                span: prop.span,
            }]),
            Ok(v) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::Integer,
                found: v.kind(),
                span: 0..0, // TODO: improve span reporting
            }]),
            Err(e) => Err(vec![e]),
        }
    }
}

/// Priority (RFC 5545 Section 3.8.1.9)
///
/// This property defines the priority for a calendar component.
/// Value must be between 0 and 9, where 0 defines an undefined priority.
#[derive(Debug, Clone, Copy)]
pub struct Priority {
    /// Priority value (0-9, where 0 is undefined)
    pub value: u8,
}

impl Priority {
    /// Get the property kind for `Priority`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Priority
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Priority {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        match take_single_value(Self::kind(), prop.values) {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok(Value::Integer(i)) if (0..=9).contains(&i) => Ok(Self { value: i as u8 }),
            Ok(Value::Integer(_)) => Err(vec![TypedError::PropertyInvalidValue {
                property: prop.kind,
                value: "Priority must be 0-9".to_string(),
                span: prop.span,
            }]),
            Ok(v) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::Integer,
                found: v.kind(),
                span: 0..0, // TODO: improve span reporting
            }]),
            Err(e) => Err(vec![e]),
        }
    }
}

simple_property_wrapper!(
    /// Multi-valued text property wrapper (RFC 5545 Section 3.8.1.10)
    Resources<'src>: Texts<'src> => Resources
);

simple_property_wrapper!(
    /// Multi-valued text property wrapper (RFC 5545 Section 3.8.1.2)
    Categories<'src>: Texts<'src> => Categories
);

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.12)
    Summary<'src>: Text<'src> => Summary
);
