// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Enumeration types for iCalendar semantic components.
//!
//! This module contains shared enums used across multiple components.
//! Component-specific enums are defined in their respective component modules.

use crate::semantic::{DateTime, Duration, Uri};

/// Classification of calendar data
#[derive(Debug, Clone, Copy)]
pub enum Classification {
    /// Public classification
    Public,

    /// Private classification
    Private,

    /// Confidential classification
    Confidential,
    // /// Custom classification
    // Custom(String),
}

/// Period of time (start-end or start-duration)
#[derive(Debug, Clone)]
pub enum Period {
    /// Start and end date/time
    DateTimeRange {
        /// Start of the period
        start: DateTime,
        /// End of the period
        end: DateTime,
    },

    /// Start date/time and duration
    Duration {
        /// Start of the period
        start: DateTime,
        /// Duration from the start
        duration: Duration,
    },
}

/// Attachment value (URI or binary)
#[derive(Debug, Clone)]
pub enum AttachmentValue {
    /// URI reference
    Uri(Uri),

    /// Binary data
    Binary(Vec<u8>),
}
