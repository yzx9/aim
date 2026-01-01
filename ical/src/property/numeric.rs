// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Numeric property types.
//!
//! This module contains property types that represent numeric values.
//! All types implement `kind()` methods and validate their property kind
//! during conversion from `ParsedProperty`:
//!
//! - 3.8.1.8: `PercentComplete` - Percent complete for todos (0-100)
//! - 3.8.1.9: `Priority` - Priority level (0-9, undefined = 0)
//! - 3.8.2.5: `Duration` - Duration value
//! - 3.8.6.2: `Repeat` - Alarm repeat count
//! - 3.8.7.4: `Sequence` - Revision sequence number

use std::convert::TryFrom;

use crate::parameter::ValueKind;
use crate::property::PropertyKind;
use crate::property::util::take_single_value;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueDuration};

/// Percent Complete (RFC 5545 Section 3.8.1.8)
///
/// This property defines the percent complete for a todo.
/// Value must be between 0 and 100.
#[derive(Debug, Clone, Copy)]
pub struct PercentComplete {
    /// Percent complete (0-100)
    pub value: u8,
}

impl PercentComplete {
    /// Get the property kind for `PercentComplete`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::PercentComplete
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for PercentComplete {
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
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok(Value::Integer(i)) if (0..=100).contains(&i) => Ok(Self { value: i as u8 }),
            Ok(Value::Integer(_)) => Err(vec![TypedError::PropertyInvalidValue {
                property: prop.kind,
                value: "Percent complete must be 0-100".to_string(),
                span: prop.span,
            }]),
            Ok(v) => {
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueKind::Integer,
                    found: v.kind(),
                    span: 0..0, // TODO: improve span reporting
                }])
            }
            Err(e) => Err(vec![e]),
        }
    }
}

/// Priority (RFC 5545 Section 3.8.1.9)
///
/// This property defines the priority for a calendar component.
/// Value must be between 0 and 9, where 0 defines an undefined priority.
#[derive(Debug, Clone, Copy)]
pub struct Priority {
    /// Priority value (0-9, where 0 is undefined)
    pub value: u8,
}

impl Priority {
    /// Get the property kind for `Priority`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Priority
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Priority {
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
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok(Value::Integer(i)) if (0..=9).contains(&i) => Ok(Self { value: i as u8 }),
            Ok(Value::Integer(_)) => Err(vec![TypedError::PropertyInvalidValue {
                property: prop.kind,
                value: "Priority must be 0-9".to_string(),
                span: prop.span,
            }]),
            Ok(v) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::Integer,
                found: v.kind(),
                span: 0..0, // TODO: improve span reporting
            }]),
            Err(e) => Err(vec![e]),
        }
    }
}

/// Duration (RFC 5545 Section 3.8.2.5)
///
/// This property specifies a duration of time.
#[derive(Debug, Clone, Copy)]
pub struct Duration {
    /// Duration value
    pub value: ValueDuration,
}

impl Duration {
    /// Get the property kind for `Duration`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Duration
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Duration {
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
            Ok(Value::Duration(d)) => Ok(Self { value: d }),
            Ok(v) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::Duration,
                found: v.kind(),
                span: prop.span,
            }]),
            Err(e) => Err(vec![e]),
        }
    }
}

/// Repeat Count (RFC 5545 Section 3.8.6.2)
///
/// This property defines the number of times the alarm should repeat.
#[derive(Debug, Clone, Copy)]
pub struct Repeat {
    /// Number of repetitions
    pub value: u32,
}

impl Repeat {
    /// Get the property kind for `Repeat`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Repeat
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Repeat {
    type Error = Vec<TypedError<'src>>;

    #[expect(clippy::cast_sign_loss)]
    fn try_from(mut prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        match prop.values.len() {
            0 => Err(vec![TypedError::PropertyMissingValue {
                property: prop.kind,
                span: prop.span,
            }]),
            1 => match prop.values.pop().unwrap() {
                Value::Integer(i) if i > 0 => Ok(Self { value: i as u32 }),
                Value::Integer(i) => Err(vec![TypedError::PropertyInvalidValue {
                    property: prop.kind,
                    value: format!("Repeat count must be non-negative: {i}"),
                    span: prop.span,
                }]),
                v => Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueKind::Integer,
                    found: v.kind(),
                    span: prop.span,
                }]),
            },
            len => Err(vec![TypedError::PropertyInvalidValueCount {
                property: PropertyKind::Repeat,
                expected: 1,
                found: len,
                span: prop.span,
            }]),
        }
    }
}

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
            Ok(Value::Integer(value)) => Ok(Self { value }),
            Ok(v) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::Integer,
                found: v.kind(),
                span: 0..0, // TODO: improve span reporting
            }]),
            Err(e) => Err(vec![e]),
        }
    }
}
