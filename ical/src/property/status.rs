// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Status Properties (RFC 5545 Section 3.8.1.11)
//!
//! This module contains status enum types for different calendar components.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::keyword::{
    KW_STATUS_CANCELLED, KW_STATUS_COMPLETED, KW_STATUS_CONFIRMED, KW_STATUS_DRAFT,
    KW_STATUS_FINAL, KW_STATUS_IN_PROCESS, KW_STATUS_NEEDS_ACTION, KW_STATUS_TENTATIVE,
};
use crate::semantic::SemanticError;
use crate::typed::{PropertyKind, TypedProperty, Value, ValueType};

/// Event/To-do/Journal status (RFC 5545 Section 3.8.1.11)
///
/// This enum represents the status of calendar components such as events,
/// to-dos, and journal entries. Each variant corresponds to a specific status
/// defined in the iCalendar specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Event is tentative
    Tentative,

    /// Event is confirmed
    Confirmed,

    /// To-do needs action
    NeedsAction,

    /// To-do is completed
    Completed,

    /// To-do is in process
    InProcess,

    /// Journal entry is draft
    Draft,

    /// Journal entry is final
    Final,

    /// Event/To-do/Journal is cancelled
    Cancelled,
}

impl FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_STATUS_TENTATIVE => Ok(Self::Tentative),
            KW_STATUS_CONFIRMED => Ok(Self::Confirmed),
            KW_STATUS_NEEDS_ACTION => Ok(Self::NeedsAction),
            KW_STATUS_COMPLETED => Ok(Self::Completed),
            KW_STATUS_IN_PROCESS => Ok(Self::InProcess),
            KW_STATUS_DRAFT => Ok(Self::Draft),
            KW_STATUS_FINAL => Ok(Self::Final),
            KW_STATUS_CANCELLED => Ok(Self::Cancelled),
            _ => Err(format!("Invalid status: {s}")),
        }
    }
}

impl AsRef<str> for Status {
    fn as_ref(&self) -> &str {
        match self {
            Self::Tentative => KW_STATUS_TENTATIVE,
            Self::Confirmed => KW_STATUS_CONFIRMED,
            Self::NeedsAction => KW_STATUS_NEEDS_ACTION,
            Self::Completed => KW_STATUS_COMPLETED,
            Self::InProcess => KW_STATUS_IN_PROCESS,
            Self::Draft => KW_STATUS_DRAFT,
            Self::Final => KW_STATUS_FINAL,
            Self::Cancelled => KW_STATUS_CANCELLED,
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'src> TryFrom<TypedProperty<'src>> for Status {
    type Error = Vec<SemanticError>;

    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                vec![SemanticError::UnexpectedType {
                    property: PropertyKind::Status,
                    expected: ValueType::Text,
                }]
            })?;

        text.parse().map_err(|e| {
            vec![SemanticError::InvalidValue {
                property: PropertyKind::Status,
                value: e,
            }]
        })
    }
}
