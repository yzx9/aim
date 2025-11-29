// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use std::{collections::HashMap, sync::LazyLock};

use chumsky::error::Rich;

use crate::property_spec::{PROPERTY_SPECS, PropertySpec};
use crate::property_value::{PropertyValue, PropertyValueKind, property_value};
use crate::syntax::{SpannedSegments, SyntaxComponent, SyntaxProperty};

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
) -> Result<Vec<TypedComponent<'_>>, Vec<Rich<'_, char>>> {
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
pub struct TypedComponent<'src> {
    pub name: &'src str, // "VCALENDAR" / "VEVENT" / "VTIMEZONE" / "VALARM" / ...
    pub properties: Vec<TypedProperty<'src>>, // Keep the original order
    pub children: Vec<TypedComponent<'src>>,
}

fn typed_component(comp: SyntaxComponent<'_>) -> Result<TypedComponent<'_>, Vec<Rich<'_, char>>> {
    let mut properties = Vec::new();
    let mut errors = Vec::new();
    for prop in comp.properties {
        match typed_property(prop) {
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
    pub name: SpannedSegments<'src>, // Case insensitive, keep original for writing back
    pub params: Vec<TypedParameter<'src>>, // Allow duplicates & multi-values
    pub values: Vec<PropertyValue<'src>>,
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

fn typed_property(prop: SyntaxProperty<'_>) -> Result<TypedProperty<'_>, Vec<Rich<'_, char>>> {
    let prop_name = prop.name.resolve().to_ascii_uppercase();
    let kind = kind_of(&prop_name, &prop);

    // TODO: cache parser
    let value = property_value(kind, prop.value)?;

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
        name: prop.name,
        params,
        values: vec![value],
    })
}

fn kind_of(prop_name: &str, prop: &SyntaxProperty) -> PropertyValueKind {
    // find VALUE= param
    let value_param = prop
        .params
        .iter()
        .find(|p| p.name.resolve().to_uppercase() == "VALUE")
        .and_then(|p| p.values.first())
        .map(|s| s.value.resolve().to_uppercase());

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
