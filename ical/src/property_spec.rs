// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use crate::keyword::{KW_CATEGORIES, KW_DTEND, KW_DTSTART, KW_DURATION, KW_RRULE};
use crate::property_value::PropertyValueKind;

#[derive(Debug, Clone)]
pub struct PropertySpec<'a> {
    pub name: &'a str,
    pub default_kind: PropertyValueKind,
    #[allow(dead_code)]
    pub allowed_kinds: &'a [PropertyValueKind],
    #[allow(dead_code)]
    pub multiple_valued: bool,
}

pub static PROPERTY_SPECS: &[PropertySpec] = &[
    PropertySpec {
        name: KW_DTSTART,
        default_kind: PropertyValueKind::DateTime,
        allowed_kinds: &[PropertyValueKind::Date, PropertyValueKind::DateTime],
        multiple_valued: false,
    },
    PropertySpec {
        name: KW_DTEND,
        default_kind: PropertyValueKind::DateTime,
        allowed_kinds: &[PropertyValueKind::Date, PropertyValueKind::DateTime],
        multiple_valued: false,
    },
    PropertySpec {
        name: KW_DURATION,
        default_kind: PropertyValueKind::Duration,
        allowed_kinds: &[PropertyValueKind::Duration],
        multiple_valued: false,
    },
    PropertySpec {
        name: KW_RRULE,
        default_kind: PropertyValueKind::Rrule,
        allowed_kinds: &[PropertyValueKind::Rrule],
        multiple_valued: false,
    },
    PropertySpec {
        name: KW_CATEGORIES,
        default_kind: PropertyValueKind::Text,
        allowed_kinds: &[PropertyValueKind::Text],
        multiple_valued: true,
    },
    // TODO: ...
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_unique_property_names() {
        let names = PROPERTY_SPECS
            .iter()
            .map(|spec| spec.name)
            .collect::<HashSet<_>>();

        assert_eq!(
            names.len(),
            PROPERTY_SPECS.len(),
            "Property names should be unique"
        );
    }

    #[test]
    fn test_property_specs() {
        for spec in PROPERTY_SPECS {
            let name = spec.name;
            assert!(!name.is_empty(), "Property name should not be empty");
            assert!(
                name.chars().all(|a| a.is_ascii_uppercase()),
                "Property {name}: name should be uppercase ASCII"
            );
            assert!(
                spec.allowed_kinds.contains(&spec.default_kind),
                "Property {name}: default_kind should be in allowed_kinds"
            );
        }
    }
}
