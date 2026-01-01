// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, str::FromStr, sync::OnceLock};

use chrono::format::{Parsed, StrftimeItems};
use chrono::{
    DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Timelike,
};
use regex::Regex;
use serde::de;

use crate::LooseDateTime;
use crate::datetime::util::{end_of_day, from_local_datetime, start_of_day};

/// Represents a date and time anchor that can be used to calculate relative dates and times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeAnchor {
    /// A specific number of days in the future or past.
    InDays(i64),

    /// A specific number of seconds in the future or past.
    Relative(i64),

    /// A specific date and time.
    DateTime(LooseDateTime),

    /// A specific time.
    Time(NaiveTime),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DayBoundary {
    Start,
    End,
}

impl DateTimeAnchor {
    /// Represents the current time.
    #[must_use]
    pub fn now() -> Self {
        DateTimeAnchor::Relative(0)
    }

    /// Represents the current date.
    #[must_use]
    pub fn today() -> Self {
        DateTimeAnchor::InDays(0)
    }

    /// Represents tomorrow, which is one day after today.
    #[must_use]
    pub fn tomorrow() -> Self {
        DateTimeAnchor::InDays(1)
    }

    /// Represents yesterday, which is one day before today.
    #[must_use]
    pub fn yesterday() -> Self {
        DateTimeAnchor::InDays(-1)
    }

    /// Resolve datetime at the start of the day based on the provided current local time.
    pub fn resolve_at_start_of_day<Tz: TimeZone>(&self, now: &DateTime<Tz>) -> DateTime<Tz> {
        self.resolve_at_day_boundary(now, DayBoundary::Start)
    }

    /// Resolve datetime at the end of the day based on the provided current local time.
    pub fn resolve_at_end_of_day<Tz: TimeZone>(&self, now: &DateTime<Tz>) -> DateTime<Tz> {
        self.resolve_at_day_boundary(now, DayBoundary::End)
    }

    /// Resolve datetime based on the provided current local time and day boundary.
    fn resolve_at_day_boundary<Tz: TimeZone>(
        &self,
        now: &DateTime<Tz>,
        boundary: DayBoundary,
    ) -> DateTime<Tz> {
        match self {
            DateTimeAnchor::InDays(n) => {
                let dt = match boundary {
                    DayBoundary::Start => start_of_day(now),
                    DayBoundary::End => end_of_day(now),
                };
                dt + TimeDelta::days(*n)
            }
            DateTimeAnchor::Relative(n) => now.clone() + TimeDelta::seconds(*n),
            DateTimeAnchor::DateTime(dt) => {
                let naive = match boundary {
                    DayBoundary::Start => dt.with_start_of_day(),
                    DayBoundary::End => dt.with_end_of_day(),
                };
                from_local_datetime(&now.timezone(), naive)
            }
            DateTimeAnchor::Time(t) => {
                let naive = NaiveDateTime::new(now.date_naive(), *t);
                from_local_datetime(&now.timezone(), naive)
            }
        }
    }

