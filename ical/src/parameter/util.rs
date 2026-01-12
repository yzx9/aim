// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::parameter::{ParameterKindRef, ParameterRef};
use crate::string_storage::SpannedSegments;
use crate::syntax::{SyntaxParameterRef, SyntaxParameterValueRef};
use crate::typed::TypedError;

pub type ParseResult<'src> = Result<ParameterRef<'src>, Vec<TypedError<'src>>>;

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
    param: &mut SyntaxParameterRef<'src>,
    kind: ParameterKindRef<'src>,
) -> Result<SyntaxParameterValueRef<'src>, Vec<TypedError<'src>>> {
    match param.values.len() {
        1 => Ok(param.values.pop().unwrap()),
        _ => Err(vec![TypedError::ParameterMultipleValuesDisallowed {
            parameter: kind,
            span: param.span,
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
    param: &mut SyntaxParameterRef<'src>,
    kind: ParameterKindRef<'src>,
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
            span: param.span,
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
    param: &mut SyntaxParameterRef<'src>,
    kind: ParameterKindRef<'src>,
) -> Result<SpannedSegments<'src>, Vec<TypedError<'src>>> {
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
            span: param.span,
        }]),
    }
}

/// Parse multiple quoted values from a parameter.
///
/// # Errors
///
/// Returns an error if any of the values are not quoted.
pub fn parse_multiple_quoted<'src>(
    param: SyntaxParameterRef<'src>,
    kind: &ParameterKindRef<'src>,
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

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$variant => $kw.fmt(f),
                    )*
                }
            }
        }

        $pvis fn $parse_fn(mut param: crate::syntax::SyntaxParameterRef<'_>) -> crate::parameter::util::ParseResult<'_> {
            parse_single_not_quoted(&mut param, crate::parameter::ParameterKind::$name).and_then(|value| {
                match $name::try_from(value) {
                    Ok(value) => Ok(crate::parameter::Parameter::$name {
                        value,
                        span: param.span,
                    }),
                    Err(value) => Err(vec![TypedError::ParameterValueInvalid {
                        span: value.span(),
                        parameter: crate::parameter::ParameterKind::$name,
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

        ref    = $rvis:vis type $name_ref:ident;
        owned  = $ovis:vis type $name_owned:ident;
        parser = $pvis:vis fn $parse_fn:ident;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        #[allow(missing_docs)]
        $vis enum $name<S: crate::string_storage::StringStorage> {
            $(
                $(#[$vmeta])*
                $variant,
            )*
            /// Custom experimental x-name value (must start with "X-" or "x-")
            XName(S),
            /// Unrecognized value (not a known standard value)
            Unrecognized(S),
        }

        /// Borrowed type alias for parameter with lifetime `'src`
        $rvis type $name_ref<'src> = $name<SpannedSegments<'src>>;
        /// Owned type alias for parameter
        $ovis type $name_owned = $name<String>;

        impl<T: crate::string_storage::StringStorage> ::core::fmt::Display for $name<T> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    $(
                        Self::$variant => $kw.fmt(f),
                    )*
                    Self::XName(segs) | Self::Unrecognized(segs) => write!(f, "{segs}"),
                }
            }
        }

        impl<'src> ::core::convert::From<SpannedSegments<'src>> for $name_ref<'src> {
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

        impl<'src> $name_ref<'src> {
            /// Convert borrowed type to owned type
            #[must_use]
            pub fn to_owned(&self) -> $name_owned {
                match self {
                    $(
                        Self::$variant => $name_owned::$variant,
                    )*
                    Self::XName(s) => $name_owned::XName(s.to_string()),
                    Self::Unrecognized(s) => $name_owned::Unrecognized(s.to_string()),
                }
            }
        }

        $pvis fn $parse_fn(mut param: SyntaxParameterRef<'_>) -> ParseResult<'_> {
            parse_single_not_quoted(&mut param, crate::parameter::ParameterKind::$name).map(|value| {
                let enum_value = $name::try_from(value).unwrap(); // Never fails due to XName/Unrecognized variants
                Parameter::$name {
                    value: enum_value,
                    span: param.span,
                }
            })
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident => $kw:ident
            ),* $(,)?
        }

        ref    = $rvis:vis type $name_ref:ident;
        owned  = $ovis:vis type $name_owned:ident;
        parser = $pvis:vis fn $parse_fn:ident;
        gen_eq_known;
    ) => {
        define_param_enum_with_unknown! {
            $(#[$meta])*
            $vis enum $name {
            $(
                $(#[$vmeta])*
                $variant => $kw
            ),*
            }
            ref    = $rvis type $name_ref;
            owned  = $ovis type $name_owned;
            parser = $pvis fn $parse_fn;
        }

        impl<T: crate::string_storage::StringStorage> $name<T> {
            /// Tries to compare two values for equality if both are standard values.
            /// Returns None if either value is x-name/unrecognized.
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
    }
}
