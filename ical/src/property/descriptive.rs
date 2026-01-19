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

use chumsky::input::{Input, Stream};
use chumsky::prelude::*;
use chumsky::{Parser, error::Rich, extra};

use crate::keyword::{
    KW_CLASS_CONFIDENTIAL, KW_CLASS_PRIVATE, KW_CLASS_PUBLIC, KW_STATUS_CANCELLED,
    KW_STATUS_COMPLETED, KW_STATUS_CONFIRMED, KW_STATUS_DRAFT, KW_STATUS_FINAL,
    KW_STATUS_IN_PROCESS, KW_STATUS_NEEDS_ACTION, KW_STATUS_TENTATIVE,
};
use crate::parameter::{Encoding, Parameter, ValueType};
use crate::property::PropertyKind;
use crate::property::common::{Text, take_single_text, take_single_value};
use crate::string_storage::{Segments, StringStorage};
use crate::syntax::RawParameter;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueText, values_float_semicolon};

/// Attachment information (RFC 5545 Section 3.8.1.1)
#[derive(Debug, Clone)]
pub struct Attachment<S: StringStorage> {
    /// URI or binary data
    pub value: AttachmentValue<S>,
    /// Format type (optional)
    pub fmt_type: Option<S>,
    /// Encoding (optional)
    pub encoding: Option<Encoding>,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

/// Attachment value (URI or binary)
#[derive(Debug, Clone)]
pub enum AttachmentValue<S: StringStorage> {
    /// URI reference
    Uri(S),
    /// Binary data
    Binary(S),
}

impl<'src> TryFrom<ParsedProperty<'src>> for Attachment<Segments<'src>> {
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
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::FormatType { .. } if fmt_type.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.kind().into(),
                    });
                }
                Parameter::FormatType { value, .. } => fmt_type = Some(value),

                p @ Parameter::Encoding { .. } if encoding.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.kind().into(),
                    });
                }
                Parameter::Encoding { value, .. } => encoding = Some(value),

                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        // Get value
        let value = match take_single_value(&PropertyKind::Attach, prop.value) {
            Ok(Value::Binary { value, .. }) => Some(AttachmentValue::Binary(value)),
            Ok(Value::Uri { value, .. }) => Some(AttachmentValue::Uri(value)),
            Ok(v) => {
                const EXPECTED: &[ValueType<String>] = &[ValueType::Uri, ValueType::Binary];
                errors.push(TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: EXPECTED,
                    found: v.kind().into(),
                    span: v.span(),
                });
                None
            }
            Err(e) => {
                errors.extend(e);
                None
            }
        };

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Attachment {
            value: value.unwrap(), // SAFETY: checked errors above
            fmt_type,
            encoding,
            x_parameters,
            retained_parameters,
            span: prop.span,
        })
    }
}

impl Attachment<Segments<'_>> {
    /// Convert borrowed `Attachment` to owned `Attachment`
    #[must_use]
    pub fn to_owned(&self) -> Attachment<String> {
        Attachment {
            value: self.value.to_owned(),
            fmt_type: self.fmt_type.as_ref().map(Segments::to_owned),
            encoding: self.encoding,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> Attachment<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

impl AttachmentValue<Segments<'_>> {
    /// Convert borrowed `AttachmentValue` to owned `AttachmentValue`
    #[must_use]
    pub fn to_owned(&self) -> AttachmentValue<String> {
        match self {
            AttachmentValue::Uri(uri) => AttachmentValue::Uri(uri.to_owned()),
            AttachmentValue::Binary(data) => AttachmentValue::Binary(data.to_owned()),
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
pub struct Classification<S: StringStorage> {
    /// Classification value
    pub value: ClassificationValue,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Classification<Segments<'src>> {
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
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
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
            retained_parameters,
            span: prop.span,
        })
    }
}

impl Classification<Segments<'_>> {
    /// Convert borrowed `Classification` to owned `Classification`
    #[must_use]
    pub fn to_owned(&self) -> Classification<String> {
        Classification {
            value: self.value,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> Classification<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.4)
    pub Comment<S> => Text
);

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.5)
    pub Description<S> => Text
);

/// Geographic position (RFC 5545 Section 3.8.1.6)
#[derive(Debug, Clone)]
pub struct Geo<S: StringStorage> {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Geo<Segments<'src>> {
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
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        let value_span = prop.value.span();
        let text = take_single_text(&PropertyKind::Geo, prop.value)?;

