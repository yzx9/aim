// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, offset::LocalResult};
use chrono_tz::Tz;
use icalendar::{CalendarDateTime, DatePerhapsTime};

/// A date and time that may be in different formats, such as date only, floating time, or local time with timezone.
#[derive(Debug, Clone, Copy)]
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

/// Sort order, either ascending or descending.
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    /// Ascending order.
    Asc,

    /// Descending order.
    Desc,
}

impl SortOrder {
    /// Converts to a string representation suitable for SQL queries.
    pub(crate) fn sql_keyword(&self) -> &str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
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
    /// No priority.
    None,
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
            Priority::None => 0,
            Priority::P1 => 1,
            Priority::P2 => 2,
            Priority::P3 => 3,
            Priority::P4 => 4,
            Priority::P5 => 5,
            Priority::P6 => 6,
            Priority::P7 => 7,
            Priority::P8 => 8,
            Priority::P9 => 9,
        }
    }
}

impl From<Priority> for u32 {
    fn from(value: Priority) -> Self {
        u8::from(value).into()
    }
}
