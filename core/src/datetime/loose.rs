// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::ops::Add;

use aimcal_ical::Segments;
use aimcal_ical::{DateTime as IcalDateTime, Time};
use chrono::{
    DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc,
    offset::LocalResult,
};
use chrono_tz::Tz;

use crate::RangePosition;
use crate::datetime::util::{
    STABLE_FORMAT_DATEONLY, STABLE_FORMAT_FLOATING, STABLE_FORMAT_LOCAL, end_of_day_naive,
    start_of_day_naive,
};

/// A date and time that may be in different formats, such as date only, floating time, or local time with timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LooseDateTime {
    /// Date only without time.
    DateOnly(NaiveDate),

    /// Floating date and time without timezone.
    Floating(NaiveDateTime),

    /// Local date and time with timezone.
    /// NOTE: This is always in the local timezone of the system running the code.
    Local(DateTime<Local>),
}

impl LooseDateTime {
    /// The date part.
    #[must_use]
    pub fn date(&self) -> NaiveDate {
        match self {
            LooseDateTime::DateOnly(d) => *d,
            LooseDateTime::Floating(dt) => dt.date(),
            LooseDateTime::Local(dt) => dt.date_naive(),
        }
    }

    /// The time part, if available.
    #[must_use]
    pub fn time(&self) -> Option<NaiveTime> {
        match self {
            LooseDateTime::DateOnly(_) => None,
            LooseDateTime::Floating(dt) => Some(dt.time()),
            LooseDateTime::Local(dt) => Some(dt.time()),
        }
    }

    /// Converts to a datetime with default start time (00:00:00) if time is missing.
    pub fn with_start_of_day(&self) -> NaiveDateTime {
        NaiveDateTime::new(self.date(), self.time().unwrap_or_else(start_of_day_naive))
    }

    /// Converts to a datetime with default end time (23:59:59.999999999) if time is missing.
    pub fn with_end_of_day(&self) -> NaiveDateTime {
        NaiveDateTime::new(self.date(), self.time().unwrap_or_else(end_of_day_naive))
    }

