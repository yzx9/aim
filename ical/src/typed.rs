// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use std::collections::HashMap;
use std::sync::LazyLock;

use chumsky::error::Error;
use chumsky::input::Stream;
use chumsky::label::LabelError;

use crate::lexer::{SpannedTokens, SpannedTokensChars};
use crate::property_spec::{PROPERTY_SPECS, PropertySpec};
use crate::property_value::{
    PropertyValue, PropertyValueExpected, PropertyValueKind, PropertyValueParser,
};
use crate::syntax::{RawComponent, RawProperty};

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
pub fn typed_analysis<'src, Err>(
    components: Vec<RawComponent<'src>>,
) -> Result<Vec<TypedComponent<'src>>, Vec<Err>>
where
    Err: Error<'src, Stream<SpannedTokensChars<'src>>>
        + LabelError<'src, Stream<SpannedTokensChars<'src>>, PropertyValueExpected>
        + 'src,
{
    let prop_parser = PropertyValueParser::<'src, Err>::new();

    let mut errors = Vec::new();
    let mut typed_components = Vec::new();
    for comp in components {
        match typed_component(&prop_parser, comp) {
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
pub struct TypedComponent<'src> {
    pub name: &'src str, // "VCALENDAR" / "VEVENT" / "VTIMEZONE" / "VALARM" / ...
    pub properties: Vec<TypedProperty<'src>>, // Keep the original order
    pub children: Vec<TypedComponent<'src>>,
}

fn typed_component<'src, 'b, Err>(
    parser: &'b PropertyValueParser<'src, Err>,
    comp: RawComponent<'src>,
) -> Result<TypedComponent<'src>, Vec<Err>>
where
    Err: Error<'src, Stream<SpannedTokensChars<'src>>>
        + LabelError<'src, Stream<SpannedTokensChars<'src>>, PropertyValueExpected>
        + 'src,
{
    let mut errors = Vec::new();
    let mut properties = Vec::new();
    for prop in comp.properties {
        match type_property(parser, prop) {
            Ok(prop) => properties.push(prop),
            Err(errs) => errors.extend(errs),
        }
    }

    let mut children = Vec::new();
    for comp in comp.children {
        match typed_component(parser, comp) {
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
    pub name: SpannedTokens<'src>, // Case insensitive, keep original for writing back
    pub params: Vec<TypedParameter<'src>>, // Allow duplicates & multi-values
    pub values: Vec<PropertyValue<'src>>,
}

#[derive(Debug, Clone)]
pub struct TypedParameter<'src> {
    pub name: SpannedTokens<'src>,
    pub values: Vec<TypedParameterValue<'src>>, // Split by commas
}

#[derive(Debug, Clone)]
pub struct TypedParameterValue<'src> {
    pub value: SpannedTokens<'src>,
    pub quoted: bool,
}

fn type_property<'b, 'src: 'b, Err>(
    parser: &'b PropertyValueParser<'src, Err>,
    prop: RawProperty<'src>,
) -> Result<TypedProperty<'src>, Vec<Err>>
where
    Err: Error<'src, Stream<SpannedTokensChars<'src>>>
        + LabelError<'src, Stream<SpannedTokensChars<'src>>, PropertyValueExpected>
        + 'src,
{
    let prop_name = prop.name.to_string().to_ascii_uppercase();
    let kind = kind_of(&prop_name, &prop);

    let mut errors = Vec::new();
    let mut values = Vec::new();
    for v in prop.value {
        match parser.parse(kind, v) {
            Ok(v) => values.push(v),
            Err(errs) => errors.extend(errs),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
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

    Ok(TypedProperty::<'src> {
        name: prop.name,
        params,
        values,
    })
}

fn kind_of(prop_name: &str, prop: &RawProperty) -> PropertyValueKind {
    // find VALUE= param
    let value_param = prop
        .params
        .iter()
        .find(|p| p.name.to_string().to_uppercase() == "VALUE")
        .and_then(|p| p.values.first())
        .map(|s| s.value.to_string().to_uppercase());

    if let Some(spec) = PROP_TABLE.get(prop_name) {
        if let Some(v) = value_param {
            let kind = v.parse();
            // TODO: check if allowed
            kind.unwrap_or(PropertyValueKind::Text) // TODO: should throw error
        } else {
            spec.default_kind
        }
    } else if let Some(v) = value_param {
        v.parse().unwrap_or(PropertyValueKind::Text) // TODO: should throw error
    } else {
        PropertyValueKind::Text // TODO: should throw error
    }
}
