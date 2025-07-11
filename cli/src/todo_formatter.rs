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
    pub columns: Vec<ColumnConfig>,
    pub separator: String,
    pub padding: bool,
    pub now: NaiveDateTime,
}

impl TodoFormatter {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            columns: vec![
                ColumnConfig::Status,
                ColumnConfig::Id,
                ColumnConfig::Priority,
                ColumnConfig::DueAt,
                ColumnConfig::Summary,
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
        let table = todos.iter().map(|a| self.cells(a)).collect();
        let columns = self.compute_columns(&table);

        let is_white_separator = self.separator.chars().all(|c| c == ' ');
        for (cells, todo) in table.into_iter().zip(todos) {
            let row = self.compute_row(todo);

            for (i, (col, cell)) in columns.iter().zip(cells.into_iter()).enumerate() {
                let cell = col.format_cell(cell);
                if let Some(c) = row.color {
                    write!(w, "{}", cell.color(c))?;
                } else {
                    write!(w, "{}", cell)?;
                }

                if i == columns.len() - 1 {
                    write!(w, "\n")?;
                } else if row.color.is_some() && !is_white_separator {
                    write!(w, "{}", self.separator.color(row.color.unwrap()))?;
                } else {
                    write!(w, "{}", self.separator)?;
                }
            }
        }

        Ok(())
    }

    fn cells(&self, todo: &impl Todo) -> Vec<String> {
        self.columns
            .iter()
            .map(|col| match col {
                ColumnConfig::DueAt => todo.due().map_or("".to_string(), |a| a.format()),
                ColumnConfig::Id => todo.id().to_string(),
                ColumnConfig::Priority => format_priority(todo).to_string(),
                ColumnConfig::Status => format_status(todo),
                ColumnConfig::Summary => todo.summary().to_string(),
            })
            .collect()
    }

    fn compute_columns(&self, table: &Vec<Vec<String>>) -> Vec<Column> {
        let max_lengths = if self.padding {
            Some(compute_column_max_width(table))
        } else {
            None
        };

        let mut columns = Vec::with_capacity(self.columns.len());
        for (i, col) in self.columns.iter().enumerate() {
            let padding_direction = match col {
                ColumnConfig::Id | ColumnConfig::Priority => PaddingDirection::Right,
                _ => PaddingDirection::Left,
            };

            let padding = if max_lengths.is_none()
                || (i == self.columns.len() - 1 && padding_direction == PaddingDirection::Left)
            {
                None // Last column does not need padding if it's left-aligned
            } else {
                Some((max_lengths.as_ref().map_or(0, |m| m[i]), padding_direction))
            };

            columns.push(Column { padding });
        }
        columns
    }

    fn compute_row(&self, todo: &impl Todo) -> Row {
        Row {
            color: self.compute_row_color(todo),
        }
    }

    fn compute_row_color(&self, todo: &impl Todo) -> Option<Color> {
        const COLOR_OVERDUE: Option<Color> = Some(Color::Red);
        const COLOR_TODAY: Option<Color> = Some(Color::Yellow);

        let Some(due) = todo.due() else { return None };

        let due_date = due.date;
        let now_date = self.now.date();
        if due_date > now_date {
            None
        } else if due_date < now_date {
            COLOR_OVERDUE
        } else if let Some(due_time) = due.time {
            if due_time < self.now.time() {
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
pub enum ColumnConfig {
    DueAt,
    Id,
    Priority,
    Status,
    Summary,
}

#[derive(Debug, Clone)]
struct Column {
    /// padding width and direction
    padding: Option<(usize, PaddingDirection)>,
}

impl Column {
    fn format_cell(&self, cell: String) -> String {
        match self.padding {
            Some((width, PaddingDirection::Left)) => {
                format!("{:<width$}", cell, width = width)
            }
            Some((width, PaddingDirection::Right)) => {
                format!("{:>width$}", cell, width = width)
            }
            _ => cell,
        }
    }
}

#[derive(Debug, Clone)]
struct Row {
    color: Option<Color>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaddingDirection {
    Left,
    Right,
}

fn format_priority<T: Todo>(todo: &T) -> &str {
    match todo.priority() {
        // TODO: Use a more sophisticated mapping for priority to string
        Priority::P1 | Priority::P2 | Priority::P3 => "!!!",
        Priority::P4 | Priority::P5 | Priority::P6 => "!!",
        Priority::P7 | Priority::P8 | Priority::P9 => "!",
        _ => "",
    }
}

fn format_status<T: Todo>(todo: &T) -> String {
    match todo.status() {
        Some(TodoStatus::NeedsAction) => "[ ]".to_string(),
        Some(TodoStatus::Completed) => "[x]".to_string(),
        Some(TodoStatus::Cancelled) => " ✗ ".to_string(),
        Some(TodoStatus::InProcess) => {
            if let Some(percent) = todo.percent() {
                format!("{:<2}%", percent)
            } else {
                "[ ]".to_string()
            }
        }
        None => "".to_string(),
    }
}

fn compute_column_max_width(table: &Vec<Vec<String>>) -> Vec<usize> {
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
