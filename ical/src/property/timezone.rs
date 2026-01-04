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
use std::ops::{Deref, DerefMut};

use crate::parameter::ValueKind;
use crate::property::PropertyKind;
use crate::property::util::{Text, take_single_value};
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueUtcOffset};

/// Simple text property wrapper for `TzId` (RFC 5545 Section 3.8.3.1)
#[derive(Debug, Clone)]
pub struct TzId<'src>(pub Text<'src>);

impl<'src> Deref for TzId<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TzId<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TzId<'_> {
    /// Get the property kind for `TzId`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::TzId
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TzId<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(TzId)
    }
}

/// Simple text property wrapper for `TzName` (RFC 5545 Section 3.8.3.2)
#[derive(Debug, Clone)]
pub struct TzName<'src>(pub Text<'src>);

impl<'src> Deref for TzName<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TzName<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TzName<'_> {
    /// Get the property kind for `TzName`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::TzName
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TzName<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(TzName)
    }
}

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
            Ok((Value::UtcOffset(offset), _)) => Ok(Self(offset)),
            Ok((v, span)) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::UtcOffset,
                found: v.kind(),
                span,
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
            Ok((Value::UtcOffset(offset), _)) => Ok(Self(offset)),
            Ok((v, span)) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::UtcOffset,
                found: v.kind(),
                span,
            }]),
            Err(e) => Err(vec![e]),
        }
    }
}

/// Simple text property wrapper for `TzUrl` (RFC 5545 Section 3.8.3.5)
#[derive(Debug, Clone)]
pub struct TzUrl<'src>(pub Text<'src>);

impl<'src> Deref for TzUrl<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TzUrl<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TzUrl<'_> {
    /// Get the property kind for `TzUrl`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::TzUrl
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TzUrl<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(TzUrl)
    }
}
