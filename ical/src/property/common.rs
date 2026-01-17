// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions and types for property parsing.
//!
//! This module provides utility functions and common types for extracting
//! and converting property values from typed properties to semantic types.

use std::convert::TryFrom;

use crate::parameter::{Parameter, ValueType};
use crate::property::PropertyKind;
use crate::string_storage::{Segments, StringStorage};
use crate::syntax::RawParameter;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueText};

/// Get the first value from a property, ensuring it has exactly one value
pub fn take_single_value<'src>(
    kind: &PropertyKind<Segments<'src>>,
    value: Value<Segments<'src>>,
) -> Result<Value<Segments<'src>>, Vec<TypedError<'src>>> {
    if !value.len() == 1 {
        return Err(vec![TypedError::PropertyInvalidValueCount {
            property: kind.clone(),
            expected: 1,
            found: value.len(),
            span: value.span(),
        }]);
    }

    Ok(value)
}

/// Get a single calendar user address value from a property
pub fn take_single_cal_address<'src>(
    kind: &PropertyKind<Segments<'src>>,
    value: Value<Segments<'src>>,
) -> Result<Segments<'src>, Vec<TypedError<'src>>> {
    const EXPECTED: &[ValueType<String>] = &[ValueType::CalendarUserAddress];
    let value = take_single_value(kind, value)?;
    match value {
        Value::CalAddress { value, .. } => Ok(value),
        v => Err(vec![TypedError::PropertyUnexpectedValue {
            property: kind.clone(),
            expected: EXPECTED,
            found: v.kind().into(),
            span: v.span(),
        }]),
    }
}

/// Get a single URI value from a property
pub fn take_single_uri<'src>(
    kind: &PropertyKind<Segments<'src>>,
    value: Value<Segments<'src>>,
) -> Result<Segments<'src>, Vec<TypedError<'src>>> {
    const EXPECTED: &[ValueType<String>] = &[ValueType::Uri];
    let value = take_single_value(kind, value)?;
    match value {
        Value::Uri { value, .. } => Ok(value),
        v => Err(vec![TypedError::PropertyUnexpectedValue {
            property: kind.clone(),
            expected: EXPECTED,
            found: v.kind().into(),
            span: v.span(),
        }]),
    }
}

/// Get a single text value from a property
pub fn take_single_text<'src>(
    kind: &PropertyKind<Segments<'src>>,
    value: Value<Segments<'src>>,
) -> Result<ValueText<Segments<'src>>, Vec<TypedError<'src>>> {
    const EXPECTED: &[ValueType<String>] = &[ValueType::Text];
    let value = take_single_value(kind, value)?;

    match value {
        Value::Text { mut values, .. } if values.len() == 1 => Ok(values.pop().unwrap()),
        Value::Text { ref values, .. } => {
            let span = value.span();
            Err(vec![TypedError::PropertyInvalidValueCount {
                property: kind.clone(),
                expected: 1,
                found: values.len(),
                span,
            }])
        }
        v => {
            let span = v.span();
            Err(vec![TypedError::PropertyUnexpectedValue {
                property: kind.clone(),
                expected: EXPECTED,
                found: v.kind().into(),
                span,
            }])
        }
    }
}

/// URI property with parameters
///
/// This type is used by properties that have a URI value type, such as:
/// - 3.8.3.5: `TzUrl` - Time zone URL
/// - 3.8.4.6: `Url` - Uniform Resource Locator
#[derive(Debug, Clone)]
pub struct UriProperty<S: StringStorage> {
    /// The URI value
    pub uri: S,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for UriProperty<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
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

        let uri = take_single_uri(&prop.kind, prop.value)?;

        Ok(UriProperty {
            uri,
            x_parameters,
            retained_parameters,
        })
    }
}

