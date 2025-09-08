// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{str::FromStr, sync::OnceLock};

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};
use regex::Regex;

use crate::LooseDateTime;
use crate::datetime::util::{end_of_day, from_local_datetime, start_of_day};

/// Represents a date and time anchor that can be used to calculate relative dates and times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Parses the `DateTimeAnchor` to a `LooseDateTime` based on the provided current local time.
    pub fn parse_from_loose(self, now: &LooseDateTime) -> LooseDateTime {
        match self {
            DateTimeAnchor::InHours(n) => *now + TimeDelta::hours(n),
            DateTimeAnchor::InDays(n) => *now + TimeDelta::days(n),
            DateTimeAnchor::DateTime(dt) => dt,
            DateTimeAnchor::Time(t) => match now {
                LooseDateTime::Local(dt) => {
                    let dt = NaiveDateTime::new(dt.date_naive(), t);
                    from_local_datetime(&Local, dt).into()
                }
                LooseDateTime::Floating(dt) => {
                    let dt = NaiveDateTime::new(dt.date(), t);
                    LooseDateTime::Floating(dt)
                }
                LooseDateTime::DateOnly(date) => {
                    let dt = NaiveDateTime::new(*date, t);
                    LooseDateTime::from_local_datetime(dt)
                }
            },
        }
    }

    /// Parses the `DateTimeAnchor` to a `LooseDateTime` based on the provided current time in any timezone.
    pub fn parse_from_dt<Tz: TimeZone>(self, now: &DateTime<Tz>) -> LooseDateTime {
        match self {
            DateTimeAnchor::InHours(n) => {
                let dt = now.clone() + TimeDelta::hours(n);
                LooseDateTime::Local(dt.with_timezone(&Local))
            }
            DateTimeAnchor::InDays(n) => {
                let date = now.date_naive() + TimeDelta::days(n);
                let dt = NaiveDateTime::new(date, NaiveTime::from_hms_opt(9, 0, 0).unwrap());
                LooseDateTime::from_local_datetime(dt)
            }
            DateTimeAnchor::DateTime(dt) => dt,
            DateTimeAnchor::Time(t) => {
                let date = now.date_naive();
                // If the time has already passed today, use tomorrow
                let delta = if now.time() <= t {
                    TimeDelta::zero()
                } else {
                    TimeDelta::days(1)
                };
                let dt = NaiveDateTime::new(date, t) + delta;
                LooseDateTime::from_local_datetime(dt)
            }
        }
    }
}

impl FromStr for DateTimeAnchor {
    type Err = String;

    fn from_str(t: &str) -> Result<Self, Self::Err> {
        // Handle keywords
        match t {
            "yesterday" => return Ok(Self::yesterday()),
            "tomorrow" => return Ok(Self::tomorrow()),
            "today" => return Ok(Self::today()),
            "now" => return Ok(Self::now()),
            _ => {}
        }

        if let Ok(dt) = NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M") {
            // Parse as datetime
            Ok(Self::DateTime(LooseDateTime::from_local_datetime(dt)))
        } else if let Ok(time) = NaiveTime::parse_from_str(t, "%H:%M") {
            // Parse as time only
            Ok(Self::Time(time))
        } else if let Some(hours) = parse_hours(t) {
            // Parse as hours (e.g., "10h", "10 hours", "10hours", "in 10hours")
            Ok(Self::InHours(hours))
        } else if let Some(days) = parse_days(t) {
            // Parse as days (e.g., "10d", "in 10d", "in 10 days")
            Ok(Self::InDays(days))
        } else {
            Err(format!("Invalid timedelta format: {t}"))
        }
    }
}

/// Parse hours from string formats like "10h", "10 hours", "10hours", "in 10hours"
fn parse_hours(s: &str) -> Option<i64> {
    const RE: &str = r"(?i)^\s*(?:in\s*)?(\d+)\s*h(?:ours)?\s*$";
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let re = REGEX.get_or_init(|| Regex::new(RE).unwrap());
    if let Some(captures) = re.captures(s)
        && let Ok(num) = captures[1].parse::<i64>()
    {
        return Some(num);
    }

    None
}

