// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use std::collections::HashSet;

use chumsky::error::Rich;
use thiserror::Error;

use crate::lexer::Span;
use crate::syntax::{SpannedSegments, SyntaxComponent, SyntaxParameter, SyntaxProperty};
use crate::typed::parameter::TypedParameter;
use crate::typed::parameter_type::ValueType;
use crate::typed::property_spec::{
    PropertyCardinality, PropertyKind, PropertySpec, ValueCardinality,
};
use crate::typed::value::{Value, parse_values};

/// Perform typed analysis on raw components, returning typed components or errors.
///
/// ## Errors
/// If there are typing errors, a vector of errors will be returned.
pub fn typed_analysis(
    components: Vec<SyntaxComponent<'_>>,
) -> Result<Vec<TypedComponent<'_>>, Vec<TypedAnalysisError<'_>>> {
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

fn typed_component(
    comp: SyntaxComponent<'_>,
) -> Result<TypedComponent<'_>, Vec<TypedAnalysisError<'_>>> {
    let mut existing_props = HashSet::with_capacity(comp.properties.len());
    let mut properties = Vec::with_capacity(comp.properties.len());
    let mut errors = Vec::new();
    for prop in comp.properties {
        match typed_property(&mut existing_props, prop) {
            Ok(prop) => properties.push(prop),
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

fn typed_property<'src>(
    existing: &mut HashSet<&str>,
    prop: SyntaxProperty<'src>,
) -> Result<TypedProperty<'src>, Vec<TypedAnalysisError<'src>>> {
    let Ok(prop_kind) = PropertyKind::try_from(&prop.name) else {
        return Err(vec![TypedAnalysisError::PropertyUnknown {
            span: prop.name.span(),
            property: prop.name,
        }]);
    };
    let spec = prop_kind.spec();

    // Check if property can appear multiple times in the component
    if matches!(spec.property_cardinality, PropertyCardinality::AtMostOnce) {
        if existing.contains(spec.name()) {
            return Err(vec![TypedAnalysisError::PropertyDuplicated {
                property: spec.name(),
                span: prop.name.span(),
            }]);
        }

        existing.insert(spec.name());
    }

    let parameters = parameters(spec, prop.parameters)?;
    let value_types = value_types(spec, &parameters)?;

    // PERF: cache parser
    let values = parse_values(&value_types, &prop.value).map_err(|errs| {
        errs.into_iter()
            .map(|err| TypedAnalysisError::ValueSyntax {
                value: prop.value.clone(),
                err,
            })
            .collect::<Vec<_>>()
    })?;

    // Validate value count based on ValueCardinality specification
    match &spec.value_cardinality {
        ValueCardinality::Exactly(n) => {
            let expected = n.get() as usize;
            if values.len() != expected {
                return Err(vec![TypedAnalysisError::PropertyInvalidValueCount {
                    property: spec.name(),
                    expected: n.get(),
                    found: values.len(),
                    span: prop.name.span(),
                }]);
            }
        }
        ValueCardinality::AtLeast(n) => {
            let min = n.get() as usize;
            if values.len() < min {
                return Err(vec![TypedAnalysisError::PropertyInsufficientValues {
                    property: spec.name(),
                    min: n.get(),
                    found: values.len(),
                    span: prop.name.span(),
                }]);
            }
        }
    }

    Ok(TypedProperty {
        name: spec.name(),
        parameters,
        values,
    })
}

/// A typed iCalendar component with validated properties and nested child components.
#[derive(Debug, Clone)]
pub struct TypedComponent<'src> {
    /// Component name (e.g., "VCALENDAR", "VEVENT", "VTIMEZONE", "VALARM")
    pub name: &'src str,
    /// Properties in original order
    pub properties: Vec<TypedProperty<'src>>,
    /// Nested child components
    pub children: Vec<TypedComponent<'src>>,
}

/// A typed iCalendar property with validated parameters and values.
#[derive(Debug, Clone)]
pub struct TypedProperty<'src> {
    /// Property name (standardized in UPPERCASE)
    pub name: &'src str,
    /// Property parameters
    pub parameters: Vec<TypedParameter<'src>>,
    /// Property values
    pub values: Vec<Value<'src>>,
}

/// Errors that can occur during typed analysis of iCalendar components.
#[non_exhaustive]
#[derive(Error, Debug, Clone)]
pub enum TypedAnalysisError<'src> {
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

    /// Property has an invalid value count.
    #[error("Property '{property}' requires exactly {expected} value(s), but found {found}")]
    PropertyInvalidValueCount {
        /// The property name
        property: &'src str,
        /// Expected number of values
        expected: u8,
        /// Actual number of values found
        found: usize,
        /// The span of the error
        span: Span,
    },

    /// Property has insufficient values.
    #[error("Property '{property}' requires at least {min} value(s), but found {found}")]
    PropertyInsufficientValues {
        /// The property name
        property: &'src str,
        /// Minimum required number of values
        min: u8,
        /// Actual number of values found
        found: usize,
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
        value_type: ValueType,
        /// The expected value types
        expected_types: &'src [ValueType],
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
}

impl TypedAnalysisError<'_> {
    /// Get the span of this error.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            TypedAnalysisError::PropertyUnknown { span, .. }
            | TypedAnalysisError::PropertyDuplicated { span, .. }
            | TypedAnalysisError::PropertyInvalidValueCount { span, .. }
            | TypedAnalysisError::PropertyInsufficientValues { span, .. }
            | TypedAnalysisError::PropertyMultipleValuesDisallowed { span, .. }
            | TypedAnalysisError::ParameterUnknown { span, .. }
            | TypedAnalysisError::ParameterDuplicated { span, .. }
            | TypedAnalysisError::ParameterMultipleValuesDisallowed { span, .. }
            | TypedAnalysisError::ParameterDisallowedForProperty { span, .. }
            | TypedAnalysisError::ParameterValueMustBeQuoted { span, .. }
            | TypedAnalysisError::ParameterValueMustNotBeQuoted { span, .. }
            | TypedAnalysisError::ParameterValueInvalid { span, .. }
            | TypedAnalysisError::ValueTypeDisallowed { span, .. } => span.clone(),

            TypedAnalysisError::ParameterValueSyntax { err, .. }
            | TypedAnalysisError::ValueSyntax { err, .. } => err.span().into_range(),
        }
    }
}

