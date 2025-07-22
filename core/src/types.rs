// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::Tz;

#[derive(Debug, Clone, Copy)]
pub struct DatePerhapsTime {
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub tz: Option<Tz>,
}

impl DatePerhapsTime {
    pub fn format(&self) -> String {
        if let Some(time) = self.time {
            format!("{} {}", self.date.format("%Y-%m-%d"), time.format("%H:%M"))
        } else {
            self.date.format("%Y-%m-%d").to_string()
        }
    }

    pub fn to_dt_tz(&self, date_format: &str, datetime_format: &str) -> (String, String) {
        let t = if let Some(t) = self.time {
            let dt = NaiveDateTime::new(self.date, t);
            dt.format(datetime_format).to_string()
        } else {
            self.date.format(date_format).to_string()
        };
        (t, self.tz.map_or("", |tz| tz.name()).to_string())
    }

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

#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub struct Pager {
    pub limit: i64,
    pub offset: i64,
}

impl From<(i64, i64)> for Pager {
    fn from((limit, offset): (i64, i64)) -> Self {
        Pager { limit, offset }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
    P8,
    P9,
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
        (value as u32).into()
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
        Into::<u8>::into(value).into()
    }
}
