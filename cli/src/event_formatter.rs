use crate::table::{Column, PaddingDirection, Table};
use aim_core::Event;
use chrono::NaiveDateTime;
use colored::Color;
use std::io;

#[derive(Debug)]
pub struct EventFormatter {
    pub columns: Vec<EventColumn>,
    pub now: NaiveDateTime,
}

impl EventFormatter {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            columns: vec![
                EventColumn::Uid(EventColumnUid),
                EventColumn::TimeRange(EventColumnTimeRange),
                EventColumn::Summary(EventColumnSummary),
            ],
            now,
        }
    }

    pub fn write_to(
        &self,
        w: &mut impl io::Write,
        events: &Vec<impl Event>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Table::new(&self.columns, &events, &(self.now,)).write_to(w)
    }
}

#[derive(Debug, Clone)]
pub enum EventColumn {
    Summary(EventColumnSummary),
    TimeRange(EventColumnTimeRange),
    Uid(EventColumnUid),
}

type Prior = (NaiveDateTime,);

impl<T: Event> Column<T, Prior> for EventColumn {
    fn format(&self, _prior: &Prior, data: &T) -> String {
        match self {
            EventColumn::Summary(a) => a.format(data),
            EventColumn::TimeRange(a) => a.format(data),
            EventColumn::Uid(a) => a.format(data),
        }
    }

    fn padding_direction(&self, _prior: &Prior, _data: &T) -> PaddingDirection {
        match self {
            EventColumn::Uid(_) => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, _prior: &Prior, _data: &T) -> Option<Color> {
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
