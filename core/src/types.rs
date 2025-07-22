// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::Tz;

/// Represents a date that may or may not include a time component, along with an optional timezone.
#[derive(Debug, Clone, Copy)]
pub struct DatePerhapsTime {
    /// The date component.
    pub date: NaiveDate,

    /// The optional time component.
    pub time: Option<NaiveTime>,

    /// The optional timezone.
    pub tz: Option<Tz>,
}

impl DatePerhapsTime {
    /// Creates a new `DatePerhapsTime` instance with the given date, optional time, and optional timezone.
    pub fn format(&self) -> String {
        if let Some(time) = self.time {
            format!("{} {}", self.date.format("%Y-%m-%d"), time.format("%H:%M"))
        } else {
            self.date.format("%Y-%m-%d").to_string()
        }
    }

    /// Converts the `DatePerhapsTime` to a string representation of date and timezone.
    pub fn to_dt_tz(&self, date_format: &str, datetime_format: &str) -> (String, String) {
        let t = if let Some(t) = self.time {
            let dt = NaiveDateTime::new(self.date, t);
            dt.format(datetime_format).to_string()
        } else {
            self.date.format(date_format).to_string()
        };
        (t, self.tz.map_or("", |tz| tz.name()).to_string())
    }

    /// Parses a date or datetime string with an optional timezone into a `DatePerhapsTime`.
    pub fn from_dt_tz(
        dt: &str,
        tz: &str,
        date_format: &str,
        datetime_format: &str,
    ) -> Option<DatePerhapsTime> {
        if dt.is_empty() {
            return None;
        }

        let tz: Option<Tz> = tz.parse().ok();
        match dt.len() {
            10 => NaiveDate::parse_from_str(dt, date_format)
                .ok()
                .map(|d| DatePerhapsTime {
                    date: d,
                    time: None,
                    tz,
                }),
            19 => NaiveDateTime::parse_from_str(dt, datetime_format)
                .ok()
                .map(|d| DatePerhapsTime {
                    date: d.date(),
                    time: Some(d.time()),
                    tz,
                }),
            _ => None,
        }
    }
}

impl From<icalendar::DatePerhapsTime> for DatePerhapsTime {
    fn from(date: icalendar::DatePerhapsTime) -> Self {
        match date {
            icalendar::DatePerhapsTime::DateTime(dt) => match dt {
                icalendar::CalendarDateTime::Floating(dt) => Self {
                    date: dt.date(),
                    time: Some(dt.time()),
                    tz: None,
                },
                icalendar::CalendarDateTime::Utc(dt) => {
                    // NOTE: always use local time, so we need refresh cache when system time changes
                    let local = dt.with_timezone(&Local).naive_local();
                    Self {
                        date: local.date(),
                        time: Some(local.time()),
                        tz: Some(Tz::UTC),
                    }
                }
                icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => {
                    let tz: Option<Tz> = tzid.parse().ok();
                    if let Some(tz) = tz {
                        let local = tz
                            .from_local_datetime(&date_time)
                            .unwrap()
                            .with_timezone(&Local)
                            .naive_local();
                        Self {
                            date: local.date(),
                            time: Some(local.time()),
                            tz: Some(tz),
                        }
                    } else {
                        log::warn!("Unknown timezone, treating as local time: {tzid}");
                        Self {
                            date: date_time.date(),
                            time: Some(date_time.time()),
                            tz: None,
                        }
                    }
                }
            },
            icalendar::DatePerhapsTime::Date(d) => Self {
                date: d,
                time: None,
                tz: None,
            },
        }
    }
}

impl From<DatePerhapsTime> for icalendar::DatePerhapsTime {
    fn from(date: DatePerhapsTime) -> Self {
        match date.time {
            Some(t) => icalendar::DatePerhapsTime::DateTime(match date.tz {
                Some(tz) => icalendar::CalendarDateTime::WithTimezone {
                    date_time: NaiveDateTime::new(date.date, t),
                    tzid: tz.name().to_string(),
                },
                None => icalendar::CalendarDateTime::Floating(NaiveDateTime::new(date.date, t)),
            }),
            None => icalendar::DatePerhapsTime::Date(date.date),
        }
    }
}

/// Sort order, either ascending or descending.
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    /// Ascending order.
    Asc,

    /// Descending order.
    Desc,
}

/// Pagination with a limit and an offset.
#[derive(Debug, Clone, Copy)]
pub struct Pager {
    /// The maximum number of items to return.
    pub limit: i64,

    /// The number of items to skip before starting to collect the result set.
    pub offset: i64,
}

impl From<(i64, i64)> for Pager {
    fn from((limit, offset): (i64, i64)) -> Self {
        Pager { limit, offset }
    }
}

/// Priority of a task or item, with values ranging from 1 to 9, and None for no priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    /// Priority 1, highest priority.
    P1,
    /// Priority 2.
    P2,
    /// Priority 3.
    P3,
    /// Priority 4.
    P4,
    /// Priority 5.
    P5,
    /// Priority 6.
    P6,
    /// Priority 7.
    P7,
    /// Priority 8.
    P8,
    /// Priority 9, lowest priority.
    P9,
    /// No priority.
    None,
}

impl From<u32> for Priority {
    fn from(value: u32) -> Self {
        match value {
            1 => Priority::P1,
            2 => Priority::P2,
            3 => Priority::P3,
            4 => Priority::P4,
            5 => Priority::P5,
            6 => Priority::P6,
            7 => Priority::P7,
            8 => Priority::P8,
            9 => Priority::P9,
            _ => Priority::None,
        }
    }
}

impl From<u8> for Priority {
    fn from(value: u8) -> Self {
        u32::from(value).into()
    }
}

impl From<Priority> for u8 {
    fn from(value: Priority) -> Self {
        match value {
            Priority::P1 => 1,
            Priority::P2 => 2,
            Priority::P3 => 3,
            Priority::P4 => 4,
            Priority::P5 => 5,
            Priority::P6 => 6,
            Priority::P7 => 7,
            Priority::P8 => 8,
            Priority::P9 => 9,
            Priority::None => 0,
        }
    }
}

impl From<Priority> for u32 {
    fn from(value: Priority) -> Self {
        u8::from(value).into()
    }
}
