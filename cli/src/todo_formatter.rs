// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, fmt};

use aimcal_core::{LooseDateTime, Priority, RangePosition, Todo, TodoStatus};
use chrono::{DateTime, Local};
use colored::Color;

use crate::table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson};
use crate::util::{OutputFormat, format_datetime};

#[derive(Debug, Clone)]
pub struct TodoFormatter {
    now: DateTime<Local>,
    columns: Vec<TodoColumn>,
    format: OutputFormat,
}

impl TodoFormatter {
    pub fn new(now: DateTime<Local>, columns: Vec<TodoColumn>, format: OutputFormat) -> Self {
        Self {
            now,
            columns,
            format,
        }
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

impl<T: Todo> fmt::Display for Display<'_, T> {
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
                let table = Table::new(TableStyleJson::new(), &columns, self.todos);
                write!(f, "{table}")
            }
            OutputFormat::Table => {
                let table = Table::new(TableStyleBasic::new(), &columns, self.todos);
                write!(f, "{table}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TodoColumn {
    Due,
    Id,
    Priority,
    ShortId,
    Status,
    Summary,
    Uid,
    UidLegacy,
}

#[derive(Debug, Clone, Copy)]
struct ColumnMeta<'a> {
    column: &'a TodoColumn,
    now: DateTime<Local>,
}

impl<T: Todo> TableColumn<T> for ColumnMeta<'_> {
    fn name(&self) -> Cow<'_, str> {
        match self.column {
            TodoColumn::Due => "Due",
            TodoColumn::Id => "ID",
            TodoColumn::Priority => "Priority",
            TodoColumn::ShortId => "Short ID",
            TodoColumn::Status => "Status",
            TodoColumn::Summary => "Summary",
            TodoColumn::Uid | TodoColumn::UidLegacy => "UID",
        }
        .into()
    }

    fn format<'b>(&self, data: &'b T) -> Cow<'b, str> {
        match self.column {
            TodoColumn::Due => format_due(data),
            TodoColumn::Id => format_id(data),
            TodoColumn::Priority => format_priority(data),
            TodoColumn::ShortId => format_short_id(data),
            TodoColumn::Status => format_status(data),
            TodoColumn::Summary => format_summary(data),
            TodoColumn::Uid => format_uid(data),
            TodoColumn::UidLegacy => format_uid_legacy(data),
        }
    }

    fn padding_direction(&self) -> PaddingDirection {
        use TodoColumn::{Id, Priority, ShortId, Uid};
        match self.column {
            Id | Priority | Uid | ShortId => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, data: &T) -> Option<Color> {
        match self.column {
            TodoColumn::Due => get_color_due(data, &self.now),
            TodoColumn::Priority => get_color_priority(),
            _ => None,
        }
    }
}

fn format_id(todo: &impl Todo) -> Cow<'_, str> {
    if let Some(short_id) = todo.short_id() {
        short_id.to_string().into()
    } else {
        let uid = todo.uid(); // Fallback to the full UID if no short ID is available
        tracing::warn!(uid, "todo does not have a short ID, using UID instead.",);
        uid.into()
    }
}

fn format_due(todo: &impl Todo) -> Cow<'_, str> {
    todo.due().map_or("".into(), |a| format_datetime(a).into())
}

fn get_color_due(todo: &impl Todo, now: &DateTime<Local>) -> Option<Color> {
    let due = todo.due()?; // Ensure due date is present
    get_color_due_impl(due, now)
}

fn get_color_due_impl(due: LooseDateTime, now: &DateTime<Local>) -> Option<Color> {
    const COLOR_LONG_OVERDUE: Option<Color> = Some(Color::Red);
    const COLOR_OVERDUE: Option<Color> = Some(Color::BrightRed);
    const COLOR_COMING: Option<Color> = Some(Color::Yellow);

    let t = now.naive_local();
    let same_day = due.date() == t.date();
    match LooseDateTime::position_in_range(&t, &None, &Some(due)) {
        RangePosition::InRange if same_day => COLOR_COMING, // not due && due in today
        RangePosition::InRange => None,                     // not due
        RangePosition::After if same_day => COLOR_OVERDUE,  // overdue && due in today
        RangePosition::After => COLOR_LONG_OVERDUE,         // overdue
        pos => {
            tracing::error!(?due, now = ?t, ?pos, "Invalid state when computing due date color.");
            None
        }
    }
}

fn format_priority(todo: &impl Todo) -> Cow<'_, str> {
    match todo.priority() {
        Priority::P1 | Priority::P2 | Priority::P3 => "!!!",
        Priority::P4 | Priority::P5 | Priority::P6 => "!!",
        Priority::P7 | Priority::P8 | Priority::P9 => "!",
        Priority::None => "",
    }
    .into()
}

#[allow(clippy::unnecessary_wraps)]
fn get_color_priority() -> Option<Color> {
    Some(Color::Red)
}

fn format_status(todo: &impl Todo) -> Cow<'_, str> {
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

fn format_summary(todo: &impl Todo) -> Cow<'_, str> {
    todo.summary().replace('\n', "↵").into()
}

fn format_short_id(todo: &impl Todo) -> Cow<'_, str> {
    todo.short_id()
        .map(|a| a.to_string())
        .unwrap_or_default()
        .into()
}

fn format_uid(todo: &impl Todo) -> Cow<'_, str> {
    todo.uid().into()
}

fn format_uid_legacy(todo: &impl Todo) -> Cow<'_, str> {
    format!("#{}", todo.uid()).into()
}

#[cfg(test)]
mod tests {
    use chrono::{Local, NaiveDate, TimeZone};
    use colored::Color;

    use super::*;

    #[test]
    fn test_compute_color() {
        let due = LooseDateTime::Floating(
            NaiveDate::from_ymd_opt(2025, 8, 5)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap(),
        );

        for (title, now, expected) in [
            (
                "Overdue yesterday",
                Local.with_ymd_and_hms(2025, 8, 6, 10, 0, 0).unwrap(),
                Some(Color::Red),
            ),
            (
                "Today before due time",
                Local.with_ymd_and_hms(2025, 8, 5, 12, 0, 0).unwrap(),
                Some(Color::Yellow),
            ),
            (
                "Today after due time",
                Local.with_ymd_and_hms(2025, 8, 5, 14, 0, 0).unwrap(),
                Some(Color::BrightRed),
            ),
            (
                "Overdue by one day",
                Local.with_ymd_and_hms(2025, 8, 6, 12, 0, 0).unwrap(),
                Some(Color::Red),
            ),
            (
                "Future date",
                Local.with_ymd_and_hms(2025, 8, 4, 10, 0, 0).unwrap(),
                None,
            ),
        ] {
            let color = get_color_due_impl(due, &now);
            assert_eq!(color, expected, "Failed for case: {title}");
        }
    }
}
