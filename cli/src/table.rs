// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use colored::{Color, Colorize};
use std::{
    borrow::Cow,
    fmt::{self, Display},
    marker::PhantomData,
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct Table<'a, T, C: TableColumn<T>, S: TableStyle<'a, T, C>> {
    columns: &'a [C],
    data: &'a [T],
    style: S,
}

impl<'a, T, C: TableColumn<T>, S: TableStyle<'a, T, C>> Table<'a, T, C, S> {
    pub fn new(style: S, columns: &'a [C], data: &'a [T]) -> Self {
        Self {
            columns,
            data,
            style,
        }
    }
}

impl<'a, T, C: TableColumn<T>, S: TableStyle<'a, T, C>> Display for Table<'a, T, C, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let table = self
            .data
            .iter()
            .map(|data| self.columns.iter().map(|col| col.format(data)).collect())
            .collect::<Vec<_>>();

        let columns = self.style.build(self.columns, &table);

        write!(f, "{}", self.style.table_starting(&columns))?;
        for (i, (cells, data)) in table.into_iter().zip(self.data).enumerate() {
            write!(f, "{}", self.style.row_starting(data))?;
            for (j, (col, cell)) in columns.iter().zip(cells.into_iter()).enumerate() {
                write!(f, "{}", self.style.cell_stylize(data, col, cell))?;

                if j < columns.len() - 1 {
                    write!(f, "{}", self.style.cell_separator())?;
                };
            }

            write!(f, "{}", self.style.row_ending(data))?;

            if i < self.data.len() - 1 {
                write!(f, "{}", self.style.row_separator())?;
            }
        }
        write!(f, "{}", self.style.table_ending(&columns))
    }
}

pub trait TableStyle<'a, T, C: TableColumn<T>> {
    type ColumnMeta;

    fn build<'b>(&self, columns: &'a [C], table: &'b [Vec<Cow<'a, str>>]) -> Vec<Self::ColumnMeta>;

    fn table_starting(&self, _columns: &[Self::ColumnMeta]) -> &str {
        ""
    }
    fn table_ending(&self, _columns: &[Self::ColumnMeta]) -> &str {
        ""
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
    fn cell_stylize(
        &self,
        _data: &'a T,
        _column: &Self::ColumnMeta,
        cell: Cow<'a, str>,
    ) -> Cow<'a, str> {
        cell
    }
    fn cell_separator(&self) -> &str {
        " "
    }
}

pub trait TableColumn<T> {
    /// Get the name of the column.
    fn name(&self) -> Cow<'_, str>;

    /// Format the data for the column.
    fn format<'a>(&self, data: &'a T) -> Cow<'a, str>;

    /// Determine the padding direction for the column.
    fn padding_direction(&self) -> PaddingDirection {
        PaddingDirection::Left
    }

    /// Get the color for the column based on the data.
    fn get_color(&self, _data: &T) -> Option<Color> {
        None
    }
}

