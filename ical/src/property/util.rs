// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions and types for property parsing.
//!
//! This module provides utility functions and common types for extracting
//! and converting property values from typed properties to semantic types.

use std::convert::TryFrom;

use crate::lexer::Span;
use crate::parameter::{Parameter, ValueType};
use crate::property::PropertyKind;
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueText, Values};

/// Get the first value from a property, or return an error
///
/// # Errors
/// Returns `SemanticError::ConstraintViolation` if there are multiple values
pub fn take_single_value<'src>(
    kind: &PropertyKind<'src>,
    mut values: Values<'src>,
) -> Result<(Value<'src>, Span), Vec<TypedError<'src>>> {
    if values.len() > 1 {
        return Err(vec![TypedError::PropertyInvalidValueCount {
            property: kind.clone(),
            expected: 1,
            found: values.len(),
            span: values.span,
        }]);
    }

    match values.pop() {
        Some(value) => Ok((value, values.span)),
        None => Err(vec![TypedError::PropertyMissingValue {
            property: kind.clone(),
            span: values.span,
        }]),
    }
}

/// Get a single text value from a property
///
/// # Errors
/// Returns `TypedError::PropertyUnexpectedValue` if the value is not text
pub fn take_single_text<'src>(
    kind: &PropertyKind<'src>,
    values: Values<'src>,
) -> Result<ValueText<'src>, Vec<TypedError<'src>>> {
    match take_single_value(kind, values) {
        Ok((Value::Text(text), _)) => Ok(text),
        Ok((v, span)) => Err(vec![TypedError::PropertyUnexpectedValue {
            property: kind.clone(),
            expected: ValueType::Text,
            found: v.into_kind(),
            span,
        }]),
        Err(e) => Err(e),
    }
}

/// Get a single string value from a property
///
/// # Errors
/// Returns `TypedError::PropertyUnexpectedValue` if the value is not text
pub fn take_single_string<'src>(
    kind: &PropertyKind<'src>,
    values: Values<'src>,
) -> Result<String, Vec<TypedError<'src>>> {
    match take_single_value(kind, values) {
        Ok((Value::Text(v), _)) => Ok(v.resolve().to_string()), // TODO: avoid allocation
        Ok((v, span)) => Err(vec![TypedError::PropertyUnexpectedValue {
            property: kind.clone(),
            expected: ValueType::Text,
            found: v.into_kind(),
            span,
        }]),
        Err(e) => Err(e),
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

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Text<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let content = take_single_text(&prop.kind, prop.values)?;

        let mut errors = Vec::new();

        // Extract language, altrep, and unknown parameters
        let mut language = None;
        let mut altrep = None;
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::Language { .. } if language.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::Language { value, .. } => language = Some(value),

                p @ Parameter::AlternateText { .. } if altrep.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::AlternateText { value, .. } => altrep = Some(value),

                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
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
            x_parameters,
            unrecognized_parameters,
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

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Texts<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let mut language = None;
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::Language { value, .. } => language = Some(value),
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let values = prop
            .values
            .into_iter()
            .filter_map(|v| match v {
                Value::Text(content) => Some(Text {
                    content,
                    language: language.clone(),
                    altrep: None, // ALTREP not applicable to multi-valued text properties
                    x_parameters: Vec::new(),
                    unrecognized_parameters: Vec::new(),
                }),
                _ => None,
            })
            .collect();

        Ok(Self {
            values,
            x_parameters,
            unrecognized_parameters,
        })
    }
}

macro_rules! simple_property_wrapper {
    (
        $(#[$meta:meta])*
        $name:ident <'src> : $inner:ty => $kind:ident
        $(, derives = [$($derive:ident),* $(,)?])?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone $(, $($($derive),*)? )?)]
        pub struct $name<'src>(pub $inner);

        impl<'src> ::core::ops::Deref for $name<'src> {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::core::ops::DerefMut for $name<'_> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<'src> ::core::convert::TryFrom<crate::typed::ParsedProperty<'src>> for $name<'src>
        where
            $inner: ::core::convert::TryFrom<crate::typed::ParsedProperty<'src>, Error = Vec<crate::typed::TypedError<'src>>>,
        {
            type Error = Vec<crate::typed::TypedError<'src>>;

            fn try_from(prop: crate::typed::ParsedProperty<'src>) -> Result<Self, Self::Error> {
                if !matches!(prop.kind, crate::property::PropertyKind::$kind) {
                    return Err(vec![crate::typed::TypedError::PropertyUnexpectedKind {
                        expected: crate::property::PropertyKind::$kind,
                        found: prop.kind,
                        span: prop.span,
                    }]);
                }

                <$inner>::try_from(prop).map($name)
            }
        }
    };
}