    /// Resolve the `DateTimeAnchor` to a `LooseDateTime` based on the provided current local time.
    #[must_use]
    pub fn resolve_at(self, now: &LooseDateTime) -> LooseDateTime {
        match self {
            DateTimeAnchor::InDays(n) => *now + TimeDelta::days(n),
            DateTimeAnchor::Relative(n) => *now + TimeDelta::seconds(n),
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
                LooseDateTime::DateOnly(d) => {
                    let dt = NaiveDateTime::new(*d, t);
                    LooseDateTime::from_local_datetime(dt)
                }
            },
        }
    }

    /// Resolve the `DateTimeAnchor` to a `LooseDateTime` starting from the provided `DateTime<Tz>`.
    #[must_use]
    pub fn resolve_since(self, start: &LooseDateTime) -> LooseDateTime {
        match self {
            DateTimeAnchor::InDays(n) => match n {
                0 => match start {
                    LooseDateTime::Local(dt) => next_suggested_time(&dt.naive_local()),
                    LooseDateTime::Floating(dt) => next_suggested_time(dt),
                    LooseDateTime::DateOnly(d) => first_suggested_time(*d),
                },
                _ => {
                    let date = start.date() + TimeDelta::days(n);
                    let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap_or_default();
                    let dt = NaiveDateTime::new(date, time);
                    LooseDateTime::from_local_datetime(dt)
                }
            },
            DateTimeAnchor::Relative(n) => *start + TimeDelta::seconds(n),
            DateTimeAnchor::DateTime(dt) => dt,
            DateTimeAnchor::Time(t) => {
                let date = start.date();
                // If the time has already passed today, use tomorrow
                let delta = if let Some(s) = start.time()
                    && s >= t
                {
                    TimeDelta::days(1)
                } else {
                    TimeDelta::zero()
                };
                let dt = NaiveDateTime::new(date, t) + delta;
                LooseDateTime::from_local_datetime(dt)
            }
        }
    }

    /// Resolve the `DateTimeAnchor` to a `LooseDateTime` starting from the provided `DateTime<Tz>`.
    #[must_use]
    pub fn resolve_since_datetime<Tz: TimeZone>(self, start: &DateTime<Tz>) -> LooseDateTime {
        match self {
            DateTimeAnchor::InDays(n) => match n {
                0 => next_suggested_time(&start.naive_local()),
                _ => {
                    let date = start.date_naive() + TimeDelta::days(n);
                    let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap_or_default();
                    let dt = NaiveDateTime::new(date, time);
                    LooseDateTime::from_local_datetime(dt)
                }
            },
            DateTimeAnchor::Relative(n) => {
                let dt = start.clone() + TimeDelta::seconds(n);
                LooseDateTime::Local(dt.with_timezone(&Local))
            }
            DateTimeAnchor::DateTime(dt) => dt,
            DateTimeAnchor::Time(t) => {
                let date = start.date_naive();
                // If the time has already passed today, use tomorrow
                let delta = if start.time() >= t {
                    TimeDelta::days(1)
                } else {
                    TimeDelta::zero()
                };
                let dt = NaiveDateTime::new(date, t) + delta;
                LooseDateTime::from_local_datetime(dt)
            }
        }
    }

    /// Parses the `DateTimeAnchor` enum based on the current time.
    // TODO: remove this function in 0.12.0
    #[deprecated(
        since = "0.9.0",
        note = "use `resolve_at_start_of_day` method instead, will be removed in 0.12.0"
    )]
    pub fn parse_as_start_of_day<Tz: TimeZone>(&self, now: &DateTime<Tz>) -> DateTime<Tz> {
        self.resolve_at_start_of_day(now)
    }

    /// Parses the `DateTimeAnchor` enum based on the current time.
    // TODO: remove this function in 0.12.0
    #[deprecated(
        since = "0.9.0",
        note = "use `resolve_at_end_of_day` method instead, will be removed in 0.12.0"
    )]
    pub fn parse_as_end_of_day<Tz: TimeZone>(&self, now: &DateTime<Tz>) -> DateTime<Tz> {
        self.resolve_at_end_of_day(now)
    }

    /// Parses the `DateTimeAnchor` to a `LooseDateTime` based on the provided current local time.
    #[deprecated(
        since = "0.9.0",
        note = "use `resolve_at` method instead, will be removed in 0.12.0"
    )]
    #[must_use]
    // TODO: remove this function in 0.12.0
    pub fn parse_from_loose(self, now: &LooseDateTime) -> LooseDateTime {
        self.resolve_at(now)
    }

    /// Parses the `DateTimeAnchor` to a `LooseDateTime` based on the provided current time in any timezone.
    #[deprecated(
        since = "0.9.0",
        note = "use `resolve_since_datetime` method instead, will be removed in 0.12.0"
    )]
    // TODO: remove this function in 0.12.0
    pub fn parse_from_dt<Tz: TimeZone>(self, now: &DateTime<Tz>) -> LooseDateTime {
        self.resolve_since_datetime(now)
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
        } else if let Ok(date) = NaiveDate::parse_from_str(t, "%Y-%m-%d") {
            // Parse as date only
            Ok(Self::DateTime(LooseDateTime::DateOnly(date)))
        } else if let Some(date) = parse_md_with_year(t, Local::now().year()) {
            // Parse as date only (with current year)
            // TODO: handle year
            Ok(Self::DateTime(LooseDateTime::DateOnly(date)))
        } else if let Ok(time) = NaiveTime::parse_from_str(t, "%H:%M") {
            // Parse as time only
            Ok(Self::Time(time))
        } else if let Some(hours) = parse_seconds(t) {
            // Parse as hours (e.g., "10s", "10sec", "10 seconds")
            Ok(Self::Relative(hours))
        } else if let Some(minutes) = parse_minutes(t) {
            // Parse as hours (e.g., "10m", "10min", "10 minutes")
            Ok(Self::Relative(60 * minutes))
        } else if let Some(hours) = parse_hours(t) {
            // Parse as hours (e.g., "10h", "10hours", "10hours")
            Ok(Self::Relative(60 * 60 * hours))
        } else if let Some(days) = parse_days(t) {
            // Parse as days (e.g., "10d", "10days")
            Ok(Self::InDays(days))
        } else {
            Err(format!("Invalid datetime anchor: {t}"))
        }
    }
}

