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
use std::fmt::{self, Display};

use chumsky::{Parser, error::Rich, extra, input::Stream};

use crate::keyword::{
    KW_CLASS_CONFIDENTIAL, KW_CLASS_PRIVATE, KW_CLASS_PUBLIC, KW_STATUS_CANCELLED,
    KW_STATUS_COMPLETED, KW_STATUS_CONFIRMED, KW_STATUS_DRAFT, KW_STATUS_FINAL,
    KW_STATUS_IN_PROCESS, KW_STATUS_NEEDS_ACTION, KW_STATUS_TENTATIVE,
};
use crate::parameter::{Encoding, Parameter, ValueType};
use crate::property::PropertyKind;
use crate::property::common::{Text, take_single_text, take_single_value};
use crate::string_storage::{SpannedSegments, StringStorage};
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

/// Type alias for borrowed attachment value
pub type AttachmentValueRef<'src> = AttachmentValue<SpannedSegments<'src>>;
/// Type alias for owned attachment value
pub type AttachmentValueOwned = AttachmentValue<String>;

impl<'src> TryFrom<ParsedProperty<'src>> for Attachment<SpannedSegments<'src>> {
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
                let span = v.span();
                errors.push(TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueType::Uri, // TODO: include Binary as well
                    found: v.kind().into(),
                    span,
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

impl Attachment<SpannedSegments<'_>> {
    /// Convert borrowed `Attachment` to owned `Attachment`
    #[must_use]
    pub fn to_owned(&self) -> Attachment<String> {
        Attachment {
            value: self.value.to_owned(),
            fmt_type: self.fmt_type.as_ref().map(SpannedSegments::to_owned),
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

impl AttachmentValue<SpannedSegments<'_>> {
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

impl<S: StringStorage> Display for Classification<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Classification<SpannedSegments<'src>> {
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

impl Classification<SpannedSegments<'_>> {
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

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.4)
    pub Comment<S> => Text

    ref   = pub type CommentRef;
    owned = pub type CommentOwned;
);

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.5)
    pub Description<S> => Text

    ref   = pub type DescriptionRef;
    owned = pub type DescriptionOwned;
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

impl<'src> TryFrom<ParsedProperty<'src>> for Geo<SpannedSegments<'src>> {
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
        let text = take_single_text(&PropertyKind::Geo, prop.value)?.to_string();

        // Use the typed phase's float parser with semicolon separator
        let stream = Stream::from_iter(text.chars()); // TODO: fix span
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

impl Geo<SpannedSegments<'_>> {
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

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.7)
    pub Location<S> => Text

    ref   = pub type LocationRef;
    owned = pub type LocationOwned;
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

impl<'src> TryFrom<ParsedProperty<'src>> for Status<SpannedSegments<'src>> {
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

impl Status<SpannedSegments<'_>> {
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

impl<'src> TryFrom<ParsedProperty<'src>> for PercentComplete<SpannedSegments<'src>> {
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
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok(Value::Integer {
                values: mut ints, ..
            }) if ints.len() == 1 => {
                let i = ints.pop().unwrap();
                if (0..=100).contains(&i) {
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
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueType::Integer,
                    found: v.kind().into(),
                    span,
                }])
            }
            Err(e) => Err(e),
        }
    }
}

impl PercentComplete<SpannedSegments<'_>> {
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

impl<'src> TryFrom<ParsedProperty<'src>> for Priority<SpannedSegments<'src>> {
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
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueType::Integer,
                    found: v.kind().into(),
                    span,
                }])
            }
            Err(e) => Err(e),
        }
    }
}

impl Priority<SpannedSegments<'_>> {
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

/// Borrowed type alias for [`Categories`]
pub type CategoriesRef<'src> = Categories<SpannedSegments<'src>>;

/// Owned type alias for [`Categories`]
pub type CategoriesOwned = Categories<String>;

impl<'src> TryFrom<ParsedProperty<'src>> for Categories<SpannedSegments<'src>> {
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
            let span = prop.value.span();
            return Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueType::Text,
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

impl Categories<SpannedSegments<'_>> {
    /// Convert borrowed `Categories` to owned `Categories`
    #[must_use]
    pub fn to_owned(&self) -> CategoriesOwned {
        Categories {
            values: self.values.iter().map(ValueText::to_owned).collect(),
            language: self.language.as_ref().map(SpannedSegments::to_owned),
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

/// Borrowed type alias for [`Resources`]
pub type ResourcesRef<'src> = Resources<SpannedSegments<'src>>;

/// Owned type alias for [`Resources`]
pub type ResourcesOwned = Resources<String>;

impl<'src> TryFrom<ParsedProperty<'src>> for Resources<SpannedSegments<'src>> {
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
            let span = prop.value.span();
            return Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueType::Text,
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

impl Resources<SpannedSegments<'_>> {
    /// Convert borrowed `Resources` to owned `Resources`
    #[must_use]
    pub fn to_owned(&self) -> ResourcesOwned {
        Resources {
            values: self.values.iter().map(ValueText::to_owned).collect(),
            language: self.language.as_ref().map(SpannedSegments::to_owned),
            altrep: self.altrep.as_ref().map(SpannedSegments::to_owned),
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

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.1.12)
    pub Summary<S> => Text

    ref   = pub type SummaryRef;
    owned = pub type SummaryOwned;
);
