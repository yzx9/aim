// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use std::collections::HashSet;
use std::fmt::Display;
use std::{collections::HashMap, sync::LazyLock};

use chumsky::error::Rich;

use crate::lexer::Span;
use crate::property_spec::{PROPERTY_SPECS, PropertySpec};
use crate::syntax::{SpannedSegments, SyntaxComponent, SyntaxProperty};
use crate::value::{Value, ValueKind, values};

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

#[derive(Debug, Clone)]
pub enum TypedAnalysisError<'src> {
    ParamNotAllowedMultiple {
        property_name: String,
        param_name: String,
        span: Span,
    },
    ParamValueKindNotDefined {
        property_name: String,
        kind: String,
        span: Span,
    },
    ParamValueKindNotAllowed {
        property_name: String,
        kind: ValueKind,
        span: Span,
    },
    PropertyNotDefined((String, Span)),
    PropertyNotAllowedMultiple((String, Span)),
    ValueSyntax(Rich<'src, char>),
}

impl TypedAnalysisError<'_> {
    pub fn span(&self) -> Span {
        match self {
            TypedAnalysisError::ParamNotAllowedMultiple { span, .. }
            | TypedAnalysisError::ParamValueKindNotDefined { span, .. }
            | TypedAnalysisError::ParamValueKindNotAllowed { span, .. }
            | TypedAnalysisError::PropertyNotDefined((_, span))
            | TypedAnalysisError::PropertyNotAllowedMultiple((_, span)) => span.clone(),
            TypedAnalysisError::ValueSyntax(err) => err.span().into_range(),
        }
    }
}

impl Display for TypedAnalysisError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedAnalysisError::ParamNotAllowedMultiple {
                property_name,
                param_name,
                ..
            } => {
                write!(
                    f,
                    "Parameter '{param_name}' for property '{property_name}' is not allowed to appear multiple times."
                )
            }
            TypedAnalysisError::ParamValueKindNotDefined {
                property_name,
                kind: value,
                ..
            } => {
                write!(
                    f,
                    "Parameter 'VALUE={value}' for property '{property_name}' is not defined."
                )
            }
            TypedAnalysisError::ParamValueKindNotAllowed {
                property_name,
                kind,
                ..
            } => {
                write!(
                    f,
                    "Parameter 'VALUE={kind}' for property '{property_name}' is not allowed here."
                )
            }
            TypedAnalysisError::PropertyNotDefined((name, _)) => {
                write!(f, "Property '{name}' is not defined in the specification.")
            }
            TypedAnalysisError::PropertyNotAllowedMultiple((name, _)) => {
                write!(
                    f,
                    "Property '{name}' is not allowed to appear multiple times."
                )
            }
            TypedAnalysisError::ValueSyntax(err) => write!(f, "{err}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypedComponent<'src> {
    pub name: &'src str, // "VCALENDAR" / "VEVENT" / "VTIMEZONE" / "VALARM" / ...
    pub properties: Vec<TypedProperty<'src>>, // Keep the original order
    pub children: Vec<TypedComponent<'src>>,
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

#[derive(Debug, Clone)]
pub struct TypedProperty<'src> {
    pub name: String,                      // UPPERCASE
    pub params: Vec<TypedParameter<'src>>, // Allow duplicates & multi-values
    pub values: Vec<Value<'src>>,
}

#[derive(Debug, Clone)]
pub struct TypedParameter<'src> {
    pub name: SpannedSegments<'src>,
    pub values: Vec<TypedParameterValue<'src>>, // Split by commas
}

#[derive(Debug, Clone)]
pub struct TypedParameterValue<'src> {
    pub value: SpannedSegments<'src>,
    pub quoted: bool,
}

fn typed_property<'src>(
    existing: &mut HashSet<String>,
    prop: SyntaxProperty<'src>,
) -> Result<TypedProperty<'src>, Vec<TypedAnalysisError<'src>>> {
    let name = prop.name.resolve().to_ascii_uppercase();
    let Some(spec) = PROP_TABLE.get(&name.as_ref()) else {
        return Err(vec![TypedAnalysisError::PropertyNotDefined((
            name,
            prop.name.span(),
        ))]);
    };

    if !spec.multiple_valued {
        if existing.contains(&name) {
            return Err(vec![TypedAnalysisError::PropertyNotAllowedMultiple((
                name,
                prop.name.span(),
            ))]);
        }

        existing.insert(name.clone()); // PERF: avoid clone
    }

    let kind = kind_of(&prop, spec).map_err(|a| vec![a])?;

    // TODO: cache parser
    let values = values(kind, prop.value).map_err(|errs| {
        errs.into_iter()
            .map(TypedAnalysisError::ValueSyntax)
            .collect::<Vec<_>>()
    })?;

    if !spec.multiple_valued && values.len() > 1 {
        return Err(vec![TypedAnalysisError::PropertyNotAllowedMultiple((
            name,
            prop.name.span(),
        ))]);
    }

    let params = prop
        .params
        .into_iter()
        .map(|p| TypedParameter {
            name: p.name,
            values: p
                .values
                .into_iter()
                .map(|v| TypedParameterValue {
                    value: v.value,
                    quoted: v.quoted,
                })
                .collect(),
        })
        .collect();

    Ok(TypedProperty {
        name,
        params,
        values,
    })
}

fn kind_of<'src>(
    prop: &SyntaxProperty<'src>,
    spec: &PropertySpec<'_>,
) -> Result<ValueKind, TypedAnalysisError<'src>> {
    // find VALUE= param
    let Some(value_params) = prop
        .params
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case("VALUE"))
    else {
        return Ok(spec.default_kind);
    };

    if value_params.values.len() > 1 {
        // multiple VALUE= params
        return Err(TypedAnalysisError::ParamNotAllowedMultiple {
            property_name: prop.name.resolve().to_string(),
            param_name: "VALUE".to_string(),
            span: prop.name.span(),
        });
    }
    let value_param = value_params.values.first().unwrap();

    let Ok(kind) = (&value_param.value).try_into() else {
        return Err(TypedAnalysisError::ParamValueKindNotDefined {
            property_name: prop.name.resolve().to_string(),
            kind: value_param.value.resolve().to_string(),
            span: prop.name.span(),
        });
    };

    if !spec.allowed_kinds.contains(&kind) {
        return Err(TypedAnalysisError::ParamValueKindNotAllowed {
            property_name: prop.name.resolve().to_string(),
            kind,
            span: prop.name.span(),
        });
    }

    Ok(kind)
}
