// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use colored::{Color, Colorize};
use std::{borrow::Cow, io, marker::PhantomData};
use unicode_width::UnicodeWidthStr;

pub struct Table<'a, T, C: TableColumn<T>, S: TableStyle<'a, T, C>> {
    columns: &'a Vec<C>,
    data: &'a Vec<T>,
    style: S,
}

impl<'a, T, C: TableColumn<T>, S: TableStyle<'a, T, C>> Table<'a, T, C, S> {
    pub fn new(style: S, columns: &'a Vec<C>, data: &'a Vec<T>) -> Self {
        Self {
            columns,
            data,
            style,
        }
    }

    pub fn write_to(&self, w: &mut impl io::Write) -> Result<(), Box<dyn std::error::Error>> {
        let table = self
            .data
            .iter()
            .map(|data| self.columns.iter().map(|col| col.format(data)).collect())
            .collect();

        // let columns = self.compute_columns(&table);
        let columns = self.style.build(&self.columns, &self.data, &table);

        write!(w, "{}", self.style.table_starting(&columns))?;
        for (i, (cells, data)) in table.into_iter().zip(self.data).enumerate() {
            write!(w, "{}", self.style.row_starting(&data))?;
            for (j, (col, cell)) in columns.iter().zip(cells.into_iter()).enumerate() {
                write!(w, "{}", self.style.cell_stylize(data, &col, cell))?;

                if j < columns.len() - 1 {
                    write!(w, "{}", self.style.cell_separator())?;
                };
            }

            write!(w, "{}", self.style.row_ending(&data))?;

            if i < self.data.len() - 1 {
                write!(w, "{}", self.style.row_separator())?;
            }
        }
        write!(w, "{}", self.style.table_ending(&columns))?;

        Ok(())
    }
}

pub trait TableStyle<'a, T, C: TableColumn<T>> {
    type ColumnMeta;

    fn build<'b>(
        &self,
        columns: &'a Vec<C>,
        data: &'a Vec<T>,
        table: &'b Vec<Vec<String>>,
    ) -> Vec<Self::ColumnMeta>;

    fn table_starting(&self, _columns: &Vec<Self::ColumnMeta>) -> &str {
        ""
    }
    fn table_ending(&self, _columns: &Vec<Self::ColumnMeta>) -> &str {
        "\n"
    }
    fn row_starting(&self, _data: &T) -> &str {
        ""
    }
    fn row_ending(&self, _data: &T) -> &str {
        ""
    }
    fn row_separator(&self) -> &str {
        "\n"
    }
    fn cell_stylize(&self, _data: &T, _column: &Self::ColumnMeta, cell: String) -> String {
        cell
    }
    fn cell_separator(&self) -> &str {
        " "
    }
}

pub trait TableColumn<T> {
    fn name(&self) -> Cow<'_, str>;
    /// Format the data for the column.
    fn format(&self, data: &T) -> String;
    /// Determine the padding direction for the column.
    fn padding_direction(&self, data: &T) -> PaddingDirection;
    /// Get the color for the column based on the data.
    fn get_color(&self, data: &T) -> Option<Color>;
}

impl<T, C: TableColumn<T> + ?Sized> TableColumn<T> for Box<C> {
    fn name(&self) -> Cow<'_, str> {
        self.as_ref().name()
    }
    fn format(&self, data: &T) -> String {
        self.as_ref().format(data)
    }
    fn padding_direction(&self, data: &T) -> PaddingDirection {
        self.as_ref().padding_direction(data)
    }
    fn get_color(&self, data: &T) -> Option<Color> {
        self.as_ref().get_color(data)
    }
}

#[derive(Debug, Clone)]
pub struct TableStyleBasic {
    padding: bool,
}

impl TableStyleBasic {
    pub fn new() -> Self {
        Self { padding: true }
    }
}

impl<'a, T, C: 'a + TableColumn<T>> TableStyle<'a, T, C> for TableStyleBasic {
    type ColumnMeta = TodoColumnBasicMeta<'a, T, C>;

    fn build<'b>(
        &self,
        columns: &'a Vec<C>,
        data: &'a Vec<T>,
        table: &'b Vec<Vec<String>>,
    ) -> Vec<TodoColumnBasicMeta<'a, T, C>> {
        let max_lengths = self.padding.then(|| get_column_max_width(table));
        columns
            .iter()
            .zip(data)
            .enumerate()
            .map(|(i, (col, data))| {
                let padding_direction = col.padding_direction(data);

                let padding = if max_lengths.is_none()
                    || (i == columns.len() - 1 && padding_direction == PaddingDirection::Left)
                {
                    None // Last column does not need padding if it's left-aligned
                } else {
                    Some((max_lengths.as_ref().map_or(0, |m| m[i]), padding_direction))
                };

                TodoColumnBasicMeta::new(col, padding)
            })
            .collect()
    }

    fn cell_stylize(
        &self,
        data: &T,
        column: &TodoColumnBasicMeta<'a, T, C>,
        cell: String,
    ) -> String {
        column.stylize_cell(data, cell)
    }
}

