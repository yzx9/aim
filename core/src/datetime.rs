// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chrono::{
    DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc, offset::LocalResult,
};
use chrono_tz::Tz;
use icalendar::{CalendarDateTime, DatePerhapsTime};

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
    /// Returns the date part
    pub fn date(&self) -> NaiveDate {
        match self {
            LooseDateTime::DateOnly(d) => *d,
            LooseDateTime::Floating(dt) => dt.date(),
            LooseDateTime::Local(dt) => dt.date_naive(),
        }
    }

    /// Returns the time part, if available.
    pub fn time(&self) -> Option<NaiveTime> {
        match self {
            LooseDateTime::DateOnly(_) => None,
            LooseDateTime::Floating(dt) => Some(dt.time()),
            LooseDateTime::Local(dt) => Some(dt.time()),
        }
    }

    /// Converts to a datetime with default start time (00:00:00) if time is missing.
    pub fn with_start_of_day(&self) -> NaiveDateTime {
        NaiveDateTime::new(
            self.date(),
            self.time()
                .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
        )
    }

    /// Converts to a datetime with default end time (23:59:59.999999999) if time is missing.
    pub fn with_end_of_day(&self) -> NaiveDateTime {
        NaiveDateTime::new(
            self.date(),
            self.time().unwrap_or_else(|| {
                // Using a leap second to represent the end of the day
                NaiveTime::from_hms_nano_opt(23, 59, 59, 1_999_999_999).unwrap()
            }),
        )
    }

    /// Determines the position of a given datetime relative to a start and optional end date.
    pub fn position_in_range(
        t: &NaiveDateTime,
        start: &LooseDateTime,
        end: &Option<LooseDateTime>,
    ) -> RangePosition {
        let start_dt = start.with_start_of_day(); // 00:00
        match end {
            Some(end) => {
                let end_dt = end.with_end_of_day(); // 23:59
                if start_dt > end_dt {
                    RangePosition::InvalidRange
                } else if t > &end_dt {
                    RangePosition::After
                } else if t <= &start_dt {
                    RangePosition::Before
                } else {
                    RangePosition::InRange
                }
            }
            None => match &start_dt <= t {
                true => RangePosition::InRange,
                false => RangePosition::Before,
            },
        }
    }

    /// NOTE: Used for storing in the database, so it should be stable across different runs.
    const DATEONLY_FORMAT: &str = "%Y-%m-%d";
    const FLOATING_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
    const LOCAL_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

    /// Converts to a string representation of date and time.
    pub(crate) fn format_stable(&self) -> String {
        match self {
            LooseDateTime::DateOnly(d) => d.format(Self::DATEONLY_FORMAT).to_string(),
            LooseDateTime::Floating(dt) => dt.format(Self::FLOATING_FORMAT).to_string(),
            LooseDateTime::Local(dt) => dt.format(Self::LOCAL_FORMAT).to_string(),
        }
    }

    pub(crate) fn parse_stable(s: &str) -> Option<Self> {
        match s.len() {
            // 2006-01-02
            10 => NaiveDate::parse_from_str(s, Self::DATEONLY_FORMAT)
                .map(Self::DateOnly)
                .ok(),

            // 2006-01-02T15:04:05
            19 => NaiveDateTime::parse_from_str(s, Self::FLOATING_FORMAT)
                .map(Self::Floating)
                .ok(),

            // 2006-01-02T15:04:05Z
            20.. => DateTime::parse_from_str(s, Self::LOCAL_FORMAT)
                .map(|a| Self::Local(a.with_timezone(&Local)))
                .ok(),

            _ => None,
        }
    }
}

