// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

/// Sort order, either ascending or descending.
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    /// Ascending order.
    Asc,

    /// Descending order.
    Desc,
}

impl SortOrder {
    /// Converts to a string representation suitable for SQL queries.
    pub(crate) fn sql_keyword(&self) -> &str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

/// Pagination with a limit and an offset.
#[derive(Debug, Clone, Copy)]
pub struct Pager {
    /// The maximum number of items to return.
    pub limit: i64,

    /// The number of items to skip before starting to collect the result set.
    pub offset: i64,
}

impl From<(i64, i64)> for Pager {
    fn from((limit, offset): (i64, i64)) -> Self {
        Pager { limit, offset }
    }
}

/// Priority of a task or item, with values ranging from 1 to 9, and None for no priority.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Priority {
    /// No priority.
    #[default]
    #[cfg_attr(feature = "clap", clap(name = "none", alias = "0"))]
    #[cfg_attr(feature = "serde", serde(rename = "none", alias = "0"))]
    None,

    /// Priority 1, highest priority.
    #[cfg_attr(feature = "clap", clap(name = "1", hide = true))]
    #[cfg_attr(feature = "serde", serde(rename = "1"))]
    P1,

    /// Priority 2, high priority.
    #[cfg_attr(feature = "clap", clap(name = "high", alias = "2"))]
    #[cfg_attr(feature = "serde", serde(rename = "2", alias = "high"))]
    P2,

    /// Priority 3.
    #[cfg_attr(feature = "clap", clap(name = "3", hide = true))]
    #[cfg_attr(feature = "serde", serde(rename = "3"))]
    P3,

    /// Priority 4.
    #[cfg_attr(feature = "clap", clap(name = "4", hide = true))]
    #[cfg_attr(feature = "serde", serde(rename = "4"))]
    P4,

    /// Priority 5, medium priority.
    #[cfg_attr(feature = "clap", clap(name = "mid", alias = "5"))]
    #[cfg_attr(feature = "serde", serde(rename = "5", alias = "mid"))]
    P5,

    /// Priority 6.
    #[cfg_attr(feature = "clap", clap(name = "6", hide = true))]
    #[cfg_attr(feature = "serde", serde(rename = "6"))]
    P6,

    /// Priority 7.
    #[cfg_attr(feature = "clap", clap(name = "7", hide = true))]
    #[cfg_attr(feature = "serde", serde(rename = "7"))]
    P7,

    /// Priority 8, low priority.
    #[cfg_attr(feature = "clap", clap(name = "low", alias = "8"))]
    #[cfg_attr(feature = "serde", serde(rename = "8", alias = "low"))]
    P8,

    /// Priority 9, lowest priority.
    #[cfg_attr(feature = "clap", clap(name = "9", hide = true))]
    #[cfg_attr(feature = "serde", serde(rename = "9"))]
    P9,
}

impl From<u32> for Priority {
    fn from(value: u32) -> Self {
        match value {
            1 => Priority::P1,
            2 => Priority::P2,
            3 => Priority::P3,
            4 => Priority::P4,
            5 => Priority::P5,
            6 => Priority::P6,
            7 => Priority::P7,
            8 => Priority::P8,
            9 => Priority::P9,
            _ => Priority::None,
        }
    }
}

impl From<u8> for Priority {
    fn from(value: u8) -> Self {
        u32::from(value).into()
    }
}

impl From<Priority> for u8 {
    fn from(value: Priority) -> Self {
        match value {
            Priority::None => 0,
            Priority::P1 => 1,
            Priority::P2 => 2,
            Priority::P3 => 3,
            Priority::P4 => 4,
            Priority::P5 => 5,
            Priority::P6 => 6,
            Priority::P7 => 7,
            Priority::P8 => 8,
            Priority::P9 => 9,
        }
    }
}

impl From<Priority> for u32 {
    fn from(value: Priority) -> Self {
        u8::from(value).into()
    }
}