/// Direction for padding in table cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingDirection {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct TodoColumnBasicMeta<'a, T, C: TableColumn<T>> {
    column: &'a C,
    /// padding width and direction
    padding: Option<(usize, PaddingDirection)>,
    _marker: PhantomData<T>,
}

impl<'a, T, C: TableColumn<T>> TodoColumnBasicMeta<'a, T, C> {
    pub fn new(column: &'a C, padding: Option<(usize, PaddingDirection)>) -> Self {
        Self {
            column,
            padding,
            _marker: PhantomData,
        }
    }

    pub fn stylize_cell(&self, data: &T, cell: String) -> String {
        let cell = match self.padding {
            Some((width, PaddingDirection::Left)) => format!("{:<width$}", cell, width = width),
            Some((width, PaddingDirection::Right)) => format!("{:>width$}", cell, width = width),
            _ => cell,
        };

        self.colorize_cell(data, cell)
    }

    fn colorize_cell(&self, data: &T, cell: String) -> String {
        match self.column.get_color(data) {
            Some(color) => cell.color(color).to_string(),
            _ => cell,
        }
    }
}

/// A simple JSON table style implementation, which formats the table as a JSON object of arrays.
#[derive(Debug, Clone)]
pub struct TableStyleJson;

impl TableStyleJson {
    pub fn new() -> Self {
        Self
    }
}

