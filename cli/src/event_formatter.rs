// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, fmt};

use aimcal_core::{Event, LooseDateTime, RangePosition};
use chrono::{DateTime, Local};
use colored::Color;

use crate::table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson};
use crate::util::{format_datetime, ArgOutputFormat};

#[derive(Debug, Clone)]
pub struct EventFormatter {
    now: DateTime<Local>,
    columns: Vec<EventColumn>,
    format: ArgOutputFormat,
}

impl EventFormatter {
    pub fn new(now: DateTime<Local>, columns: Vec<EventColumn>) -> Self {
        Self {
            now,
            columns,
            format: ArgOutputFormat::Table,
        }
    }

    pub fn with_output_format(mut self, format: ArgOutputFormat) -> Self {
        self.format = format;
        self
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
    Id(EventColumnId),
    Summary(EventColumnSummary),
    TimeRange(EventColumnTimeRange),
    Uid(EventColumnUid),
}

#[derive(Debug, Clone, Copy)]
struct ColumnMeta<'a> {
    column: &'a EventColumn,
    now: DateTime<Local>,
}

impl<'a, E: Event> TableColumn<E> for ColumnMeta<'a> {
    fn name(&self) -> Cow<'_, str> {
        match self.column {
            EventColumn::Id(_) => "ID",
            EventColumn::Summary(_) => "Summary",
            EventColumn::TimeRange(_) => "Time Range",
            EventColumn::Uid(_) => "UID",
        }
        .into()
    }

    fn format<'b>(&self, data: &'b E) -> Cow<'b, str> {
        match self.column {
            EventColumn::Id(a) => a.format(data),
            EventColumn::Summary(a) => a.format(data),
            EventColumn::TimeRange(a) => a.format(data),
            EventColumn::Uid(a) => a.format(data),
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
            EventColumn::TimeRange(v) => v.get_color(data, &self.now),
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
        event.summary().into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventColumnTimeRange;

impl EventColumnTimeRange {
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
            (None, None) => "".to_string().into(),
        }
    }

    fn get_color(&self, event: &impl Event, now: &DateTime<Local>) -> Option<Color> {
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
pub struct EventColumnUid;

impl EventColumnUid {
    fn format<'a>(&self, event: &'a impl Event) -> Cow<'a, str> {
        format!("#{}", event.uid()).into()
    }
}
