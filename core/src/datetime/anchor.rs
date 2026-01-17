// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, str::FromStr, sync::OnceLock};

use jiff::civil::{self, Date, Time, date, time};
use jiff::tz::TimeZone;
use jiff::{Span, Zoned};
use regex::Regex;
use serde::de;

use crate::LooseDateTime;

/// Represents a date and time anchor that can be used to calculate relative dates and times.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTimeAnchor {
    /// A specific number of days in the future or past.
    InDays(i64),

    /// A specific number of seconds in the future or past.
    Relative(i64),

    /// A specific date and time.
    DateTime(LooseDateTime),

    /// A specific time.
    Time(Time),
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
    ///
    /// # Errors
    ///
    /// Returns an error if date/time operations fail.
    pub fn resolve_at_start_of_day(&self, now: &Zoned) -> Result<Zoned, String> {
        match self {
            DateTimeAnchor::InDays(n) => {
                let start = now
                    .start_of_day()
                    .map_err(|e| format!("Failed to get start of day: {e}"))?;
                start
                    .checked_add(Span::new().days(*n))
                    .map_err(|e| format!("Failed to add days to start of day: {e}"))
            }
            DateTimeAnchor::Relative(n) => now
                .checked_add(Span::new().seconds(*n))
                .map_err(|e| format!("Failed to add relative seconds: {e}")),
            DateTimeAnchor::DateTime(dt) => dt
                .with_start_of_day()
                .to_zoned(TimeZone::system())
                .map_err(|e| format!("Failed to convert to zoned: {e}")),
            DateTimeAnchor::Time(t) => now
                .with()
                .hour(t.hour())
                .minute(t.minute())
                .second(t.second())
                .subsec_nanosecond(t.subsec_nanosecond())
                .build()
                .map_err(|e| format!("Failed to build zoned: {e}")),
        }
    }

    /// Resolve datetime at the end of the day based on the provided current local time.
    ///
    /// # Errors
    ///
    /// Returns an error if date/time operations fail.
    pub fn resolve_at_end_of_day(&self, now: &Zoned) -> Result<Zoned, String> {
        match self {
            DateTimeAnchor::InDays(n) => {
                let end = now
                    .end_of_day()
                    .map_err(|e| format!("Failed to get end of day: {e}"))?;
                end.checked_add(Span::new().days(*n))
                    .map_err(|e| format!("Failed to add days to end of day: {e}"))
            }
            DateTimeAnchor::Relative(n) => now
                .checked_add(Span::new().seconds(*n))
                .map_err(|e| format!("Failed to add relative seconds: {e}")),
            DateTimeAnchor::DateTime(dt) => dt
                .with_end_of_day()
                .to_zoned(TimeZone::system())
                .map_err(|e| format!("Failed to convert to zoned: {e}")),
            DateTimeAnchor::Time(t) => now
                .with()
                .hour(t.hour())
                .minute(t.minute())
                .second(t.second())
                .subsec_nanosecond(t.subsec_nanosecond())
                .build()
                .map_err(|e| format!("Failed to build zoned: {e}")),
        }
    }

    /// Resolve the `DateTimeAnchor` to a `LooseDateTime` based on the provided current local time.
    #[must_use]
    pub fn resolve_at(self, now: &LooseDateTime) -> LooseDateTime {
        match self {
            DateTimeAnchor::InDays(n) => now.clone() + Span::new().days(n),
            DateTimeAnchor::Relative(n) => now.clone() + Span::new().seconds(n),
            DateTimeAnchor::DateTime(dt) => dt,
            DateTimeAnchor::Time(t) => {
                let dt = civil::DateTime::from_parts(now.date(), t);
                LooseDateTime::from_local_datetime(dt)
            }
        }
    }

    /// Resolve the `DateTimeAnchor` to a `LooseDateTime` starting from the provided `LooseDateTime`.
    ///
    /// # Errors
    ///
    /// Returns an error if date/time operations fail.
    pub fn resolve_since(self, start: &LooseDateTime) -> Result<LooseDateTime, String> {
        match self {
            DateTimeAnchor::InDays(n) => match n {
                0 => Ok(match start {
                    LooseDateTime::Local(zoned) => next_suggested_time(&zoned.datetime()),
                    LooseDateTime::Floating(dt) => next_suggested_time(dt),
                    LooseDateTime::DateOnly(d) => first_suggested_time(*d),
                }),
                _ => {
                    let date = start
                        .date()
                        .checked_add(Span::new().days(n))
                        .map_err(|e| format!("Failed to add days to start date: {e}"))?;
                    let t = time(9, 0, 0, 0);
                    let dt = civil::DateTime::from_parts(date, t);
                    Ok(LooseDateTime::from_local_datetime(dt))
                }
            },
            DateTimeAnchor::Relative(n) => Ok(start.clone() + Span::new().seconds(n)),
            DateTimeAnchor::DateTime(dt) => Ok(dt),
            DateTimeAnchor::Time(t) => {
                let mut date = start.date();
                // If the time has already passed today, use tomorrow
                if start.time().is_some_and(|s| s >= t) {
                    date = date
                        .checked_add(Span::new().days(1))
                        .map_err(|e| format!("Failed to add day to date: {e}"))?;
                }
                let dt = civil::DateTime::from_parts(date, t);
                Ok(LooseDateTime::from_local_datetime(dt))
            }
        }
    }

    /// Resolve the `DateTimeAnchor` to a `LooseDateTime` starting from the provided `Zoned`.
    ///
    /// # Errors
    ///
    /// Returns an error if date/time operations fail.
    pub fn resolve_since_zoned(self, start: &Zoned) -> Result<LooseDateTime, String> {
        match self {
            DateTimeAnchor::InDays(n) => match n {
                0 => Ok(next_suggested_time(&start.datetime())),
                _ => {
                    let date = start
                        .datetime()
                        .date()
                        .checked_add(Span::new().days(n))
                        .map_err(|e| format!("Failed to add days to start date: {e}"))?;
                    let t = time(9, 0, 0, 0);
                    let dt = civil::DateTime::from_parts(date, t);
                    Ok(LooseDateTime::from_local_datetime(dt))
                }
            },
            DateTimeAnchor::Relative(n) => {
                let zoned = start
                    .checked_add(Span::new().seconds(n))
                    .map_err(|e| format!("Failed to add relative seconds: {e}"))?;
                Ok(LooseDateTime::Local(zoned))
            }
            DateTimeAnchor::DateTime(dt) => Ok(dt),
            DateTimeAnchor::Time(t) => {
                let mut date = start.date();
                // If the time has already passed today, use tomorrow
                if start.time() >= t {
                    date = date
                        .checked_add(Span::new().days(1))
                        .map_err(|e| format!("Failed to add day to date: {e}"))?;
                }
                let dt = civil::DateTime::from_parts(date, t);
                Ok(LooseDateTime::from_local_datetime(dt))
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

        // Try datetime
        if let Ok(dt) = civil::DateTime::strptime("%Y-%m-%d %H:%M", t) {
            return Ok(Self::DateTime(LooseDateTime::from_local_datetime(dt)));
        }

        // Try date
        if let Ok(d) = Date::strptime("%Y-%m-%d", t) {
            return Ok(Self::DateTime(LooseDateTime::DateOnly(d)));
        }

        // Try date with current year
        if let Some(d) = parse_md_with_year(t, i32::from(date(2025, 1, 1).year())) {
            return Ok(Self::DateTime(LooseDateTime::DateOnly(d)));
        }

        // Try time
        if let Ok(time) = Time::strptime("%H:%M", t) {
            return Ok(Self::Time(time));
        }

        // Try durations
        if let Some(seconds) = parse_seconds(t) {
            return Ok(Self::Relative(seconds));
        }
        if let Some(minutes) = parse_minutes(t) {
            return Ok(Self::Relative(minutes * 60));
        }
        if let Some(hours) = parse_hours(t) {
            return Ok(Self::Relative(hours * 60 * 60));
        }
        if let Some(days) = parse_days(t) {
            return Ok(Self::InDays(days));
        }

        Err(format!("Invalid datetime anchor: {t}"))
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
    ($fn:ident, $re:expr) => {
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

const HOURS: [i8; 3] = [9, 13, 18];

fn next_suggested_time(now: &civil::DateTime) -> LooseDateTime {
    let date = now.date();
    let current_hour = now.hour();
    for &hour in &HOURS {
        if current_hour < hour {
            let dt =
                civil::DateTime::new(date.year(), date.month(), date.day(), hour, 0, 0, 0).unwrap();
            return LooseDateTime::from_local_datetime(dt);
        }
    }

    LooseDateTime::DateOnly(date)
}

fn first_suggested_time(date: Date) -> LooseDateTime {
    let dt =
        civil::DateTime::new(date.year(), date.month(), date.day(), HOURS[0], 0, 0, 0).unwrap();
    LooseDateTime::from_local_datetime(dt)
}

fn parse_md_with_year(s: &str, year: i32) -> Option<Date> {
    // Prepend year to the month-day string for parsing
    let full_str = format!("{year}-{s}");
    Date::strptime("%Y-%m-%d", &full_str).ok()
}

#[cfg(test)]
mod tests {
    use jiff::civil::{date, datetime};
    use jiff::tz::TimeZone;

    use super::*;

    #[test]
    fn resolves_now_anchor_to_current_time() {
        let now = date(2025, 1, 1)
            .at(15, 30, 45, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        assert_eq!(
            DateTimeAnchor::now().resolve_at_start_of_day(&now).unwrap(),
            now
        );
        assert_eq!(
            DateTimeAnchor::now().resolve_at_end_of_day(&now).unwrap(),
            now
        );
    }

    #[test]
    fn resolves_indays_anchor_to_day_boundary() {
        let now = date(2025, 1, 1)
            .at(15, 30, 45, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let anchor = DateTimeAnchor::InDays(1);

        let expected = date(2025, 1, 2)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let parsed = anchor.resolve_at_start_of_day(&now).unwrap();
        assert_eq!(parsed, expected);

        let parsed = anchor.resolve_at_end_of_day(&now).unwrap();
        assert!(
            parsed
                > date(2025, 1, 2)
                    .at(23, 59, 59, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap()
        );
        assert!(
            parsed
                < date(2025, 1, 3)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap()
        );
    }

    #[test]
    fn resolves_datetime_anchor_to_day_boundary() {
        let now = date(2025, 1, 1)
            .at(15, 30, 45, 0)
            .to_zoned(TimeZone::system())
            .unwrap();
        let d = date(2025, 1, 5);
        let anchor = DateTimeAnchor::DateTime(LooseDateTime::DateOnly(d));

        let parsed = anchor.resolve_at_start_of_day(&now).unwrap();
        let expected = date(2025, 1, 5)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::system())
            .unwrap();
        assert_eq!(parsed, expected);

        let parsed = anchor.resolve_at_end_of_day(&now).unwrap();
        assert!(
            parsed
                > date(2025, 1, 5)
                    .at(23, 59, 59, 0)
                    .to_zoned(TimeZone::system())
                    .unwrap()
        );
        assert!(
            parsed
                < date(2025, 1, 6)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::system())
                    .unwrap()
        );
    }

    #[test]
    fn resolves_various_anchor_types_correctly() {
        let now = date(2025, 1, 1)
            .at(15, 30, 45, 0)
            .to_zoned(TimeZone::system())
            .unwrap();
        for (name, anchor, expected) in [
            (
                "Relative (+2h30m45s)",
                DateTimeAnchor::Relative(2 * 60 * 60 + 30 * 60 + 45),
                date(2025, 1, 1)
                    .at(18, 1, 30, 0)
                    .to_zoned(TimeZone::system())
                    .unwrap(),
            ),
            (
                "Floating",
                {
                    let dt = datetime(2025, 1, 5, 14, 30, 0, 0);
                    DateTimeAnchor::DateTime(LooseDateTime::Floating(dt))
                },
                date(2025, 1, 5)
                    .at(14, 30, 0, 0)
                    .to_zoned(TimeZone::system())
                    .unwrap(),
            ),
            (
                "Local",
                {
                    let zoned = date(2025, 1, 5)
                        .at(14, 30, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap();
                    DateTimeAnchor::DateTime(LooseDateTime::Local(zoned))
                },
                date(2025, 1, 5)
                    .at(14, 30, 0, 0)
                    .to_zoned(TimeZone::system())
                    .unwrap(),
            ),
        ] {
            let parsed = anchor.resolve_at_start_of_day(&now).unwrap();
            assert_eq!(parsed, expected, "start_of_day failed for {name}");

            let parsed = anchor.resolve_at_end_of_day(&now).unwrap();
            assert_eq!(parsed, expected, "end_of_day failed for {name}");
        }
    }

    #[test]
    fn resolves_time_anchor_to_specific_time() {
        let t = time(14, 30, 0, 0);
        let anchor = DateTimeAnchor::Time(t);

        let now = date(2025, 1, 1)
            .at(10, 0, 0, 0)
            .to_zoned(TimeZone::system())
            .unwrap();
        let parsed_start = anchor.resolve_at_start_of_day(&now).unwrap();
        let parsed_end = anchor.resolve_at_end_of_day(&now).unwrap();

        // Both should have the same time (14:30) on the same date (2025-01-01)
        assert_eq!(parsed_start.date(), now.date());
        assert_eq!(parsed_start.time(), t);
        assert_eq!(parsed_end.date(), now.date());
        assert_eq!(parsed_end.time(), t);
    }

    #[test]
    fn resolves_anchor_from_loose_datetime() {
        let dt = |y, m, d, h, mm, s| {
            let zoned = date(y, m, d)
                .at(h, mm, s, 0)
                .to_zoned(TimeZone::system())
                .unwrap();
            LooseDateTime::Local(zoned)
        };

        let now: LooseDateTime = LooseDateTime::Local(
            date(2025, 1, 1)
                .at(12, 0, 0, 0)
                .to_zoned(TimeZone::system())
                .unwrap(),
        );

        for (name, anchor, expected) in [
            (
                "AtInDays (same datetime)",
                DateTimeAnchor::DateTime(LooseDateTime::Local(
                    date(2025, 1, 1)
                        .at(12, 0, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                )),
                LooseDateTime::Local(
                    date(2025, 1, 1)
                        .at(12, 0, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                ),
            ),
            (
                "Relative (+3 hours)",
                DateTimeAnchor::Relative(3 * 60 * 60),
                dt(2025, 1, 1, 15, 0, 0),
            ),
            (
                "DateTime (absolute 10:00)",
                DateTimeAnchor::DateTime(LooseDateTime::Local(
                    date(2025, 1, 1)
                        .at(10, 0, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                )),
                LooseDateTime::Local(
                    date(2025, 1, 1)
                        .at(10, 0, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                ),
            ),
            (
                "Time (10:00 today)",
                DateTimeAnchor::Time(time(10, 0, 0, 0)),
                LooseDateTime::Local(
                    date(2025, 1, 1)
                        .at(10, 0, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                ),
            ),
        ] {
            let result = anchor.resolve_at(&now);
            assert_eq!(result, expected, "resolve_at failed for case: {name}");
        }
    }

    #[test]
    fn resolves_anchor_since_loose_datetime() {
        let dt = |y, m, d, h, mm, s| {
            LooseDateTime::Local(
                date(y, m, d)
                    .at(h, mm, s, 0)
                    .to_zoned(TimeZone::system())
                    .unwrap(),
            )
        };

        let now: LooseDateTime = LooseDateTime::Local(
            date(2025, 1, 1)
                .at(12, 0, 0, 0)
                .to_zoned(TimeZone::system())
                .unwrap(),
        );

        for (name, anchor, expected) in [
            (
                "DateTime == now",
                DateTimeAnchor::DateTime(now.clone()),
                now.clone(),
            ),
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
                DateTimeAnchor::Time(time(10, 0, 0, 0)),
                dt(2025, 1, 2, 10, 0, 0),
            ),
            (
                "Time after now -> today 14:00",
                DateTimeAnchor::Time(time(14, 0, 0, 0)),
                dt(2025, 1, 1, 14, 0, 0),
            ),
        ] {
            let result = anchor.resolve_since(&now).unwrap();
            assert_eq!(result, expected, "case failed: {name}");
        }
    }

    #[test]
    fn resolves_anchor_since_zoned() {
        let now = date(2025, 1, 1)
            .at(12, 0, 0, 0)
            .to_zoned(TimeZone::system())
            .unwrap();

        for (name, anchor, expected) in [
            (
                "DateTimeAnchor::DateTime (same datetime)",
                DateTimeAnchor::DateTime(LooseDateTime::Local(now.clone())),
                LooseDateTime::Local(now.clone()),
            ),
            (
                "DateTimeAnchor::Relative (3h25m45s later)",
                DateTimeAnchor::Relative(3 * 60 * 60 + 25 * 60 + 45),
                LooseDateTime::Local(
                    date(2025, 1, 1)
                        .at(15, 25, 45, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                ),
            ),
            (
                "DateTimeAnchor::DateTime (specific datetime before now)",
                DateTimeAnchor::DateTime(LooseDateTime::Local(
                    date(2025, 1, 1)
                        .at(10, 0, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                )),
                LooseDateTime::Local(
                    date(2025, 1, 1)
                        .at(10, 0, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                ),
            ),
            (
                "DateTimeAnchor::Time (before now → tomorrow)",
                DateTimeAnchor::Time(time(10, 0, 0, 0)),
                LooseDateTime::Local(
                    date(2025, 1, 2)
                        .at(10, 0, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                ),
            ),
            (
                "DateTimeAnchor::Time (after now → today)",
                DateTimeAnchor::Time(time(14, 0, 0, 0)),
                LooseDateTime::Local(
                    date(2025, 1, 1)
                        .at(14, 0, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                ),
            ),
        ] {
            let result = anchor.resolve_since_zoned(&now).unwrap();
            assert_eq!(result, expected, "failed: {name} → resolve_since_zoned");
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
                DateTimeAnchor::DateTime(LooseDateTime::DateOnly(date(2025, 1, 15))),
            ),
            // DateTime
            (
                "2025-01-15 14:30",
                DateTimeAnchor::DateTime(LooseDateTime::Local(
                    date(2025, 1, 15)
                        .at(14, 30, 0, 0)
                        .to_zoned(TimeZone::system())
                        .unwrap(),
                )),
            ),
            // Time only
            ("14:30", DateTimeAnchor::Time(time(14, 30, 0, 0))),
        ] {
            let anchor: DateTimeAnchor = s.parse().unwrap();
            assert_eq!(anchor, expected, "Failed to parse '{s}'");
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
            let now = datetime(2025, 1, 1, hour, min, 0, 0);
            let result = next_suggested_time(&now);
            let zoned = date(2025, 1, 1)
                .at(expected_hour, 0, 0, 0)
                .to_zoned(TimeZone::system())
                .unwrap();
            let expected = LooseDateTime::Local(zoned);
            assert_eq!(result, expected, "{description}");
        }
    }

    #[test]
    fn suggests_dateonly_after_business_hours() {
        // After 6 PM, should suggest DateOnly (next day)
        let now = datetime(2025, 1, 1, 19, 30, 0, 0);
        let result = next_suggested_time(&now);
        let expected = LooseDateTime::DateOnly(date(2025, 1, 1));
        assert_eq!(result, expected, "After 6 PM, should suggest DateOnly");

        // Exactly at 6 PM, should suggest DateOnly (next day)
        let now = datetime(2025, 1, 1, 18, 0, 0, 0);
        let result = next_suggested_time(&now);
        let expected = LooseDateTime::DateOnly(date(2025, 1, 1));
        assert_eq!(result, expected, "Exactly at 6 PM, should suggest DateOnly");
    }
}
