// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, fmt};

use aimcal_core::{LooseDateTime, Priority, RangePosition, Todo, TodoStatus};
use colored::Color;
use jiff::Zoned;

use crate::table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson};
use crate::util::{OutputFormat, format_datetime};

#[derive(Debug, Clone)]
pub struct TodoFormatter {
    now: Zoned,
    columns: Vec<TodoColumn>,
    format: OutputFormat,
}

impl TodoFormatter {
    pub fn new(now: Zoned, columns: Vec<TodoColumn>, format: OutputFormat) -> Self {
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
                now: self.formatter.now.clone(),
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

#[derive(Debug, Clone)]
struct ColumnMeta<'a> {
    column: &'a TodoColumn,
    now: Zoned,
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
        tracing::warn!(
            uid = uid.as_ref(),
            "todo does not have a short ID, using UID instead."
        );
        uid
    }
}

fn format_due(todo: &impl Todo) -> Cow<'_, str> {
    todo.due().map_or("".into(), |a| format_datetime(a).into())
}

fn get_color_due(todo: &impl Todo, now: &Zoned) -> Option<Color> {
    let due = todo.due()?; // Ensure due date is present
    get_color_due_impl(due, now)
}

fn get_color_due_impl(due: LooseDateTime, now: &Zoned) -> Option<Color> {
    const COLOR_LONG_OVERDUE: Option<Color> = Some(Color::Red);
    const COLOR_OVERDUE: Option<Color> = Some(Color::BrightRed);
    const COLOR_COMING: Option<Color> = Some(Color::Yellow);

    let same_day = due.date() == now.date();
    match LooseDateTime::position_in_range(&now.datetime(), &None, &Some(due.clone())) {
        RangePosition::InRange if same_day => COLOR_COMING, // not due && due in today
        RangePosition::InRange => None,                     // not due
        RangePosition::After if same_day => COLOR_OVERDUE,  // overdue && due in today
        RangePosition::After => COLOR_LONG_OVERDUE,         // overdue
        pos => {
            tracing::error!(?due, now = ?now, ?pos, "Invalid state when computing due date color.");
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

#[expect(clippy::unnecessary_wraps)]
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
    todo.uid()
}

fn format_uid_legacy(todo: &impl Todo) -> Cow<'_, str> {
    format!("#{}", todo.uid()).into()
}

#[cfg(test)]
mod tests {
    use colored::Color;
    use jiff::civil::{DateTime, date, time};

    use super::*;

    #[test]
    fn computes_color_based_on_due_date() {
        let due_date = date(2025, 8, 5);
        let due_time = time(12, 0, 0, 0);
        let due = LooseDateTime::Floating(DateTime::from_parts(due_date, due_time));

        for (title, year, month, day, hour, minute, second, expected) in [
            ("Overdue yesterday", 2025, 8, 6, 10, 0, 0, Some(Color::Red)),
            (
                "Today before due time",
                2025,
                8,
                5,
                12,
                0,
                0,
                Some(Color::Yellow),
            ),
            (
                "Today after due time",
                2025,
                8,
                5,
                14,
                0,
                0,
                Some(Color::BrightRed),
            ),
            ("Overdue by one day", 2025, 8, 6, 12, 0, 0, Some(Color::Red)),
            ("Future date", 2025, 8, 4, 10, 0, 0, None),
        ] {
            let date = date(year, month, day);
            let time = time(hour, minute, second, 0);
            let now = DateTime::from_parts(date, time)
                .to_zoned(jiff::tz::TimeZone::system())
                .unwrap();
            let color = get_color_due_impl(due.clone(), &now);
            assert_eq!(color, expected, "Failed for case: {title}");
        }
    }
}
