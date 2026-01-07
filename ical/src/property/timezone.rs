// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Time Zone Component Properties (RFC 5545 Section 3.8.3)
//!
//! - 3.8.3.1: `TzId` - Time zone identifier
//! - 3.8.3.2: `TzName` - Time zone name
//! - 3.8.3.3: `TzOffsetFrom` - Time zone offset from standard time
//! - 3.8.3.4: `TzOffsetTo` - Time zone offset to daylight saving time
//! - 3.8.3.5: `TzUrl` - Time zone URL

use std::convert::TryFrom;

use crate::parameter::{Parameter, ValueType};
use crate::property::util::{Text, take_single_value};
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueUtcOffset};

simple_property_wrapper!(
    /// Simple text property wrapper for `TzId` (RFC 5545 Section 3.8.3.1)
    TzId<'src>: Text<'src> => TzId
);

simple_property_wrapper!(
    /// Simple text property wrapper for `TzName` (RFC 5545 Section 3.8.3.2)
    TzName<'src>: Text<'src> => TzName
);

/// UTC offset property with parameters (RFC 5545 Section 3.8.3.3 & 3.8.3.4)
///
/// This type implements `TryFrom<ParsedProperty>` for use with
/// the `simple_property_wrapper!` macro.
#[derive(Debug, Clone)]
pub struct UtcOffsetProperty<'src> {
    /// UTC offset value
    pub value: ValueUtcOffset,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for UtcOffsetProperty<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let kind = prop.kind.clone();

        match take_single_value(&kind, prop.value) {
            Ok(Value::UtcOffset { value, .. }) => Ok(Self {
                value,
                x_parameters,
                unrecognized_parameters,
            }),
            Ok(v) => {
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: kind,
                    expected: ValueType::UtcOffset,
                    found: v.into_kind(),
                    span,
                }])
            }
            Err(e) => Err(e),
        }
    }
}

simple_property_wrapper!(
    /// Time Zone Offset From property wrapper (RFC 5545 Section 3.8.3.3)
    TzOffsetFrom<'src>: UtcOffsetProperty<'src> => TzOffsetFrom
);

simple_property_wrapper!(
    /// Time Zone Offset To property wrapper (RFC 5545 Section 3.8.3.4)
    TzOffsetTo<'src>: UtcOffsetProperty<'src> => TzOffsetTo
);

simple_property_wrapper!(
    /// Simple text property wrapper for `TzUrl` (RFC 5545 Section 3.8.3.5)
    TzUrl<'src>: Text<'src> => TzUrl
);
