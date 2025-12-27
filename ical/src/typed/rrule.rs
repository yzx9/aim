// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Recurrence rule type definitions for iCalendar.

use crate::typed::ValueDateTime;

/// Recurrence rule
#[derive(Debug, Clone)]
pub struct RecurrenceRule {
    /// Frequency of recurrence
    pub freq: RecurrenceFrequency,

    /// Until date for recurrence
    pub until: Option<ValueDateTime>,

    /// Number of occurrences
    pub count: Option<u32>,

    /// Interval between recurrences
    pub interval: Option<u32>,

    /// Second specifier
    pub by_second: Vec<u8>,

    /// Minute specifier
    pub by_minute: Vec<u8>,

    /// Hour specifier
    pub by_hour: Vec<u8>,

    /// Day of month specifier
    pub by_month_day: Vec<i8>,

    /// Day of year specifier
    pub by_year_day: Vec<i16>,

    /// Week number specifier
    pub by_week_no: Vec<i8>,

    /// Month specifier
    pub by_month: Vec<u8>,

    /// Day of week specifier
    pub by_day: Vec<WeekDay>,

    /// Position in month
    pub by_set_pos: Vec<i16>,

    /// Start day of week
    pub wkst: Option<WeekDay>,
}

/// Recurrence frequency
#[derive(Debug, Clone, Copy)]
pub enum RecurrenceFrequency {
    /// Secondly
    Secondly,

    /// Minutely
    Minutely,

    /// Hourly
    Hourly,

    /// Daily
    Daily,

    /// Weekly
    Weekly,

    /// Monthly
    Monthly,

    /// Yearly
    Yearly,
}

/// Day of week with optional occurrence
#[derive(Debug, Clone, Copy)]
pub struct WeekDay {
    /// Day of the week
    pub day: Day,

    /// Occurrence in month (optional)
    pub occurrence: Option<i8>,
}

/// Day of the week
#[derive(Debug, Clone, Copy)]
pub enum Day {
    /// Sunday
    Sunday,

    /// Monday
    Monday,

    /// Tuesday
    Tuesday,

    /// Wednesday
    Wednesday,

    /// Thursday
    Thursday,

    /// Friday
    Friday,

    /// Saturday
    Saturday,
}