fn parameters<'src>(
    spec: &PropertySpec<'src>,
    params: Vec<SyntaxParameter<'src>>,
) -> Result<Vec<TypedParameter<'src>>, Vec<TypedAnalysisError<'src>>> {
    let mut existing = HashSet::with_capacity(params.len());
    let mut typed_params = Vec::with_capacity(params.len());
    let mut errors = Vec::new();
    for param in params {
        match TypedParameter::try_from(param) {
            Ok(typed) => {
                let param_kind = typed.kind();
                let param_name = typed.name();
                if !spec.parameters.contains(&param_kind) {
                    // Check if parameter is allowed for this property
                    errors.push(TypedAnalysisError::ParameterDisallowedForProperty {
                        parameter: param_name,
                        property: spec.name(),
                        span: typed.span(),
                    });
                } else if existing.contains(&param_kind) {
                    errors.push(TypedAnalysisError::ParameterDuplicated {
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
    spec: &'src PropertySpec,
    params: &Vec<TypedParameter<'src>>,
) -> Result<Vec<ValueType>, Vec<TypedAnalysisError<'src>>> {
    use ValueType::Binary;

    // If VALUE parameter is explicitly specified, use only that type
    if let Some(TypedParameter::ValueType { value, span }) = params
        .iter()
        .find(|param| matches!(param, TypedParameter::ValueType { .. }))
    {
        if spec.value_types.contains(value) {
            // Return only the explicitly specified type
            Ok(vec![*value])
        } else {
            Err(vec![TypedAnalysisError::ValueTypeDisallowed {
                property: spec.name(),
                value_type: *value,
                expected_types: spec.value_types,
                span: span.clone(),
            }])
        }
    } else {
        // No VALUE parameter specified - return all allowed types for type inference,
        // EXCEPT for BINARY which MUST be explicitly specified with VALUE=BINARY
        // (per RFC 5545 Section 3.3.1 and 3.8.1.1)
        Ok(spec
            .value_types
            .iter()
            .filter(|&&t| t != Binary)
            .copied()
            .collect())
    }
}
