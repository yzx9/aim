// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use jiff::civil::Time;

/// NOTE: Used for storing in the database, so it should be stable across different runs.
pub const STABLE_FORMAT_DATEONLY: &str = "%Y-%m-%d";
pub const STABLE_FORMAT_FLOATING: &str = "%Y-%m-%dT%H:%M:%S";
pub const STABLE_FORMAT_LOCAL: &str = "%Y-%m-%dT%H:%M:%S%z";

/// The position of a date relative to a range defined by a start and optional end date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangePosition {
    /// The date is before the start of the range.
    Before,
    /// The date is within the range.
    InRange,
    /// The date is after the start of the range.
    After,
    /// The range is invalid, e.g., start date is after end date.
    InvalidRange,
}

pub const fn start_of_day() -> Time {
    Time::constant(0, 0, 0, 0)
}

/// Using a leap second to represent the end of the day
pub const fn end_of_day() -> Time {
    Time::constant(23, 59, 59, 999_999_999)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_start_of_day() {
        let time = start_of_day();
        assert!(time.hour() == 0);
        assert!(time.minute() == 0);
        assert!(time.second() == 0);
    }

    #[test]
    fn returns_end_of_day() {
        let time = end_of_day();
        assert!(time.hour() == 23);
        assert!(time.minute() == 59);
        assert!(time.second() == 59);
    }

    #[test]
    fn validates_day_boundary_constants() {
        // Test that the constants are what we expect
        let start = start_of_day();
        let end = end_of_day();

        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);
        assert_eq!(start.subsec_nanosecond(), 0);

        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
        assert_eq!(end.second(), 59);
        assert_eq!(end.subsec_nanosecond(), 999_999_999);
    }
}