impl<'de> serde::Deserialize<'de> for DateTimeAnchor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl de::Visitor<'_> for Visitor {
            type Value = DateTimeAnchor;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a datetime")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

macro_rules! parse_with_regex {
    ($fn: ident, $re:expr) => {
        fn $fn(s: &str) -> Option<i64> {
            static REGEX: OnceLock<Regex> = OnceLock::new();
            let re = REGEX.get_or_init(|| Regex::new($re).unwrap());
            if let Some(captures) = re.captures(s)
                && let Ok(num) = captures[1].parse::<i64>()
            {
                return Some(num);
            }
            None
        }
    };
}

parse_with_regex!(parse_seconds, r"^\s*(\d+)\s*s(?:ec|econds)?\s*$"); // "10s", "10 sec", "10 seconds"
parse_with_regex!(parse_minutes, r"^\s*(\d+)\s*m(?:in|inutes)?\s*$"); // "10m", "10 min", "10minutes"

// TODO: remove "in xxx" support?
parse_with_regex!(parse_hours, r"(?i)^\s*(?:in\s*)?(\d+)\s*h(?:ours)?\s*$"); // "10h", "10 hours", "10hours", "in 10hours"
parse_with_regex!(parse_days, r"(?i)^\s*(?:in\s*)?(\d+)\s*d(?:ays)?\s*$"); // "10d", "in 10d", "in 10 days"

const HOURS: [u32; 3] = [9, 13, 18];

fn next_suggested_time(now: &NaiveDateTime) -> LooseDateTime {
    let date = now.date();
    let current_hour = now.hour();
    for hour in HOURS {
        if current_hour < hour {
            let dt = NaiveDateTime::new(date, NaiveTime::from_hms_opt(hour, 0, 0).unwrap());
            return LooseDateTime::from_local_datetime(dt);
        }
    }

    LooseDateTime::DateOnly(date)
}

fn first_suggested_time(date: NaiveDate) -> LooseDateTime {
    let dt = NaiveDateTime::new(date, NaiveTime::from_hms_opt(HOURS[0], 0, 0).unwrap());
    LooseDateTime::from_local_datetime(dt)
}