    /// Determines the position of a given datetime relative to a start and optional end date.
    #[must_use]
    pub fn position_in_range(
        t: &NaiveDateTime,
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

    /// Creates a `LooseDateTime` from a `NaiveDateTime` in the local timezone.
    pub(crate) fn from_local_datetime(dt: NaiveDateTime) -> LooseDateTime {
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

    /// Converts to a string representation of date and time.
    pub(crate) fn format_stable(&self) -> String {
        match self {
            LooseDateTime::DateOnly(d) => d.format(STABLE_FORMAT_DATEONLY).to_string(),
            LooseDateTime::Floating(dt) => dt.format(STABLE_FORMAT_FLOATING).to_string(),
            LooseDateTime::Local(dt) => dt.format(STABLE_FORMAT_LOCAL).to_string(),
        }
    }

    pub(crate) fn parse_stable(s: &str) -> Option<Self> {
        match s.len() {
            // 2006-01-02
            10 => NaiveDate::parse_from_str(s, STABLE_FORMAT_DATEONLY)
                .map(Self::DateOnly)
                .ok(),

            // 2006-01-02T15:04:05
            19 => NaiveDateTime::parse_from_str(s, STABLE_FORMAT_FLOATING)
                .map(Self::Floating)
                .ok(),

            // 2006-01-02T15:04:05Z
            20.. => DateTime::parse_from_str(s, STABLE_FORMAT_LOCAL)
                .map(|a| Self::Local(a.with_timezone(&Local)))
                .ok(),

            _ => None,
        }
    }
}

impl From<IcalDateTime<Segments<'_>>> for LooseDateTime {
    #[tracing::instrument]
    #[expect(clippy::cast_sign_loss)]
    fn from(dt: IcalDateTime<Segments<'_>>) -> Self {
        match dt {
            IcalDateTime::Floating { date, time, .. } => {
                let naive_dt = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        i32::from(date.year),
                        date.month as u32,
                        date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(
                        u32::from(time.hour),
                        u32::from(time.minute),
                        u32::from(time.second),
                    )
                    .unwrap(),
                );
                LooseDateTime::Floating(naive_dt)
            }
            IcalDateTime::Utc { date, time, .. } => {
                let naive_dt = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        i32::from(date.year),
                        date.month as u32,
                        date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(
                        u32::from(time.hour),
                        u32::from(time.minute),
                        u32::from(time.second),
                    )
                    .unwrap(),
                );
                LooseDateTime::Local(
                    DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).with_timezone(&Local),
                )
            }
            IcalDateTime::Zoned {
                date, time, tz_id, ..
            } => {
                let naive_dt = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        i32::from(date.year),
                        date.month as u32,
                        date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(
                        u32::from(time.hour),
                        u32::from(time.minute),
                        u32::from(time.second),
                    )
                    .unwrap(),
                );
                let tz_id_str = tz_id.to_string();
                match tz_id_str.parse::<Tz>() {
                    Ok(tz) => match tz.from_local_datetime(&naive_dt) {
                        LocalResult::Single(dt_in_tz) => dt_in_tz.into(),
                        LocalResult::Ambiguous(dt1, _) => {
                            tracing::warn!(tzid = %tz_id_str, "ambiguous local time, picking earliest");
                            dt1.into()
                        }
                        LocalResult::None => {
                            tracing::warn!(tzid = %tz_id_str, "invalid local time, falling back to floating");
                            LooseDateTime::Floating(naive_dt)
                        }
                    },
                    Err(_) => {
                        tracing::warn!(tzid = %tz_id_str, "unknown timezone, treating as floating");
                        LooseDateTime::Floating(naive_dt)
                    }
                }
            }
            IcalDateTime::Date { date, .. } => LooseDateTime::DateOnly(
                NaiveDate::from_ymd_opt(i32::from(date.year), date.month as u32, date.day as u32)
                    .unwrap(),
            ),
        }
    }
}

impl From<IcalDateTime<String>> for LooseDateTime {
    #[expect(clippy::cast_sign_loss)]
    fn from(dt: IcalDateTime<String>) -> Self {
        match dt {
            IcalDateTime::Floating { date, time, .. } => {
                let naive_dt = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        i32::from(date.year),
                        date.month as u32,
                        date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(
                        u32::from(time.hour),
                        u32::from(time.minute),
                        u32::from(time.second),
                    )
                    .unwrap(),
                );
                LooseDateTime::Floating(naive_dt)
            }
            IcalDateTime::Utc { date, time, .. } => {
                let naive_dt = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        i32::from(date.year),
                        date.month as u32,
                        date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(
                        u32::from(time.hour),
                        u32::from(time.minute),
                        u32::from(time.second),
                    )
                    .unwrap(),
                );
                LooseDateTime::Local(
                    DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).with_timezone(&Local),
                )
            }
            IcalDateTime::Zoned {
                date, time, tz_id, ..
            } => {
                let naive_dt = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        i32::from(date.year),
                        date.month as u32,
                        date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(
                        u32::from(time.hour),
                        u32::from(time.minute),
                        u32::from(time.second),
                    )
                    .unwrap(),
                );
                match tz_id.parse::<Tz>() {
                    Ok(tz) => match tz.from_local_datetime(&naive_dt) {
                        LocalResult::Single(dt_in_tz) => dt_in_tz.into(),
                        LocalResult::Ambiguous(dt1, _) => {
                            tracing::warn!(tzid = %tz_id, "ambiguous local time, picking earliest");
                            dt1.into()
                        }
                        LocalResult::None => {
                            tracing::warn!(tzid = %tz_id, "invalid local time, falling back to floating");
                            LooseDateTime::Floating(naive_dt)
                        }
                    },
                    Err(_) => {
                        tracing::warn!(tzid = %tz_id, "unknown timezone, treating as floating");
                        LooseDateTime::Floating(naive_dt)
                    }
                }
            }
            IcalDateTime::Date { date, .. } => LooseDateTime::DateOnly(
                NaiveDate::from_ymd_opt(i32::from(date.year), date.month as u32, date.day as u32)
                    .unwrap(),
            ),
        }
    }
}

