// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, fmt};

use aimcal_core::{Event, LooseDateTime, RangePosition};
use chrono::{DateTime, Datelike, Local, NaiveDate};
use colored::Color;

use crate::table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson};
use crate::util::{OutputFormat, format_datetime};

#[derive(Debug, Clone)]
pub struct EventFormatter {
    now: DateTime<Local>,
    columns: Vec<EventColumn>,
    format: OutputFormat,
}

impl EventFormatter {
    pub fn new(now: DateTime<Local>, columns: Vec<EventColumn>, format: OutputFormat) -> Self {
        Self {
            now,
            columns,
            format,
        }
    }

    pub fn format<'a, E: Event>(&'a self, events: &'a [E]) -> Display<'a, E> {
        Display {
            events,
            formatter: self,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Display<'a, E: Event> {
    events: &'a [E],
    formatter: &'a EventFormatter,
}

impl<'a, E: Event> fmt::Display for Display<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let columns: Vec<_> = self
            .formatter
            .columns
            .iter()
            .map(|column| ColumnMeta {
                column,
                now: self.formatter.now,
            })
            .collect();

        match self.formatter.format {
            OutputFormat::Json => {
                let table = Table::new(TableStyleJson::new(), &columns, self.events);
                write!(f, "{table}")
            }
            OutputFormat::Table => {
                let table = Table::new(TableStyleBasic::new(), &columns, self.events);
                write!(f, "{table}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventColumn {
    DateTimeSpan,
    Id,
    ShortId,
    Summary,
    TimeSpan { date: NaiveDate },
    Uid,
    UidLegacy,
}

#[derive(Debug, Clone, Copy)]
struct ColumnMeta<'a> {
    column: &'a EventColumn,
    now: DateTime<Local>,
}

impl<'a, E: Event> TableColumn<E> for ColumnMeta<'a> {
    fn name(&self) -> Cow<'_, str> {
        match self.column {
            EventColumn::DateTimeSpan => "Date Time",
            EventColumn::Id => "ID",
            EventColumn::ShortId => "Short ID",
            EventColumn::Summary => "Summary",
            EventColumn::TimeSpan { date: _ } => "Time",
            EventColumn::Uid => "UID",
            EventColumn::UidLegacy => "UID",
        }
        .into()
    }

    fn format<'b>(&self, data: &'b E) -> Cow<'b, str> {
        match self.column {
            EventColumn::DateTimeSpan => format_datetime_span(data),
            EventColumn::Id => format_id(data),
            EventColumn::ShortId => format_short_id(data),
            EventColumn::Summary => format_summary(data),
            EventColumn::TimeSpan { date } => format_time_span(data, *date),
            EventColumn::Uid => format_uid(data),
            EventColumn::UidLegacy => format_uid_legacy(data),
        }
    }

    fn padding_direction(&self) -> PaddingDirection {
        match self.column {
            EventColumn::Id | EventColumn::Uid => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, data: &E) -> Option<Color> {
        match &self.column {
            EventColumn::DateTimeSpan => get_color_datetime_span(data, self.now),
            EventColumn::TimeSpan { date: _ } => get_color_time_span(data, self.now),
            _ => None,
        }
    }
}

fn format_id(event: &impl Event) -> Cow<'_, str> {
    if let Some(short_id) = event.short_id() {
        short_id.to_string().into()
    } else {
        let uid = event.uid(); // Fallback to the full UID if no short ID is available
        tracing::warn!(uid, "event does not have a short ID, using UID instead.",);
        uid.into()
    }
}

fn format_short_id(event: &impl Event) -> Cow<'_, str> {
    event
        .short_id()
        .map(|a| a.to_string())
        .unwrap_or_default()
        .into()
}

fn format_summary(event: &impl Event) -> Cow<'_, str> {
    event.summary().replace('\n', "↵").into()
}

