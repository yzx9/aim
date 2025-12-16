// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt::Display;
use std::{collections::HashMap, sync::LazyLock};

use chumsky::error::Rich;

use crate::keyword::KW_VALUE;
use crate::lexer::Span;
use crate::syntax::{SpannedSegments, SyntaxComponent, SyntaxParameter, SyntaxProperty};
use crate::typed::parameter::{FreeBusyType, ParamEncoding, ParamValueType, TypedParameter};
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
    existing: &mut HashSet<String>,
    prop: SyntaxProperty<'src>,
) -> Result<TypedProperty<'src>, Vec<TypedAnalysisError<'src>>> {
    let name = prop.name.resolve().to_ascii_uppercase();
    let Some(spec) = PROP_TABLE.get(&name.as_ref()) else {
        return Err(vec![TypedAnalysisError::PropertyUnknown {
            property: name,
            span: prop.name.span(),
        }]);
    };

    if !spec.multiple_valued {
        if existing.contains(&name) {
            return Err(vec![TypedAnalysisError::PropertyDuplicated {
                property: name,
                span: prop.name.span(),
            }]);
        }

        existing.insert(name.clone()); // PERF: avoid clone
    }

    let parameters: TypedParameters = prop.parameters.try_into()?;
    let kind = parameters.value_type.unwrap_or(spec.default_kind);

    // PERF: cache parser
    let values = parse_values(kind, prop.value).map_err(|errs| {
        errs.into_iter()
            .map(TypedAnalysisError::ValueSyntax)
            .collect::<Vec<_>>()
    })?;

    if !spec.multiple_valued && values.len() > 1 {
        return Err(vec![TypedAnalysisError::PropertyDuplicated {
            property: name,
            span: prop.name.span(),
        }]);
    }

    Ok(TypedProperty {
        name,
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
    pub name: String, // UPPERCASE
    pub parameters: TypedParameters<'src>,
    pub values: Vec<Value<'src>>,
}

#[derive(Default, Debug, Clone)]
pub struct TypedParameters<'src> {
    alternate_text: Option<SpannedSegments<'src>>,
    common_name: Option<SpannedSegments<'src>>,
    calendar_user_type: Option<SpannedSegments<'src>>,
    delegators: Option<Vec<SpannedSegments<'src>>>,
    delegatees: Option<Vec<SpannedSegments<'src>>>,
    directory: Option<SpannedSegments<'src>>,
    encoding: Option<ParamEncoding>,
    format_type: Option<SpannedSegments<'src>>,
    free_busy_type: Option<FreeBusyType>,
    language: Option<SpannedSegments<'src>>,
    group_or_list_membership: Option<Vec<SpannedSegments<'src>>>,
    participation_status: Option<SpannedSegments<'src>>,
    recurrence_id_range: Option<SpannedSegments<'src>>,
    alarm_trigger_relationship: Option<SpannedSegments<'src>>,
    relationship_type: Option<SpannedSegments<'src>>,
    participation_role: Option<SpannedSegments<'src>>,
    send_by: Option<SpannedSegments<'src>>,
    rsvp_expectation: Option<bool>,
    time_zone_identifier: Option<SpannedSegments<'src>>,
    value_type: Option<ParamValueType>,
}

impl<'src> TryFrom<Vec<SyntaxParameter<'src>>> for TypedParameters<'src> {
    type Error = Vec<TypedAnalysisError<'src>>;

    fn try_from(params: Vec<SyntaxParameter<'src>>) -> Result<Self, Self::Error> {
        fn assign_or_error<T>(
            slot: &mut Option<T>,
            errors: &mut Vec<TypedAnalysisError>,
            value: T,
            property: &str,
            param: &SyntaxParameter,
        ) {
            match slot {
                Some(_) => errors.push(TypedAnalysisError::ParameterDuplicated {
                    property: property.to_string(),
                    parameter: param.name.resolve().to_string(),
                    span: param.name.span(),
                }),
                None => *slot = Some(value),
            }
        }

        let mut result = Self::default();
        let mut errors = Vec::new();
        for param in params {
            match TypedParameter::try_from(param.clone()) {
                Ok(TypedParameter::AlternateText(p)) => {
                    assign_or_error(&mut result.alternate_text, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::CommonName(p)) => {
                    assign_or_error(&mut result.common_name, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::CalendarUserType(p)) => {
                    assign_or_error(&mut result.calendar_user_type, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::Delegators(p)) => {
                    // NOTE: should we allow multiple parameters to merge values?
                    assign_or_error(&mut result.delegators, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::Delegatees(p)) => {
                    // NOTE: should we allow multiple parameters to merge values?
                    assign_or_error(&mut result.delegatees, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::Directory(p)) => {
                    assign_or_error(&mut result.directory, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::Encoding(p)) => {
                    assign_or_error(&mut result.encoding, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::FormatType(p)) => {
                    assign_or_error(&mut result.format_type, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::FreeBusyType(p)) => {
                    assign_or_error(&mut result.free_busy_type, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::Language(p)) => {
                    assign_or_error(&mut result.language, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::GroupOrListMembership(p)) => {
                    // NOTE: should we allow multiple parameters to merge values?
                    assign_or_error(
                        &mut result.group_or_list_membership,
                        &mut errors,
                        p,
                        "",
                        &param,
                    );
                }
                Ok(TypedParameter::ParticipationStatus(p)) => {
                    assign_or_error(&mut result.participation_status, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::RecurrenceIdRange(p)) => {
                    assign_or_error(&mut result.recurrence_id_range, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::AlarmTriggerRelationship(p)) => {
                    assign_or_error(
                        &mut result.alarm_trigger_relationship,
                        &mut errors,
                        p,
                        "",
                        &param,
                    );
                }
                Ok(TypedParameter::RelationshipType(p)) => {
                    assign_or_error(&mut result.relationship_type, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::ParticipationRole(p)) => {
                    assign_or_error(&mut result.participation_role, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::RsvpExpectation(p)) => {
                    assign_or_error(&mut result.rsvp_expectation, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::SendBy(p)) => {
                    assign_or_error(&mut result.send_by, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::TimeZoneIdentifier(p)) => {
                    assign_or_error(&mut result.time_zone_identifier, &mut errors, p, "", &param);
                }
                Ok(TypedParameter::ValueType(p)) => {
                    assign_or_error(&mut result.value_type, &mut errors, p, "", &param);
                }
                Err(e) => errors.extend(e),
            }
        }

        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypedAnalysisError<'src> {
    PropertyUnknown {
        property: String,
        span: Span,
    },
    PropertyDuplicated {
        property: String,
        span: Span,
    },
    ParameterDuplicated {
        property: String,
        parameter: String,
        span: Span,
    },
    ParameterMultipleValuesDisallowed {
        property: String,
        parameter: String,
        span: Span,
    },
    ParameterValueKindUnknown {
        property: String,
        kind: String,
        span: Span,
    },
    ParameterValueKindDisallowed {
        property: String,
        kind: ParamValueType,
        span: Span,
    },
    ParameterValueSyntax {
        property: String,
        parameter: String,
        err: Rich<'src, char>,
    },
    ValueSyntax(Rich<'src, char>),
}

impl TypedAnalysisError<'_> {
    pub fn span(&self) -> Span {
        match self {
            TypedAnalysisError::ParameterDuplicated { span, .. }
            | TypedAnalysisError::ParameterMultipleValuesDisallowed { span, .. }
            | TypedAnalysisError::ParameterValueKindUnknown { span, .. }
            | TypedAnalysisError::ParameterValueKindDisallowed { span, .. }
            | TypedAnalysisError::PropertyUnknown { span, .. }
            | TypedAnalysisError::PropertyDuplicated { span, .. } => span.clone(),
            TypedAnalysisError::ParameterValueSyntax { err, .. }
            | TypedAnalysisError::ValueSyntax(err) => err.span().into_range(),
        }
    }
}

impl Display for TypedAnalysisError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedAnalysisError::PropertyUnknown { property, .. } => {
                write!(f, "Unknown property '{property}'")
            }
            TypedAnalysisError::PropertyDuplicated { property, .. } => {
                write!(f, "Property '{property}' occurs multiple times")
            }
            TypedAnalysisError::ParameterDuplicated {
                property,
                parameter: param,
                ..
            } => write!(
                f,
                "Parameter '{param}' occurs multiple times on property '{property}'"
            ),
            TypedAnalysisError::ParameterMultipleValuesDisallowed {
                property,
                parameter: param,
                ..
            } => write!(
                f,
                "Parameter '{param}' on property '{property}' does not allow multiple values"
            ),
            TypedAnalysisError::ParameterValueKindUnknown { property, kind, .. } => write!(
                f,
                "Unknown parameter '${KW_VALUE}={kind}' on property '{property}'"
            ),
            TypedAnalysisError::ParameterValueKindDisallowed { property, kind, .. } => write!(
                f,
                "Parameter '{KW_VALUE}={kind}' is not allowed on property '{property}'"
            ),
            TypedAnalysisError::ParameterValueSyntax {
                property,
                parameter,
                err,
            } => write!(
                f,
                "Syntax error in value of parameter '{parameter}' on property '{property}': {err}"
            ),
            TypedAnalysisError::ValueSyntax(err) => write!(f, "{err}"),
        }
    }
}
