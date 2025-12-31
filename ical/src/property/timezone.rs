// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Time Zone Component Properties (RFC 5545 Section 3.8.3)
//!
//! This module contains property types for the "Time Zone Component Properties"
//! section of RFC 5545, including:
//! - 3.8.3.3 Time Zone Offset From
//! - 3.8.3.4 Time Zone Offset To

use std::convert::TryFrom;

use crate::semantic::SemanticError;
use crate::typed::{PropertyKind, TypedProperty, Value};

/// Timezone offset (RFC 5545 Section 3.8.3.3, 3.8.3.4)
#[derive(Debug, Clone, Copy)]
pub struct TimeZoneOffset {
    /// Whether the offset is positive
    pub positive: bool,

    /// Hours
    pub hours: u8,

    /// Minutes
    pub minutes: u8,
}

impl TimeZoneOffset {
    /// Try to convert from a Value with `PropertyKind` context
    ///
    /// # Errors
    ///
    /// Returns `Err` if the value is not a `UtcOffset`
    pub fn try_from_value(value: &Value<'_>, kind: PropertyKind) -> Result<Self, SemanticError> {
        match value {
            Value::UtcOffset(offset) => Ok(TimeZoneOffset {
                positive: offset.positive,
                hours: offset.hour,
                minutes: offset.minute,
            }),
            _ => Err(SemanticError::InvalidValue {
                property: kind,
                value: format!("Expected UTC offset value, got {value:?}"),
            }),
        }
    }
}

impl TryFrom<Value<'_>> for TimeZoneOffset {
    type Error = SemanticError;

    fn try_from(value: Value<'_>) -> Result<Self, Self::Error> {
        match value {
            Value::UtcOffset(offset) => Ok(TimeZoneOffset {
                positive: offset.positive,
                hours: offset.hour,
                minutes: offset.minute,
            }),
            _ => Err(SemanticError::InvalidValue {
                property: PropertyKind::TzOffsetFrom, // Default fallback
                value: format!("Expected UTC offset value, got {value:?}"),
            }),
        }
    }
}

impl<'src> TryFrom<TypedProperty<'src>> for TimeZoneOffset {
    type Error = Vec<SemanticError>;

    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
        let Some(value) = prop.values.first() else {
            return Err(vec![SemanticError::MissingValue {
                property: prop.kind,
            }]);
        };

        TimeZoneOffset::try_from_value(value, prop.kind).map_err(|e| vec![e])
    }
}
