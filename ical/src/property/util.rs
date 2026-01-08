// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions and types for property parsing.
//!
//! This module provides utility functions and common types for extracting
//! and converting property values from typed properties to semantic types.

use std::convert::TryFrom;
use std::fmt::Display;

use crate::parameter::{Parameter, ValueTypeRef};
use crate::property::PropertyKindRef;
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueRef, ValueText, ValueTextRef};

/// Get the first value from a property, ensuring it has exactly one value
///
/// # Errors
/// Returns `TypedError::PropertyInvalidValueCount` if there are multiple or zero values
pub fn take_single_value<'src>(
    kind: &PropertyKindRef<'src>,
    value: ValueRef<'src>,
) -> Result<ValueRef<'src>, Vec<TypedError<'src>>> {
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

/// Get a single text value from a property
///
/// # Errors
/// Returns `TypedError::PropertyUnexpectedValue` if the value is not text
/// Returns `TypedError::PropertyInvalidValueCount` if there are multiple or zero values
pub fn take_single_text<'src>(
    kind: &PropertyKindRef<'src>,
    value: ValueRef<'src>,
) -> Result<ValueTextRef<'src>, Vec<TypedError<'src>>> {
    let value = take_single_value(kind, value)?;

    match value {
        Value::Text { mut values, .. } if values.len() == 1 => Ok(values.pop().unwrap()),
        v => {
            let span = v.span();
            Err(vec![TypedError::PropertyUnexpectedValue {
                property: kind.clone(),
                expected: ValueTypeRef::Text,
                found: v.into_kind(),
                span,
            }])
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
pub struct Text<S: Clone + Display> {
    /// The actual text content
    pub content: ValueText<S>,

    /// Language code (optional)
    pub language: Option<S>,

    /// Alternate text representation URI (optional)
    ///
    /// Per RFC 5545, this parameter is not applicable to TZNAME and CATEGORIES
    /// properties, but may be present in other text properties like DESCRIPTION,
    /// SUMMARY, LOCATION, CONTACT, and RESOURCES.
    pub altrep: Option<S>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Text<SpannedSegments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let content = take_single_text(&prop.kind, prop.value)?;

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
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
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
pub struct Texts<S: Clone + Display> {
    /// List of text values
    pub values: Vec<ValueText<S>>,

    /// Language code (optional, applied to all values)
    pub language: Option<S>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Texts<SpannedSegments<'src>> {
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
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let Value::Text { values, .. } = prop.value else {
            let span = prop.value.span();
            return Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueTypeRef::Text,
                found: prop.value.into_kind(),
                span,
            }]);
        };

        Ok(Self {
            values,
            language,
            x_parameters,
            unrecognized_parameters,
        })
    }
}

/// Macro to define simple property wrappers with generic storage parameter.
///
/// This is similar to `simple_property_wrapper!` but generates generic wrappers
/// that accept a storage parameter `S: Clone + Display` instead of hardcoding
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
/// pub struct Comment<S: Clone + Display>(pub Text<S>);
/// ```
macro_rules! simple_property_wrapper {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident <S> => $inner:ident

        $(#[$rmeta:meta])*
        ref   = $rvis:vis type $name_ref:ident;
        $(#[$ometa:meta])*
        owned = $ovis:vis type $name_owned:ident;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        $vis struct $name<S: Clone + ::core::fmt::Display>(pub $inner<S>);

        #[doc = concat!("Borrowed type alias for [`", stringify!($name), "`]")]
        $(#[$rmeta])*
        $rvis type $name_ref<'src> = $name<crate::syntax::SpannedSegments<'src>>;
        #[doc = concat!("Owned type alias for [`", stringify!($name), "`]")]
        $(#[$ometa])*
        $ovis type $name_owned = $name<String>;

        impl<S: ::core::clone::Clone + ::core::fmt::Display> ::core::ops::Deref for $name<S> {
            type Target = $inner<S>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<S: ::core::clone::Clone + ::core::fmt::Display> ::core::ops::DerefMut for $name<S> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<'src> ::core::convert::TryFrom<crate::typed::ParsedProperty<'src>> for $name_ref<'src>
        where
            $inner<crate::syntax::SpannedSegments<'src>>: ::core::convert::TryFrom<crate::typed::ParsedProperty<'src>, Error = Vec<crate::typed::TypedError<'src>>>,
        {
            type Error = Vec<crate::typed::TypedError<'src>>;

            fn try_from(prop: crate::typed::ParsedProperty<'src>) -> Result<Self, Self::Error> {
                if !matches!(prop.kind, crate::property::PropertyKindRef::$name) {
                    return Err(vec![crate::typed::TypedError::PropertyUnexpectedKind {
                        expected: crate::property::PropertyKindRef::$name,
                        found: prop.kind,
                        span: prop.span,
                    }]);
                }

                <$inner<crate::syntax::SpannedSegments<'src>>>::try_from(prop).map($name)
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


        impl<'src> TryFrom<crate::value::ValueTextRef<'src>> for $Name {
            type Error = crate::value::ValueTextRef<'src>;

            fn try_from(segs: crate::value::ValueTextRef<'src>) -> Result<Self, Self::Error> {
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
