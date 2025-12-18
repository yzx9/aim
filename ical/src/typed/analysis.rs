// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use std::{collections::HashMap, collections::HashSet, sync::LazyLock};

use chumsky::error::Rich;
use thiserror::Error;

use crate::lexer::Span;
use crate::syntax::{SpannedSegments, SyntaxComponent, SyntaxProperty};
use crate::typed::parameter::{TypedParameters, ValueType};
use crate::typed::property_spec::{PROPERTY_SPECS, PropertySpec};
use crate::typed::value::{Value, parse_values};

static PROP_TABLE: LazyLock<HashMap<&'static str, &'static PropertySpec>> = LazyLock::new(|| {
    PROPERTY_SPECS
        .iter()
        .map(|spec| (spec.name, spec))
        .collect()
});

/// Perform typed analysis on raw components, returning typed components or errors.
///
/// ## Errors
/// If there are typing errors, a vector of errors will be returned.
pub fn typed_analysis(
    components: Vec<SyntaxComponent<'_>>,
) -> Result<Vec<TypedComponent<'_>>, Vec<TypedAnalysisError<'_>>> {
    let mut typed_components = Vec::new();
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
    let mut existing_props = HashSet::new();
    let mut properties = Vec::new();
    let mut errors = Vec::new();
    for prop in comp.properties {
        match typed_property(&mut existing_props, prop) {
            Ok(prop) => properties.push(prop),
            Err(errs) => errors.extend(errs),
        }
    }

    let mut children = Vec::new();
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
    let prop_name_upper = prop.name.resolve().to_ascii_uppercase();
    let Some(spec) = PROP_TABLE.get::<str>(prop_name_upper.as_ref()) else {
        return Err(vec![TypedAnalysisError::PropertyUnknown {
            span: prop.name.span(),
            property: prop.name,
        }]);
    };

    if !spec.multiple_valued {
        if existing.contains(spec.name) {
            return Err(vec![TypedAnalysisError::PropertyDuplicated {
                property: spec.name,
                span: prop.name.span(),
            }]);
        }

        existing.insert(spec.name);
    }

    let parameters: TypedParameters = prop.parameters.try_into()?;
    let (value_type, _value_type_span) = match (&parameters.value_type, &parameters.value_type_span)
    {
        (Some(value_type), Some(span)) => {
            if !spec.allowed_kinds.contains(value_type) {
                return Err(vec![TypedAnalysisError::ValueTypeDisallowed {
                    property: spec.name,
                    value_type: *value_type,
                    expected_types: spec.allowed_kinds,
                    span: span.clone(),
                }]);
            }
            (*value_type, span.clone())
        }
        (Some(value_type), None) => {
            // This shouldn't happen if parsing is correct, but handle gracefully
            if !spec.allowed_kinds.contains(value_type) {
                return Err(vec![TypedAnalysisError::ValueTypeDisallowed {
                    property: spec.name,
                    value_type: *value_type,
                    expected_types: spec.allowed_kinds,
                    span: prop.name.span(),
                }]);
            }
            (*value_type, prop.name.span())
        }
        (None, _) => (spec.default_kind, prop.name.span()),
    };

    // PERF: cache parser, avoid cloning the value
    let values = parse_values(value_type, prop.value.clone()).map_err(|errs| {
        errs.into_iter()
            .map(|err| TypedAnalysisError::ValueSyntax {
                value: prop.value.clone(),
                err,
            })
            .collect::<Vec<_>>()
    })?;

    if !spec.multiple_valued && values.len() > 1 {
        return Err(vec![TypedAnalysisError::PropertyMultipleValuesDisallowed {
            property: spec.name,
            span: prop.name.span(),
        }]);
    }

    Ok(TypedProperty {
        name: spec.name,
        parameters,
        values,
    })
}

#[derive(Debug, Clone)]
pub struct TypedComponent<'src> {
    pub name: &'src str, // "VCALENDAR" / "VEVENT" / "VTIMEZONE" / "VALARM" / ...
    pub properties: Vec<TypedProperty<'src>>, // Keep the original order
    pub children: Vec<TypedComponent<'src>>,
}

#[derive(Debug, Clone)]
pub struct TypedProperty<'src> {
    pub name: &'src str, // Standardized property name in UPPERCASE
    pub parameters: TypedParameters<'src>,
    pub values: Vec<Value<'src>>,
}
#[derive(Error, Debug, Clone)]
pub enum TypedAnalysisError<'src> {
    #[error("Unknown property '{property}'")]
    PropertyUnknown {
        property: SpannedSegments<'src>,
        span: Span,
    },

    #[error("Property '{property}' occurs multiple times")]
    PropertyDuplicated { property: &'src str, span: Span },

    #[error("Property '{property}' does not allow multiple values")]
    PropertyMultipleValuesDisallowed { property: &'src str, span: Span },

    #[error("Parameter '{parameter}' is unknown")]
    ParameterUnknown {
        parameter: SpannedSegments<'src>,
        span: Span,
    },

    #[error("Parameter '{parameter}' occurs multiple times")]
    ParameterDuplicated { parameter: &'src str, span: Span },

    #[error("Parameter '{parameter}' does not allow multiple values")]
    ParameterMultipleValuesDisallowed { parameter: &'src str, span: Span },

    #[error("Invalid value for parameter '{parameter}={value}'")]
    ParameterInvalidValue {
        parameter: &'src str,
        value: SpannedSegments<'src>,
        span: Span,
    },

    #[error("Syntax error in value of parameter '{parameter}': {err}")]
    ParameterValueSyntax {
        parameter: &'src str,
        err: Rich<'src, char>,
    },

    #[error("Invalid value type '{value_type}' for property '{property}'")]
    ValueTypeDisallowed {
        property: &'src str,
        value_type: ValueType,
        expected_types: &'src [ValueType],
        span: Span,
    },

    #[error("Syntax error in value '{value}': {err}")]
    ValueSyntax {
        value: SpannedSegments<'src>,
        err: Rich<'src, char>,
    },
}

impl TypedAnalysisError<'_> {
    pub fn span(&self) -> Span {
        match self {
            TypedAnalysisError::PropertyUnknown { span, .. }
            | TypedAnalysisError::PropertyDuplicated { span, .. }
            | TypedAnalysisError::PropertyMultipleValuesDisallowed { span, .. }
            | TypedAnalysisError::ParameterUnknown { span, .. }
            | TypedAnalysisError::ParameterDuplicated { span, .. }
            | TypedAnalysisError::ParameterMultipleValuesDisallowed { span, .. }
            | TypedAnalysisError::ParameterInvalidValue { span, .. }
            | TypedAnalysisError::ValueTypeDisallowed { span, .. } => span.clone(),

            TypedAnalysisError::ParameterValueSyntax { err, .. }
            | TypedAnalysisError::ValueSyntax { err, .. } => err.span().into_range(),
        }
    }
}