impl From<DatePerhapsTime> for LooseDateTime {
    fn from(dt: DatePerhapsTime) -> Self {
        match dt {
            DatePerhapsTime::DateTime(dt) => match dt {
                CalendarDateTime::Floating(dt) => LooseDateTime::Floating(dt),
                CalendarDateTime::Utc(dt) => LooseDateTime::Local(dt.into()),
                CalendarDateTime::WithTimezone { date_time, tzid } => match tzid.parse::<Tz>() {
                    Ok(tz) => match tz.from_local_datetime(&date_time) {
                        // Use the parsed timezone to interpret the datetime
                        LocalResult::Single(dt_in_tz) => {
                            LooseDateTime::Local(dt_in_tz.with_timezone(&Local))
                        }
                        LocalResult::Ambiguous(dt1, _) => {
                            log::warn!(
                                "Ambiguous local time for {date_time} in {tzid}, picking earliest"
                            );
                            LooseDateTime::Local(dt1.with_timezone(&Local))
                        }
                        LocalResult::None => {
                            log::warn!(
                                "Invalid local time for {date_time} in {tzid}, falling back to floating"
                            );
                            LooseDateTime::Floating(date_time)
                        }
                    },
                    _ => {
                        log::warn!("Unknown timezone, treating as floating: {tzid}");
                        LooseDateTime::Floating(date_time)
                    }
                },
            },
            DatePerhapsTime::Date(d) => LooseDateTime::DateOnly(d),
        }
    }
}

impl From<LooseDateTime> for DatePerhapsTime {
    fn from(dt: LooseDateTime) -> Self {
        use DatePerhapsTime::*;
        match dt {
            LooseDateTime::DateOnly(d) => Date(d),
            LooseDateTime::Floating(dt) => DateTime(CalendarDateTime::Floating(dt)),
            LooseDateTime::Local(dt) => match iana_time_zone::get_timezone() {
                Ok(tzid) => DateTime(CalendarDateTime::WithTimezone {
                    date_time: dt.naive_local(),
                    tzid,
                }),
                Err(_) => DateTime(CalendarDateTime::Utc(dt.into())),
            },
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

impl From<DateTime<Local>> for LooseDateTime {
    fn from(dt: DateTime<Local>) -> Self {
        LooseDateTime::Local(dt)
    }
}

impl From<DateTime<Utc>> for LooseDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        LooseDateTime::Local(dt.with_timezone(&Local))
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    #[test]
    fn test_date_and_time_methods() {
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
    fn test_with_start_of_day() {
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
    fn test_with_end_of_day() {
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

    #[test]
    fn test_position_in_range_with_end() {
        let start = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let end = Some(LooseDateTime::DateOnly(
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        ));

        let t_before = NaiveDate::from_ymd_opt(2023, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let t_in = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let t_after = NaiveDate::from_ymd_opt(2024, 1, 4)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        assert_eq!(
            LooseDateTime::position_in_range(&t_before, &start, &end),
            RangePosition::Before
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_in, &start, &end),
            RangePosition::InRange
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t_after, &start, &end),
            RangePosition::After
        );
    }

    #[test]
    fn test_position_in_range_without_end() {
        let start = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        let t1 = NaiveDate::from_ymd_opt(2023, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let t2 = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        assert_eq!(
            LooseDateTime::position_in_range(&t1, &start, &None),
            RangePosition::Before
        );
        assert_eq!(
            LooseDateTime::position_in_range(&t2, &start, &None),
            RangePosition::InRange
        );
    }

    #[test]
    fn test_invalid_range() {
        let start = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap());
        let end = LooseDateTime::DateOnly(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let t = NaiveDate::from_ymd_opt(2024, 1, 3)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        assert_eq!(
            LooseDateTime::position_in_range(&t, &start, &Some(end)),
            RangePosition::InvalidRange
        );
    }

    #[test]
    fn test_format_and_parse_stable() {
        let date = NaiveDate::from_ymd_opt(2024, 7, 18).unwrap();
        let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
        let datetime = NaiveDateTime::new(date, time);
        let local = TimeZone::with_ymd_and_hms(&Local, 2024, 7, 18, 12, 30, 45).unwrap();

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
}
