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

use crate::parameter::ValueKind;
use crate::property::util::take_single_value;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::Value;

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

impl<'src> TryFrom<ParsedProperty<'src>> for TimeZoneOffset {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        match take_single_value(prop.kind, prop.values) {
            Ok(Value::UtcOffset(offset)) => Ok(TimeZoneOffset {
                positive: offset.positive,
                hours: offset.hour,
                minutes: offset.minute,
            }),
            Ok(v) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::UtcOffset,
                found: v.kind(),
                span: prop.span,
            }]),
            Err(e) => Err(vec![e]),
        }
    }
}