impl<T, C: TableColumn<T> + ?Sized> TableColumn<T> for Box<C> {
    fn name(&self) -> Cow<'_, str> {
        self.as_ref().name()
    }
    fn format<'a>(&self, data: &'a T) -> Cow<'a, str> {
        self.as_ref().format(data)
    }
    fn padding_direction(&self) -> PaddingDirection {
        self.as_ref().padding_direction()
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
        columns: &'a [C],
        table: &'b [Vec<Cow<'a, str>>],
    ) -> Vec<TodoColumnBasicMeta<'a, T, C>> {
        let max_lengths = self.padding.then(|| get_column_max_width(table));
        columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let padding_direction = col.padding_direction();

                let padding = match &max_lengths {
                    Some(lengths)
                        // Last column does not need padding if it's left-aligned
                        if !(i == columns.len() - 1 && padding_direction == PaddingDirection::Left) =>
                            Some((*lengths.get(i).unwrap_or(&0), padding_direction)),
                    _ => None,
                };

                TodoColumnBasicMeta::new(col, padding)
            })
            .collect()
    }

    fn cell_stylize(
        &self,
        data: &'a T,
        column: &TodoColumnBasicMeta<'a, T, C>,
        cell: Cow<'a, str>,
    ) -> Cow<'a, str> {
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

    pub fn stylize_cell(&self, data: &T, cell: Cow<'a, str>) -> Cow<'a, str> {
        let cell = match self.padding {
            Some((width, PaddingDirection::Left)) => format!("{cell:<width$}").into(),
            Some((width, PaddingDirection::Right)) => format!("{cell:>width$}").into(),
            _ => cell,
        };

        self.column
            .get_color(data)
            .map(|color| cell.color(color).to_string().into())
            .unwrap_or(cell)
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
        columns: &'a [C],
        _table: &'b [Vec<Cow<'a, str>>],
    ) -> Vec<Self::ColumnMeta> {
        columns.iter().map(|col| col.name()).collect()
    }

    fn table_starting(&self, _columns: &[Self::ColumnMeta]) -> &str {
        "["
    }
    fn table_ending(&self, _columns: &[Self::ColumnMeta]) -> &str {
        "]"
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
    fn cell_stylize(
        &self,
        _data: &T,
        column: &Self::ColumnMeta,
        cell: Cow<'a, str>,
    ) -> Cow<'a, str> {
        // A simple JSON string escaper. This is not fully compliant.
        let escaped = cell
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");

        format!(r#""{column}":"{escaped}""#).into()
    }
    fn cell_separator(&self) -> &str {
        ","
    }
}