fn format_datetime_span(event: &impl Event) -> Cow<'_, str> {
    match (event.start(), event.end()) {
        (Some(start), Some(end)) => match start.date() == end.date() {
            true => match (start.time(), end.time()) {
                (Some(stime), Some(etime)) => format!(
                    "{} {}~{}",
                    start.date().format("%Y-%m-%d"),
                    stime.format("%H:%M"),
                    etime.format("%H:%M")
                ),
                (Some(stime), None) => format!(
                    "{} {}~24:00",
                    start.date().format("%Y-%m-%d"),
                    stime.format("%H:%M")
                ),
                (None, Some(etime)) => format!(
                    "{} 00:00~{}",
                    start.date().format("%Y-%m-%d"),
                    etime.format("%H:%M")
                ),
                (None, None) => start.date().format("%Y-%m-%d").to_string(),
            },
            false => format!("{}~{}", format_datetime(start), format_datetime(end)),
        }
        .into(),
        (Some(start), None) => format_datetime(start).into(),
        (None, Some(end)) => format!("~{}", format_datetime(end)).into(),
        (None, None) => String::new().into(),
    }
}

fn get_color_datetime_span(event: &impl Event, now: DateTime<Local>) -> Option<Color> {
    const COLOR_CURRENT: Option<Color> = Some(Color::Yellow);
    const COLOR_TODAY_LATE: Option<Color> = Some(Color::Green);

    let start = event.start()?; // If no start time, no color
    match LooseDateTime::position_in_range(&now.naive_local(), &Some(start), &event.end()) {
        RangePosition::Before => COLOR_TODAY_LATE,
        RangePosition::InRange => COLOR_CURRENT,
        RangePosition::After => None,
        RangePosition::InvalidRange => {
            tracing::warn!(uid = event.uid(), "invalid range for event");
            None
        }
    }
}

fn format_time_span<'a>(event: &'a impl Event, date: NaiveDate) -> Cow<'a, str> {
    fn format_date(d: NaiveDate) -> String {
        d.format("%Y-%m-%d").to_string()
    }

    match (event.start(), event.end()) {
        (Some(start), Some(end)) => {
            let sdate = start.date();
            let edate = end.date();
            if edate < sdate {
                String::new() // Invalid range
            } else if edate < date {
                format!("⇥{}", format_date(edate)).to_string() // in the past
            } else if sdate > date {
                format!("{}↦", format_date(sdate)).to_string() // in the future
            } else if sdate == date && sdate == edate {
                // today is the only day
                match (start.time(), end.time()) {
                    (Some(stime), Some(etime)) => {
                        format!("{}~{}", stime.format("%H:%M"), etime.format("%H:%M"))
                    }
                    (Some(stime), None) => format!("{}⇥", stime.format("%H:%M")),
                    (None, Some(etime)) => format!("     ↦{}", etime.format("%H:%M")),
                    (None, None) => format!("⇹{}", format_date(date)),
                }
            } else if sdate == date
                && sdate < edate
                && let Some(stime) = start.time()
            {
                // starts today with time, ends later
                format!("{}↦", stime.format("%H:%M")).to_string()
            } else if edate == date
                && let Some(etime) = end.time()
            {
                // ends today with time, started earlier
                format!("⇥{}", etime.format("%H:%M"))
            } else if sdate.year() == date.year() && edate.year() == date.year() {
                // sdate <= self.date <= edate, no time, same year, only show month and day
                format!("{}~{}", sdate.format("%m-%d"), edate.format("%m-%d")).to_string()
            } else {
                // sdate <= self.date <= edate, no time, different year, show full date
                format!("⇸{}", format_date(edate)).to_string()
            }
        }
        .into(),
        (Some(start), None) => format_datetime(start).into(),
        (None, Some(end)) => format!("↦{}", format_datetime(end)).into(),
        (None, None) => String::new().into(),
    }
}

fn get_color_time_span(event: &impl Event, now: DateTime<Local>) -> Option<Color> {
    get_color_datetime_span(event, now)
}

fn format_uid(event: &impl Event) -> Cow<'_, str> {
    event.uid().into()
}

fn format_uid_legacy(event: &impl Event) -> Cow<'_, str> {
    format!("#{}", event.uid()).into()
}
