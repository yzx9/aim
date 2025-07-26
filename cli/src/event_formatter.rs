// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parser::{ArgOutputFormat, format_datetime},
    short_id::EventWithShortId,
    table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson},
};
use aimcal_core::{Event, LooseDateTime, RangePosition};
use chrono::NaiveDateTime;
use colored::Color;
use std::{borrow::Cow, fmt};

#[derive(Debug, Clone)]
pub struct EventFormatter {
    columns: Vec<EventColumn>,
    now: NaiveDateTime,
    format: ArgOutputFormat,
}

impl EventFormatter {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            columns: vec![
                EventColumn::ShortId(EventColumnShortId),
                EventColumn::TimeRange(EventColumnTimeRange),
                EventColumn::Summary(EventColumnSummary),
            ],
            now,
            format: ArgOutputFormat::Table,
        }
    }

    pub fn with_output_format(mut self, format: ArgOutputFormat) -> Self {
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

#[derive(Debug, Clone, Copy)]
pub struct Display<'a, E: Event> {
    events: &'a [EventWithShortId<E>],
    formatter: &'a EventFormatter,
}

impl<'a, E: Event> fmt::Display for Display<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let columns = self
            .formatter
            .columns
            .iter()
            .map(|column| ColumnMeta {
                column,
                now: self.formatter.now,
            })
            .collect::<Vec<_>>();

        match self.formatter.format {
            ArgOutputFormat::Json => write!(
                f,
                "{}",
                Table::new(TableStyleJson::new(), &columns, self.events)
            ),
            ArgOutputFormat::Table => write!(
                f,
                "{}",
                Table::new(TableStyleBasic::new(), &columns, self.events)
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventColumn {
    ShortId(EventColumnShortId),
    Summary(EventColumnSummary),
    TimeRange(EventColumnTimeRange),
    #[allow(dead_code)]
    Uid(EventColumnUid),
}

#[derive(Debug, Clone, Copy)]
struct ColumnMeta<'a> {
    column: &'a EventColumn,
    now: NaiveDateTime,
}

impl<'a, E: Event> TableColumn<EventWithShortId<E>> for ColumnMeta<'a> {
    fn name(&self) -> Cow<'_, str> {
        match self.column {
            EventColumn::ShortId(_) => "Display Number",
            EventColumn::Summary(_) => "Summary",
            EventColumn::TimeRange(_) => "Time Range",
            EventColumn::Uid(_) => "UID",
        }
        .into()
    }

    fn format<'b>(&self, data: &'b EventWithShortId<E>) -> Cow<'b, str> {
        match self.column {
            EventColumn::ShortId(a) => a.format(data),
            EventColumn::Summary(a) => a.format(data),
            EventColumn::TimeRange(a) => a.format(data),
            EventColumn::Uid(a) => a.format(data),
        }
    }

    fn padding_direction(&self) -> PaddingDirection {
        match self.column {
            EventColumn::ShortId(_) | EventColumn::Uid(_) => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, data: &EventWithShortId<E>) -> Option<Color> {
        match &self.column {
            EventColumn::TimeRange(v) => v.get_color(data, &self.now),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnShortId;

impl EventColumnShortId {
    fn format<'a>(&self, event: &'a EventWithShortId<impl Event>) -> Cow<'a, str> {
        event.short_id.to_string().into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnSummary;

impl EventColumnSummary {
    fn format<'a>(&self, event: &'a EventWithShortId<impl Event>) -> Cow<'a, str> {
        event.inner.summary().into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnTimeRange;

impl EventColumnTimeRange {
    fn format<'a>(&self, event: &'a EventWithShortId<impl Event>) -> Cow<'a, str> {
        match (event.inner.start(), event.inner.end()) {
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
            (None, None) => "".to_string().into(),
        }
    }

    fn get_color(
        &self,
        event: &EventWithShortId<impl Event>,
        now: &NaiveDateTime,
    ) -> Option<Color> {
        const COLOR_CURRENT: Option<Color> = Some(Color::Yellow);
        const COLOR_TODAY_LATE: Option<Color> = Some(Color::Green);

        let start = event.inner.start()?;
        match LooseDateTime::position_in_range(now, &start, &event.inner.end()) {
            RangePosition::Before => COLOR_TODAY_LATE,
            RangePosition::InRange => COLOR_CURRENT,
            RangePosition::After => None,
            RangePosition::InvalidRange => {
                log::warn!("Invalid range for event: {}", event.inner.uid());
                None
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnUid;

impl EventColumnUid {
    fn format<'a>(&self, event: &'a EventWithShortId<impl Event>) -> Cow<'a, str> {
        format!("#{}", event.inner.uid()).into()
    }
}
