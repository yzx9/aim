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

use crate::parameter::ValueType;
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

impl<'src> TryFrom<ParsedProperty<'src>> for TzId<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::TzId) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::TzId,
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

impl<'src> TryFrom<ParsedProperty<'src>> for TzName<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::TzName) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::TzName,
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
        if !matches!(prop.kind, PropertyKind::TzOffsetFrom) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::TzOffsetFrom,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        match take_single_value(&prop.kind, prop.values) {
            Ok((Value::UtcOffset(offset), _)) => Ok(Self(offset)),
            Ok((v, span)) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueType::UtcOffset,
                found: v.into_kind(),
                span,
            }]),
            Err(e) => Err(e),
        }
    }
}

/// Time Zone Offset To property wrapper (RFC 5545 Section 3.8.3.4)
#[derive(Debug, Clone, Copy)]
pub struct TzOffsetTo(ValueUtcOffset);

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
        if !matches!(prop.kind, PropertyKind::TzOffsetTo) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::TzOffsetTo,
                found: prop.kind,
                span: prop.span,
            }]);
        }
        match take_single_value(&prop.kind, prop.values) {
            Ok((Value::UtcOffset(offset), _)) => Ok(Self(offset)),
            Ok((v, span)) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueType::UtcOffset,
                found: v.into_kind(),
                span,
            }]),
            Err(e) => Err(e),
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

impl<'src> TryFrom<ParsedProperty<'src>> for TzUrl<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::TzUrl) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::TzUrl,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(TzUrl)
    }
}
