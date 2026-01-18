// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::ops::Add;

use aimcal_ical as ical;
use aimcal_ical::Segments;
use jiff::civil::{self, Date, DateTime};
use jiff::tz::TimeZone;
use jiff::{Span, Zoned};

use crate::RangePosition;
use crate::datetime::util::{
    STABLE_FORMAT_DATEONLY, STABLE_FORMAT_FLOATING, STABLE_FORMAT_LOCAL, end_of_day, start_of_day,
};

/// A date and time that may be in different formats, such as date only, floating time, or local time with timezone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LooseDateTime {
    /// Date only without time.
    DateOnly(Date),
    /// Floating date and time without timezone.
    Floating(DateTime),
    /// Local date and time with timezone.
    /// NOTE: This is always in the local timezone of the system running the code.
    Local(Zoned),
}

impl LooseDateTime {
    /// The date part.
    #[must_use]
    pub fn date(&self) -> Date {
        match self {
            LooseDateTime::DateOnly(d) => *d,
            LooseDateTime::Floating(dt) => dt.date(),
            LooseDateTime::Local(zoned) => zoned.date(),
        }
    }

    /// The time part, if available.
    #[must_use]
    pub fn time(&self) -> Option<civil::Time> {
        match self {
            LooseDateTime::DateOnly(_) => None,
            LooseDateTime::Floating(dt) => Some(dt.time()),
            LooseDateTime::Local(zoned) => Some(zoned.time()),
        }
    }

    /// Converts to a datetime with default start time (00:00:00) if time is missing.
    pub fn with_start_of_day(&self) -> DateTime {
        let d = self.date();
        let t = self.time().unwrap_or_else(start_of_day);
        DateTime::from_parts(d, t)
    }

    /// Converts to a datetime with default end time (23:59:59.999999999) if time is missing.
    pub fn with_end_of_day(&self) -> DateTime {
        let d = self.date();
        let t = self.time().unwrap_or_else(end_of_day);
        DateTime::from_parts(d, t)
    }

    /// Determines the position of a given datetime relative to a start and optional end date.
    #[must_use]
    pub fn position_in_range(
        t: &DateTime,
        start: &Option<LooseDateTime>,
        end: &Option<LooseDateTime>,
    ) -> RangePosition {
        match (start, end) {
            (Some(start), Some(end)) => {
                let start_dt = start.with_start_of_day(); // 00:00
                let end_dt = end.with_end_of_day(); // 23:59
                if start_dt > end_dt {
                    RangePosition::InvalidRange
                } else if t > &end_dt {
                    RangePosition::After
                } else if t < &start_dt {
                    RangePosition::Before
                } else {
                    RangePosition::InRange
                }
            }
            (Some(start), None) => match t >= &start.with_start_of_day() {
                true => RangePosition::InRange,
                false => RangePosition::Before,
            },
            (None, Some(end)) => match t > &end.with_end_of_day() {
                true => RangePosition::After,
                false => RangePosition::InRange,
            },
            (None, None) => RangePosition::InvalidRange,
        }
    }

    /// Creates a `LooseDateTime` from a `DateTime` in the local timezone.
    pub(crate) fn from_local_datetime(dt: DateTime) -> LooseDateTime {
        // Try to interpret the datetime in the system timezone
        let tz = TimeZone::system();
        match dt.to_zoned(tz) {
            Ok(zoned) => LooseDateTime::Local(zoned),
            Err(_) => {
                // Fallback to floating if timezone conversion fails
                tracing::warn!(
                    ?dt,
                    "failed to convert to local timezone, treating as floating"
                );
                LooseDateTime::Floating(dt)
            }
        }
    }

    /// Converts to a string representation of date and time.
    pub(crate) fn format_stable(&self) -> String {
        match self {
            LooseDateTime::DateOnly(d) => d.strftime(STABLE_FORMAT_DATEONLY).to_string(),
            LooseDateTime::Floating(dt) => dt.strftime(STABLE_FORMAT_FLOATING).to_string(),
            LooseDateTime::Local(zoned) => zoned.strftime(STABLE_FORMAT_LOCAL).to_string(),
        }
    }

