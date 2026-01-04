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

use crate::parameter::ValueKind;
use crate::property::util::take_single_value;
use crate::property::{DateTime, PropertyKind};
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
#[derive(Debug, Clone, Copy)]
pub struct Sequence {
    /// Sequence number
    pub value: i32,
}

impl Sequence {
    /// Get the property kind for `Sequence`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Sequence
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Sequence {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        match take_single_value(Self::kind(), prop.values) {
            Ok((Value::Integer(value), _)) => Ok(Self { value }),
            Ok((v, span)) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::Integer,
                found: v.kind(),
                span,
            }]),
            Err(e) => Err(vec![e]),
        }
    }
}