impl UriProperty<Segments<'_>> {
    /// Convert borrowed `UriProperty` to owned `UriProperty`
    #[must_use]
    pub fn to_owned(&self) -> UriProperty<String> {
        UriProperty {
            uri: self.uri.to_owned(),
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
        }
    }
}

/// Plain text property without standard parameters
///
/// This is a helper type used by text properties that do NOT support any
/// standard parameters (LANGUAGE, ALTREP, etc.):
/// - 3.8.3.1: `TzId` - Time zone identifier
/// - 3.8.4.7: `Uid` - Unique identifier
///
/// All standard parameters are preserved in `retained_parameters` for
/// round-trip compatibility.
#[derive(Debug, Clone)]
pub struct TextOnly<S: StringStorage> {
    /// The actual text content
    pub content: ValueText<S>,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for TextOnly<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let content = take_single_text(&prop.kind, prop.value)?;

        let mut x_parameters = Vec::new();
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // ALL standard parameters go to retained_parameters for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        Ok(Self {
            content,
            x_parameters,
            retained_parameters,
        })
    }
}

impl TextOnly<Segments<'_>> {
    /// Convert borrowed `TextOnly` to owned `TextOnly`
    #[must_use]
    pub fn to_owned(&self) -> TextOnly<String> {
        TextOnly {
            content: self.content.to_owned(),
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
        }
    }
}

/// Text with language parameter only
///
/// This is a helper type used by text properties that support ONLY the LANGUAGE
/// parameter (not ALTREP):
/// - 3.8.3.2: `TzName` - Time zone name
/// - 3.8.8.3: `RequestStatus` - Request status
///
/// ALTREP and other standard parameters are preserved in `retained_parameters`
/// for round-trip compatibility.
#[derive(Debug, Clone)]
pub struct TextWithLanguage<S: StringStorage> {
    /// The actual text content
    pub content: ValueText<S>,

    /// Language code (optional)
    pub language: Option<S>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,

    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for TextWithLanguage<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let content = take_single_text(&prop.kind, prop.value)?;

        let mut errors = Vec::new();
        let mut language = None;
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

                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            content,
            language,
            x_parameters,
            retained_parameters,
        })
    }
}

impl TextWithLanguage<Segments<'_>> {
    /// Convert borrowed `TextWithLanguage` to owned `TextWithLanguage`
    #[must_use]
    pub fn to_owned(&self) -> TextWithLanguage<String> {
        TextWithLanguage {
            content: self.content.to_owned(),
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
        }
    }
}

/// Text with language and alternate representation information
///
/// This is a helper type used by text properties that support both LANGUAGE
/// and ALTREP parameters:
/// - 3.8.1.4: `Comment`
/// - 3.8.1.5: `Description`
/// - 3.8.1.7: `Location`
/// - 3.8.1.10: `Resources` (multi-valued)
/// - 3.8.1.12: `Summary`
/// - 3.8.4.2: `Contact`
#[derive(Debug, Clone)]
pub struct Text<S: StringStorage> {
    /// The actual text content
    pub content: ValueText<S>,

    /// Language code (optional)
    pub language: Option<S>,

    /// Alternate text representation URI (optional)
    ///
    /// Per RFC 5545 Section 3.2.1, this parameter specifies a URI that points
    /// to an alternate representation for the textual property value.
    pub altrep: Option<S>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,

    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Text<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let content = take_single_text(&prop.kind, prop.value)?;

        let mut errors = Vec::new();

        // Extract language, altrep, and unknown parameters
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

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            content,
            language,
            altrep,
            x_parameters,
            retained_parameters,
        })
    }
}

impl Text<Segments<'_>> {
    /// Convert borrowed Text to owned Text
    #[must_use]
    pub fn to_owned(&self) -> Text<String> {
        Text {
            content: self.content.to_owned(),
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
        }
    }
}

