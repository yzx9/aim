// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions and types for property parsing.
//!
//! This module provides utility functions and common types for extracting
//! and converting property values from typed properties to semantic types.

use std::convert::TryFrom;

use crate::lexer::Span;
use crate::parameter::{Parameter, ValueKind};
use crate::property::PropertyKind;
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueText, Values};

/// Get the first value from a property, or return an error
///
/// # Errors
/// Returns `SemanticError::ConstraintViolation` if there are multiple values
pub fn take_single_value(
    kind: PropertyKind,
    mut values: Values<'_>,
) -> Result<(Value<'_>, Span), TypedError<'_>> {
    let len = values.len();
    if len > 1 {
        return Err(TypedError::PropertyInvalidValueCount {
            property: kind,
            expected: 1,
            found: len,
            span: values.span,
        });
    }

    match values.pop() {
        Some(value) => Ok((value, values.span)),
        None => Err(TypedError::PropertyMissingValue {
            property: kind,
            span: values.span,
        }),
    }
}

/// Get a single text value from a property
///
/// # Errors
/// Returns `SemanticError::UnexpectedType` if the value is not text
pub fn take_single_text(
    kind: PropertyKind,
    values: Values<'_>,
) -> Result<ValueText<'_>, TypedError<'_>> {
    match take_single_value(kind, values) {
        Ok((Value::Text(text), _)) => Ok(text),
        Ok((v, span)) => Err(TypedError::PropertyUnexpectedValue {
            property: kind,
            expected: ValueKind::Text,
            found: v.kind(),
            span,
        }),
        Err(e) => Err(e),
    }
}

/// Get a single string value from a property
///
/// # Errors
/// Returns `SemanticError::UnexpectedType` if the value is not text
pub fn take_single_string(
    kind: PropertyKind,
    values: Values<'_>,
) -> Result<String, TypedError<'_>> {
    match take_single_value(kind, values) {
        Ok((Value::Text(v), _)) => Ok(v.resolve().to_string()), // TODO: avoid allocation
        Ok((v, span)) => Err(TypedError::PropertyUnexpectedValue {
            property: kind,
            expected: ValueKind::Text,
            found: v.kind(),
            span,
        }),
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

        impl $name<'_> {
            /// Get the property kind for this type
            #[must_use]
            pub const fn kind() -> crate::property::PropertyKind {
                crate::property::PropertyKind::$kind
            }
        }

        impl<'src> ::core::convert::TryFrom<crate::typed::ParsedProperty<'src>> for $name<'src>
        where
            $inner: ::core::convert::TryFrom<crate::typed::ParsedProperty<'src>, Error = Vec<crate::typed::TypedError<'src>>>,
        {
            type Error = Vec<crate::typed::TypedError<'src>>;

            fn try_from(prop: crate::typed::ParsedProperty<'src>) -> Result<Self, Self::Error> {
                if prop.kind != Self::kind() {
                    return Err(vec![crate::typed::TypedError::PropertyUnexpectedKind {
                        expected: Self::kind(),
                        found: prop.kind,
                        span: prop.span,
                    }]);
                }

                <$inner>::try_from(prop).map($name)
            }
        }
    };
}
