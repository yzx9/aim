// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    OutputFormat,
    short_id::TodoWithShortId,
    table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson},
};
use aim_core::{Priority, Todo, TodoStatus};
use chrono::NaiveDateTime;
use colored::Color;
use std::{borrow::Cow, fmt};

#[derive(Debug)]
pub struct TodoFormatter {
    columns: Vec<TodoColumn>,
    now: NaiveDateTime,
    format: OutputFormat,
}

impl TodoFormatter {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            columns: vec![
                TodoColumn::Status(TodoColumnStatus),
                TodoColumn::DisplayNumber(TodoColumnDisplayNumber),
                TodoColumn::Priority(TodoColumnPriority),
                TodoColumn::Due(TodoColumnDue),
                TodoColumn::Summary(TodoColumnSummary),
            ],
            now,
            format: OutputFormat::Table,
        }
    }

    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    pub fn format<'a, T: Todo>(&'a self, todos: &'a [TodoWithShortId<T>]) -> Display<'a, T> {
        Display {
            todos,
            formatter: self,
        }
    }
}

pub struct Display<'a, T: Todo> {
    todos: &'a [TodoWithShortId<T>],
    formatter: &'a TodoFormatter,
}

impl<'a, T: Todo> fmt::Display for Display<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let columns = self
            .formatter
            .columns
            .iter()
            .map(|column| TodoColumnMeta {
                column,
                now: self.formatter.now,
            })
            .collect::<Vec<_>>();

        match self.formatter.format {
            OutputFormat::Json => write!(
                f,
                "{}",
                Table::new(TableStyleJson::new(), &columns, self.todos)
            ),
            OutputFormat::Table => write!(
                f,
                "{}",
                Table::new(TableStyleBasic::new(), &columns, self.todos)
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TodoColumn {
    DisplayNumber(TodoColumnDisplayNumber),
    Due(TodoColumnDue),
    Priority(TodoColumnPriority),
    Status(TodoColumnStatus),
    Summary(TodoColumnSummary),
    #[allow(dead_code)]
    Uid(TodoColumnUid),
}

struct TodoColumnMeta<'a> {
    column: &'a TodoColumn,
    now: NaiveDateTime,
}

impl<'a, T: Todo> TableColumn<TodoWithShortId<T>> for TodoColumnMeta<'a> {
    fn name(&self) -> Cow<'_, str> {
        match &self.column {
            TodoColumn::DisplayNumber(_) => "Display Number",
            TodoColumn::Due(_) => "Due",
            TodoColumn::Priority(_) => "Priority",
            TodoColumn::Status(_) => "Status",
            TodoColumn::Summary(_) => "Summary",
            TodoColumn::Uid(_) => "UID",
        }
        .into()
    }

    fn format<'b>(&self, data: &'b TodoWithShortId<T>) -> Cow<'b, str> {
        match &self.column {
            TodoColumn::DisplayNumber(a) => a.format(data),
            TodoColumn::Due(a) => a.format(data),
            TodoColumn::Priority(a) => a.format(data),
            TodoColumn::Status(a) => a.format(data),
            TodoColumn::Summary(a) => a.format(data),
            TodoColumn::Uid(a) => a.format(data),
        }
    }

    fn padding_direction(&self, _data: &TodoWithShortId<T>) -> PaddingDirection {
        match &self.column {
            TodoColumn::DisplayNumber(_) | TodoColumn::Priority(_) | TodoColumn::Uid(_) => {
                PaddingDirection::Right
            }
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, data: &TodoWithShortId<T>) -> Option<Color> {
        match &self.column {
            TodoColumn::Due(v) => v.get_color(data, &self.now),
            TodoColumn::Priority(v) => v.get_color(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoColumnDisplayNumber;

impl TodoColumnDisplayNumber {
    fn format<'a>(&self, todo: &'a TodoWithShortId<impl Todo>) -> Cow<'a, str> {
        todo.short_id.to_string().into()
    }
}

#[derive(Debug, Clone)]
pub struct TodoColumnDue;

impl TodoColumnDue {
    pub fn format<'a>(&self, todo: &'a TodoWithShortId<impl Todo>) -> Cow<'a, str> {
        todo.inner.due().map_or("".into(), |a| a.format().into())
    }

    fn get_color(&self, todo: &TodoWithShortId<impl Todo>, now: &NaiveDateTime) -> Option<Color> {
        const COLOR_OVERDUE: Option<Color> = Some(Color::Red);
        const COLOR_TODAY: Option<Color> = Some(Color::Yellow);

        let due = todo.inner.due()?;
        let due_date = due.date;
        let now_date = now.date();
        if due_date > now_date {
            None
        } else if due_date < now_date {
            COLOR_OVERDUE
        } else if let Some(due_time) = due.time {
            if due_time < now.time() {
                COLOR_OVERDUE
            } else {
                COLOR_TODAY
            }
        } else {
            COLOR_TODAY
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoColumnPriority;

impl TodoColumnPriority {
    fn format<'a>(&self, todo: &'a TodoWithShortId<impl Todo>) -> Cow<'a, str> {
        match todo.inner.priority() {
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

#[derive(Debug, Clone)]
pub struct TodoColumnStatus;

impl TodoColumnStatus {
    fn format<'a>(&self, todo: &'a TodoWithShortId<impl Todo>) -> Cow<'a, str> {
        match todo.inner.status() {
            Some(TodoStatus::NeedsAction) => "[ ]".into(),
            Some(TodoStatus::Completed) => "[x]".into(),
            Some(TodoStatus::Cancelled) => " ✗ ".into(),
            Some(TodoStatus::InProcess) => match todo.inner.percent() {
                Some(percent) if percent > 0 => {
                    format!("[{}]", "x".repeat(percent as usize)).into()
                }
                _ => "[ ]".into(),
            },
            None => "".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoColumnSummary;

impl TodoColumnSummary {
    fn format<'a>(&self, todo: &'a TodoWithShortId<impl Todo>) -> Cow<'a, str> {
        todo.inner.summary().into()
    }
}

#[derive(Debug, Clone)]
pub struct TodoColumnUid;

impl TodoColumnUid {
    fn format<'a>(&self, todo: &'a TodoWithShortId<impl Todo>) -> Cow<'a, str> {
        todo.inner.uid().into()
    }
}
