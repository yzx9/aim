// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Operations on iCalendar components.
//!
//! This module provides extension traits for performing operations on parsed
//! iCalendar components, such as `RRule` expansion and conflict detection.
//!
//! # Modules
//!
//! - [`rrule`] - `RRule` expansion and computation utilities
//! - [`conflict`] - Event conflict detection utilities

pub mod conflict;
pub mod rrule;

#[cfg(feature = "jiff")]
pub use conflict::ConflictExt;
#[cfg(feature = "jiff")]
pub use rrule::{DateRange, EventOccurrence, RRuleExt, VEventExt};