        // Use the typed phase's float parser with semicolon separator
        let stream = make_input(text.clone()); // TODO: avoid clone
        let parser = values_float_semicolon::<_, extra::Err<Rich<char, _>>>();

        match parser.parse(stream).into_result() {
            Ok(result) => {
                if result.len() != 2 {
                    return Err(vec![TypedError::PropertyInvalidValue {
                        property: PropertyKind::Geo,
                        value: format!(
                            "Expected exactly 2 float values (lat;long), got {}",
                            result.len()
                        ),
                        span: value_span,
                    }]);
                }

                Ok(Geo {
                    lat: result.first().copied().unwrap_or_default(),
                    lon: result.get(1).copied().unwrap_or_default(),
                    x_parameters,
                    retained_parameters,
                    span: prop.span,
                })
            }
            Err(_) => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Geo,
                value: format!("Expected 'lat;long' format with semicolon separator, got {text}"),
                span: value_span,
            }]),
        }
    }
}

impl Geo<Segments<'_>> {
    /// Convert borrowed `Geo` to owned `Geo`
    #[must_use]
    pub fn to_owned(&self) -> Geo<String> {
        Geo {
            lat: self.lat,
            lon: self.lon,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> Geo<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.7)
    pub Location<S> => Text
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
pub struct Status<S: StringStorage> {
    /// Status value
    pub value: StatusValue,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Status<Segments<'src>> {
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
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
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
            retained_parameters,
            span: prop.span,
        })
    }
}

impl Status<Segments<'_>> {
    /// Convert borrowed `Status` to owned `Status`
    #[must_use]
    pub fn to_owned(&self) -> Status<String> {
        Status {
            value: self.value,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> Status<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

impl Status<String> {
    /// Create a new `Status<String>` from a status value.
    #[must_use]
    pub fn new(value: StatusValue) -> Self {
        Self {
            value,
            x_parameters: Vec::new(),
            retained_parameters: Vec::new(),
            span: (),
        }
    }
}

/// Percent Complete (RFC 5545 Section 3.8.1.8)
///
/// This property defines the percent complete for a todo.
/// Value must be between 0 and 100.
#[derive(Debug, Clone)]
pub struct PercentComplete<S: StringStorage> {
    /// Percent complete (0-100)
    pub value: u8,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for PercentComplete<Segments<'src>> {
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
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        let value_span = prop.value.span();
        match take_single_value(&PropertyKind::PercentComplete, prop.value) {
            Ok(Value::Integer {
                values: mut ints, ..
            }) if ints.len() == 1 => {
                let i = ints.pop().unwrap();
                if (0..=100).contains(&i) {
                    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    Ok(Self {
                        value: i as u8,
                        x_parameters,
                        retained_parameters,
                        span: prop.span,
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
                const EXPECTED: &[ValueType<String>] = &[ValueType::Integer];
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: EXPECTED,
                    found: v.kind().into(),
                    span,
                }])
            }
            Err(e) => Err(e),
        }
    }
}

impl PercentComplete<Segments<'_>> {
    /// Convert borrowed `PercentComplete` to owned `PercentComplete`
    #[must_use]
    pub fn to_owned(&self) -> PercentComplete<String> {
        PercentComplete {
            value: self.value,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> PercentComplete<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

impl PercentComplete<String> {
    /// Create a new `PercentComplete<String>` from a percent value (0-100).
    #[must_use]
    pub fn new(value: u8) -> Self {
        Self {
            value,
            x_parameters: Vec::new(),
            retained_parameters: Vec::new(),
            span: (),
        }
    }
}

/// Priority (RFC 5545 Section 3.8.1.9)
///
/// This property defines the priority for a calendar component.
/// Value must be between 0 and 9, where 0 defines an undefined priority.
#[derive(Debug, Clone)]
pub struct Priority<S: StringStorage> {
    /// Priority value (0-9, where 0 is undefined)
    pub value: u8,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Priority<Segments<'src>> {
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
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
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
                        retained_parameters,
                        span: prop.span,
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
                const EXPECTED: &[ValueType<String>] = &[ValueType::Integer];
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: EXPECTED,
                    found: v.kind().into(),
                    span,
                }])
            }
            Err(e) => Err(e),
        }
    }
}

