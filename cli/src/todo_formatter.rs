// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, fmt};

use aimcal_core::{LooseDateTime, Priority, RangePosition, Todo, TodoStatus};
use chrono::{DateTime, Local};
use colored::Color;

use crate::table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson};
use crate::util::{ArgOutputFormat, format_datetime};

#[derive(Debug, Clone)]
pub struct TodoFormatter {
    now: DateTime<Local>,
    columns: Vec<TodoColumn>,
    format: ArgOutputFormat,
}

impl TodoFormatter {
    pub fn new(now: DateTime<Local>) -> Self {
        Self {
            now,
            columns: vec![
                TodoColumn::Status(TodoColumnStatus),
                TodoColumn::Id(TodoColumnId),
                TodoColumn::Priority(TodoColumnPriority),
                TodoColumn::Due(TodoColumnDue),
                TodoColumn::Summary(TodoColumnSummary),
            ],
            format: ArgOutputFormat::Table,
        }
    }

    pub fn with_output_format(mut self, format: ArgOutputFormat) -> Self {
        self.format = format;
        self
    }

    pub fn format<'a, T: Todo>(&'a self, todos: &'a [T]) -> Display<'a, T> {
        Display {
            todos,
            formatter: self,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Display<'a, T: Todo> {
    todos: &'a [T],
    formatter: &'a TodoFormatter,
}

impl<'a, T: Todo> fmt::Display for Display<'a, T> {
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
                Table::new(TableStyleJson::new(), &columns, self.todos)
            ),
            ArgOutputFormat::Table => write!(
                f,
                "{}",
                Table::new(TableStyleBasic::new(), &columns, self.todos)
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TodoColumn {
    Id(TodoColumnId),
    Due(TodoColumnDue),
    Priority(TodoColumnPriority),
    Status(TodoColumnStatus),
    Summary(TodoColumnSummary),
    #[allow(dead_code)]
    Uid(TodoColumnUid),
}

#[derive(Debug, Clone, Copy)]
struct ColumnMeta<'a> {
    column: &'a TodoColumn,
    now: DateTime<Local>,
}

impl<'a, T: Todo> TableColumn<T> for ColumnMeta<'a> {
    fn name(&self) -> Cow<'_, str> {
        match &self.column {
            TodoColumn::Id(_) => "ID",
            TodoColumn::Due(_) => "Due",
            TodoColumn::Priority(_) => "Priority",
            TodoColumn::Status(_) => "Status",
            TodoColumn::Summary(_) => "Summary",
            TodoColumn::Uid(_) => "UID",
        }
        .into()
    }

    fn format<'b>(&self, data: &'b T) -> Cow<'b, str> {
        match &self.column {
            TodoColumn::Id(a) => a.format(data),
            TodoColumn::Due(a) => a.format(data),
            TodoColumn::Priority(a) => a.format(data),
            TodoColumn::Status(a) => a.format(data),
            TodoColumn::Summary(a) => a.format(data),
            TodoColumn::Uid(a) => a.format(data),
        }
    }

    fn padding_direction(&self) -> PaddingDirection {
        match &self.column {
            TodoColumn::Id(_) | TodoColumn::Priority(_) | TodoColumn::Uid(_) => {
                PaddingDirection::Right
            }
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, data: &T) -> Option<Color> {
        match &self.column {
            TodoColumn::Due(v) => v.get_color(data, &self.now),
            TodoColumn::Priority(v) => v.get_color(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnId;

impl TodoColumnId {
    #[tracing::instrument(skip_all)]
    fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        if let Some(short_id) = todo.short_id() {
            short_id.to_string().into()
        } else {
            let uid = todo.uid(); // Fallback to the full UID if no short ID is available
            tracing::warn!(uid, "Todo does not have a short ID, using UID instead.",);
            uid.into()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnDue;

impl TodoColumnDue {
    pub fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        todo.due().map_or("".into(), |a| format_datetime(a).into())
    }

    fn get_color(&self, todo: &impl Todo, now: &DateTime<Local>) -> Option<Color> {
        let due = todo.due()?; // Ensure due date is present
        self.compute_color(due, now)
    }

    fn compute_color(&self, due: LooseDateTime, now: &DateTime<Local>) -> Option<Color> {
        const COLOR_OVERDUE: Option<Color> = Some(Color::Red);
        const COLOR_TODAY: Option<Color> = Some(Color::Yellow);

        let t = now.naive_local();
        match LooseDateTime::position_in_range(&t, &None, &Some(due)) {
            RangePosition::InRange => match LooseDateTime::position_in_range(
                &t,
                &Some(LooseDateTime::DateOnly(due.date())),
                &Some(due),
            ) {
                RangePosition::InRange => COLOR_TODAY, // due in today && 00:00 ~ due
                _ => None,
            },
            _ => COLOR_OVERDUE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnPriority;

impl TodoColumnPriority {
    fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        match todo.priority() {
            // TODO: Use a more sophisticated mapping for priority to string
            Priority::P1 | Priority::P2 | Priority::P3 => "!!!",
            Priority::P4 | Priority::P5 | Priority::P6 => "!!",
            Priority::P7 | Priority::P8 | Priority::P9 => "!",
            _ => "",
        }
        .into()
    }

    fn get_color(&self) -> Option<Color> {
        Some(Color::Red)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnStatus;

impl TodoColumnStatus {
    fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        match todo.status() {
            TodoStatus::NeedsAction => "[ ]".into(),
            TodoStatus::Completed => "[x]".into(),
            TodoStatus::Cancelled => " ✗ ".into(),
            TodoStatus::InProcess => {
                let percent = todo.percent_complete().unwrap_or_default();
                match percent {
                    0 => "[ ]".into(),
                    100 => "[x]".into(),
                    _ => format!("{percent}%").into(),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnSummary;

impl TodoColumnSummary {
    fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        todo.summary().into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnUid;

impl TodoColumnUid {
    fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        todo.uid().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, NaiveDate, TimeZone};
    use colored::Color;

    #[test]
    fn test_compute_color_overdue_yesterday() {
        let col = TodoColumnDue;
        let due = LooseDateTime::Floating(
            NaiveDate::from_ymd_opt(2025, 8, 4)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap(),
        );
        let now = Local.with_ymd_and_hms(2025, 8, 5, 10, 0, 0).unwrap();

        let color = col.compute_color(due, &now);
        assert_eq!(color, Some(Color::Red));
    }

    #[test]
    fn test_compute_color_today_before_due_time() {
        let col = TodoColumnDue;
        let due = LooseDateTime::Floating(
            NaiveDate::from_ymd_opt(2025, 8, 5)
                .unwrap()
                .and_hms_opt(18, 0, 0)
                .unwrap(),
        );
        let now = Local.with_ymd_and_hms(2025, 8, 5, 12, 0, 0).unwrap();

        let color = col.compute_color(due, &now);
        assert_eq!(color, Some(Color::Yellow));
    }

    #[test]
    fn test_compute_color_today_after_due_time() {
        let col = TodoColumnDue;
        let due = LooseDateTime::Floating(
            NaiveDate::from_ymd_opt(2025, 8, 5)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
        );
        let now = Local.with_ymd_and_hms(2025, 8, 5, 12, 0, 0).unwrap();

        let color = col.compute_color(due, &now);
        assert_eq!(color, Some(Color::Red));
    }

    #[test]
    fn test_compute_color_future_date() {
        let col = TodoColumnDue;
        let due = LooseDateTime::Floating(
            NaiveDate::from_ymd_opt(2025, 8, 6)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
        );
        let now = Local.with_ymd_and_hms(2025, 8, 5, 10, 0, 0).unwrap();

        let color = col.compute_color(due, &now);
        assert_eq!(color, None);
    }
}
