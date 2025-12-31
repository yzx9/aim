// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Status Properties (RFC 5545 Section 3.8.1.11)
//!
//! This module contains status enum types for different calendar components.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::keyword::{
    KW_EVENT_STATUS_CANCELLED, KW_EVENT_STATUS_CONFIRMED, KW_EVENT_STATUS_TENTATIVE,
    KW_JOURNAL_STATUS_CANCELLED, KW_JOURNAL_STATUS_DRAFT, KW_JOURNAL_STATUS_FINAL,
    KW_TODO_STATUS_CANCELLED, KW_TODO_STATUS_COMPLETED, KW_TODO_STATUS_IN_PROCESS,
    KW_TODO_STATUS_NEEDS_ACTION,
};

/// Event status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStatus {
    /// Event is tentative
    Tentative,

    /// Event is confirmed
    Confirmed,

    /// Event is cancelled
    Cancelled,
}

impl FromStr for EventStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_EVENT_STATUS_TENTATIVE => Ok(Self::Tentative),
            KW_EVENT_STATUS_CONFIRMED => Ok(Self::Confirmed),
            KW_EVENT_STATUS_CANCELLED => Ok(Self::Cancelled),
            _ => Err(format!("Invalid event status: {s}")),
        }
    }
}

impl Display for EventStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tentative => KW_EVENT_STATUS_TENTATIVE.fmt(f),
            Self::Confirmed => KW_EVENT_STATUS_CONFIRMED.fmt(f),
            Self::Cancelled => KW_EVENT_STATUS_CANCELLED.fmt(f),
        }
    }
}

impl AsRef<str> for EventStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::Tentative => KW_EVENT_STATUS_TENTATIVE,
            Self::Confirmed => KW_EVENT_STATUS_CONFIRMED,
            Self::Cancelled => KW_EVENT_STATUS_CANCELLED,
        }
    }
}

/// To-do status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoStatus {
    /// To-do needs action
    NeedsAction,

    /// To-do is completed
    Completed,

    /// To-do is in process
    InProcess,

    /// To-do is cancelled
    Cancelled,
}

impl FromStr for TodoStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_TODO_STATUS_NEEDS_ACTION => Ok(Self::NeedsAction),
            KW_TODO_STATUS_COMPLETED => Ok(Self::Completed),
            KW_TODO_STATUS_IN_PROCESS => Ok(Self::InProcess),
            KW_TODO_STATUS_CANCELLED => Ok(Self::Cancelled),
            _ => Err(format!("Invalid todo status: {s}")),
        }
    }
}

impl Display for TodoStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeedsAction => KW_TODO_STATUS_NEEDS_ACTION.fmt(f),
            Self::Completed => KW_TODO_STATUS_COMPLETED.fmt(f),
            Self::InProcess => KW_TODO_STATUS_IN_PROCESS.fmt(f),
            Self::Cancelled => KW_TODO_STATUS_CANCELLED.fmt(f),
        }
    }
}

impl AsRef<str> for TodoStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::NeedsAction => KW_TODO_STATUS_NEEDS_ACTION,
            Self::Completed => KW_TODO_STATUS_COMPLETED,
            Self::InProcess => KW_TODO_STATUS_IN_PROCESS,
            Self::Cancelled => KW_TODO_STATUS_CANCELLED,
        }
    }
}

/// Journal status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalStatus {
    /// Journal entry is draft
    Draft,

    /// Journal entry is final
    Final,

    /// Journal entry is cancelled
    Cancelled,
}

impl FromStr for JournalStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_JOURNAL_STATUS_DRAFT => Ok(Self::Draft),
            KW_JOURNAL_STATUS_FINAL => Ok(Self::Final),
            KW_JOURNAL_STATUS_CANCELLED => Ok(Self::Cancelled),
            _ => Err(format!("Invalid journal status: {s}")),
        }
    }
}

impl Display for JournalStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => KW_JOURNAL_STATUS_DRAFT.fmt(f),
            Self::Final => KW_JOURNAL_STATUS_FINAL.fmt(f),
            Self::Cancelled => KW_JOURNAL_STATUS_CANCELLED.fmt(f),
        }
    }
}

impl AsRef<str> for JournalStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::Draft => KW_JOURNAL_STATUS_DRAFT,
            Self::Final => KW_JOURNAL_STATUS_FINAL,
            Self::Cancelled => KW_JOURNAL_STATUS_CANCELLED,
        }
    }
}
