// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use colored::{Color, Colorize};
use std::{io, marker::PhantomData};
use unicode_width::UnicodeWidthStr;

pub struct Table<'a, T, P, C: Column<T, P>> {
    columns: &'a Vec<C>,
    data: &'a Vec<T>,
    prior: &'a P,
    separator: String,
    padding: bool,
}

impl<'a, T, K, C: Column<T, K>> Table<'a, T, K, C> {
    pub fn new(columns: &'a Vec<C>, data: &'a Vec<T>, prior: &'a K) -> Self {
        Self {
            columns,
            data,
            prior,
            separator: " ".to_string(),
            padding: true,
        }
    }

    // pub fn set_separator<S: Into<String>>(mut self, separator: S) -> Self {
    //     self.separator = separator.into();
    //     self
    // }
    //
    // pub fn set_padding(mut self, padding: bool) -> Self {
    //     self.padding = padding;
    //     self
    // }

    pub fn write_to(&self, w: &mut impl io::Write) -> Result<(), Box<dyn std::error::Error>> {
        let table = self
            .data
            .iter()
            .map(|data| {
                self.columns
                    .iter()
                    .map(|col| col.format(self.prior, data))
                    .collect()
            })
            .collect();

        let columns = self.compute_columns(&table);

        for (cells, todo) in table.into_iter().zip(self.data) {
            for (j, (col, cell)) in columns.iter().zip(cells.into_iter()).enumerate() {
                let cell = col.stylize_cell(todo, cell);
                write!(w, "{}", cell)?;

                if j < columns.len() - 1 {
                    write!(w, "{}", self.separator)?;
                } else {
                    write!(w, "\n")?;
                }
            }
        }

        Ok(())
    }

    fn compute_columns(&self, table: &Vec<Vec<String>>) -> Vec<ColumnStylizer<T, K, C>> {
        let max_lengths = self.padding.then(|| get_column_max_width(table));

        let mut columns = Vec::with_capacity(self.columns.len());
        for (i, (col, data)) in self.columns.iter().zip(self.data).enumerate() {
            let padding_direction = col.padding_direction(self.prior, data);

            let padding = if max_lengths.is_none()
                || (i == self.columns.len() - 1 && padding_direction == PaddingDirection::Left)
            {
                None // Last column does not need padding if it's left-aligned
            } else {
                Some((max_lengths.as_ref().map_or(0, |m| m[i]), padding_direction))
            };

            columns.push(ColumnStylizer {
                config: col,
                prior: self.prior,
                padding,
                _marker: PhantomData,
            });
        }
        columns
    }
}

pub trait Column<T, P> {
    /// Format the data for the column.
    fn format(&self, prior: &P, data: &T) -> String;
    /// Determine the padding direction for the column.
    fn padding_direction(&self, prior: &P, data: &T) -> PaddingDirection;
    /// Get the color for the column based on the data and prior.
    fn get_color(&self, prior: &P, data: &T) -> Option<Color>;
}

#[derive(Debug, Clone)]
struct ColumnStylizer<'a, T, P, C: Column<T, P>> {
    config: &'a C,
    prior: &'a P,
    /// padding width and direction
    padding: Option<(usize, PaddingDirection)>,
    _marker: PhantomData<T>,
}

impl<'a, T, K, C: Column<T, K>> ColumnStylizer<'a, T, K, C> {
    pub fn stylize_cell(&self, data: &T, cell: String) -> String {
        let cell = match self.padding {
            Some((width, PaddingDirection::Left)) => format!("{:<width$}", cell, width = width),
            Some((width, PaddingDirection::Right)) => format!("{:>width$}", cell, width = width),
            _ => cell,
        };

        self.colorize_cell(data, cell)
    }

    fn colorize_cell(&self, data: &T, cell: String) -> String {
        match self.config.get_color(self.prior, data) {
            Some(color) => cell.color(color).to_string(),
            _ => cell,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingDirection {
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
