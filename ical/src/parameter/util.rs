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