    pub(crate) fn parse_stable(s: &str) -> Option<Self> {
        match s.len() {
            // 2006-01-02
            10 => Date::strptime(STABLE_FORMAT_DATEONLY, s)
                .ok()
                .map(Self::DateOnly),
            // 2006-01-02T15:04:05
            19 => DateTime::strptime(STABLE_FORMAT_FLOATING, s)
                .ok()
                .map(Self::Floating),
            // 2006-01-02T15:04:05Z or 2006-01-02T15:04:05+00:00
            20.. => Zoned::strptime(STABLE_FORMAT_LOCAL, s)
                .ok()
                .map(Self::Local),
            _ => None,
        }
    }
}

impl From<ical::DateTime<Segments<'_>>> for LooseDateTime {
    #[tracing::instrument]
    fn from(dt: ical::DateTime<Segments<'_>>) -> Self {
        match dt {
            ical::DateTime::Floating { date, time, .. } => {
                let civil_dt = DateTime::from_parts(date.civil_date(), time.civil_time());
                LooseDateTime::Floating(civil_dt)
            }
            ical::DateTime::Utc { date, time, .. } => {
                let civil_dt = DateTime::from_parts(date.civil_date(), time.civil_time());
                LooseDateTime::Local(civil_dt.to_zoned(TimeZone::UTC).unwrap())
            }
            ical::DateTime::Zoned {
                date, time, tz_id, ..
            } => {
                let civil_dt = DateTime::from_parts(date.civil_date(), time.civil_time());
                let tz_id_str = tz_id.to_string();
                match TimeZone::get(tz_id_str.as_str()) {
                    Ok(tz) => match civil_dt.to_zoned(tz) {
                        Ok(zoned) => LooseDateTime::Local(zoned),
                        Err(_) => {
                            tracing::warn!(tzid = %tz_id_str, "unknown timezone, treating as floating");
                            LooseDateTime::Floating(civil_dt)
                        }
                    },
                    Err(_) => {
                        tracing::warn!(tzid = %tz_id_str, "unknown timezone, treating as floating");
                        LooseDateTime::Floating(civil_dt)
                    }
                }
            }
            ical::DateTime::Date { date, .. } => LooseDateTime::DateOnly(date.into()),
        }
    }
}

impl From<ical::DateTime<String>> for LooseDateTime {
    fn from(dt: ical::DateTime<String>) -> Self {
        match dt {
            ical::DateTime::Floating { date, time, .. } => {
                let civil_dt = DateTime::from_parts(date.civil_date(), time.civil_time());
                LooseDateTime::Floating(civil_dt)
            }
            ical::DateTime::Utc { date, time, .. } => {
                let civil_dt = DateTime::from_parts(date.civil_date(), time.civil_time());
                LooseDateTime::Local(civil_dt.to_zoned(TimeZone::UTC).unwrap())
            }
            ical::DateTime::Zoned {
                date, time, tz_id, ..
            } => {
                let civil_dt = DateTime::from_parts(date.civil_date(), time.civil_time());
                match TimeZone::get(tz_id.as_str()) {
                    Ok(tz) => match civil_dt.to_zoned(tz) {
                        Ok(zoned) => LooseDateTime::Local(zoned),
                        Err(_) => {
                            tracing::warn!(tzid = %tz_id, "unknown timezone, treating as floating");
                            LooseDateTime::Floating(civil_dt)
                        }
                    },
                    Err(_) => {
                        tracing::warn!(tzid = %tz_id, "unknown timezone, treating as floating");
                        LooseDateTime::Floating(civil_dt)
                    }
                }
            }
            ical::DateTime::Date { date, .. } => LooseDateTime::DateOnly(date.into()),
        }
    }
}

