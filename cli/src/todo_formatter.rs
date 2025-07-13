// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::table::{Column, PaddingDirection, Table};
use aim_core::{Priority, Todo, TodoStatus};
use chrono::NaiveDateTime;
use colored::Color;
use std::io;

#[derive(Debug)]
pub struct TodoFormatter {
    pub columns: Vec<TodoColumn>,
    pub now: NaiveDateTime,
}

impl TodoFormatter {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            columns: vec![
                TodoColumn::Status(TodoColumnStatus),
                TodoColumn::Id(TodoColumnId),
                TodoColumn::Priority(TodoColumnPriority),
                TodoColumn::Due(TodoColumnDue),
                TodoColumn::Summary(TodoColumnSummary),
            ],
            now,
        }
    }

    pub fn write(
        &self,
        w: &mut impl io::Write,
        todos: &Vec<impl Todo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Table::new(&self.columns, &todos, &(self.now,)).write_to(w)
    }
}

#[derive(Debug, Clone)]
pub enum TodoColumn {
    Due(TodoColumnDue),
    Id(TodoColumnId),
    Priority(TodoColumnPriority),
    Status(TodoColumnStatus),
    Summary(TodoColumnSummary),
}

type Prior = (NaiveDateTime,);

impl<T: Todo> Column<T, Prior> for TodoColumn {
    fn format(&self, _prior: &Prior, data: &T) -> String {
        match self {
            TodoColumn::Due(a) => a.format(data),
            TodoColumn::Id(a) => a.format(data),
            TodoColumn::Priority(a) => a.format(data),
            TodoColumn::Status(a) => a.format(data),
            TodoColumn::Summary(a) => a.format(data),
        }
    }

    fn padding_direction(&self, _prior: &Prior, _data: &T) -> PaddingDirection {
        match self {
            TodoColumn::Id(_) | TodoColumn::Priority(_) => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }

    fn get_color(&self, prior: &Prior, data: &T) -> Option<Color> {
        match self {
            TodoColumn::Due(v) => v.get_color(data, &prior.0),
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
pub struct TodoColumnId;

impl TodoColumnId {
    fn format(&self, todo: &impl Todo) -> String {
        todo.id().to_string()
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
