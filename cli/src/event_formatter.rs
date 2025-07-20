// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    OutputFormat,
    short_id::EventWithShortId,
    table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson},
};
use aim_core::Event;
use chrono::NaiveDateTime;
use std::{borrow::Cow, fmt};

#[derive(Debug)]
pub struct EventFormatter {
    columns: Vec<EventColumn>,
    _now: NaiveDateTime,
    format: OutputFormat,
}

impl EventFormatter {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            columns: vec![
                EventColumn::ShortId(EventColumnShortId),
                EventColumn::TimeRange(EventColumnTimeRange),
                EventColumn::Summary(EventColumnSummary),
            ],
            _now: now,
            format: OutputFormat::Table,
        }
    }

    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    pub fn format<'a, E: Event>(&'a self, events: &'a [EventWithShortId<E>]) -> Display<'a, E> {
        Display {
            events,
            formatter: self,
        }
    }
}

pub struct Display<'a, E: Event> {
    events: &'a [EventWithShortId<E>],
    formatter: &'a EventFormatter,
}

impl<'a, E: Event> fmt::Display for Display<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.formatter.format {
            OutputFormat::Json => write!(
                f,
                "{}",
                Table::new(TableStyleJson::new(), &self.formatter.columns, self.events)
            ),
            OutputFormat::Table => write!(
                f,
                "{}",
                Table::new(TableStyleBasic::new(), &self.formatter.columns, self.events)
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventColumn {
    ShortId(EventColumnShortId),
    Summary(EventColumnSummary),
    TimeRange(EventColumnTimeRange),
    #[allow(dead_code)]
    Uid(EventColumnUid),
}

impl<E: Event> TableColumn<EventWithShortId<E>> for EventColumn {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        match self {
            EventColumn::ShortId(_) => "Display Number",
            EventColumn::Summary(_) => "Summary",
            EventColumn::TimeRange(_) => "Time Range",
            EventColumn::Uid(_) => "UID",
        }
        .into()
    }

    fn format<'a>(&self, data: &'a EventWithShortId<E>) -> Cow<'a, str> {
        match self {
            EventColumn::ShortId(a) => a.format(data),
            EventColumn::Summary(a) => a.format(data),
            EventColumn::TimeRange(a) => a.format(data),
            EventColumn::Uid(a) => a.format(data),
        }
    }

    fn padding_direction(&self, _data: &EventWithShortId<E>) -> PaddingDirection {
        match self {
            EventColumn::ShortId(_) | EventColumn::Uid(_) => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventColumnShortId;

impl EventColumnShortId {
    fn format<'a>(&self, event: &'a EventWithShortId<impl Event>) -> Cow<'a, str> {
        event.short_id.to_string().into()
    }
}

#[derive(Debug, Clone)]
pub struct EventColumnSummary;

impl EventColumnSummary {
    fn format<'a>(&self, event: &'a EventWithShortId<impl Event>) -> Cow<'a, str> {
        event.inner.summary().into()
    }
}

#[derive(Debug, Clone)]
pub struct EventColumnTimeRange;

impl EventColumnTimeRange {
    fn format<'a>(&self, event: &'a EventWithShortId<impl Event>) -> Cow<'a, str> {
        match (event.inner.start(), event.inner.end()) {
            (Some(start), Some(end)) => match start.date == end.date {
                true => match (start.time, end.time) {
                    (Some(stime), Some(etime)) => format!(
                        "{} {}~{}",
                        start.date.format("%Y-%m-%d"),
                        stime.format("%H:%M"),
                        etime.format("%H:%M")
                    ),
                    (Some(stime), None) => format!(
                        "{} {}~24:00",
                        start.date.format("%Y-%m-%d"),
                        stime.format("%H:%M")
                    ),
                    (None, Some(etime)) => format!(
                        "{} 00:00~{}",
                        start.date.format("%Y-%m-%d"),
                        etime.format("%H:%M")
                    ),
                    (None, None) => start.date.format("%Y-%m-%d").to_string(),
                },
                false => format!("{}~{}", start.format(), end.format()),
            }
            .into(),
            (Some(start), None) => start.format().into(),
            (None, Some(end)) => format!("~{}", end.format()).into(),
            (None, None) => "".to_string().into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventColumnUid;

impl EventColumnUid {
    fn format<'a>(&self, event: &'a EventWithShortId<impl Event>) -> Cow<'a, str> {
        format!("#{}", event.inner.uid()).into()
    }
}