impl From<LooseDateTime> for ical::DateTime<String> {
    #[allow(clippy::cast_sign_loss)]
    fn from(dt: LooseDateTime) -> Self {
        match dt {
            LooseDateTime::DateOnly(d) => ical::DateTime::Date {
                date: d.into(),
                x_parameters: Vec::new(),
                retained_parameters: Vec::new(),
            },
            LooseDateTime::Floating(dt) => ical::DateTime::Floating {
                date: dt.date().into(),
                time: dt.time().into(),
                x_parameters: Vec::new(),
                retained_parameters: Vec::new(),
            },
            LooseDateTime::Local(zoned) => {
                let tz = zoned.time_zone();
                if *tz != TimeZone::UTC
                    && let Some(tz_name) = tz.iana_name()
                {
                    ical::DateTime::Zoned {
                        date: zoned.date().into(),
                        time: zoned.time().into(),
                        tz_id: tz_name.to_string(),
                        tz_jiff: zoned.time_zone().clone(),
                        x_parameters: Vec::new(),
                        retained_parameters: Vec::new(),
                    }
                } else {
                    // Conveto UTC for iCalendar output
                    let utc_dt = zoned.with_time_zone(TimeZone::UTC);
                    ical::DateTime::Utc {
                        date: utc_dt.date().into(),
                        time: utc_dt.time().into(),
                        x_parameters: Vec::new(),
                        retained_parameters: Vec::new(),
                    }
                }
            }
        }
    }
}

impl From<Date> for LooseDateTime {
    fn from(d: Date) -> Self {
        LooseDateTime::DateOnly(d)
    }
}

impl From<DateTime> for LooseDateTime {
    fn from(dt: DateTime) -> Self {
        LooseDateTime::Floating(dt)
    }
}

impl From<Zoned> for LooseDateTime {
    fn from(zoned: Zoned) -> Self {
        LooseDateTime::Local(zoned)
    }
}

impl Add<Span> for LooseDateTime {
    type Output = Self;

    fn add(self, rhs: Span) -> Self::Output {
        match self {
            LooseDateTime::DateOnly(d) => LooseDateTime::DateOnly(d.checked_add(rhs).unwrap()),
            LooseDateTime::Floating(dt) => LooseDateTime::Floating(dt.checked_add(rhs).unwrap()),
            LooseDateTime::Local(zoned) => LooseDateTime::Local(zoned.checked_add(rhs).unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use jiff::Span;
    use jiff::civil::{date, datetime, time};
    use jiff::tz::TimeZone;

    use super::*;

    #[test]
    fn provides_date_and_time_accessors() {
        let date = date(2024, 7, 18);
        let time = time(12, 30, 45, 0);
        let datetime = datetime(2024, 7, 18, 12, 30, 45, 0);
        let tz = TimeZone::UTC;
        let zoned_dt = datetime.to_zoned(tz).unwrap();

        let d1 = LooseDateTime::DateOnly(date);
        let d2 = LooseDateTime::Floating(datetime);
        let d3 = LooseDateTime::Local(zoned_dt);

        // Date
        assert_eq!(d1.date(), date);
        assert_eq!(d2.date(), date);
        assert_eq!(d3.date(), date);

        // Time
        assert_eq!(d1.time(), None);
        assert_eq!(d2.time(), Some(time));
        assert_eq!(d3.time(), Some(time));
    }

    #[test]
    fn sets_time_to_start_of_day() {
        let d = date(2024, 7, 18);
        let t = time(12, 30, 0, 0);
        let datetime = DateTime::from_parts(d, t);
        let tz = TimeZone::UTC;
        let zoned_dt = datetime.to_zoned(tz).unwrap();

        let d1 = LooseDateTime::DateOnly(d);
        let d2 = LooseDateTime::Floating(datetime);
        let d3 = LooseDateTime::Local(zoned_dt);

        assert_eq!(
            d1.with_start_of_day(),
            DateTime::from_parts(d, time(0, 0, 0, 0))
        );
        assert_eq!(d2.with_start_of_day(), datetime);
        assert_eq!(d3.with_start_of_day(), datetime);
    }

    #[test]
    fn sets_time_to_end_of_day() {
        let d = date(2024, 7, 18);
        let t = time(12, 30, 0, 0);
        let datetime = DateTime::from_parts(d, t);
        let tz = TimeZone::UTC;
        let zoned_dt = datetime.to_zoned(tz).unwrap();

        let d1 = LooseDateTime::DateOnly(d);
        let d2 = LooseDateTime::Floating(datetime);
        let d3 = LooseDateTime::Local(zoned_dt);

        assert_eq!(
            d1.with_end_of_day(),
            DateTime::from_parts(d, time(23, 59, 59, 999_999_999))
        );
        assert_eq!(d2.with_end_of_day(), datetime);
        assert_eq!(d3.with_end_of_day(), datetime);
    }

    #[test]
    fn calculates_position_in_date_date_range() {
        let start = LooseDateTime::DateOnly(date(2024, 1, 1));
        let end = LooseDateTime::DateOnly(date(2024, 1, 3));

        let t_before = datetime(2023, 12, 31, 23, 59, 59, 0);
        let t_in_s = datetime(2024, 1, 1, 12, 0, 0, 0);
        let t_in_e = datetime(2024, 1, 3, 12, 0, 0, 0);
        let t_after = datetime(2024, 1, 4, 0, 0, 0, 0);

        assert_eq!(
            LooseDateTime::position_in_range(&t_before, &Some(start.clone()), &Some(end.clone())),
            RangePosition::Before
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_s, &Some(start.clone()), &Some(end.clone())),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_e, &Some(start.clone()), &Some(end.clone())),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_after, &Some(start), &Some(end)),
            RangePosition::After
        );
    }

