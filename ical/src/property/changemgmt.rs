// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Change Management Component Properties (RFC 5545 Section 3.8.7)
//!
//! This module contains property types for the "Change Management Component Properties"
//! section of RFC 5545. These properties track the creation, modification, and
//! versioning of calendar components.
//!
//! - 3.8.7.1: `Created` - Date-time created
//! - 3.8.7.2: `DtStamp` - Date-time stamp
//! - 3.8.7.3: `LastModified` - Last modified
//! - 3.8.7.4: `Sequence` - Revision sequence number

use std::convert::TryFrom;

use crate::parameter::{Parameter, ValueType};
use crate::property::DateTime;
use crate::property::{PropertyKind, util::take_single_value};
use crate::typed::{ParsedProperty, TypedError};
use crate::value::Value;

simple_property_wrapper!(
    /// Created property wrapper (RFC 5545 Section 3.8.7.1)
    Created<'src>: DateTime<'src> => Created
);

simple_property_wrapper!(
    /// Date-Time Stamp property wrapper (RFC 5545 Section 3.8.7.2)
    DtStamp<'src>: DateTime<'src> => DtStamp
);

simple_property_wrapper!(
    /// Last Modified property wrapper (RFC 5545 Section 3.8.7.3)
    LastModified<'src>: DateTime<'src> => LastModified
);

/// Sequence Number (RFC 5545 Section 3.8.7.4)
///
/// This property defines the revision sequence number for the calendar component.
#[derive(Debug, Clone)]
pub struct Sequence<'src> {
    /// Sequence number
    pub value: i32,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Sequence<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Sequence) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Sequence,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        match take_single_value(&PropertyKind::Sequence, prop.value) {
            Ok(Value::Integer {
                values: mut ints, ..
            }) if ints.len() == 1 => Ok(Self {
                value: ints.pop().unwrap(),
                x_parameters,
                unrecognized_parameters,
            }),
            Ok(v) => {
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueType::Integer,
                    found: v.into_kind(),
                    span,
                }])
            }
            Err(e) => Err(e),
        }
    }
}
