// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aim_core::{Todo, TodoStatus};
use chrono::{DateTime, Utc};
use colored::{Color, Colorize};
use std::io;

#[derive(Debug)]
pub struct TodoFormatter {
    pub columns: Vec<ColumnConfig>,
    pub separator: String,
    pub padding: bool,
    pub now: chrono::DateTime<chrono::Utc>,
}

impl TodoFormatter {
    pub fn new(now: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            columns: vec![
                ColumnConfig::Status,
                ColumnConfig::Id,
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
                ColumnConfig::Id => todo.id().to_string(),
                ColumnConfig::Status => format_status(todo),
                ColumnConfig::DueAt => todo.due_at().map_or("".to_string(), |a| format_time(&a)),
                ColumnConfig::Summary => todo.summary().to_string(),
            })
            .collect()
    }

    fn compute_columns(&self, table: &Vec<Vec<String>>) -> Vec<Column> {
        let max_lengths = if self.padding {
            Some(compute_column_max_length(table))
        } else {
            None
        };

        let mut columns = Vec::with_capacity(self.columns.len());
        for (i, col) in self.columns.iter().enumerate() {
            let padding_direction = match col {
                ColumnConfig::Id => PaddingDirection::Right,
                _ => PaddingDirection::Left,
            };

            let padding = if max_lengths.is_none()
                || (i == self.columns.len() - 1 && padding_direction != PaddingDirection::Left)
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
        let color = match todo.due_at() {
            Some(due_at) if due_at < self.now => Some(colored::Color::Red),
            Some(due_at) if due_at.date_naive() == self.now.date_naive() => {
                Some(colored::Color::Yellow)
            }
            _ => None,
        };
        Row { color }
    }
}

#[derive(Debug, Clone)]
pub enum ColumnConfig {
    Id,
    Status,
    DueAt,
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

fn format_status<T: Todo>(todo: &T) -> String {
    match todo.status() {
        Some(TodoStatus::NeedsAction) => "[ ]",
        Some(TodoStatus::Completed) => "[x]",
        Some(TodoStatus::Cancelled) => " ✗ ",
        Some(TodoStatus::InProcess) => "[ ]", // TODO: xx%
        None => "",
    }
    .to_string()
}

fn format_time(time: &DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M").to_string()
}

fn compute_column_max_length(table: &Vec<Vec<String>>) -> Vec<usize> {
    let mut max_lengths = vec![0; table[0].len()];
    for row in table {
        for (i, cell) in row.iter().enumerate() {
            if cell.len() > max_lengths[i] {
                max_lengths[i] = cell.len();
            }
        }
    }
    max_lengths
}
