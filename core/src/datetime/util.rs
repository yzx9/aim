// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, NaiveDateTime, NaiveTime, TimeZone, Utc, offset::LocalResult};

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

pub const fn start_of_day_naive() -> NaiveTime {
    NaiveTime::from_hms_opt(0, 0, 0).expect("00:00:00 must exist in NaiveTime")
}

/// Using a leap second to represent the end of the day
pub const fn end_of_day_naive() -> NaiveTime {
    NaiveTime::from_hms_nano_opt(23, 59, 59, 1_999_999_999)
        .expect("23:59:59:1_999_999_999 must exist in NaiveTime")
}

/// The start of the day (00:00:00) for the given `DateTime` in the same timezone.
pub fn start_of_day<Tz: TimeZone>(dt: &DateTime<Tz>) -> DateTime<Tz> {
    let naive = NaiveDateTime::new(dt.date_naive(), start_of_day_naive());
    from_local_datetime(&dt.timezone(), naive)
}

/// The end of the day (23:59:59) for the given `DateTime` in the same timezone.
pub fn end_of_day<Tz: TimeZone>(dt: &DateTime<Tz>) -> DateTime<Tz> {
    let last_nano_sec = end_of_day_naive();
    let naive = NaiveDateTime::new(dt.date_naive(), last_nano_sec);
    from_local_datetime(&dt.timezone(), naive)
}

/// Convert the `NaiveDateTime` to the local timezone, handles local time ambiguities:
/// - `Single(dt)` returns directly;
/// - `Ambiguous(a, b)` takes the earlier one;
/// - `None` (local time does not exist, e.g., due to DST transition): falls back to UTC
///   combination and then converts.
pub fn from_local_datetime<Tz: TimeZone>(tz: &Tz, naive: NaiveDateTime) -> DateTime<Tz> {
    match tz.from_local_datetime(&naive) {
        LocalResult::Single(x) => x,
        LocalResult::Ambiguous(a, b) => {
            // Choose the earlier one
            if a <= b { a } else { b }
        }
        LocalResult::None => Utc.from_utc_datetime(&naive).with_timezone(tz),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn returns_start_of_day_constant() {
        let time = start_of_day_naive();
        assert!(time <= NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn returns_end_of_day_constant() {
        let time = end_of_day_naive();
        assert!(time >= NaiveTime::from_hms_opt(23, 59, 59).unwrap());
    }

    #[test]
    fn returns_start_of_day_for_datetime() {
        let dt = Utc.with_ymd_and_hms(2025, 1, 15, 14, 30, 45).unwrap();
        let start = start_of_day(&dt);
        assert_eq!(start.date_naive(), dt.date_naive());
        assert!(start <= Utc.with_ymd_and_hms(2025, 1, 15, 14, 30, 45).unwrap());
    }

    #[test]
    fn returns_end_of_day_for_datetime() {
        let dt = Utc.with_ymd_and_hms(2025, 1, 15, 14, 30, 45).unwrap();
        let end = end_of_day(&dt);
        assert_eq!(end.date_naive(), dt.date_naive());
        assert!(end >= Utc.with_ymd_and_hms(2025, 1, 15, 14, 30, 45).unwrap());
    }

    #[test]
    fn converts_naive_to_datetime_in_timezone() {
        let tz = Utc;
        // Use the newer DateTime::from_timestamp instead of deprecated NaiveDateTime::from_timestamp_opt
        let dt = DateTime::from_timestamp(1_609_459_200, 0).unwrap(); // 2021-01-01 00:00:00 UTC
        let naive = dt.naive_utc();
        let result = from_local_datetime(&tz, naive);
        assert_eq!(result.timestamp(), 1_609_459_200);
    }

    #[test]
    fn validates_day_boundary_constants() {
        // Test that the constants are what we expect
        let start = start_of_day_naive();
        let end = end_of_day_naive();

        assert_eq!(start, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        assert_eq!(
            end,
            NaiveTime::from_hms_nano_opt(23, 59, 59, 1_999_999_999).unwrap()
        );
    }
}
