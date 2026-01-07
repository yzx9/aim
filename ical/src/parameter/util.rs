// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::parameter::{Parameter, ParameterKind};
use crate::syntax::{SpannedSegments, SyntaxParameter, SyntaxParameterValue};
use crate::typed::TypedError;

pub type ParseResult<'src> = Result<Parameter<'src>, Vec<TypedError<'src>>>;

/// Parse a single value from a parameter.
///
/// # Errors
///
/// Returns an error if the parameter does not have exactly one value.
///
/// # Panics
///
/// Panics if the parameter has exactly one value but `Vec::pop()` returns `None`.
/// This should never happen in practice as the length check ensures there is
/// exactly one value.
pub fn parse_single<'src>(
    param: &mut SyntaxParameter<'src>,
    kind: ParameterKind<'src>,
) -> Result<SyntaxParameterValue<'src>, Vec<TypedError<'src>>> {
    match param.values.len() {
        1 => Ok(param.values.pop().unwrap()),
        _ => Err(vec![TypedError::ParameterMultipleValuesDisallowed {
            parameter: kind,
            span: param.span(),
        }]),
    }
}

/// Parse a single quoted value from a parameter.
///
/// # Errors
///
/// Returns an error if:
/// - The parameter does not have exactly one value
/// - The value is not quoted
pub fn parse_single_quoted<'src>(
    param: &mut SyntaxParameter<'src>,
    kind: ParameterKind<'src>,
) -> Result<SpannedSegments<'src>, Vec<TypedError<'src>>> {
    match param.values.len() {
        1 => {
            let v = param.values.pop().unwrap(); // SAFETY: length check
            if v.quoted {
                Ok(v.value)
            } else {
                Err(vec![TypedError::ParameterValueMustBeQuoted {
                    parameter: kind,
                    span: v.value.span(),
                    value: v.value,
                }])
            }
        }
        _ => Err(vec![TypedError::ParameterMultipleValuesDisallowed {
            parameter: kind,
            span: param.span(),
        }]),
    }
}

/// Parse a single unquoted value from a parameter.
///
/// # Errors
///
/// Returns an error if:
/// - The parameter does not have exactly one value
/// - The value is quoted
pub fn parse_single_not_quoted<'src>(
    param: &mut SyntaxParameter<'src>,
    kind: ParameterKind<'src>,
) -> Result<SpannedSegments<'src>, Vec<TypedError<'src>>> {
    // TODO: avoid clone kind
    match param.values.len() {
        1 => {
            let v = param.values.pop().unwrap();
            if v.quoted {
                Err(vec![TypedError::ParameterValueMustNotBeQuoted {
                    parameter: kind,
                    span: v.value.span(),
                    value: v.value,
                }])
            } else {
                Ok(v.value)
            }
        }
        _ => Err(vec![TypedError::ParameterMultipleValuesDisallowed {
            parameter: kind,
            span: param.span(),
        }]),
    }
}

/// Parse multiple quoted values from a parameter.
///
/// # Errors
///
/// Returns an error if any of the values are not quoted.
pub fn parse_multiple_quoted<'src>(
    param: SyntaxParameter<'src>,
    kind: &ParameterKind<'src>,
) -> Result<Vec<SpannedSegments<'src>>, Vec<TypedError<'src>>> {
    let mut values = Vec::with_capacity(param.values.len());
    let mut errors = Vec::new();
    for v in param.values {
        if v.quoted {
            values.push(v.value);
        } else {
            errors.push(TypedError::ParameterValueMustBeQuoted {
                parameter: kind.clone(),
                span: v.value.span(),
                value: v.value,
            });
        }
    }

    if errors.is_empty() {
        Ok(values)
    } else {
        Err(errors)
    }
}

/// Macro to define parameter enums without x-name/iana-token support.
///
/// This generates simple enums with Copy semantics for RFC 5545 parameter values
/// that don't support extensions.
macro_rules! define_param_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident => $kw:ident
            ),* $(,)?
        }

        parser = $pvis:vis fn $parse_fn:ident;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[allow(missing_docs)]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                $variant,
            )*
        }

        impl<'src> TryFrom<SpannedSegments<'src>> for $name {
            type Error = SpannedSegments<'src>;

            fn try_from(segs: SpannedSegments<'src>) -> Result<Self, Self::Error> {
                $(
                    if segs.eq_str_ignore_ascii_case($kw) {
                        return Ok(Self::$variant);
                    }
                )*
                Err(segs)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$variant => $kw.fmt(f),
                    )*
                }
            }
        }

        $pvis fn $parse_fn(mut param: SyntaxParameter<'_>) -> ParseResult<'_> {
            parse_single_not_quoted(&mut param, ParameterKind::$name).and_then(|value| {
                match $name::try_from(value) {
                    Ok(value) => Ok(Parameter::$name {
                        value,
                        span: param.span(),
                    }),
                    Err(value) => Err(vec![TypedError::ParameterValueInvalid {
                        span: value.span(),
                        parameter: ParameterKind::$name,
                        value,
                    }])
                }
            })
        }
    };
}

/// Macro to define parameter enums with x-name and unrecognized value support.
///
/// This generates enums with lifetime parameters for zero-copy storage of
/// extension values per RFC 5545.
macro_rules! define_param_enum_with_unknown {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident => $kw:ident
            ),* $(,)?
        }

        parser = $pvis:vis fn $parse_fn:ident;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        #[allow(missing_docs)]
        $vis enum $name<'src> {
            $(
                $(#[$vmeta])*
                $variant,
            )*
            /// Custom experimental x-name value (must start with "X-" or "x-")
            XName(SpannedSegments<'src>),
            /// Unrecognized value (not a known standard value)
            Unrecognized(SpannedSegments<'src>),
        }

        impl<'src> ::core::convert::From<SpannedSegments<'src>> for $name<'src> {
            fn from(segs: SpannedSegments<'src>) -> Self {
                $(
                    if segs.eq_str_ignore_ascii_case($kw) {
                        return Self::$variant;
                    }
                )*

                // Check for x-name prefix
                if segs.starts_with_str_ignore_ascii_case("X-") {
                    Self::XName(segs.clone())
                } else {
                    // Otherwise, treat as unrecognized value
                    Self::Unrecognized(segs.clone())
                }
            }
        }

        impl<'src> ::core::fmt::Display for $name<'src> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    $(
                        Self::$variant => $kw.fmt(f),
                    )*
                    Self::XName(segs) | Self::Unrecognized(segs) => write!(f, "{segs}"),
                }
            }
        }

        impl $name<'_> {
            /// Tries to compare two values for equality if both are standard values.
            /// Returns None if either value is x-name/unrecognized.
            #[allow(dead_code)]
            #[must_use]
            pub(crate) fn try_eq_known(&self, other: &Self) -> Option<bool> {
                match self {
                    $(
                        Self::$variant => match other {
                            Self::$variant => Some(true),
                            Self::XName(_) | Self::Unrecognized(_) => None, // not standard
                            _ => Some(false),
                        },
                    )*
                    Self::XName(_) | Self::Unrecognized(_) => None, // not standard
                }
            }
        }

        $pvis fn $parse_fn(mut param: SyntaxParameter<'_>) -> ParseResult<'_> {
            parse_single_not_quoted(&mut param, ParameterKind::$name).map(|value| {
                let enum_value = $name::try_from(value).unwrap(); // Never fails due to XName/Unrecognized variants
                Parameter::$name {
                    value: enum_value,
                    span: param.span(),
                }
            })
        }
    };
}