impl From<LooseDateTime> for IcalDateTime<String> {
    #[expect(clippy::cast_possible_truncation)]
    fn from(dt: LooseDateTime) -> Self {
        match dt {
            LooseDateTime::DateOnly(d) => IcalDateTime::Date {
                date: aimcal_ical::value::ValueDate {
                    year: d.year() as i16,
                    month: d.month() as i8,
                    day: d.day() as i8,
                },
                x_parameters: Vec::new(),
                retained_parameters: Vec::new(),
            },
            LooseDateTime::Floating(naive_dt) => {
                let time = Time::new(
                    naive_dt.hour() as u8,
                    naive_dt.minute() as u8,
                    naive_dt.second() as u8,
                )
                .expect("time values should be valid");
                IcalDateTime::Floating {
                    date: aimcal_ical::value::ValueDate {
                        year: naive_dt.year() as i16,
                        month: naive_dt.month() as i8,
                        day: naive_dt.day() as i8,
                    },
                    time,
                    x_parameters: Vec::new(),
                    retained_parameters: Vec::new(),
                }
            }
            LooseDateTime::Local(dt) => {
                // For owned data, use UTC instead of trying to get system timezone
                // This avoids the complexity of tz_jiff field when jiff feature is enabled
                let utc_dt = dt.with_timezone(&Utc);
                let time = Time::new(
                    utc_dt.hour() as u8,
                    utc_dt.minute() as u8,
                    utc_dt.second() as u8,
                )
                .expect("time values should be valid");
                IcalDateTime::Utc {
                    date: aimcal_ical::value::ValueDate {
                        year: utc_dt.year() as i16,
                        month: utc_dt.month() as i8,
                        day: utc_dt.day() as i8,
                    },
                    time,
                    x_parameters: Vec::new(),
                    retained_parameters: Vec::new(),
                }
            }
        }
    }
}

impl From<NaiveDate> for LooseDateTime {
    fn from(d: NaiveDate) -> Self {
        LooseDateTime::DateOnly(d)
    }
}

impl From<NaiveDateTime> for LooseDateTime {
    fn from(dt: NaiveDateTime) -> Self {
        LooseDateTime::Floating(dt)
    }
}

impl<Tz: TimeZone> From<DateTime<Tz>> for LooseDateTime {
    fn from(dt: DateTime<Tz>) -> Self {
        LooseDateTime::Local(dt.with_timezone(&Local))
    }
}

impl Add<chrono::TimeDelta> for LooseDateTime {
    type Output = Self;

