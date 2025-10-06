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

impl<'a, T: Todo> fmt::Display for Display<'a, T> {
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
    Due(TodoColumnDue),
    Id(TodoColumnId),
    Priority(TodoColumnPriority),
    ShortId(TodoColumnShortId),
    Status(TodoColumnStatus),
    Summary(TodoColumnSummary),
    Uid(TodoColumnUid),
    UidLegacy(TodoColumnUidLegacy),
}

impl TodoColumn {
    pub fn due() -> Self {
        TodoColumn::Due(TodoColumnDue)
    }

    pub fn id() -> Self {
        TodoColumn::Id(TodoColumnId)
    }

    pub fn priority() -> Self {
        TodoColumn::Priority(TodoColumnPriority)
    }

    pub fn short_id() -> Self {
        TodoColumn::ShortId(TodoColumnShortId)
    }

    pub fn status() -> Self {
        TodoColumn::Status(TodoColumnStatus)
    }

    pub fn summary() -> Self {
        TodoColumn::Summary(TodoColumnSummary)
    }

    pub fn uid() -> Self {
        TodoColumn::Uid(TodoColumnUid)
    }

    pub fn uid_legacy() -> Self {
        TodoColumn::UidLegacy(TodoColumnUidLegacy)
    }
}

#[derive(Debug, Clone, Copy)]
struct ColumnMeta<'a> {
    column: &'a TodoColumn,
    now: DateTime<Local>,
}

impl<'a, T: Todo> TableColumn<T> for ColumnMeta<'a> {
    fn name(&self) -> Cow<'_, str> {
        match self.column {
            TodoColumn::Due(_) => "Due",
            TodoColumn::Id(_) => "ID",
            TodoColumn::Priority(_) => "Priority",
            TodoColumn::ShortId(_) => "Short ID",
            TodoColumn::Status(_) => "Status",
            TodoColumn::Summary(_) => "Summary",
            TodoColumn::Uid(_) => "UID",
            TodoColumn::UidLegacy(_) => "UID",
        }
        .into()
    }

    fn format<'b>(&self, data: &'b T) -> Cow<'b, str> {
        match self.column {
            TodoColumn::Due(a) => a.format(data),
            TodoColumn::Id(a) => a.format(data),
            TodoColumn::Priority(a) => a.format(data),
            TodoColumn::ShortId(a) => a.format(data),
            TodoColumn::Status(a) => a.format(data),
            TodoColumn::Summary(a) => a.format(data),
            TodoColumn::Uid(a) => a.format(data),
            TodoColumn::UidLegacy(a) => a.format(data),
        }
    }

    fn padding_direction(&self) -> PaddingDirection {
        use TodoColumn::*;
        match self.column {
            Id(_) | Priority(_) | Uid(_) | ShortId(_) => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, data: &T) -> Option<Color> {
        match self.column {
            TodoColumn::Due(v) => v.get_color(data, &self.now),
            TodoColumn::Priority(v) => v.get_color(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnId;

impl TodoColumnId {
    fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        if let Some(short_id) = todo.short_id() {
            short_id.to_string().into()
        } else {
            let uid = todo.uid(); // Fallback to the full UID if no short ID is available
            tracing::warn!(uid, "todo does not have a short ID, using UID instead.",);
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
        todo.summary().replace('\n', "↵").into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnShortId;

impl TodoColumnShortId {
    fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        todo.short_id()
            .map(|a| a.to_string())
            .unwrap_or_default()
            .into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnUid;

impl TodoColumnUid {
    fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        todo.uid().into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoColumnUidLegacy;

impl TodoColumnUidLegacy {
    fn format<'a>(&self, todo: &'a impl Todo) -> Cow<'a, str> {
        format!("#{}", todo.uid()).into()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, NaiveDate, TimeZone};
    use colored::Color;

    use super::*;

    #[test]
    fn test_compute_color() {
        let col = TodoColumnDue;
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
            let color = col.compute_color(due, &now);
            assert_eq!(color, expected, "Failed for case: {}", title);
        }
    }
}
