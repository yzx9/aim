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

use chumsky::{Parser, error::Rich, extra, input::Stream};

use crate::keyword::{
    KW_CLASS_CONFIDENTIAL, KW_CLASS_PRIVATE, KW_CLASS_PUBLIC, KW_STATUS_CANCELLED,
    KW_STATUS_COMPLETED, KW_STATUS_CONFIRMED, KW_STATUS_DRAFT, KW_STATUS_FINAL,
    KW_STATUS_IN_PROCESS, KW_STATUS_NEEDS_ACTION, KW_STATUS_TENTATIVE,
};
use crate::parameter::{Encoding, Parameter, ValueType};
use crate::property::PropertyKind;
use crate::property::util::{Text, Texts, take_single_text, take_single_value};
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

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
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
        if !matches!(prop.kind, PropertyKind::Attach) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Attach,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut errors = Vec::new();

        // Collect all optional parameters in a single pass
        let mut fmt_type = None;
        let mut encoding = None;
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::FormatType { .. } if fmt_type.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::FormatType { value, .. } => fmt_type = Some(value),

                p @ Parameter::Encoding { .. } if encoding.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::Encoding { value, .. } => encoding = Some(value),

                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        // Get value
        let value = match take_single_value(&PropertyKind::Attach, prop.value) {
            Ok(v) => v,
            Err(e) => {
                errors.extend(e);
                return Err(errors);
            }
        };

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        match value {
            Value::Text {
                values: mut uris, ..
            } if uris.len() == 1 => Ok(Attachment {
                value: AttachmentValue::Uri(uris.pop().unwrap()),
                fmt_type,
                encoding,
                x_parameters,
                unrecognized_parameters,
            }),
            Value::Binary { raw: data, .. } => Ok(Attachment {
                value: AttachmentValue::Binary(data.clone()),
                fmt_type,
                encoding,
                x_parameters,
                unrecognized_parameters,
            }),
            _ => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Attach,
                value: "Expected URI or binary value".to_string(),
                span: value.span(),
            }]),
        }
    }
}

define_prop_value_enum! {
    /// Classification value (RFC 5545 Section 3.8.1.3)
    #[derive(Default)]
    pub enum ClassificationValue {
        /// Public classification
        #[default]
        Public => KW_CLASS_PUBLIC,

        /// Private classification
        Private => KW_CLASS_PRIVATE,

        /// Confidential classification
        Confidential => KW_CLASS_CONFIDENTIAL,
    }
}

/// Classification of calendar data (RFC 5545 Section 3.8.1.3)
#[derive(Debug, Clone)]
pub struct Classification<'src> {
    /// Classification value
    pub value: ClassificationValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl fmt::Display for Classification<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Classification<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Class) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Class,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let value_span = prop.value.span();
        let text = take_single_text(&PropertyKind::Class, prop.value)?;
        let value = text.try_into().map_err(|text| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Class,
                value: format!("Invalid classification: {text}"),
                span: value_span,
            }]
        })?;

        Ok(Self {
            value,
            x_parameters,
            unrecognized_parameters,
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
#[derive(Debug, Clone)]
pub struct Geo<'src> {
    /// Latitude
    pub lat: f64,

    /// Longitude
    pub lon: f64,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Geo<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Geo) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Geo,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let value_span = prop.value.span();
        let text = take_single_text(&PropertyKind::Geo, prop.value)?
            .resolve()
            .to_string();

        // Use the typed phase's float parser with semicolon separator
        let stream = Stream::from_iter(text.chars());
        let parser = values_float_semicolon::<_, extra::Err<Rich<char, _>>>();

        match parser.parse(stream).into_result() {
            Ok(result) => match (result.first(), result.get(1)) {
                (Some(&lat), Some(&lon)) => Ok(Geo {
                    lat,
                    lon,
                    x_parameters,
                    unrecognized_parameters,
                }),
                (_, _) => Err(vec![TypedError::PropertyInvalidValue {
                    property: PropertyKind::Geo,
                    value: format!(
                        "Expected exactly 2 float values (lat;long), got {}",
                        result.len()
                    ),
                    span: value_span,
                }]),
            },
            Err(_) => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Geo,
                value: format!("Expected 'lat;long' format with semicolon separator, got {text}"),
                span: value_span,
            }]),
        }
    }
}

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.7)
    Location<'src>: Text<'src> => Location
);

