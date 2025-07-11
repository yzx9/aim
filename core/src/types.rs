// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chrono::{Local, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Tz;

#[derive(Debug, Clone)]
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
                        log::warn!("Unknown timezone, treating as local time: {}", tzid);
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

#[derive(Debug, Clone, Copy)]
pub struct Pager {
    pub limit: i64,
    pub offset: i64,
}

impl Into<Pager> for (i64, i64) {
    fn into(self) -> Pager {
        Pager {
            limit: self.0,
            offset: self.1,
        }
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
