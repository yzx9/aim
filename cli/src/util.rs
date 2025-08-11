// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aimcal_core::LooseDateTime;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, offset::LocalResult};
use clap::{Arg, ArgMatches, arg, value_parser};
use unicode_width::UnicodeWidthStr;

/// The output format for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ArgOutputFormat {
    Json,
    Table,
}

impl ArgOutputFormat {
    pub fn arg() -> Arg {
        arg!(--"output-format" <FORMAT> "Output format")
            .value_parser(value_parser!(ArgOutputFormat))
            .default_value("table")
    }

    pub fn from(matches: &ArgMatches) -> Self {
        matches
            .get_one("output-format")
            .copied()
            .unwrap_or(ArgOutputFormat::Table)
    }
}

pub fn parse_datetime(dt: &str) -> Result<Option<LooseDateTime>, &'static str> {
    if dt.is_empty() {
        Ok(None)
    } else if let Ok(dt) = NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M") {
        Ok(Some(match Local.from_local_datetime(&dt) {
            LocalResult::Single(dt) => dt.into(),
            LocalResult::Ambiguous(dt1, _) => {
                tracing::warn!(?dt, "ambiguous local time in local, picking earliest");
                dt1.into()
            }
            LocalResult::None => {
                tracing::warn!(?dt, "invalid local time in local, falling back to floating");
                dt.into()
            }
        }))
    } else if let Ok(time) = NaiveTime::parse_from_str(dt, "%H:%M") {
        // If the input is just a time, we assume it's today
        match Local::now().with_time(time) {
            LocalResult::Single(dt) => Ok(Some(dt.into())),
            LocalResult::Ambiguous(dt1, _) => {
                tracing::warn!(?dt, "ambiguous local time in local, picking earliest");
                Ok(Some(dt1.into()))
            }
            LocalResult::None => Err("Invalid local time"),
        }
    } else if let Ok(date) = NaiveDate::parse_from_str(dt, "%Y-%m-%d") {
        Ok(Some(date.into()))
    } else {
        Err("Invalid date format. Expected format: YYYY-MM-DD, HH:MM and YYYY-MM-DD HH:MM")
    }
}

pub fn format_datetime(t: LooseDateTime) -> String {
    match t {
        LooseDateTime::DateOnly(d) => d.format("%Y-%m-%d"),
        LooseDateTime::Floating(dt) => dt.format("%Y-%m-%d %H:%M"),
        LooseDateTime::Local(dt) => dt.format("%Y-%m-%d %H:%M"),
    }
    .to_string()
}

pub fn unicode_width_of_slice(s: &str, first_n_chars: usize) -> usize {
    if first_n_chars == 0 || s.is_empty() {
        0
    } else if let Some((idx, ch)) = s.char_indices().nth(first_n_chars - 1) {
        let byte_idx = idx + ch.len_utf8();
        s[..byte_idx].width()
    } else {
        s.width()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn test_unicode_width_ascii_only() {
        let s = "hello world";
        assert_eq!(unicode_width_of_slice(s, 100), 11);
        assert_eq!(unicode_width_of_slice(s, 5), 5);
        assert_eq!(unicode_width_of_slice(s, 0), 0);
    }

    #[test]
    fn test_unicode_width_mixed_english_chinese() {
        let s = "abc中文def";
        // "abc" + "中"
        assert_eq!(unicode_width_of_slice(s, 4), "abc中".width());
        // Full string
        assert_eq!(unicode_width_of_slice(s, 8), s.width());
        assert_eq!(unicode_width_of_slice(s, 9), s.width());
    }

    #[test]
    fn test_unicode_width_emoji() {
        let s = "a😀b";
        // "a😀" => 1 (a) + 2 (😀)
        assert_eq!(unicode_width_of_slice(s, 2), "a😀".width());
    }

    #[test]
    fn test_unicode_width_out_of_bounds_char_index() {
        let s = "hi";
        assert_eq!(unicode_width_of_slice(s, 10), s.width());
    }

    #[test]
    fn test_unicode_width_empty_string() {
        let s = "";
        assert_eq!(unicode_width_of_slice(s, 0), 0);
    }

    #[test]
    fn test_unicode_width_full_width_characters() {
        let s = "ＡＢＣ"; // Full-width Latin letters
        assert_eq!(unicode_width_of_slice(s, 2), "ＡＢ".width());
    }
}