/// Parse days from string formats like "10d", "in 10d", "in 10 days"
fn parse_days(s: &str) -> Option<i64> {
    const RE: &str = r"(?i)^\s*(?:in\s*)?(\d+)\s*d(?:ays)?\s*$";
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let re = REGEX.get_or_init(|| Regex::new(RE).unwrap());
    if let Some(captures) = re.captures(s)
        && let Ok(num) = captures[1].parse::<i64>()
    {
        return Some(num);
    }

    None
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, Utc};

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
    fn test_time_parsing() {
        // Test parsing of DateTimeAnchor::Time variant
        let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
        let anchor = DateTimeAnchor::Time(time);

        // Test with a sample date for parsing
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap();
        let parsed_start = anchor.parse_as_start_of_day(&now);
        let parsed_end = anchor.parse_as_end_of_day(&now);

        // Both should have the same time (14:30) on the same date (2025-01-01)
        assert_eq!(parsed_start.date_naive(), now.date_naive());
        assert_eq!(parsed_start.time(), time);
        assert_eq!(parsed_end.date_naive(), now.date_naive());
        assert_eq!(parsed_end.time(), time);
    }

    #[test]
    fn test_parse_from_loose_in_days() {
        let expected = LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap());
        let anchor = DateTimeAnchor::DateTime(expected);
        let result = anchor.parse_from_loose(&expected);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_from_loose_in_hours() {
        let anchor = DateTimeAnchor::InHours(3);
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let result = anchor.parse_from_loose(&now.into());
        let expected = LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 15, 0, 0).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_from_loose_datetime() {
        let anchor = DateTimeAnchor::DateTime(LooseDateTime::Local(
            Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap(),
        ));
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let result = anchor.parse_from_loose(&now.into());
        let expected = LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_from_loose_time() {
        let anchor = DateTimeAnchor::Time(NaiveTime::from_hms_opt(10, 0, 0).unwrap());
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let result = anchor.parse_from_loose(&now.into());
        let expected = LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_from_dt_in_days() {
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let expected = now.into();
        let anchor = DateTimeAnchor::DateTime(expected);
        let result = anchor.parse_from_dt(&now);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_from_dt_in_hours() {
        let anchor = DateTimeAnchor::InHours(3);
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let result = anchor.parse_from_dt(&now);
        let expected = LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 15, 0, 0).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_from_dt_datetime() {
        let anchor = DateTimeAnchor::DateTime(LooseDateTime::Local(
            Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap(),
        ));
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let result = anchor.parse_from_dt(&now);
        let expected = LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_from_dt_time_before_now() {
        // Test "HH:MM" (before now, should be tomorrow)
        let anchor = DateTimeAnchor::Time(NaiveTime::from_hms_opt(10, 0, 0).unwrap());
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let result = anchor.parse_from_dt(&now);
        let expected = LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 2, 10, 0, 0).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_from_dt_time_after_now() {
        // Test "HH:MM" (after now, should be today)
        let anchor = DateTimeAnchor::Time(NaiveTime::from_hms_opt(14, 0, 0).unwrap());
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let result = anchor.parse_from_dt(&now);
        let expected = LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 14, 0, 0).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_str_keywords() {
        for (s, expected) in [
            ("now", DateTimeAnchor::now()),
            ("today", DateTimeAnchor::today()),
            ("yesterday", DateTimeAnchor::yesterday()),
            ("tomorrow", DateTimeAnchor::tomorrow()),
        ] {
            let anchor = DateTimeAnchor::from_str(s).unwrap();
            assert_eq!(anchor, expected);
        }
    }

    #[test]
    fn test_from_str_datetime() {
        let anchor = DateTimeAnchor::from_str("2025-01-15 14:30").unwrap();
        let expected = DateTimeAnchor::DateTime(LooseDateTime::Local(
            Local.with_ymd_and_hms(2025, 1, 15, 14, 30, 0).unwrap(),
        ));
        assert_eq!(anchor, expected);
    }

    #[test]
    fn test_from_str_time() {
        let anchor = DateTimeAnchor::from_str("14:30").unwrap();
        let expected = DateTimeAnchor::Time(NaiveTime::from_hms_opt(14, 30, 0).unwrap());
        assert_eq!(anchor, expected);
    }

    #[test]
    fn test_from_str_invalid() {
        let result = DateTimeAnchor::from_str("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid timedelta format"));
    }

    #[test]
    fn test_from_str_hours() {
        for s in [
            "in 10hours",
            "in 10H",
            "   IN   10   hours   ",
            "10hours",
            "10 HOURS",
            "   10   hours   ",
            "10h",
            "10 H",
            "   10   h   ",
        ] {
            let anchor = DateTimeAnchor::from_str(s).unwrap();
            let expected = DateTimeAnchor::InHours(10);
            assert_eq!(anchor, expected, "Failed to parse '{}'", s);
        }
    }

    #[test]
    fn test_from_str_days() {
        for s in [
            "in 10days",
            "in 10D",
            "   IN   10   days   ",
            "10days",
            "10 DAYS",
            "   10   days   ",
            "10d",
            "10 D",
            "   10   d   ",
        ] {
            let anchor = DateTimeAnchor::from_str(s).unwrap();
            let expected = DateTimeAnchor::InDays(10);
            assert_eq!(anchor, expected, "Failed to parse '{}'", s);
        }
    }
}