fn parse_md_with_year(s: &str, year: i32) -> Option<NaiveDate> {
    let mut p = Parsed::new();
    chrono::format::parse(&mut p, s, StrftimeItems::new("%m-%d")).ok()?;
    let month = p.month?;
    let day = p.day?;
    NaiveDate::from_ymd_opt(year, month, day)
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, NaiveDate, Utc};

    use super::*;

    #[test]
    fn resolves_now_anchor_to_current_time() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        assert_eq!(DateTimeAnchor::now().resolve_at_start_of_day(&now), now);
        assert_eq!(DateTimeAnchor::now().resolve_at_end_of_day(&now), now);

        #[expect(deprecated)]
        {
            assert_eq!(DateTimeAnchor::now().parse_as_start_of_day(&now), now);
            assert_eq!(DateTimeAnchor::now().parse_as_end_of_day(&now), now);
        }
    }

    #[test]
    fn resolves_indays_anchor_to_day_boundary() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        let anchor = DateTimeAnchor::InDays(1);

        let expected = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();

        let parsed = anchor.resolve_at_start_of_day(&now);
        assert_eq!(parsed, expected);

        let parsed = anchor.resolve_at_end_of_day(&now);
        assert!(parsed > Utc.with_ymd_and_hms(2025, 1, 2, 23, 59, 59).unwrap());
        assert!(parsed < Utc.with_ymd_and_hms(2025, 1, 3, 0, 0, 0).unwrap());

        // Test deprecated functions still work
        #[expect(deprecated)]
        {
            let parsed = anchor.parse_as_start_of_day(&now);
            assert_eq!(parsed, expected);

            let parsed = anchor.parse_as_end_of_day(&now);
            assert!(parsed > Utc.with_ymd_and_hms(2025, 1, 2, 23, 59, 59).unwrap());
            assert!(parsed < Utc.with_ymd_and_hms(2025, 1, 3, 0, 0, 0).unwrap());
        }
    }

    #[test]
    fn resolves_datetime_anchor_to_day_boundary() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let loose_date = LooseDateTime::DateOnly(date);
        let anchor = DateTimeAnchor::DateTime(loose_date);

        let parsed = anchor.resolve_at_start_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 5, 0, 0, 0).unwrap();
        assert_eq!(parsed, expected);

        let parsed = anchor.resolve_at_end_of_day(&now);
        assert!(parsed > Utc.with_ymd_and_hms(2025, 1, 5, 23, 59, 59).unwrap());
        assert!(parsed < Utc.with_ymd_and_hms(2025, 1, 6, 0, 0, 0).unwrap());

        // Test deprecated functions still work
        #[expect(deprecated)]
        {
            let parsed = anchor.parse_as_start_of_day(&now);
            assert_eq!(parsed, expected);

            let parsed = anchor.parse_as_end_of_day(&now);
            assert!(parsed > Utc.with_ymd_and_hms(2025, 1, 5, 23, 59, 59).unwrap());
            assert!(parsed < Utc.with_ymd_and_hms(2025, 1, 6, 0, 0, 0).unwrap());
        }
    }

    #[test]
    fn resolves_various_anchor_types_correctly() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 15, 30, 45).unwrap();
        for (name, anchor, expected) in [
            (
                "Relative (+2h30m45s)",
                DateTimeAnchor::Relative(2 * 60 * 60 + 30 * 60 + 45),
                Utc.with_ymd_and_hms(2025, 1, 1, 18, 1, 30).unwrap(),
            ),
            (
                "Floating",
                {
                    let date = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
                    let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
                    let datetime = NaiveDateTime::new(date, time);
                    DateTimeAnchor::DateTime(LooseDateTime::Floating(datetime))
                },
                Utc.with_ymd_and_hms(2025, 1, 5, 14, 30, 0).unwrap(),
            ),
            (
                "Local",
                {
                    let local_dt = Local.with_ymd_and_hms(2025, 1, 5, 14, 30, 0).unwrap();
                    DateTimeAnchor::DateTime(LooseDateTime::Local(local_dt))
                },
                Utc.with_ymd_and_hms(2025, 1, 5, 14, 30, 0).unwrap(),
            ),
        ] {
            let parsed = anchor.resolve_at_start_of_day(&now);
            assert_eq!(parsed, expected, "start_of_day failed for {name}");

            let parsed = anchor.resolve_at_end_of_day(&now);
            assert_eq!(parsed, expected, "end_of_day failed for {name}");

            #[expect(deprecated)]
            {
                let parsed = anchor.parse_as_start_of_day(&now);
                assert_eq!(parsed, expected, "parse_as_start_of_day failed for {name}",);

                let parsed = anchor.parse_as_end_of_day(&now);
                assert_eq!(parsed, expected, "parse_as_end_of_day failed for {name}");
            }
        }
    }

    #[test]
    fn calculates_start_of_day() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 59).unwrap();
        let parsed = start_of_day(&now);
        let expected = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn calculates_end_of_day() {
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 0).unwrap();
        let parsed = end_of_day(&now);
        let last_sec = Utc.with_ymd_and_hms(2025, 1, 1, 23, 59, 59).unwrap();
        let next_day = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();
        assert!(parsed > last_sec);
        assert!(parsed < next_day);
    }

    #[test]
    fn handles_dst_ambiguity_by_picking_earliest() {
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
    fn resolves_time_anchor_to_specific_time() {
        // Test parsing of DateTimeAnchor::Time variant
        let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
        let anchor = DateTimeAnchor::Time(time);

        // Test with a sample date for parsing
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap();
        let parsed_start = anchor.resolve_at_start_of_day(&now);
        let parsed_end = anchor.resolve_at_end_of_day(&now);

        // Both should have the same time (14:30) on the same date (2025-01-01)
        assert_eq!(parsed_start.date_naive(), now.date_naive());
        assert_eq!(parsed_start.time(), time);
        assert_eq!(parsed_end.date_naive(), now.date_naive());
        assert_eq!(parsed_end.time(), time);

        // Test deprecated functions still work
        #[expect(deprecated)]
        {
            let parsed_start = anchor.parse_as_start_of_day(&now);
            let parsed_end = anchor.parse_as_end_of_day(&now);

            // Both should have the same time (14:30) on the same date (2025-01-01)
            assert_eq!(parsed_start.date_naive(), now.date_naive());
            assert_eq!(parsed_start.time(), time);
            assert_eq!(parsed_end.date_naive(), now.date_naive());
            assert_eq!(parsed_end.time(), time);
        }
    }

    #[test]
    fn resolves_anchor_from_loose_datetime() {
        for (name, anchor, now, expected) in [
            (
                "AtInDays (same datetime)",
                DateTimeAnchor::DateTime(LooseDateTime::Local(
                    Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
                )),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap()),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap()),
            ),
            (
                "Relative (+3 hours)",
                DateTimeAnchor::Relative(3 * 60 * 60),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap()),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 15, 0, 0).unwrap()),
            ),
            (
                "DateTime (absolute 10:00)",
                DateTimeAnchor::DateTime(LooseDateTime::Local(
                    Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap(),
                )),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap()),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap()),
            ),
            (
                "Time (10:00 today)",
                DateTimeAnchor::Time(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap()),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap()),
            ),
        ] {
            let result = anchor.resolve_at(&now);
            assert_eq!(result, expected, "resolve_at failed for case: {name}");

            #[expect(deprecated)]
            {
                let result = anchor.parse_from_loose(&now);
                assert_eq!(result, expected, "parse_from_loose failed for case: {name}");
            }
        }
    }

    #[test]
    fn resolves_anchor_since_loose_datetime() {
        let dt = |y, m, d, hh, mm, ss| {
            LooseDateTime::Local(Local.with_ymd_and_hms(y, m, d, hh, mm, ss).unwrap())
        };
        let now: LooseDateTime = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap().into();
        for (name, anchor, expected) in [
            ("DateTime == now", DateTimeAnchor::DateTime(now), now),
            (
                "Relative +3:25:45",
                DateTimeAnchor::Relative(3 * 60 * 60 + 25 * 60 + 45),
                dt(2025, 1, 1, 15, 25, 45),
            ),
            (
                "Explicit DateTime 10:00",
                DateTimeAnchor::DateTime(dt(2025, 1, 1, 10, 0, 0)),
                dt(2025, 1, 1, 10, 0, 0),
            ),
            (
                "Time before now -> tomorrow 10:00",
                DateTimeAnchor::Time(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
                dt(2025, 1, 2, 10, 0, 0),
            ),
            (
                "Time after now -> today 14:00",
                DateTimeAnchor::Time(NaiveTime::from_hms_opt(14, 0, 0).unwrap()),
                dt(2025, 1, 1, 14, 0, 0),
            ),
        ] {
            let result = anchor.resolve_since(&now);
            assert_eq!(result, expected, "case failed: {name}");
        }
    }

    #[test]
    fn resolves_anchor_since_datetime() {
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        for (name, anchor, expected) in [
            (
                "DateTimeAnchor::DateTime (same datetime)",
                DateTimeAnchor::DateTime(LooseDateTime::Local(now)),
                LooseDateTime::Local(now),
            ),
            (
                "DateTimeAnchor::Relative (3h25m45s later)",
                DateTimeAnchor::Relative(3 * 60 * 60 + 25 * 60 + 45),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 15, 25, 45).unwrap()),
            ),
            (
                "DateTimeAnchor::DateTime (specific datetime before now)",
                DateTimeAnchor::DateTime(LooseDateTime::Local(
                    Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap(),
                )),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap()),
            ),
            (
                "DateTimeAnchor::Time (before now → tomorrow)",
                DateTimeAnchor::Time(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 2, 10, 0, 0).unwrap()),
            ),
            (
                "DateTimeAnchor::Time (after now → today)",
                DateTimeAnchor::Time(NaiveTime::from_hms_opt(14, 0, 0).unwrap()),
                LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 1, 14, 0, 0).unwrap()),
            ),
        ] {
            let result = anchor.resolve_since_datetime(&now);
            assert_eq!(result, expected, "failed: {name} → resolve_since_datetime",);

            // Deprecated API should behave identically
            #[expect(deprecated)]
            {
                let deprecated_result = anchor.parse_from_dt(&now);
                assert_eq!(
                    deprecated_result, expected,
                    "failed: {name} → parse_from_dt (deprecated)",
                );
            }
        }
    }

    #[test]
    fn parses_string_to_datetime_anchor() {
        for (s, expected) in [
            // Keywords
            ("now", DateTimeAnchor::now()),
            ("today", DateTimeAnchor::today()),
            ("yesterday", DateTimeAnchor::yesterday()),
            ("tomorrow", DateTimeAnchor::tomorrow()),
            // Date only
            (
                "2025-01-15",
                DateTimeAnchor::DateTime(LooseDateTime::DateOnly(
                    NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                )),
            ),
            // Date only with current year
            (
                "01-15",
                DateTimeAnchor::DateTime(LooseDateTime::DateOnly(
                    NaiveDate::from_ymd_opt(Local::now().year(), 1, 15).unwrap(),
                )),
            ),
            // DateTime
            (
                "2025-01-15 14:30",
                DateTimeAnchor::DateTime(LooseDateTime::Local(
                    Local.with_ymd_and_hms(2025, 1, 15, 14, 30, 0).unwrap(),
                )),
            ),
            // Time only
            (
                "14:30",
                DateTimeAnchor::Time(NaiveTime::from_hms_opt(14, 30, 0).unwrap()),
            ),
        ] {
            let anchor: DateTimeAnchor = s.parse().unwrap();
            assert_eq!(anchor, expected, "Failed to parse '{s}'",);
        }
    }

    #[test]
    fn returns_error_for_invalid_string() {
        let result = DateTimeAnchor::from_str("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid datetime anchor"));
    }

    #[test]
    fn parses_seconds_and_minutes_durations() {
        for (tests, expected) in [
            (
                [
                    "10s",
                    "10sec",
                    "10seconds",
                    "   10   s   ",
                    "   10   sec   ",
                    "   10   seconds   ",
                ],
                DateTimeAnchor::Relative(10),
            ),
            (
                [
                    "10m",
                    "10min",
                    "10minutes",
                    "   10   m   ",
                    "   10   min   ",
                    "   10   minutes   ",
                ],
                DateTimeAnchor::Relative(10 * 60),
            ),
        ] {
            for s in tests {
                let anchor: DateTimeAnchor = s.parse().unwrap();
                assert_eq!(anchor, expected, "Failed to parse '{s}'");

                // No "in " prefix allowed for seconds
                let prefix_in = DateTimeAnchor::from_str(&format!("in {s}"));
                assert!(prefix_in.is_err());

                // No uppercase allowed for seconds
                let uppercase = DateTimeAnchor::from_str(&s.to_uppercase());
                assert!(uppercase.is_err());
            }
        }
    }

    #[test]
    fn parses_hours_and_days_durations() {
        for (tests, expected) in [
            (
                [
                    "in 10hours",
                    "in 10H",
                    "   IN   10   hours   ",
                    "10hours",
                    "10 HOURS",
                    "   10   hours   ",
                    "10h",
                    "10 H",
                    "   10   h   ",
                ],
                DateTimeAnchor::Relative(10 * 60 * 60),
            ),
            (
                [
                    "in 10days",
                    "in 10D",
                    "   IN   10   days   ",
                    "10days",
                    "10 DAYS",
                    "   10   days   ",
                    "10d",
                    "10 D",
                    "   10   d   ",
                ],
                DateTimeAnchor::InDays(10),
            ),
        ] {
            for s in tests {
                let anchor: DateTimeAnchor = s.parse().unwrap();
                assert_eq!(anchor, expected, "Failed to parse '{s}'");
            }
        }
    }

    #[test]
    fn suggests_next_available_time_slot() {
        for (hour, min, expected_hour, description) in [
            (8, 30, 9, "Before 9 AM, should suggest 9 AM"),
            (
                10,
                30,
                13,
                "After 9 AM but before 1 PM, should suggest 1 PM",
            ),
            (
                14,
                30,
                18,
                "After 1 PM but before 6 PM, should suggest 6 PM",
            ),
            (9, 0, 13, "Exactly at 9 AM, should suggest 1 PM"),
            (13, 0, 18, "Exactly at 1 PM, should suggest 6 PM"),
        ] {
            let now = Local.with_ymd_and_hms(2025, 1, 1, hour, min, 0).unwrap();
            let result = next_suggested_time(&now.naive_local());
            let dt = Local
                .with_ymd_and_hms(2025, 1, 1, expected_hour, 0, 0)
                .unwrap();
            let expected = LooseDateTime::Local(dt);
            assert_eq!(result, expected, "{description}");
        }
    }

    #[test]
    fn suggests_dateonly_after_business_hours() {
        // After 6 PM, should suggest DateOnly (next day)
        let now = Local.with_ymd_and_hms(2025, 1, 1, 19, 30, 0).unwrap();
        let result = next_suggested_time(&now.naive_local());
        let expected = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert_eq!(result, expected, "After 6 PM, should suggest DateOnly");

        // Exactly at 6 PM, should suggest DateOnly (next day)
        let now = Local.with_ymd_and_hms(2025, 1, 1, 18, 0, 0).unwrap();
        let result = next_suggested_time(&now.naive_local());
        let expected = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert_eq!(result, expected, "Exactly at 6 PM, should suggest DateOnly");
    }
}
