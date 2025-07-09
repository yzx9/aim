// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aim_core::{Todo, TodoStatus};
use chrono::{DateTime, Utc};
use colored::{ColoredString, Colorize};

pub struct TodoFormatter {
    pub columns: Vec<TodoFormatterColumn>,
    pub separator: String,
    pub padding: bool,
    pub now: DateTime<Utc>,
}

impl TodoFormatter {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            columns: vec![
                TodoFormatterColumn::Status,
                TodoFormatterColumn::Id,
                TodoFormatterColumn::DueAt,
                TodoFormatterColumn::Summary,
            ],
            separator: " ".to_string(),
            padding: true,
            now,
        }
    }

    pub fn format(&self, todos: &Vec<impl Todo>) -> Vec<ColoredString> {
        let table = todos.iter().map(|a| self.cells(a)).collect();
        let max_lengths = compute_column_max_length(&table);
        table
            .into_iter()
            .zip(todos)
            .map(|(cells, todo)| {
                let cells = if self.padding {
                    self.padding_cell(&max_lengths, cells)
                } else {
                    cells // TODO: dont compute max_lengths
                };
                let formatted = cells.join(&self.separator);
                let colorized = self.colorize_row(todo, formatted);
                colorized
            })
            .collect()
    }

    fn cells(&self, todo: &impl Todo) -> Vec<String> {
        vec![
            format_status(todo),
            todo.id().to_string(),
            todo.due_at()
                .map(|a| format_time(&a))
                .unwrap_or("".to_string()),
            todo.summary().to_string(),
        ]
    }

    fn padding_cell(&self, max_lengths: &Vec<usize>, cells: Vec<String>) -> Vec<String> {
        cells
            .iter()
            .enumerate()
            .map(|(i, cell)| match self.columns[i] {
                TodoFormatterColumn::Id => format!("{:>width$}", cell, width = max_lengths[i]),
                _ if i == self.columns.len() - 1 => cell.into(),
                _ => format!("{:<width$}", cell, width = max_lengths[i]),
            })
            .collect()
    }

    fn colorize_row(&self, todo: &impl Todo, row: String) -> ColoredString {
        match todo.due_at() {
            Some(due_at) if due_at < self.now => row.red(),
            Some(due_at) if due_at.date_naive() == self.now.date_naive() => row.yellow(),
            _ => row.into(),
        }
    }
}

pub enum TodoFormatterColumn {
    Status,
    Id,
    DueAt,
    Summary,
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

fn format_time(time: &DateTime<chrono::Utc>) -> String {
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