    #[test]
    fn calculates_position_in_date_floating_range() {
        let start = LooseDateTime::DateOnly(date(2024, 1, 1));
        let end = LooseDateTime::Floating(datetime(2024, 1, 3, 13, 0, 0, 0));

        let t_before = datetime(2023, 12, 31, 23, 59, 59, 0);
        let t_in_s = datetime(2024, 1, 1, 12, 0, 0, 0);
        let t_in_e = datetime(2024, 1, 3, 12, 0, 0, 0);
        let t_after = datetime(2024, 1, 3, 14, 0, 0, 0);

        assert_eq!(
            LooseDateTime::position_in_range(&t_before, &Some(start.clone()), &Some(end.clone())),
            RangePosition::Before
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_s, &Some(start.clone()), &Some(end.clone())),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_e, &Some(start.clone()), &Some(end.clone())),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_after, &Some(start), &Some(end)),
            RangePosition::After
        );
    }

    #[test]
    fn calculates_position_in_floating_date_range() {
        let start = LooseDateTime::Floating(datetime(2024, 1, 1, 13, 0, 0, 0));
        let end = LooseDateTime::DateOnly(date(2024, 1, 1));

        let t_before = datetime(2024, 1, 1, 12, 0, 0, 0);
        let t_in_s = datetime(2024, 1, 1, 14, 0, 0, 0);
        let t_in_e = datetime(2024, 1, 1, 23, 59, 59, 0);
        let t_after = datetime(2024, 1, 2, 0, 0, 0, 0);

        assert_eq!(
            LooseDateTime::position_in_range(&t_before, &Some(start.clone()), &Some(end.clone())),
            RangePosition::Before
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_s, &Some(start.clone()), &Some(end.clone())),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_e, &Some(start.clone()), &Some(end.clone())),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_after, &Some(start), &Some(end)),
            RangePosition::After
        );
    }

    #[test]
    fn calculates_position_with_end_only() {
        let t1 = datetime(2023, 12, 31, 23, 59, 59, 0);
        let t2 = datetime(2024, 1, 1, 20, 0, 0, 0);

        for end in [
            LooseDateTime::DateOnly(date(2023, 12, 31)),
            LooseDateTime::Floating(datetime(2023, 12, 31, 23, 59, 59, 0)),
        ] {
            assert_eq!(
                LooseDateTime::position_in_range(&t1, &None, &Some(end.clone())),
                RangePosition::InRange,
                "end = {end:?}"
            );
            assert_eq!(
                LooseDateTime::position_in_range(&t2, &None, &Some(end.clone())),
                RangePosition::After,
                "end = {end:?}"
            );
        }
    }

    #[test]
    fn calculates_position_with_start_only() {
        let t1 = datetime(2023, 12, 31, 23, 59, 59, 0);
        let t2 = datetime(2024, 1, 1, 0, 0, 0, 0);

        for start in [
            LooseDateTime::DateOnly(date(2024, 1, 1)),
            LooseDateTime::Floating(datetime(2024, 1, 1, 0, 0, 0, 0)),
        ] {
            assert_eq!(
                LooseDateTime::position_in_range(&t1, &Some(start.clone()), &None),
                RangePosition::Before,
                "start = {start:?}"
            );
            assert_eq!(
                LooseDateTime::position_in_range(&t2, &Some(start.clone()), &None),
                RangePosition::InRange,
                "start = {start:?}"
            );
        }
    }

    #[test]
    fn returns_invalid_range_for_inverted_or_missing_bounds() {
        let start = LooseDateTime::DateOnly(date(2024, 1, 5));
        let end = LooseDateTime::DateOnly(date(2024, 1, 1));

        let t = datetime(2024, 1, 3, 12, 0, 0, 0);

        assert_eq!(
            LooseDateTime::position_in_range(&t, &Some(start), &Some(end)),
            RangePosition::InvalidRange
        );

        assert_eq!(
            LooseDateTime::position_in_range(&t, &None, &None),
            RangePosition::InvalidRange
        );
    }

    #[test]
    fn creates_from_local_datetime() {
        // Test with a valid datetime
        let date = date(2021, 1, 1);
        let time = time(0, 0, 0, 0);
        let datetime = DateTime::from_parts(date, time);
        let loose_dt = LooseDateTime::from_local_datetime(datetime);

        // Should convert to Local variant
        assert!(matches!(loose_dt, LooseDateTime::Local(_)));
    }

    #[test]
    fn serializes_and_deserializes_stably() {
        let date = date(2024, 7, 18);
        let time = time(12, 30, 45, 0);
        let datetime = DateTime::from_parts(date, time);
        let tz = TimeZone::UTC;
        let local = datetime.to_zoned(tz).unwrap();

        let d1 = LooseDateTime::DateOnly(date);
        let d2 = LooseDateTime::Floating(datetime);
        let d3 = LooseDateTime::Local(local.clone());

        // Format
        let f1 = d1.format_stable();
        let f2 = d2.format_stable();
        let f3 = d3.format_stable();

        assert_eq!(f1, "2024-07-18");
        assert_eq!(f2, "2024-07-18T12:30:45");
        assert!(f3.starts_with("2024-07-18T12:30:45"));

        // Parse
        assert_eq!(LooseDateTime::parse_stable(&f1), Some(d1));
        assert_eq!(LooseDateTime::parse_stable(&f2), Some(d2));
        let parsed3 = LooseDateTime::parse_stable(&f3);
        if let Some(LooseDateTime::Local(zoned)) = parsed3 {
            assert_eq!(zoned.datetime(), local.datetime());
        } else {
            panic!("Failed to parse local datetime");
        }
    }

    #[test]
    fn adds_span_to_dateonly() {
        let d = date(2025, 1, 1);
        let added = LooseDateTime::DateOnly(d) + Span::new().days(2).hours(3);
        let expected = LooseDateTime::DateOnly(date(2025, 1, 3));
        assert_eq!(added, expected);
    }

    #[test]
    fn adds_span_to_floating() {
        let d = date(2025, 1, 1);
        let t = time(12, 30, 45, 0);
        let dt = LooseDateTime::Floating(DateTime::from_parts(d, t));
        let added = dt + Span::new().days(2).hours(3);
        let expected_date = date(2025, 1, 3);
        let expected_time = time(15, 30, 45, 0);
        let excepted = LooseDateTime::Floating(DateTime::from_parts(expected_date, expected_time));
        assert_eq!(added, excepted);
    }

    #[test]
    fn adds_span_to_local() {
        let tz = TimeZone::UTC;
        let d = date(2025, 1, 1);
        let t = time(12, 30, 45, 0);
        let datetime = DateTime::from_parts(d, t);
        let zoned = datetime.to_zoned(tz.clone()).unwrap();
        let added = LooseDateTime::Local(zoned.clone()) + Span::new().days(2).hours(3);
        let expected_date = date(2025, 1, 3);
        let expected_time = time(15, 30, 45, 0);
        let expected_datetime = DateTime::from_parts(expected_date, expected_time);
        let excepted = LooseDateTime::Local(expected_datetime.to_zoned(tz).unwrap());
        assert_eq!(added, excepted);
    }
}