/// Computes the maximum display width for each column in a 2D table of strings.
fn get_column_max_width(table: &[Vec<Cow<'_, str>>]) -> Vec<usize> {
    let mut max_width = vec![0; table.first().map(Vec::len).unwrap_or(0)];
    for row in table {
        for (i, cell) in row.iter().enumerate() {
            let width = cell.width();
            if let Some(max_width) = max_width.get_mut(i) {
                if width > *max_width {
                    *max_width = width;
                }
            }
        }
    }
    max_width
}

#[cfg(test)]
mod tests {
    use super::*;
    use colored::Color;
    use std::sync::{Mutex, OnceLock};

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
        fn format<'a>(&self, data: &'a TestData) -> Cow<'a, str> {
            (&data.name).into()
        }
        fn get_color(&self, data: &TestData) -> Option<Color> {
            data.active.then_some(Color::Green)
        }
    }

    impl TableColumn<TestData> for AgeColumn {
        fn name(&self) -> Cow<'_, str> {
            "Age".into()
        }
        fn format<'a>(&self, data: &'a TestData) -> Cow<'a, str> {
            data.age.to_string().into()
        }
        fn padding_direction(&self) -> PaddingDirection {
            PaddingDirection::Right
        }
    }

    impl TableColumn<TestData> for ActiveColumn {
        fn name(&self) -> Cow<'_, str> {
            "Active".into()
        }
        fn format<'a>(&self, data: &'a TestData) -> Cow<'a, str> {
            if data.active { "Yes" } else { "No" }.into()
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

    static COLORED_CONTROL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn colored_control() -> &'static Mutex<()> {
        COLORED_CONTROL_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn test_table_style_basic_nocolor() {
        let data = create_test_data();
        let columns: Vec<DynColumn> = vec![
            Box::new(NameColumn),
            Box::new(AgeColumn),
            Box::new(ActiveColumn),
        ];
        let style = TableStyleBasic::new();
        let table = Table::new(style, &columns, &data);

        let result = {
            let _guard = colored_control().lock().unwrap();
            colored::control::set_override(false);
            table.to_string()
        };

        assert_eq!(
            result,
            "\
Alice   30 Yes
Bob     25 No
Charlie 35 Yes\
            ",
        );
    }

    #[test]
    fn test_table_style_basic_colorful() {
        let data = create_test_data();
        let columns: Vec<DynColumn> = vec![
            Box::new(NameColumn),
            Box::new(AgeColumn),
            Box::new(ActiveColumn),
        ];
        let style = TableStyleBasic::new();
        let table = Table::new(style, &columns, &data);

        let result = {
            let _guard = colored_control().lock().unwrap();
            colored::control::set_override(true);
            table.to_string()
        };

        assert_eq!(
            result,
            "\
\u{1b}[32mAlice  \u{1b}[0m 30 Yes
Bob     25 \u{1b}[31mNo\u{1b}[0m
\u{1b}[32mCharlie\u{1b}[0m 35 Yes\
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

        let result = {
            let _guard = colored_control().lock().unwrap();
            colored::control::set_override(false);
            table.to_string()
        };

        // Without padding, the output should be more compact
        assert_eq!(
            result,
            "\
Alice 30 Yes
Bob 25 No
Charlie 35 Yes\
",
        );
    }

    #[test]
    fn test_table_style_basic_empty() {
        let data: Vec<TestData> = vec![];
        let columns: Vec<DynColumn> = vec![
            Box::new(NameColumn),
            Box::new(AgeColumn),
            Box::new(ActiveColumn),
        ];
        let style = TableStyleBasic::new();
        let table = Table::new(style, &columns, &data);

        let result = {
            let _guard = colored_control().lock().unwrap();
            colored::control::set_override(false);
            table.to_string()
        };

        // Empty table should produce minimal output
        assert_eq!(result, "");
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

        // Check JSON structure
        assert_eq!(
            table.to_string(),
            format!(
                "[{},{},{}]",
                r#"{"Name":"Alice","Age":"30","Active":"Yes"}"#,
                r#"{"Name":"Bob","Age":"25","Active":"No"}"#,
                r#"{"Name":"Charlie","Age":"35","Active":"Yes"}"#
            )
        );
    }

    #[test]
    fn test_table_style_json_empty() {
        let data: Vec<TestData> = vec![];
        let columns: Vec<DynColumn> = vec![
            Box::new(NameColumn),
            Box::new(AgeColumn),
            Box::new(ActiveColumn),
        ];
        let style = TableStyleJson::new();
        let table = Table::new(style, &columns, &data);

        // Empty table should produce minimal output
        assert_eq!(table.to_string(), "[]");
    }

    #[test]
    fn test_single_row_table() {
        let data = vec![TestData {
            name: "Single".to_string(),
            age: 42,
            active: true,
        }];
        let columns = vec![NameColumn, NameColumn];
        let style = TableStyleBasic::new();
        let table = Table::new(style, &columns, &data);

        let result = {
            let _guard = colored_control().lock().unwrap();
            colored::control::set_override(false);
            table.to_string()
        };

        assert_eq!(result, "Single Single");
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

        // Should handle Unicode characters correctly
        let result = table.to_string();
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

        // Check that quotes are properly escaped
        assert_eq!(table.to_string(), r#"[{"Name":"Test\"Quote"}]"#);
    }

    #[test]
    fn test_get_column_max_width_empty() {
        let widths = get_column_max_width(&[]);
        assert_eq!(widths.len(), 0);
    }

    #[test]
    fn test_get_column_max_width_single_row() {
        let row = vec!["foo".into(), "long long long".into()];
        let table = vec![row];

        let widths = get_column_max_width(&table);
        assert_eq!(widths.first(), Some(&3)); // "foo"
        assert_eq!(widths.get(1), Some(&14)); // "long long long"
    }

    #[test]
    fn test_get_column_max_width_multi_rows() {
        let table = vec![
            vec!["short".into(), "medium".into()],
            vec!["very long string".into(), "x".into()],
            vec!["".into(), "normal".into()],
        ];

        let widths = get_column_max_width(&table);
        assert_eq!(widths.first(), Some(&16)); // "very long string"
        assert_eq!(widths.get(1), Some(&6)); // "medium"
    }
}
