// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use chrono::offset::LocalResult;
use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};

use crate::LooseDateTime;
use crate::datetime::util::{end_of_day, from_local_datetime, start_of_day};

/// Represents a date and time anchor that can be used to calculate relative dates and times.
#[derive(Debug, Clone, Copy)]
pub enum DateTimeAnchor {
    /// A specific number of hours in the future or past.
    InHours(i64),

    /// A specific number of days in the future or past.
    InDays(i64),

    /// A specific date and time.
    DateTime(LooseDateTime),

    /// A specific time.
    Time(NaiveTime),
}

impl DateTimeAnchor {
    /// Represents the current time.
    pub fn now() -> Self {
        DateTimeAnchor::InHours(0)
    }

    /// Represents the current date.
    pub fn today() -> Self {
        DateTimeAnchor::InDays(0)
    }

    /// Represents tomorrow, which is one day after today.
    pub fn tomorrow() -> Self {
        DateTimeAnchor::InDays(1)
    }

    /// Represents yesterday, which is one day before today.
    pub fn yesterday() -> Self {
        DateTimeAnchor::InDays(-1)
    }

    /// Parses the `DateTimeAnchor` enum based on the current time.
    pub fn parse_as_start_of_day<Tz: TimeZone>(&self, now: &DateTime<Tz>) -> DateTime<Tz> {
        match self {
            DateTimeAnchor::InHours(n) => now.clone() + TimeDelta::hours(*n),
            DateTimeAnchor::InDays(n) => start_of_day(now) + TimeDelta::days(*n),
            DateTimeAnchor::DateTime(t) => {
                let naive = t.with_start_of_day();
                from_local_datetime(&now.timezone(), naive)
            }
            DateTimeAnchor::Time(t) => {
                let naive = NaiveDateTime::new(now.date_naive(), *t);
                from_local_datetime(&now.timezone(), naive)
            }
        }
    }

    /// Parses the `DateTimeAnchor` enum based on the current time.
    pub fn parse_as_end_of_day<Tz: TimeZone>(&self, now: &DateTime<Tz>) -> DateTime<Tz> {
        match self {
            DateTimeAnchor::InHours(n) => now.clone() + TimeDelta::hours(*n),
            DateTimeAnchor::InDays(n) => end_of_day(now) + TimeDelta::days(*n),
            DateTimeAnchor::DateTime(dt) => {
                let naive = dt.with_end_of_day();
                from_local_datetime(&now.timezone(), naive)
            }
            DateTimeAnchor::Time(t) => {
                let naive = NaiveDateTime::new(now.date_naive(), *t);
                from_local_datetime(&now.timezone(), naive)
            }
        }
    }
}

impl FromStr for DateTimeAnchor {
    type Err = String;

    fn from_str(timedelta: &str) -> Result<Self, Self::Err> {
        // Handle keywords
        if timedelta == "tomorrow" {
            return Ok(Self::tomorrow());
        }

        // Try to parse as datetime
        if let Ok(dt) = NaiveDateTime::parse_from_str(timedelta, "%Y-%m-%d %H:%M") {
            return Ok(Self::DateTime(loose_from_datetime(dt)));
        }

        // Try to parse as time only
        if let Ok(time) = NaiveTime::parse_from_str(timedelta, "%H:%M") {
            return Ok(Self::Time(time));
        }

        Err(format!("Invalid timedelta format: {timedelta}"))
    }
}

