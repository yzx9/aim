// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.
//!
//! This module provides the typed analysis phase of the iCalendar parser,
//! converting syntax components into strongly-typed representations with
//! validated parameters and values.

use std::collections::HashSet;

use chumsky::error::Rich;
use thiserror::Error;

use crate::lexer::Span;
use crate::parameter::{Parameter, ValueKind};
use crate::property::{Property, PropertyKind};
use crate::syntax::{SpannedSegments, SyntaxComponent, SyntaxParameter, SyntaxProperty};
use crate::value::{Values, parse_values};

/// Perform typed analysis on raw components, returning typed components or errors.
///
/// ## Errors
/// If there are typing errors, a vector of errors will be returned.
pub fn typed_analysis(
    components: Vec<SyntaxComponent<'_>>,
) -> Result<Vec<TypedComponent<'_>>, Vec<TypedError<'_>>> {
    let mut typed_components = Vec::with_capacity(components.len());
    let mut errors = Vec::new();
    for comp in components {
        match typed_component(comp) {
            Ok(typed_comp) => typed_components.push(typed_comp),
            Err(errs) => errors.extend(errs),
        }
    }

    if errors.is_empty() {
        Ok(typed_components)
    } else {
        Err(errors)
    }
}

fn typed_component(comp: SyntaxComponent<'_>) -> Result<TypedComponent<'_>, Vec<TypedError<'_>>> {
    let mut existing_props = HashSet::with_capacity(comp.properties.len());
    let mut properties = Vec::with_capacity(comp.properties.len());
    let mut errors = Vec::new();
    for prop in comp.properties {
        match parsed_property(&mut existing_props, prop) {
            // Convert ParsedProperty to Property
            Ok(prop) => match Property::try_from(prop) {
                Ok(property) => properties.push(property),
                Err(errs) => errors.extend(errs),
            },
            Err(errs) => errors.extend(errs),
        }
    }

    let mut children = Vec::with_capacity(comp.children.len());
    for comp in comp.children {
        match typed_component(comp) {
            Ok(child) => children.push(child),
            Err(errs) => errors.extend(errs),
        }
    }

    if errors.is_empty() {
        Ok(TypedComponent {
            name: comp.name,
            properties,
            children,
        })
    } else {
        Err(errors)
    }
}

fn parsed_property<'src>(
    _existing: &mut HashSet<&str>,
    prop: SyntaxProperty<'src>,
) -> Result<ParsedProperty<'src>, Vec<TypedError<'src>>> {
    let Ok(prop_kind) = PropertyKind::try_from(&prop.name) else {
        return Err(vec![TypedError::PropertyUnknown {
            span: prop.name.span(),
            property: prop.name,
        }]);
    };

    let parameters = parameters(prop.parameters)?;
    let value_types = value_types(prop_kind, &parameters)?;

    // PERF: cache parser
    let values = parse_values(&value_types, &prop.value).map_err(|errs| {
        errs.into_iter()
            .map(|err| TypedError::ValueSyntax {
                value: prop.value.clone(),
                err,
            })
            .collect::<Vec<_>>()
    })?;

    Ok(ParsedProperty {
        kind: prop_kind,
        parameters,
        values,
        span: prop.name.span(),
    })
}

/// A typed iCalendar component with validated properties and nested child components.
#[derive(Debug, Clone)]
pub struct TypedComponent<'src> {
    /// Component name (e.g., "VCALENDAR", "VEVENT", "VTIMEZONE", "VALARM")
    pub name: &'src str,
    /// Properties in original order
    pub properties: Vec<Property<'src>>,
    /// Nested child components
    pub children: Vec<TypedComponent<'src>>,
}

/// A typed iCalendar property with validated parameters and values.
#[derive(Debug, Clone)]
pub struct ParsedProperty<'src> {
    /// Property kind
    pub kind: PropertyKind,
    /// Property parameters
    pub parameters: Vec<Parameter<'src>>,
    /// Property values
    pub values: Values<'src>,
    /// The span of the property name (for error reporting)
    pub span: Span,
}