impl Priority<Segments<'_>> {
    /// Convert borrowed `Priority` to owned `Priority`
    #[must_use]
    pub fn to_owned(&self) -> Priority<String> {
        Priority {
            value: self.value,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> Priority<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

impl Priority<String> {
    /// Create a new `Priority<String>` from a priority value (0-9).
    #[must_use]
    pub fn new(value: u8) -> Self {
        Self {
            value,
            x_parameters: Vec::new(),
            retained_parameters: Vec::new(),
            span: (),
        }
    }
}

/// Categories property (RFC 5545 Section 3.8.1.2)
///
/// This property defines the categories for a calendar component.
///
/// Per RFC 5545, CATEGORIES supports the LANGUAGE parameter but NOT ALTREP.
#[derive(Debug, Clone)]
pub struct Categories<S: StringStorage> {
    /// List of category text values
    pub values: Vec<ValueText<S>>,
    /// Language code (optional, applied to all values)
    pub language: Option<S>,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Categories<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Categories) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Categories,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut language = None;
        let mut x_parameters = Vec::new();
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::Language { .. } if language.is_some() => {
                    return Err(vec![TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.kind().into(),
                    }]);
                }
                Parameter::Language { value, .. } => language = Some(value),

                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        let Value::Text { values, .. } = prop.value else {
            const EXPECTED: &[ValueType<String>] = &[ValueType::Text];
            let span = prop.value.span();
            return Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: EXPECTED,
                found: prop.value.kind().into(),
                span,
            }]);
        };

        Ok(Self {
            values,
            language,
            x_parameters,
            retained_parameters,
            span: prop.span,
        })
    }
}

impl Categories<Segments<'_>> {
    /// Convert borrowed `Categories` to owned `Categories`
    #[must_use]
    pub fn to_owned(&self) -> Categories<String> {
        Categories {
            values: self.values.iter().map(ValueText::to_owned).collect(),
            language: self.language.as_ref().map(Segments::to_owned),
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> Categories<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

/// Resources property (RFC 5545 Section 3.8.1.10)
///
/// This property defines the equipment or resources anticipated for an activity.
///
/// Per RFC 5545, RESOURCES supports both LANGUAGE and ALTREP parameters.
#[derive(Debug, Clone)]
pub struct Resources<S: StringStorage> {
    /// List of resource text values
    pub values: Vec<ValueText<S>>,
    /// Language code (optional, applied to all values)
    pub language: Option<S>,
    /// Alternate text representation URI (optional)
    pub altrep: Option<S>,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Resources<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Resources) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Resources,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut errors = Vec::new();
        let mut language = None;
        let mut altrep = None;
        let mut x_parameters = Vec::new();
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::Language { .. } if language.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.kind().into(),
                    });
                }
                Parameter::Language { value, .. } => language = Some(value),

                p @ Parameter::AlternateText { .. } if altrep.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.kind().into(),
                    });
                }
                Parameter::AlternateText { value, .. } => altrep = Some(value),

                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let Value::Text { values, .. } = prop.value else {
            const EXPECTED: &[ValueType<String>] = &[ValueType::Text];
            let span = prop.value.span();
            return Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: EXPECTED,
                found: prop.value.kind().into(),
                span,
            }]);
        };

        Ok(Self {
            values,
            language,
            altrep,
            x_parameters,
            retained_parameters,
            span: prop.span,
        })
    }
}

impl Resources<Segments<'_>> {
    /// Convert borrowed `Resources` to owned `Resources`
    #[must_use]
    pub fn to_owned(&self) -> Resources<String> {
        Resources {
            values: self.values.iter().map(ValueText::to_owned).collect(),
            language: self.language.as_ref().map(Segments::to_owned),
            altrep: self.altrep.as_ref().map(Segments::to_owned),
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> Resources<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.12)
    pub Summary<S> => Text
);

/// Create a parser input from `ValueText` with proper span tracking.
///
/// This function creates a properly-spanned input stream from a `ValueText`,
/// enabling accurate error reporting during parsing.
fn make_input(text: ValueText<Segments<'_>>) -> impl Input<'_, Token = char, Span = SimpleSpan> {
    // Get EOI span directly from the ValueText without iteration
    let eoi = text.span().into();

    // Create the parser stream
    Stream::from_iter(text.into_spanned_chars().map(|(c, s)| (c, s.into())))
        .map(eoi, |(c, s)| (c, s))
}
