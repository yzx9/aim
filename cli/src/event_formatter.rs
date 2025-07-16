use crate::{
    OutputFormat,
    table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson},
};
use aim_core::Event;
use chrono::NaiveDateTime;
use colored::Color;
use std::io;

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
                EventColumn::Uid(EventColumnUid),
                EventColumn::TimeRange(EventColumnTimeRange),
                EventColumn::Summary(EventColumnSummary),
            ],
            _now: now,
            format: OutputFormat::Table,
        }
    }

    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    pub fn write_to(
        &self,
        w: &mut impl io::Write,
        events: &Vec<impl Event>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.format {
            OutputFormat::Json => {
                Table::new(TableStyleJson::new(), &self.columns, &events).write_to(w)
            }
            OutputFormat::Table => {
                Table::new(TableStyleBasic::new(), &self.columns, &events).write_to(w)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventColumn {
    Summary(EventColumnSummary),
    TimeRange(EventColumnTimeRange),
    Uid(EventColumnUid),
}

impl<T: Event> TableColumn<T> for EventColumn {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        match self {
            EventColumn::Summary(_) => "Summary",
            EventColumn::TimeRange(_) => "Time Range",
            EventColumn::Uid(_) => "UID",
        }
        .into()
    }

    fn format(&self, data: &T) -> String {
        match self {
            EventColumn::Summary(a) => a.format(data),
            EventColumn::TimeRange(a) => a.format(data),
            EventColumn::Uid(a) => a.format(data),
        }
    }

    fn padding_direction(&self, _data: &T) -> PaddingDirection {
        match self {
            EventColumn::Uid(_) => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, _data: &T) -> Option<Color> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct EventColumnUid;

impl EventColumnUid {
    fn format(&self, event: &impl Event) -> String {
        format!("#{}", event.uid())
    }
}

#[derive(Debug, Clone)]
pub struct EventColumnSummary;

impl EventColumnSummary {
    fn format(&self, event: &impl Event) -> String {
        event.summary().to_string()
    }
}

#[derive(Debug, Clone)]
pub struct EventColumnTimeRange;

impl EventColumnTimeRange {
    fn format(&self, event: &impl Event) -> String {
        match (event.start(), event.end()) {
            (Some(start), Some(end)) => match start.date == end.date {
                true => match (start.time, end.time) {
                    (Some(stime), Some(etime)) => format!(
                        "{} {}~{}",
                        start.date.format("%Y-%m-%d"),
                        stime.format("%H:%M").to_string(),
                        etime.format("%H:%M").to_string()
                    ),
                    (Some(stime), None) => format!(
                        "{} {}~24:00",
                        start.date.format("%Y-%m-%d"),
                        stime.format("%H:%M").to_string()
                    ),
                    (None, Some(etime)) => format!(
                        "{} 00:00~{}",
                        start.date.format("%Y-%m-%d"),
                        etime.format("%H:%M").to_string()
                    ),
                    (None, None) => start.date.format("%Y-%m-%d").to_string(),
                },
                false => format!("{}~{}", start.format(), end.format()),
            },
            (Some(start), None) => format!("{}", start.format()),
            (None, Some(end)) => format!("~{}", end.format()),
            (None, None) => "".to_string(),
        }
    }
}