impl<'a, T, C: 'a + TableColumn<T>> TableStyle<'a, T, C> for TableStyleJson {
    type ColumnMeta = Cow<'a, str>;

    fn build<'b>(
        &self,
        columns: &'a Vec<C>,
        _data: &'a Vec<T>,
        _table: &'b Vec<Vec<String>>,
    ) -> Vec<Self::ColumnMeta> {
        columns.iter().map(|col| col.name()).collect()
    }

    fn table_starting(&self, _columns: &Vec<Self::ColumnMeta>) -> &str {
        "["
    }
    fn table_ending(&self, _columns: &Vec<Self::ColumnMeta>) -> &str {
        "]\n"
    }
    fn row_starting(&self, _data: &T) -> &str {
        "{"
    }
    fn row_ending(&self, _data: &T) -> &str {
        "}"
    }
    fn row_separator(&self) -> &str {
        ","
    }
    fn cell_stylize(&self, _data: &T, column: &Self::ColumnMeta, cell: String) -> String {
        // A simple JSON string escaper. This is not fully compliant.
        let escaped = cell
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");

        format!("\"{}\": \"{}\"", column, escaped)
    }
    fn cell_separator(&self) -> &str {
        ", "
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use colored::Color;

    /// Test data structure
    #[derive(Debug, Clone)]
    struct TestData {
        name: String,
        age: u32,
        active: bool,
    }

    /// Test column implementations
    #[derive(Debug)]
    struct NameColumn;
    #[derive(Debug)]
    struct AgeColumn;
    #[derive(Debug)]
    struct ActiveColumn;

    impl TableColumn<TestData> for NameColumn {
        fn name(&self) -> Cow<'_, str> {
            "Name".into()
        }
        fn format(&self, data: &TestData) -> String {
            data.name.clone()
        }
        fn padding_direction(&self, _data: &TestData) -> PaddingDirection {
            PaddingDirection::Left
        }
        fn get_color(&self, data: &TestData) -> Option<Color> {
            data.active.then_some(Color::Green)
        }
    }

    impl TableColumn<TestData> for AgeColumn {
        fn name(&self) -> Cow<'_, str> {
            "Age".into()
        }
        fn format(&self, data: &TestData) -> String {
            data.age.to_string()
        }
        fn padding_direction(&self, _data: &TestData) -> PaddingDirection {
            PaddingDirection::Right
        }
        fn get_color(&self, _data: &TestData) -> Option<Color> {
            None
        }
    }

    impl TableColumn<TestData> for ActiveColumn {
        fn name(&self) -> Cow<'_, str> {
            "Active".into()
        }
        fn format(&self, data: &TestData) -> String {
            if data.active { "Yes" } else { "No" }.to_string()
        }
        fn padding_direction(&self, _data: &TestData) -> PaddingDirection {
            PaddingDirection::Left
        }
        fn get_color(&self, data: &TestData) -> Option<Color> {
            if !data.active { Some(Color::Red) } else { None }
        }
    }

    type DynColumn = Box<dyn TableColumn<TestData>>;

    fn create_test_data() -> Vec<TestData> {
        vec![
            TestData {
                name: "Alice".to_string(),
                age: 30,
                active: true,
            },
            TestData {
                name: "Bob".to_string(),
                age: 25,
                active: false,
            },
            TestData {
                name: "Charlie".to_string(),
                age: 35,
                active: true,
            },
        ]
    }

    #[test]
    fn test_table_style_basic() {
        let data = create_test_data();
        let columns: Vec<DynColumn> = vec![
            Box::new(NameColumn),
            Box::new(AgeColumn),
            Box::new(ActiveColumn),
        ];
        let style = TableStyleBasic::new();
        let table = Table::new(style, &columns, &data);

        let mut output = Vec::new();
        table.write_to(&mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "\
\u{1b}[32mAlice  \u{1b}[0m 30 Yes
Bob     25 \u{1b}[31mNo\u{1b}[0m
\u{1b}[32mCharlie\u{1b}[0m 35 Yes
",
        );
    }

    #[test]
    fn test_table_style_basic_no_padding() {
        let data = create_test_data();
        let columns: Vec<DynColumn> = vec![
            Box::new(NameColumn),
            Box::new(AgeColumn),
            Box::new(ActiveColumn),
        ];
        let mut style = TableStyleBasic::new();
        style.padding = false;
        let table = Table::new(style, &columns, &data);

        let mut output = Vec::new();
        table.write_to(&mut output).unwrap();

        // Without padding, the output should be more compact
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "\
\u{1b}[32mAlice\u{1b}[0m 30 Yes
Bob 25 \u{1b}[31mNo\u{1b}[0m
\u{1b}[32mCharlie\u{1b}[0m 35 Yes
",
        );
    }

    #[test]
    fn test_table_style_json() {
        let data = create_test_data();
        let columns: Vec<DynColumn> = vec![
            Box::new(NameColumn),
            Box::new(AgeColumn),
            Box::new(ActiveColumn),
        ];
        let style = TableStyleJson::new();
        let table = Table::new(style, &columns, &data);

        let mut output = Vec::new();
        table.write_to(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        // Check JSON structure
        println!("{}", result);
        assert_eq!(
            result,
            format!(
                "[{},{},{}]\n",
                r#"{"Name":"Alice","Age":"30","Active":"Yes"}"#,
                r#"{"Name":"Bob","Age":"25","Active":"No"}"#,
                r#"{"Name":"Charlie","Age":"35","Active":"Yes"}"#
            )
        );
    }

    #[test]
    fn test_single_row_table() {
        let data = vec![TestData {
            name: "Single".to_string(),
            age: 42,
            active: true,
        }];
        let columns = vec![NameColumn];
        let style = TableStyleBasic::new();
        let table = Table::new(style, &columns, &data);

        let mut output = Vec::new();
        table.write_to(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert!(result.contains("Single"));
    }

    #[test]
    fn test_unicode_width() {
        let data = vec![
            TestData {
                name: "你好".to_string(),
                age: 25,
                active: true,
            },
            TestData {
                name: "🌟".to_string(),
                age: 30,
                active: false,
            },
        ];
        let columns = vec![NameColumn];
        let style = TableStyleBasic::new();
        let table = Table::new(style, &columns, &data);

        let mut output = Vec::new();
        table.write_to(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        // Should handle Unicode characters correctly
        assert!(result.contains("你好"));
        assert!(result.contains("🌟"));
    }

    #[test]
    fn test_json_escaping() {
        let data = vec![TestData {
            name: "Test\"Quote".to_string(),
            age: 25,
            active: true,
        }];
        let columns = vec![NameColumn];
        let style = TableStyleJson::new();
        let table = Table::new(style, &columns, &data);

        let mut output = Vec::new();
        table.write_to(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        // Check that quotes are properly escaped
        assert!(result.contains(r#""Name":"Test\"Quote""#));
    }

    #[test]
    fn test_get_column_max_width() {
        let table = vec![
            vec!["short".to_string(), "medium".to_string()],
            vec!["very long string".to_string(), "x".to_string()],
            vec!["".to_string(), "normal".to_string()],
        ];

        let widths = get_column_max_width(&table);
        assert_eq!(widths[0], 16); // "very long string"
        assert_eq!(widths[1], 6); // "medium"
    }
}
