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
use crate::property::common::{TextOnly, TextWithLanguage, UriProperty, take_single_value};
use crate::string_storage::{SpannedSegments, StringStorage};
use crate::syntax::RawParameter;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueUtcOffset};

simple_property_wrapper!(
    /// Plain text property wrapper for `TzId` (RFC 5545 Section 3.8.3.1)
    ///
    /// Per RFC 5545, TZID does not support any standard parameters.
    pub TzId<S> => TextOnly
);

simple_property_wrapper!(
    /// Text property wrapper for `TzName` (RFC 5545 Section 3.8.3.2)
    ///
    /// Per RFC 5545, TZNAME supports the LANGUAGE parameter but not ALTREP.
    pub TzName<S> => TextWithLanguage
);

/// UTC offset property with parameters (RFC 5545 Section 3.8.3.3 & 3.8.3.4)
///
/// This type implements `TryFrom<ParsedProperty>` for use with
/// the `simple_property_wrapper!` macro.
#[derive(Debug, Clone)]
pub struct UtcOffsetProperty<S: StringStorage> {
    /// UTC offset value
    pub value: ValueUtcOffset,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for UtcOffsetProperty<SpannedSegments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let mut x_parameters = Vec::new();
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        let kind = prop.kind.clone();

        match take_single_value(&kind, prop.value) {
            Ok(Value::UtcOffset { value, .. }) => Ok(Self {
                value,
                x_parameters,
                retained_parameters,
            }),
            Ok(v) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: kind,
                expected: ValueType::UtcOffset,
                found: v.kind().into(),
                span: v.span(),
            }]),
            Err(e) => Err(e),
        }
    }
}

impl UtcOffsetProperty<SpannedSegments<'_>> {
    /// Convert borrowed `UtcOffsetProperty` to owned `UtcOffsetProperty`
    #[must_use]
    pub fn to_owned(&self) -> UtcOffsetProperty<String> {
        UtcOffsetProperty {
            value: self.value,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
        }
    }
}

simple_property_wrapper!(
    /// Time Zone Offset From property wrapper (RFC 5545 Section 3.8.3.3)
    pub TzOffsetFrom<S> => UtcOffsetProperty
);

simple_property_wrapper!(
    /// Time Zone Offset To property wrapper (RFC 5545 Section 3.8.3.4)
    pub TzOffsetTo<S> => UtcOffsetProperty
);

simple_property_wrapper!(
    /// URI property wrapper for `TzUrl` (RFC 5545 Section 3.8.3.5)
    pub TzUrl<S> => UriProperty
);