impl Text<String> {
    /// Create a new `Text<String>` from a string value.
    ///
    /// This constructor is provided for convenient construction of owned text properties.
    /// The input string is treated as an unescaped text value with no parameters.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self {
            content: ValueText::new(value),
            language: None,
            altrep: None,
            x_parameters: Vec::new(),
            retained_parameters: Vec::new(),
        }
    }
}

impl TextOnly<String> {
    /// Create a new `TextOnly<String>` from a string value.
    ///
    /// This constructor is provided for convenient construction of owned text-only properties.
    /// The input string is treated as an unescaped text value with no parameters.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self {
            content: ValueText::new(value),
            x_parameters: Vec::new(),
            retained_parameters: Vec::new(),
        }
    }
}

/// Macro to define simple property wrappers with generic storage parameter.
///
/// This is similar to `simple_property_wrapper!` but generates generic wrappers
/// that accept a storage parameter `S: StringStorage` instead of hardcoding
/// the lifetime `'src`.
///
/// Usage:
///
/// ```ignore
/// simple_property_wrapper!(
///     Comment<S>: Text => Comment
/// );
/// ```
///
/// This generates:
///
/// ```ignore
/// pub struct Comment<S: StringStorage>(pub Text<S>);
/// ```
macro_rules! simple_property_wrapper {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident <S> => $inner:ident
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        $vis struct $name<S: StringStorage> {
            /// Inner property value
            pub inner: $inner<S>,
            /// Span of the property in the source
            pub span: S::Span,
        }

        impl<S> ::core::ops::Deref for $name<S>
        where
            S: StringStorage,
        {
            type Target = $inner<S>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl<S> ::core::ops::DerefMut for $name<S>
        where
            S: StringStorage,
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }

        impl<'src> ::core::convert::TryFrom<crate::typed::ParsedProperty<'src>> for $name<crate::string_storage::Segments<'src>>
        where
            $inner<crate::string_storage::Segments<'src>>: ::core::convert::TryFrom<crate::typed::ParsedProperty<'src>, Error = Vec<crate::typed::TypedError<'src>>>,
        {
            type Error = Vec<crate::typed::TypedError<'src>>;

            fn try_from(prop: crate::typed::ParsedProperty<'src>) -> Result<Self, Self::Error> {
                if !matches!(prop.kind, crate::property::PropertyKind::$name) {
                    return Err(vec![crate::typed::TypedError::PropertyUnexpectedKind {
                        expected: crate::property::PropertyKind::$name,
                        found: prop.kind,
                        span: prop.span,
                    }]);
                }

                let span = prop.span;
                <$inner<crate::string_storage::Segments<'src>>>::try_from(prop).map(|inner| $name { inner, span })
            }
        }

        impl $name<crate::string_storage::Segments<'_>> {
            /// Convert borrowed type to owned type
            #[must_use]
            pub fn to_owned(&self) -> $name<String> {
                $name {
                    inner: self.inner.to_owned(),
                    span: (),
                }
            }
        }
    };
}

/// Macro to define simple enums for property values.
///
/// This generates simple enums with Copy semantics for RFC 5545 parameter values
/// that don't support extensions.
macro_rules! define_prop_value_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident {
            $(
                $(#[$vmeta:meta])*
                $Variant:ident => $kw:ident
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[allow(missing_docs)]
        $vis enum $Name {
            $(
                $(#[$vmeta])*
                $Variant,
            )*
        }


        impl<'src> TryFrom<crate::value::ValueText<Segments<'src>>> for $Name {
            type Error = crate::value::ValueText<Segments<'src>>;

            fn try_from(segs: crate::value::ValueText<Segments<'src>>) -> Result<Self, Self::Error> {
                $(
                    if segs.eq_str_ignore_ascii_case($kw) {
                        return Ok(Self::$Variant);
                    }
                )*
                Err(segs)
            }
        }

        impl ::core::fmt::Display for $Name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    $(
                        Self::$Variant => $kw.fmt(f),
                    )*
                }
            }
        }
    };
}
