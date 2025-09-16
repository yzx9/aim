// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroU32;

/// The unique identifier for a todo item, which can be either a UID or a short ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Id {
    /// The unique identifier for the todo item.
    Uid(String),

    /// Either a short identifier or a unique identifier.
    ShortIdOrUid(String),
}

impl Id {
    /// Weather the ID is a short ID or a UID.
    pub fn maybe_short_id(&self) -> Option<NonZeroU32> {
        match self {
            Id::ShortIdOrUid(id) => id.parse::<NonZeroU32>().ok(),
            Id::Uid(_) => None,
        }
    }

    /// Returns the ID as a string slice.
    pub fn as_uid(&self) -> &str {
        match self {
            Id::Uid(uid) => uid,
            Id::ShortIdOrUid(id) => id,
        }
    }
}

/// Kind of item, either an event or a todo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Kind {
    /// An event item.
    Event,

    /// A todo item.
    Todo,
}

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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Priority {
    /// No priority.
    #[default]
    #[serde(rename = "none", alias = "0")]
    #[cfg_attr(feature = "clap", clap(name = "none", alias = "0"))]
    None,

    /// Priority 1, highest priority.
    #[serde(rename = "1")]
    #[cfg_attr(feature = "clap", clap(name = "1", hide = true))]
    P1,

    /// Priority 2, high priority.
    #[serde(rename = "2", alias = "high")]
    #[cfg_attr(feature = "clap", clap(name = "high", alias = "2"))]
    P2,

    /// Priority 3.
    #[serde(rename = "3")]
    #[cfg_attr(feature = "clap", clap(name = "3", hide = true))]
    P3,

    /// Priority 4.
    #[serde(rename = "4")]
    #[cfg_attr(feature = "clap", clap(name = "4", hide = true))]
    P4,

    /// Priority 5, medium priority.
    #[serde(rename = "5", alias = "mid")]
    #[cfg_attr(feature = "clap", clap(name = "mid", alias = "5"))]
    P5,

    /// Priority 6.
    #[serde(rename = "6")]
    #[cfg_attr(feature = "clap", clap(name = "6", hide = true))]
    P6,

    /// Priority 7.
    #[serde(rename = "7")]
    #[cfg_attr(feature = "clap", clap(name = "7", hide = true))]
    P7,

    /// Priority 8, low priority.
    #[serde(rename = "8", alias = "low")]
    #[cfg_attr(feature = "clap", clap(name = "low", alias = "8"))]
    P8,

    /// Priority 9, lowest priority.
    #[serde(rename = "9")]
    #[cfg_attr(feature = "clap", clap(name = "9", hide = true))]
    P9,
}

macro_rules! priority_from_to_int {
    ($t:ty) => {
        impl From<$t> for Priority {
            fn from(value: $t) -> Self {
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

        impl From<Priority> for $t {
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
    };
}

priority_from_to_int!(i8);
priority_from_to_int!(i16);
priority_from_to_int!(i32);
priority_from_to_int!(i64);
priority_from_to_int!(u8);
priority_from_to_int!(u16);
priority_from_to_int!(u32);
priority_from_to_int!(u64);

impl<'de> serde::Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PriorityVisitor;

        impl<'de> serde::de::Visitor<'de> for PriorityVisitor {
            type Value = Priority;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter
                    .write_str(r#"a string of "none", "high", "mid", "low" or number from 0 to 9"#)
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    0..=9 => Ok(v.into()),
                    _ => Err(E::custom(format!("invalid priority: {v}"))),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "0" | "none" => Ok(Priority::None),
                    "1" => Ok(Priority::P1),
                    "2" | "high" => Ok(Priority::P2),
                    "3" => Ok(Priority::P3),
                    "4" => Ok(Priority::P4),
                    "5" | "mid" => Ok(Priority::P5),
                    "6" => Ok(Priority::P6),
                    "7" => Ok(Priority::P7),
                    "8" | "low" => Ok(Priority::P8),
                    "9" => Ok(Priority::P9),
                    _ => Err(E::custom(format!("invalid priority: {v}"))),
                }
            }
        }

        deserializer.deserialize_any(PriorityVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_maybe_short_id() {
        let id1 = Id::ShortIdOrUid("123".to_string());
        let id2 = Id::ShortIdOrUid("0".to_string());
        let id3 = Id::ShortIdOrUid("abc".to_string());
        let id4 = Id::Uid("unique-id".to_string());
        assert_eq!(id1.maybe_short_id(), NonZeroU32::new(123));
        assert_eq!(id2.maybe_short_id(), None);
        assert_eq!(id3.maybe_short_id(), None);
        assert_eq!(id4.maybe_short_id(), None);
    }

    #[test]
    fn test_sort_order_sql_keyword() {
        assert_eq!(SortOrder::Asc.sql_keyword(), "ASC");
        assert_eq!(SortOrder::Desc.sql_keyword(), "DESC");
    }

    #[test]
    fn test_priority_deserialize_all_variants() {
        for (input, expected) in [
            // integer number
            ("0", Priority::None),
            ("1", Priority::P1),
            ("2", Priority::P2),
            ("3", Priority::P3),
            ("4", Priority::P4),
            ("5", Priority::P5),
            ("6", Priority::P6),
            ("7", Priority::P7),
            ("8", Priority::P8),
            ("9", Priority::P9),
            // string number
            (r#""1""#, Priority::P1),
            (r#""2""#, Priority::P2),
            (r#""3""#, Priority::P3),
            (r#""4""#, Priority::P4),
            (r#""5""#, Priority::P5),
            (r#""6""#, Priority::P6),
            (r#""7""#, Priority::P7),
            (r#""8""#, Priority::P8),
            (r#""9""#, Priority::P9),
            // named strings
            (r#""none""#, Priority::None),
            (r#""high""#, Priority::P2),
            (r#""mid""#, Priority::P5),
            (r#""low""#, Priority::P8),
        ] {
            let actual: Priority = serde_json::from_str(input).unwrap();
            assert_eq!(actual, expected, "Failed on input: {input}");
        }
    }

    #[test]
    fn test_priority_deserialize_invalid_values() {
        for input in [
            r#""invalid""#,
            r#""urgent""#,
            r#""10""#,
            "10",
            "-1",
            r#""-1""#,
            "0.1",
            r#""0.1""#,
        ] {
            let result = serde_json::from_str::<Priority>(input);
            assert!(
                result.is_err(),
                "Expected error for input: {input}, but got Ok({:?})",
                result.ok()
            );
        }
    }
}
