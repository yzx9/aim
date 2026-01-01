// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Time Transparency Property (RFC 5545 Section 3.8.2.7)

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::keyword::{KW_TRANSP_OPAQUE, KW_TRANSP_TRANSPARENT};
use crate::property::PropertyKind;
use crate::property::util::take_single_string;
use crate::typed::{ParsedProperty, TypedError};

/// Time transparency for events (RFC 5545 Section 3.8.2.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeTransparency {
    /// Event blocks time
    #[default]
    Opaque,

    /// Event does not block time
    Transparent,
}

impl FromStr for TimeTransparency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_TRANSP_OPAQUE => Ok(Self::Opaque),
            KW_TRANSP_TRANSPARENT => Ok(Self::Transparent),
            _ => Err(format!("Invalid time transparency: {s}")),
        }
    }
}

impl Display for TimeTransparency {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Opaque => KW_TRANSP_OPAQUE.fmt(f),
            Self::Transparent => KW_TRANSP_TRANSPARENT.fmt(f),
        }
    }
}

impl AsRef<str> for TimeTransparency {
    fn as_ref(&self) -> &str {
        match self {
            Self::Opaque => KW_TRANSP_OPAQUE,
            Self::Transparent => KW_TRANSP_TRANSPARENT,
        }
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TimeTransparency {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let text = take_single_string(prop.kind, prop.values).map_err(|e| vec![e])?;
        text.parse().map_err(|e| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Transp,
                value: e,
                span: prop.span,
            }]
        })
    }
}