define_prop_value_enum! {
    /// Status value (RFC 5545 Section 3.8.1.11)
    ///
    /// This enum represents the status of calendar components such as events,
    /// to-dos, and journal entries. Each variant corresponds to a specific status
    /// defined in the iCalendar specification.
    pub enum StatusValue {
        /// Event is tentative
        Tentative => KW_STATUS_TENTATIVE,

        /// Event is confirmed
        Confirmed => KW_STATUS_CONFIRMED,

        /// To-do needs action
        NeedsAction => KW_STATUS_NEEDS_ACTION,

        /// To-do is completed
        Completed => KW_STATUS_COMPLETED,

        /// To-do is in process
        InProcess => KW_STATUS_IN_PROCESS,

        /// Journal entry is draft
        Draft => KW_STATUS_DRAFT,

        /// Journal entry is final
        Final => KW_STATUS_FINAL,

        /// Event/To-do/Journal is cancelled
        Cancelled => KW_STATUS_CANCELLED,
    }
}

/// Event/To-do/Journal status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone)]
pub struct Status<'src> {
    /// Status value
    pub value: StatusValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl fmt::Display for Status<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Status<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Status) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Status,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let value_span = prop.value.span();
        let text = take_single_text(&PropertyKind::Status, prop.value)?;
        let value = text.try_into().map_err(|text| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Status,
                value: format!("Invalid status: {text}"),
                span: value_span,
            }]
        })?;

        Ok(Self {
            value,
            x_parameters,
            unrecognized_parameters,
        })
    }
}

/// Percent Complete (RFC 5545 Section 3.8.1.8)
///
/// This property defines the percent complete for a todo.
/// Value must be between 0 and 100.
#[derive(Debug, Clone)]
pub struct PercentComplete<'src> {
    /// Percent complete (0-100)
    pub value: u8,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for PercentComplete<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::PercentComplete) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::PercentComplete,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let value_span = prop.value.span();
        match take_single_value(&PropertyKind::PercentComplete, prop.value) {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok(Value::Integer {
                values: mut ints, ..
            }) if ints.len() == 1 => {
                let i = ints.pop().unwrap();
                if (0..=100).contains(&i) {
                    Ok(Self {
                        value: i as u8,
                        x_parameters,
                        unrecognized_parameters,
                    })
                } else {
                    Err(vec![TypedError::PropertyInvalidValue {
                        property: prop.kind,
                        value: "Percent complete must be 0-100".to_string(),
                        span: value_span,
                    }])
                }
            }
            Ok(Value::Integer { .. }) => Err(vec![TypedError::PropertyInvalidValue {
                property: prop.kind,
                value: "Percent complete must be 0-100".to_string(),
                span: value_span,
            }]),
            Ok(v) => {
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueType::Integer,
                    found: v.into_kind(),
                    span,
                }])
            }
            Err(e) => Err(e),
        }
    }
}

/// Priority (RFC 5545 Section 3.8.1.9)
///
/// This property defines the priority for a calendar component.
/// Value must be between 0 and 9, where 0 defines an undefined priority.
#[derive(Debug, Clone)]
pub struct Priority<'src> {
    /// Priority value (0-9, where 0 is undefined)
    pub value: u8,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Priority<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Priority) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Priority,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let value_span = prop.value.span();
        match take_single_value(&PropertyKind::Priority, prop.value) {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok(Value::Integer {
                values: mut ints, ..
            }) if ints.len() == 1 => {
                let i = ints.pop().unwrap();
                if (0..=9).contains(&i) {
                    Ok(Self {
                        value: i as u8,
                        x_parameters,
                        unrecognized_parameters,
                    })
                } else {
                    Err(vec![TypedError::PropertyInvalidValue {
                        property: prop.kind,
                        value: "Priority must be 0-9".to_string(),
                        span: value_span,
                    }])
                }
            }
            Ok(Value::Integer { .. }) => Err(vec![TypedError::PropertyInvalidValue {
                property: prop.kind,
                value: "Priority must be 0-9".to_string(),
                span: value_span,
            }]),
            Ok(v) => {
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueType::Integer,
                    found: v.into_kind(),
                    span,
                }])
            }
            Err(e) => Err(e),
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
