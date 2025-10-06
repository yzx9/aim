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
    DateTimeSpan(EventColumnDateTimeSpan),
    Id(EventColumnId),
    ShortId(EventColumnShortId),
    Summary(EventColumnSummary),
    TimeSpan(EventColumnTimeSpan),
    Uid(EventColumnUid),
    UidLegacy(EventColumnUidLegacy),
}

impl EventColumn {
    pub fn datetime_span() -> Self {
        EventColumn::DateTimeSpan(EventColumnDateTimeSpan)
    }

    pub fn time_span(date: NaiveDate) -> Self {
        EventColumn::TimeSpan(EventColumnTimeSpan { date })
    }

    pub fn id() -> Self {
        EventColumn::Id(EventColumnId)
    }

    pub fn short_id() -> Self {
        EventColumn::ShortId(EventColumnShortId)
    }

    pub fn summary() -> Self {
        EventColumn::Summary(EventColumnSummary)
    }

    pub fn uid() -> Self {
        EventColumn::Uid(EventColumnUid)
    }

    pub fn uid_legacy() -> Self {
        EventColumn::UidLegacy(EventColumnUidLegacy)
    }
}

#[derive(Debug, Clone, Copy)]
struct ColumnMeta<'a> {
    column: &'a EventColumn,
    now: DateTime<Local>,
}

impl<'a, E: Event> TableColumn<E> for ColumnMeta<'a> {
    fn name(&self) -> Cow<'_, str> {
        match self.column {
            EventColumn::DateTimeSpan(_) => "Date Time",
            EventColumn::Id(_) => "ID",
            EventColumn::ShortId(_) => "Short ID",
            EventColumn::Summary(_) => "Summary",
            EventColumn::TimeSpan(_) => "Time",
            EventColumn::Uid(_) => "UID",
            EventColumn::UidLegacy(_) => "UID",
        }
        .into()
    }

    fn format<'b>(&self, data: &'b E) -> Cow<'b, str> {
        match self.column {
            EventColumn::DateTimeSpan(a) => a.format(data),
            EventColumn::Id(a) => a.format(data),
            EventColumn::ShortId(a) => a.format(data),
            EventColumn::Summary(a) => a.format(data),
            EventColumn::TimeSpan(a) => a.format(data),
            EventColumn::Uid(a) => a.format(data),
            EventColumn::UidLegacy(a) => a.format(data),
        }
    }

    fn padding_direction(&self) -> PaddingDirection {
        match self.column {
            EventColumn::Id(_) | EventColumn::Uid(_) => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, data: &E) -> Option<Color> {
        match &self.column {
            EventColumn::DateTimeSpan(v) => v.get_color(data, &self.now),
            EventColumn::TimeSpan(v) => v.get_color(data, &self.now),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnId;

impl EventColumnId {
    fn format<'a>(&self, event: &'a impl Event) -> Cow<'a, str> {
        if let Some(short_id) = event.short_id() {
            short_id.to_string().into()
        } else {
            let uid = event.uid(); // Fallback to the full UID if no short ID is available
            tracing::warn!(uid, "event does not have a short ID, using UID instead.",);
            uid.into()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnSummary;

impl EventColumnSummary {
    fn format<'a>(&self, event: &'a impl Event) -> Cow<'a, str> {
        event.summary().replace('\n', "↵").into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnDateTimeSpan;

impl EventColumnDateTimeSpan {
    fn format<'a>(&self, event: &'a impl Event) -> Cow<'a, str> {
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

    fn get_color(&self, event: &impl Event, now: &DateTime<Local>) -> Option<Color> {
        Self::event_color(event, now)
    }

    fn event_color(event: &impl Event, now: &DateTime<Local>) -> Option<Color> {
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
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnTimeSpan {
    date: NaiveDate,
}

impl EventColumnTimeSpan {
    fn format<'a>(&self, event: &'a impl Event) -> Cow<'a, str> {
        match (event.start(), event.end()) {
            (Some(start), Some(end)) => {
                let sdate = start.date();
                let edate = end.date();
                if edate < sdate {
                    String::new() // Invalid range
                } else if edate < self.date {
                    format!("⇥{}", Self::format_date(edate)).to_string() // in the past
                } else if sdate > self.date {
                    format!("{}↦", Self::format_date(sdate)).to_string() // in the future
                } else if sdate == self.date && sdate == edate {
                    // today is the only day
                    match (start.time(), end.time()) {
                        (Some(stime), Some(etime)) => {
                            format!("{}~{}", stime.format("%H:%M"), etime.format("%H:%M"))
                        }
                        (Some(stime), None) => format!("{}⇥", stime.format("%H:%M")),
                        (None, Some(etime)) => format!("     ↦{}", etime.format("%H:%M")),
                        (None, None) => format!("⇹{}", Self::format_date(self.date)),
                    }
                } else if sdate == self.date
                    && sdate < edate
                    && let Some(stime) = start.time()
                {
                    // starts today with time, ends later
                    format!("{}↦", stime.format("%H:%M")).to_string()
                } else if edate == self.date
                    && let Some(etime) = end.time()
                {
                    // ends today with time, started earlier
                    format!("⇥{}", etime.format("%H:%M"))
                } else if sdate.year() == self.date.year() && edate.year() == self.date.year() {
                    // sdate <= self.date <= edate, no time, same year, only show month and day
                    format!("{}~{}", sdate.format("%m-%d"), edate.format("%m-%d")).to_string()
                } else {
                    // sdate <= self.date <= edate, no time, different year, show full date
                    format!("⇸{}", Self::format_date(edate)).to_string()
                }
            }
            .into(),
            (Some(start), None) => format_datetime(start).into(),
            (None, Some(end)) => format!("↦{}", format_datetime(end)).into(),
            (None, None) => String::new().into(),
        }
    }

    fn get_color(&self, event: &impl Event, now: &DateTime<Local>) -> Option<Color> {
        EventColumnDateTimeSpan::event_color(event, now)
    }

    fn format_date(d: NaiveDate) -> String {
        d.format("%Y-%m-%d").to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnUid;

impl EventColumnUid {
    fn format<'a>(&self, event: &'a impl Event) -> Cow<'a, str> {
        event.uid().into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnUidLegacy;

impl EventColumnUidLegacy {
    fn format<'a>(&self, event: &'a impl Event) -> Cow<'a, str> {
        format!("#{}", event.uid()).into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnShortId;

impl EventColumnShortId {
    fn format<'a>(&self, event: &'a impl Event) -> Cow<'a, str> {
        event
            .short_id()
            .map(|a| a.to_string())
            .unwrap_or_default()
            .into()
    }
}