    fn add(self, rhs: chrono::TimeDelta) -> Self::Output {
        match self {
            LooseDateTime::DateOnly(d) => LooseDateTime::DateOnly(d.add(rhs)),
            LooseDateTime::Floating(dt) => LooseDateTime::Floating(dt.add(rhs)),
            LooseDateTime::Local(dt) => LooseDateTime::Local(dt.add(rhs)),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;

    #[test]
    fn provides_date_and_time_accessors() {
        let date = NaiveDate::from_ymd_opt(2024, 7, 18).unwrap();
        let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
        let datetime = NaiveDateTime::new(date, time);
        let local_dt = Local.with_ymd_and_hms(2024, 7, 18, 12, 30, 45).unwrap();

        let d1 = LooseDateTime::DateOnly(date);
        let d2 = LooseDateTime::Floating(datetime);
        let d3 = LooseDateTime::Local(local_dt);

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
        let date = NaiveDate::from_ymd_opt(2024, 7, 18).unwrap();
        let time = NaiveTime::from_hms_opt(12, 30, 0).unwrap();
        let datetime = NaiveDateTime::new(date, time);
        let local_dt = Local.with_ymd_and_hms(2024, 7, 18, 12, 30, 0).unwrap();

        let d1 = LooseDateTime::DateOnly(date);
        let d2 = LooseDateTime::Floating(datetime);
        let d3 = LooseDateTime::Local(local_dt);

        assert_eq!(
            d1.with_start_of_day(),
            NaiveDateTime::new(date, NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        );
        assert_eq!(d2.with_start_of_day(), datetime);
        assert_eq!(d3.with_start_of_day(), datetime);
    }

    #[test]
    fn sets_time_to_end_of_day() {
        let date = NaiveDate::from_ymd_opt(2024, 7, 18).unwrap();
        let time = NaiveTime::from_hms_opt(12, 30, 0).unwrap();
        let datetime = NaiveDateTime::new(date, time);
        let local_dt = Local.with_ymd_and_hms(2024, 7, 18, 12, 30, 0).unwrap();

        let d1 = LooseDateTime::DateOnly(date);
        let d2 = LooseDateTime::Floating(datetime);
        let d3 = LooseDateTime::Local(local_dt);

        assert_eq!(
            d1.with_end_of_day(),
            NaiveDateTime::new(
                date,
                NaiveTime::from_hms_nano_opt(23, 59, 59, 1_999_999_999).unwrap()
            )
        );
        assert_eq!(d2.with_end_of_day(), datetime);
        assert_eq!(d3.with_end_of_day(), datetime);
    }

    #[expect(clippy::many_single_char_names)]
    fn datetime(y: i32, m: u32, d: u32, h: u32, mm: u32, s: u32) -> Option<NaiveDateTime> {
        NaiveDate::from_ymd_opt(y, m, d).and_then(|a| a.and_hms_opt(h, mm, s))
    }

    #[test]
    fn calculates_position_in_date_date_range() {
        let start = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let end = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 3).unwrap());

        let t_before = datetime(2023, 12, 31, 23, 59, 59).unwrap();
        let t_in_s = datetime(2024, 1, 1, 12, 0, 0).unwrap();
        let t_in_e = datetime(2024, 1, 3, 12, 0, 0).unwrap();
        let t_after = datetime(2024, 1, 4, 0, 0, 0).unwrap();

        assert_eq!(
            LooseDateTime::position_in_range(&t_before, &Some(start), &Some(end)),
            RangePosition::Before
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_s, &Some(start), &Some(end)),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_e, &Some(start), &Some(end)),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_after, &Some(start), &Some(end)),
            RangePosition::After
        );
    }

    #[test]
    fn calculates_position_in_date_floating_range() {
        let start = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let end = LooseDateTime::Floating(datetime(2024, 1, 3, 13, 0, 0).unwrap());

        let t_before = datetime(2023, 12, 31, 23, 59, 59).unwrap();
        let t_in_s = datetime(2024, 1, 1, 12, 0, 0).unwrap();
        let t_in_e = datetime(2024, 1, 3, 12, 0, 0).unwrap();
        let t_after = datetime(2024, 1, 3, 14, 0, 0).unwrap();

        assert_eq!(
            LooseDateTime::position_in_range(&t_before, &Some(start), &Some(end)),
            RangePosition::Before
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_s, &Some(start), &Some(end)),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_e, &Some(start), &Some(end)),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_after, &Some(start), &Some(end)),
            RangePosition::After
        );
    }

    #[test]
    fn calculates_position_in_floating_date_range() {
        let start = LooseDateTime::Floating(datetime(2024, 1, 1, 13, 0, 0).unwrap());
        let end = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        let t_before = datetime(2024, 1, 1, 12, 0, 0).unwrap();
        let t_in_s = datetime(2024, 1, 1, 14, 0, 0).unwrap();
        let t_in_e = datetime(2024, 1, 1, 23, 59, 59).unwrap();
        let t_after = datetime(2024, 1, 2, 0, 0, 0).unwrap();

        assert_eq!(
            LooseDateTime::position_in_range(&t_before, &Some(start), &Some(end)),
            RangePosition::Before
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_s, &Some(start), &Some(end)),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in_e, &Some(start), &Some(end)),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_after, &Some(start), &Some(end)),
            RangePosition::After
        );
    }

    #[test]
    fn calculates_position_with_end_only() {
        let t1 = datetime(2023, 12, 31, 23, 59, 59).unwrap();
        let t2 = datetime(2024, 1, 1, 20, 0, 0).unwrap();

        for end in [
            LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
            LooseDateTime::Floating(datetime(2023, 12, 31, 23, 59, 59).unwrap()),
            LooseDateTime::Local(Local.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap()),
        ] {
            assert_eq!(
                LooseDateTime::position_in_range(&t1, &None, &Some(end)),
                RangePosition::InRange,
                "end = {end:?}"
            );
            assert_eq!(
                LooseDateTime::position_in_range(&t2, &None, &Some(end)),
                RangePosition::After,
                "end = {end:?}"
            );
        }
    }

    #[test]
    fn calculates_position_with_start_only() {
        let t1 = datetime(2023, 12, 31, 23, 59, 59).unwrap();
        let t2 = datetime(2024, 1, 1, 0, 0, 0).unwrap();

        for start in [
            LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            LooseDateTime::Floating(datetime(2024, 1, 1, 0, 0, 0).unwrap()),
            LooseDateTime::Local(Local.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()),
        ] {
            assert_eq!(
                LooseDateTime::position_in_range(&t1, &Some(start), &None),
                RangePosition::Before,
                "start = {start:?}"
            );
            assert_eq!(
                LooseDateTime::position_in_range(&t2, &Some(start), &None),
                RangePosition::InRange,
                "start = {start:?}"
            );
        }
    }

    #[test]
    fn returns_invalid_range_for_inverted_or_missing_bounds() {
        let start = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap());
        let end = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        let t = datetime(2024, 1, 3, 12, 0, 0).unwrap();

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
        // Test with a valid datetime that should produce a single result
        let datetime = DateTime::from_timestamp(1_609_459_200, 0)
            .expect("Valid timestamp for 2021-01-01 00:00:00")
            .naive_local();
        let loose_dt = LooseDateTime::from_local_datetime(datetime);

        // Should convert to Local variant
        assert!(matches!(loose_dt, LooseDateTime::Local(_)));
    }

    #[test]
    fn serializes_and_deserializes_stably() {
        let date = NaiveDate::from_ymd_opt(2024, 7, 18).unwrap();
        let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
        let datetime = NaiveDateTime::new(date, time);
        let local = Local.with_ymd_and_hms(2024, 7, 18, 12, 30, 45).unwrap();

        let d1 = LooseDateTime::DateOnly(date);
        let d2 = LooseDateTime::Floating(datetime);
        let d3 = LooseDateTime::Local(local);

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
        if let Some(LooseDateTime::Local(dt)) = parsed3 {
            assert_eq!(dt.naive_local(), local.naive_local());
        } else {
            panic!("Failed to parse local datetime");
        }
    }

    #[test]
    fn adds_timedelta_to_dateonly() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let added = LooseDateTime::DateOnly(date) + TimeDelta::days(2) + TimeDelta::hours(3);
        let expected = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2025, 1, 3).unwrap());
        assert_eq!(added, expected);
    }

    #[test]
    fn adds_timedelta_to_floating() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
        let dt = LooseDateTime::Floating(NaiveDateTime::new(date, time));
        let added = dt + TimeDelta::days(2) + TimeDelta::hours(3);
        let excepted = LooseDateTime::Floating(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(),
            NaiveTime::from_hms_opt(15, 30, 45).unwrap(),
        ));
        assert_eq!(added, excepted);
    }

    #[test]
    fn adds_timedelta_to_local() {
        let local = Local.with_ymd_and_hms(2025, 1, 1, 12, 30, 45).unwrap();
        let added = LooseDateTime::Local(local) + TimeDelta::days(2) + TimeDelta::hours(3);
        let excepted =
            LooseDateTime::Local(Local.with_ymd_and_hms(2025, 1, 3, 15, 30, 45).unwrap());
        assert_eq!(added, excepted);
    }
}