/// Errors that can occur during typed analysis of iCalendar components.
#[non_exhaustive]
#[derive(Error, Debug, Clone)]
pub enum TypedError<'src> {
    /// Unknown property encountered.
    #[error("Unknown property '{property}'")]
    PropertyUnknown {
        /// The property name
        property: SpannedSegments<'src>,
        /// The span of the error
        span: Span,
    },

    /// Property occurs multiple times when only one is allowed.
    #[error("Property '{property}' occurs multiple times")]
    PropertyDuplicated {
        /// The property name
        property: &'src str,
        /// The span of the error
        span: Span,
    },

    /// Property does not allow multiple values.
    #[error("Property '{property}' does not allow multiple values")]
    PropertyMultipleValuesDisallowed {
        /// The property name
        property: &'src str,
        /// The span of the error
        span: Span,
    },

    /// Unknown parameter encountered.
    #[error("Parameter '{parameter}' is unknown")]
    ParameterUnknown {
        /// The parameter name
        parameter: SpannedSegments<'src>,
        /// The span of the error
        span: Span,
    },

    /// Parameter occurs multiple times when only one is allowed.
    #[error("Parameter '{parameter}' occurs multiple times")]
    ParameterDuplicated {
        /// The parameter name
        parameter: &'src str,
        /// The span of the error
        span: Span,
    },

    /// Parameter does not allow multiple values.
    #[error("Parameter '{parameter}' does not allow multiple values")]
    ParameterMultipleValuesDisallowed {
        /// The parameter name
        parameter: &'src str,
        /// The span of the error
        span: Span,
    },

    /// Parameter is not allowed for this property.
    #[error("Parameter '{parameter}' is not allowed for property '{property}'")]
    ParameterDisallowedForProperty {
        /// The parameter name
        parameter: &'src str,
        /// The property name
        property: &'src str,
        /// The span of the error
        span: Span,
    },

    /// Parameter value must be quoted.
    #[error("Parameter '{parameter}={value}' value must be quoted")]
    ParameterValueMustBeQuoted {
        /// The parameter name
        parameter: &'src str,
        /// The parameter value
        value: SpannedSegments<'src>,
        /// The span of the error
        span: Span,
    },

    /// Parameter value must not be quoted.
    #[error("Parameter '{parameter}=\"{value}\"' value must not be quoted")]
    ParameterValueMustNotBeQuoted {
        /// The parameter name
        parameter: &'src str,
        /// The parameter value
        value: SpannedSegments<'src>,
        /// The span of the error
        span: Span,
    },

    /// Invalid parameter value.
    #[error("Invalid value for parameter '{parameter}={value}'")]
    ParameterValueInvalid {
        /// The parameter name
        parameter: &'src str,
        /// The parameter value
        value: SpannedSegments<'src>,
        /// The span of the error
        span: Span,
    },

    /// Syntax error in parameter value.
    #[error("Syntax error in value of parameter '{parameter}': {err}")]
    ParameterValueSyntax {
        /// The parameter name
        parameter: &'src str,
        /// The syntax error details
        err: Rich<'src, char>,
    },

    /// Value type is not allowed for this property.
    #[error("Invalid value type '{value_type}' for property '{property}'")]
    ValueTypeDisallowed {
        /// The property name
        property: &'src str,
        /// The value type that was provided
        value_type: ValueKind,
        /// The expected value types
        expected_types: &'src [ValueKind],
        /// The span of the error
        span: Span,
    },

    /// Syntax error in property value.
    #[error("Syntax error in value '{value}': {err}")]
    ValueSyntax {
        /// The value
        value: SpannedSegments<'src>,
        /// The syntax error details
        err: Rich<'src, char>,
    },

    /// Property kind does not match the expected type.
    #[error("Expected property kind '{expected}', found '{found}'")]
    PropertyUnexpectedKind {
        /// Expected property kind
        expected: PropertyKind,
        /// Actual property kind found
        found: PropertyKind,
        /// The span of the error
        span: Span,
    },

    /// Property has no values when at least one is required.
    #[error("Property '{property}' has no values")]
    PropertyMissingValue {
        /// The property that is missing values
        property: PropertyKind,
        /// The span of the error
        span: Span,
    },

    /// Property has an invalid value count.
    #[error("Property '{property}' requires exactly {expected} value(s), but found {found}")]
    PropertyInvalidValueCount {
        /// The property kind
        property: PropertyKind,
        /// Expected number of values
        expected: usize,
        /// Actual number of values found
        found: usize,
        /// The span of the error
        span: Span,
    },

    /// Property value is invalid or out of allowed range.
    #[error("Invalid value '{value}' for property '{property}'")]
    PropertyInvalidValue {
        /// The property that has the invalid value
        property: PropertyKind,
        /// Description of why the value is invalid
        value: String,
        /// The span of the error
        span: Span,
    },

    /// Property value has unexpected type.
    #[error("Expected {expected} value for property '{property}', found {found}")]
    PropertyUnexpectedValue {
        /// The property that has the wrong type
        property: PropertyKind,
        /// Expected value type
        expected: ValueKind,
        /// Actual value type found
        found: ValueKind,
        /// The span of the error
        span: Span,
    },
}

