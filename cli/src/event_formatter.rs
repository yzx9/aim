use crate::table::{Column, PaddingDirection, Table};
use aim_core::Event;
use chrono::NaiveDateTime;
use colored::Color;
use std::io;

#[derive(Debug)]
pub struct EventFormatter {
    pub columns: Vec<EventColumn>,
    pub separator: String,
    pub padding: bool,
    pub now: NaiveDateTime,
}

impl EventFormatter {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            columns: vec![
                EventColumn::Id(EventColumnId),
                EventColumn::TimeRange(EventColumnTimeRange),
                EventColumn::Summary(EventColumnSummary),
            ],
            separator: " ".to_string(),
            padding: true,
            now,
        }
    }

    pub fn write_to(
        &self,
        w: &mut impl io::Write,
        events: &Vec<impl Event>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Table {
            columns: self.columns.clone(),
            separator: self.separator.clone(),
            padding: self.padding,
            now: self.now,
            data: events,
        }
        .write_to(w)
    }
}

#[derive(Debug, Clone)]
pub enum EventColumn {
    Id(EventColumnId),
    Summary(EventColumnSummary),
    TimeRange(EventColumnTimeRange),
}

impl<T: Event> Column<T> for EventColumn {
    fn format(&self, data: &T) -> String {
        match self {
            EventColumn::Id(a) => a.format(data),
            EventColumn::Summary(a) => a.format(data),
            EventColumn::TimeRange(a) => a.format(data),
        }
    }

    fn padding_direction(&self) -> PaddingDirection {
        match self {
            EventColumn::Id(_) => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, _now: &NaiveDateTime, _data: &T) -> Option<Color> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct EventColumnId;

impl EventColumnId {
    fn format(&self, event: &impl Event) -> String {
        format!("#{}", event.id())
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
