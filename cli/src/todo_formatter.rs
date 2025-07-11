// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aim_core::{Priority, Todo, TodoStatus};
use chrono::NaiveDateTime;
use colored::{Color, Colorize};
use std::io;
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct TodoFormatter {
    pub columns: Vec<TodoColumn>,
    pub separator: String,
    pub padding: bool,
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
            separator: " ".to_string(),
            padding: true,
            now,
        }
    }

    pub fn write(
        &self,
        w: &mut impl io::Write,
        todos: &Vec<impl Todo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let table = todos
            .iter()
            .map(|todo| self.columns.iter().map(|col| col.format(todo)).collect())
            .collect();

        let columns = self.compute_columns(&table);

        for (cells, todo) in table.into_iter().zip(todos) {
            for (i, (col, cell)) in columns.iter().zip(cells.into_iter()).enumerate() {
                let cell = col.stylize_cell(todo, cell);
                write!(w, "{}", cell)?;

                if i == columns.len() - 1 {
                    write!(w, "\n")?;
                } else {
                    write!(w, "{}", self.separator)?;
                }
            }
        }

        Ok(())
    }

    fn compute_columns(&self, table: &Vec<Vec<String>>) -> Vec<ColumnStylizer> {
        let max_lengths = if self.padding {
            Some(get_column_max_width(table))
        } else {
            None
        };

        let mut columns = Vec::with_capacity(self.columns.len());
        for (i, col) in self.columns.iter().enumerate() {
            let padding_direction = col.padding_direction();

            let padding = if max_lengths.is_none()
                || (i == self.columns.len() - 1 && padding_direction == PaddingDirection::Left)
            {
                None // Last column does not need padding if it's left-aligned
            } else {
                Some((max_lengths.as_ref().map_or(0, |m| m[i]), padding_direction))
            };

            columns.push(ColumnStylizer {
                config: col,
                now: &self.now,
                padding,
            });
        }
        columns
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

impl TodoColumn {
    fn format(&self, todo: &impl Todo) -> String {
        match self {
            TodoColumn::Due(a) => a.format(todo),
            TodoColumn::Id(a) => a.format(todo),
            TodoColumn::Priority(a) => a.format(todo),
            TodoColumn::Status(a) => a.format(todo),
            TodoColumn::Summary(a) => a.format(todo),
        }
    }

    fn padding_direction(&self) -> PaddingDirection {
        match self {
            TodoColumn::Id(_) | TodoColumn::Priority(_) => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoColumnDue;

impl TodoColumnDue {
    pub fn format(&self, todo: &impl Todo) -> String {
        todo.due().map_or("".to_string(), |a| a.format())
    }

    fn get_color(&self, now: &NaiveDateTime, todo: &impl Todo) -> Option<Color> {
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

#[derive(Debug, Clone)]
struct ColumnStylizer<'a> {
    config: &'a TodoColumn,
    now: &'a NaiveDateTime,
    /// padding width and direction
    padding: Option<(usize, PaddingDirection)>,
}

impl<'a> ColumnStylizer<'a> {
    pub fn stylize_cell(&self, todo: &impl Todo, cell: String) -> String {
        let cell = match self.padding {
            Some((width, PaddingDirection::Left)) => format!("{:<width$}", cell, width = width),
            Some((width, PaddingDirection::Right)) => format!("{:>width$}", cell, width = width),
            _ => cell,
        };

        self.colorize_cell(todo, cell)
    }

    fn colorize_cell(&self, todo: &impl Todo, cell: String) -> String {
        let color = match self.config {
            TodoColumn::Due(v) => v.get_color(self.now, todo),
            TodoColumn::Priority(v) => v.get_color(),
            _ => None,
        };

        match color {
            Some(color) => cell.color(color).to_string(),
            _ => cell,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaddingDirection {
    Left,
    Right,
}

fn get_column_max_width(table: &Vec<Vec<String>>) -> Vec<usize> {
    let mut max_width = vec![0; table[0].len()];
    for row in table {
        for (i, cell) in row.iter().enumerate() {
            let width = cell.width();
            if width > max_width[i] {
                max_width[i] = width;
            }
        }
    }
    max_width
}