fn loose_from_datetime(dt: NaiveDateTime) -> LooseDateTime {
    match Local.from_local_datetime(&dt) {
        LocalResult::Single(dt) => dt.into(),
        LocalResult::Ambiguous(dt1, _) => {
            tracing::warn!(?dt, "ambiguous local time in local, picking earliest");
            dt1.into()
        }
        LocalResult::None => {
            tracing::warn!(?dt, "invalid local time in local, falling back to floating");
            dt.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, NaiveDate, Timelike, Utc};

    use super::*;

    #[test]
    fn test_anchor_now() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        assert_eq!(DateTimeAnchor::now().parse_as_start_of_day(&now), now);
        assert_eq!(DateTimeAnchor::now().parse_as_end_of_day(&now), now);
    }

    #[test]
    fn test_anchor_in_hours() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        let anchor = DateTimeAnchor::InHours(1);

        let parsed = anchor.parse_as_start_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 1, 16, 30, 45).unwrap();
        assert_eq!(parsed, expected);

        let parsed = anchor.parse_as_end_of_day(&now);
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_anchor_in_days() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        let anchor = DateTimeAnchor::InDays(1);

        let parsed = anchor.parse_as_start_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();
        assert_eq!(parsed, expected);

        let parsed = anchor.parse_as_end_of_day(&now);
        assert!(parsed > Utc.with_ymd_and_hms(2025, 1, 2, 23, 59, 59).unwrap());
        assert!(parsed < Utc.with_ymd_and_hms(2025, 1, 3, 0, 0, 0).unwrap());
    }

    #[test]
    fn test_anchor_time_dateonly() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let loose_date = LooseDateTime::DateOnly(date);
        let anchor = DateTimeAnchor::DateTime(loose_date);

        let parsed = anchor.parse_as_start_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 5, 0, 0, 0).unwrap();
        assert_eq!(parsed, expected);

        let parsed = anchor.parse_as_end_of_day(&now);
        assert!(parsed > Utc.with_ymd_and_hms(2025, 1, 5, 23, 59, 59).unwrap());
        assert!(parsed < Utc.with_ymd_and_hms(2025, 1, 6, 0, 0, 0).unwrap());
    }

    #[test]
    fn test_anchor_time_floating() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
        let datetime = NaiveDateTime::new(date, time);
        let loose_datetime = LooseDateTime::Floating(datetime);
        let anchor = DateTimeAnchor::DateTime(loose_datetime);

        let parsed = anchor.parse_as_start_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 5, 14, 30, 0).unwrap();
        assert_eq!(parsed, expected);

        let parsed = anchor.parse_as_end_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 5, 14, 30, 0).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_anchor_time_local() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        let local_dt = Local.with_ymd_and_hms(2025, 1, 5, 14, 30, 0).unwrap();
        let loose_local = LooseDateTime::Local(local_dt);
        let anchor = DateTimeAnchor::DateTime(loose_local);

        let parsed = anchor.parse_as_start_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 5, 14, 30, 0).unwrap();
        assert_eq!(parsed, expected);

        let parsed = anchor.parse_as_end_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 5, 14, 30, 0).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_start_of_day() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 59).unwrap();
        let parsed = start_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_end_of_day() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 0).unwrap();
        let parsed = end_of_day(&now);
        let last_sec = Utc.with_ymd_and_hms(2025, 1, 1, 23, 59, 59).unwrap();
        let next_day = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();
        assert!(parsed > last_sec);
        assert!(parsed < next_day);
    }

    #[test]
    fn test_from_local_datetime_dst_ambiguity_pick_earliest() {
        let tz = chrono_tz::America::New_York; // DST
        let now = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2025, 11, 2).unwrap(),
            NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        );

        let parsed = from_local_datetime(&tz, now).with_timezone(&Utc);
        let expected = Utc.with_ymd_and_hms(2025, 11, 2, 5, 30, 0).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_datetime_anchor_from_str_keywords() {
        // Test keyword parsing
        let anchor = DateTimeAnchor::from_str("tomorrow").unwrap();
        assert!(matches!(anchor, DateTimeAnchor::InDays(1)));

        // Test invalid keyword
        let result = DateTimeAnchor::from_str("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid timedelta format"));
    }

    #[test]
    fn test_datetime_anchor_from_str_datetime() {
        // Test datetime parsing
        let anchor = DateTimeAnchor::from_str("2025-01-15 14:30").unwrap();
        match anchor {
            DateTimeAnchor::DateTime(ldt) => match ldt {
                LooseDateTime::Local(dt) => {
                    assert_eq!(dt.year(), 2025);
                    assert_eq!(dt.month(), 1);
                    assert_eq!(dt.day(), 15);
                    assert_eq!(dt.hour(), 14);
                    assert_eq!(dt.minute(), 30);
                }
                _ => panic!("Expected Local datetime"),
            },
            _ => panic!("Expected DateTimeAnchor::DateTime"),
        }
    }

    #[test]
    fn test_datetime_anchor_from_str_time() {
        // Test time parsing
        let anchor = DateTimeAnchor::from_str("14:30").unwrap();
        match anchor {
            DateTimeAnchor::Time(time) => {
                assert_eq!(time.hour(), 14);
                assert_eq!(time.minute(), 30);
            }
            _ => panic!("Expected DateTimeAnchor::Time"),
        }
    }

    #[test]
    fn test_datetime_anchor_time_parsing() {
        // Test parsing of DateTimeAnchor::Time variant
        let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
        let anchor = DateTimeAnchor::Time(time);

        // Test with a sample date for parsing
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap();
        let parsed_start = anchor.parse_as_start_of_day(&now);
        let parsed_end = anchor.parse_as_end_of_day(&now);

        // Both should have the same time (14:30) on the same date (2025-01-01)
        assert_eq!(parsed_start.date_naive(), now.date_naive());
        assert_eq!(parsed_end.date_naive(), now.date_naive());
        assert_eq!(parsed_start.hour(), 14);
        assert_eq!(parsed_start.minute(), 30);
        assert_eq!(parsed_end.hour(), 14);
        assert_eq!(parsed_end.minute(), 30);
    }
}
