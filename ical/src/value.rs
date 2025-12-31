// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Value type parsing module for iCalendar property values.
//!
//! This module handles the parsing and validation of iCalendar value types
//! as defined in RFC 5545 Section 3.3.

mod ast;
mod datetime;
mod duration;
pub(crate) mod numeric;
mod period;
mod rrule;
mod text;

pub use ast::{Value, ValueExpected, parse_values};
pub use datetime::{ValueDate, ValueDateTime, ValueTime, ValueUtcOffset};
pub use duration::ValueDuration;
pub use numeric::values_float_semicolon;
pub use period::ValuePeriod;
pub use rrule::{Day, RecurrenceFrequency, RecurrenceRule, WeekDay};
pub use text::ValueText;