impl TypedError<'_> {
    /// Get the span of this error.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            TypedError::PropertyUnknown { span, .. }
            | TypedError::PropertyDuplicated { span, .. }
            | TypedError::PropertyInvalidValueCount { span, .. }
            | TypedError::PropertyMultipleValuesDisallowed { span, .. }
            | TypedError::ParameterUnknown { span, .. }
            | TypedError::ParameterDuplicated { span, .. }
            | TypedError::ParameterMultipleValuesDisallowed { span, .. }
            | TypedError::ParameterDisallowedForProperty { span, .. }
            | TypedError::ParameterValueMustBeQuoted { span, .. }
            | TypedError::ParameterValueMustNotBeQuoted { span, .. }
            | TypedError::ParameterValueInvalid { span, .. }
            | TypedError::ValueTypeDisallowed { span, .. }
            | TypedError::PropertyUnexpectedKind { span, .. }
            | TypedError::PropertyInvalidValue { span, .. }
            | TypedError::PropertyMissingValue { span, .. }
            | TypedError::PropertyUnexpectedValue { span, .. } => *span,

            TypedError::ParameterValueSyntax { err, .. } | TypedError::ValueSyntax { err, .. } => {
                (*err.span()).into()
            }
        }
    }
}

fn parameters(params: Vec<SyntaxParameter<'_>>) -> Result<Vec<Parameter<'_>>, Vec<TypedError<'_>>> {
    let mut existing = HashSet::with_capacity(params.len());
    let mut typed_params = Vec::with_capacity(params.len());
    let mut errors = Vec::new();
    for param in params {
        match Parameter::try_from(param) {
            Ok(typed) => {
                let param_kind = typed.kind();
                let param_name = typed.name();
                if existing.contains(&param_kind) {
                    errors.push(TypedError::ParameterDuplicated {
                        parameter: param_name,
                        span: typed.span(),
                    });
                } else {
                    typed_params.push(typed);
                    existing.insert(param_kind);
                }
            }
            Err(errs) => errors.extend(errs),
        }
    }

    if errors.is_empty() {
        Ok(typed_params)
    } else {
        Err(errors)
    }
}

fn value_types<'src>(
    prop_kind: PropertyKind,
    params: &Vec<Parameter<'src>>,
) -> Result<Vec<ValueKind>, Vec<TypedError<'src>>> {
    use ValueKind::Binary;

    let allowed_types = prop_kind.value_kinds();

    // If VALUE parameter is explicitly specified, use only that type
    if let Some(Parameter::ValueKind { value, span }) = params
        .iter()
        .find(|param| matches!(param, Parameter::ValueKind { .. }))
    {
        if allowed_types.contains(value) {
            // Return only the explicitly specified type
            Ok(vec![*value])
        } else {
            Err(vec![TypedError::ValueTypeDisallowed {
                property: prop_kind.as_str(),
                value_type: *value,
                expected_types: allowed_types,
                span: *span,
            }])
        }
    } else {
        // No VALUE parameter specified - return all allowed types for type inference,
        // EXCEPT for BINARY which MUST be explicitly specified with VALUE=BINARY
        // (per RFC 5545 Section 3.3.1 and 3.8.1.1)
        Ok(allowed_types
            .iter()
            .filter(|&&t| t != Binary)
            .copied()
            .collect())
    }
}
