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
use crate::property::common::take_single_value;
use crate::property::{DateTimeUtc, PropertyKind};
use crate::string_storage::{Segments, StringStorage};
use crate::syntax::RawParameter;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::Value;

simple_property_wrapper!(
    /// Created property wrapper (RFC 5545 Section 3.8.7.1)
    ///
    /// This property MUST be specified in UTC time format.
    pub Created<S> => DateTimeUtc
);

simple_property_wrapper!(
    /// Date-Time Stamp property wrapper (RFC 5545 Section 3.8.7.2)
    ///
    /// This property MUST be specified in UTC time format.
    pub DtStamp<S> => DateTimeUtc
);

simple_property_wrapper!(
    /// Last Modified property wrapper (RFC 5545 Section 3.8.7.3)
    ///
    /// This property MUST be specified in UTC time format.
    pub LastModified<S> => DateTimeUtc
);

/// Sequence Number (RFC 5545 Section 3.8.7.4)
///
/// This property defines the revision sequence number for the calendar component.
#[derive(Debug, Clone)]
pub struct Sequence<S: StringStorage> {
    /// Sequence number
    pub value: u32,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Sequence<Segments<'src>> {
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

        match take_single_value(&PropertyKind::Sequence, prop.value) {
            Ok(Value::Integer {
                values: mut ints, ..
            }) => {
                if ints.len() != 1 {
                    return Err(vec![TypedError::PropertyInvalidValueCount {
                        property: prop.kind,
                        expected: 1,
                        found: ints.len(),
                        span: prop.span,
                    }]);
                }

                let value = ints.pop().unwrap(); // SAFETY: checked length above
                if value < 0 {
                    return Err(vec![TypedError::PropertyInvalidValue {
                        property: PropertyKind::Sequence,
                        value: format!("Sequence must be non-negative: {value}"),
                        span: prop.span,
                    }]);
                }

                #[allow(clippy::cast_sign_loss)]
                Ok(Self {
                    value: value as u32, // SAFETY: i < i32::MAX < u32::MAX
                    x_parameters,
                    retained_parameters,
                    span: prop.span,
                })
            }
            Ok(v) => {
                const EXPECTED: &[ValueType<String>] = &[ValueType::Integer];
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: EXPECTED,
                    found: v.kind().into(),
                    span: v.span(),
                }])
            }
            Err(e) => Err(e),
        }
    }
}

impl Sequence<Segments<'_>> {
    /// Convert borrowed `Sequence` to owned `Sequence`
    #[must_use]
    pub fn to_owned(&self) -> Sequence<String> {
        Sequence {
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
            span: (),
        }
    }
}

impl<S: StringStorage> Sequence<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}
