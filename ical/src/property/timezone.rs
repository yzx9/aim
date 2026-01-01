// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Time Zone Component Properties (RFC 5545 Section 3.8.3)
//!
//! This module contains property types for the "Time Zone Component Properties"
//! section of RFC 5545. Each property type implements `Deref` and `DerefMut`
//! for convenient access to the underlying UTC offset value, and includes
//! a `kind()` method for property validation:
//!
//! - 3.8.3.3: `TzOffsetFrom` - Time zone offset from standard time
//! - 3.8.3.4: `TzOffsetTo` - Time zone offset to daylight saving time
//!
//! Both properties validate their kind during conversion, ensuring type
//! safety throughout the parsing pipeline.

use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

use crate::parameter::ValueKind;
use crate::property::PropertyKind;
use crate::property::util::take_single_value;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueUtcOffset};

/// Time Zone Offset From property wrapper (RFC 5545 Section 3.8.3.3)
#[derive(Debug, Clone, Copy)]
pub struct TzOffsetFrom(ValueUtcOffset);

impl TzOffsetFrom {
    /// Get the property kind for `TzOffsetFrom`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::TzOffsetFrom
    }
}

impl Deref for TzOffsetFrom {
    type Target = ValueUtcOffset;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TzOffsetFrom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TzOffsetFrom {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        match take_single_value(prop.kind, prop.values) {
            Ok(Value::UtcOffset(offset)) => Ok(Self(offset)),
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

/// Time Zone Offset To property wrapper (RFC 5545 Section 3.8.3.4)
#[derive(Debug, Clone, Copy)]
pub struct TzOffsetTo(ValueUtcOffset);

impl TzOffsetTo {
    /// Get the property kind for `TzOffsetTo`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::TzOffsetTo
    }
}

impl Deref for TzOffsetTo {
    type Target = ValueUtcOffset;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TzOffsetTo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TzOffsetTo {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }
        match take_single_value(prop.kind, prop.values) {
            Ok(Value::UtcOffset(offset)) => Ok(Self(offset)),
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
