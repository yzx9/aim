// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    OutputFormat,
    table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson},
};
use aim_core::{Priority, Todo, TodoStatus};
use chrono::NaiveDateTime;
use colored::Color;
use std::io;

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
                TodoColumn::Uid(TodoColumnUid),
                TodoColumn::Priority(TodoColumnPriority),
                TodoColumn::Due(TodoColumnDue),
                TodoColumn::Summary(TodoColumnSummary),
            ],
            now,
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
        todos: &Vec<impl Todo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let columns = self
            .columns
            .iter()
            .map(|col| TodoColumnMeta {
                column: &col,
                now: self.now,
            })
            .collect::<Vec<_>>();

        match self.format {
            OutputFormat::Json => Table::new(TableStyleJson::new(), &columns, &todos).write_to(w),
            OutputFormat::Table => Table::new(TableStyleBasic::new(), &columns, &todos).write_to(w),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TodoColumn {
    Due(TodoColumnDue),
    Priority(TodoColumnPriority),
    Status(TodoColumnStatus),
    Summary(TodoColumnSummary),
    Uid(TodoColumnUid),
}

struct TodoColumnMeta<'a> {
    column: &'a TodoColumn,
    now: NaiveDateTime,
}

impl<'a, T: Todo> TableColumn<T> for TodoColumnMeta<'a> {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        match &self.column {
            TodoColumn::Due(_) => "Due",
            TodoColumn::Priority(_) => "Priority",
            TodoColumn::Status(_) => "Status",
            TodoColumn::Summary(_) => "Summary",
            TodoColumn::Uid(_) => "UID",
        }
        .into()
    }

    fn format(&self, data: &T) -> String {
        match &self.column {
            TodoColumn::Due(a) => a.format(data),
            TodoColumn::Priority(a) => a.format(data),
            TodoColumn::Status(a) => a.format(data),
            TodoColumn::Summary(a) => a.format(data),
            TodoColumn::Uid(a) => a.format(data),
        }
    }

    fn padding_direction(&self, _data: &T) -> PaddingDirection {
        match &self.column {
            TodoColumn::Uid(_) | TodoColumn::Priority(_) => PaddingDirection::Right,
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

#[derive(Debug, Clone)]
pub struct TodoColumnDue;

impl TodoColumnDue {
    pub fn format(&self, todo: &impl Todo) -> String {
        todo.due().map_or("".to_string(), |a| a.format())
    }

    fn get_color(&self, todo: &impl Todo, now: &NaiveDateTime) -> Option<Color> {
        const COLOR_OVERDUE: Option<Color> = Some(Color::Red);
        const COLOR_TODAY: Option<Color> = Some(Color::Yellow);

        let Some(due) = todo.due() else { return None };

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
pub struct TodoColumnUid;

impl TodoColumnUid {
    fn format(&self, todo: &impl Todo) -> String {
        todo.uid().to_string()
    }
}

#[derive(Debug, Clone)]
pub struct TodoColumnPriority;

impl TodoColumnPriority {
    fn format(&self, todo: &impl Todo) -> String {
        match todo.priority() {
            // TODO: Use a more sophisticated mapping for priority to string
            Priority::P1 | Priority::P2 | Priority::P3 => "!!!",
            Priority::P4 | Priority::P5 | Priority::P6 => "!!",
            Priority::P7 | Priority::P8 | Priority::P9 => "!",
            _ => "",
        }
        .to_string()
    }

    fn get_color(&self) -> Option<Color> {
        Some(Color::Red)
    }
}

#[derive(Debug, Clone)]
pub struct TodoColumnStatus;

impl TodoColumnStatus {
    fn format(&self, todo: &impl Todo) -> String {
        match todo.status() {
            Some(TodoStatus::NeedsAction) => "[ ]".to_string(),
            Some(TodoStatus::Completed) => "[x]".to_string(),
            Some(TodoStatus::Cancelled) => " ✗ ".to_string(),
            Some(TodoStatus::InProcess) => match todo.percent() {
                Some(percent) if percent > 0 => format!("[{}]", "x".repeat(percent as usize)),
                _ => "[ ]".to_string(),
            },
            None => "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoColumnSummary;

impl TodoColumnSummary {
    fn format(&self, todo: &impl Todo) -> String {
        todo.summary().to_string()
    }
}
