// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::LooseDateTime;
use chrono::offset::LocalResult;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};
use clap::{Arg, ArgMatches, arg, value_parser};
use unicode_segmentation::UnicodeSegmentation;
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

pub fn arg_verbose() -> Arg {
    arg!(-v --verbose "Show more detailed information")
}

pub fn get_verbose(matches: &ArgMatches) -> bool {
    matches.get_flag("verbose")
}

pub fn parse_datetime(dt: &str) -> Result<Option<LooseDateTime>, &'static str> {
    if dt.is_empty() {
        Ok(None)
    } else if let Ok(dt) = NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M") {
        Ok(Some(local_from_datetime(dt)))
    } else if let Ok(time) = NaiveTime::parse_from_str(dt, "%H:%M") {
        // If the input is just a time, we assume it's today
        local_with_time(Local::now(), time).map(|a| Some(a.into()))
    } else if let Ok(date) = NaiveDate::parse_from_str(dt, "%Y-%m-%d") {
        Ok(Some(date.into()))
    } else {
        Err("Invalid date format. Expected format: YYYY-MM-DD, HH:MM and YYYY-MM-DD HH:MM")
    }
}

/// Parses a date range from two strings, where the first is the start date and the second is the end date.
///
/// NOTE: Don't assert that the start date is before the end date, as this function does not enforce that.
pub fn parse_datetime_range(
    start: &str,
    end: &str,
) -> Result<(Option<LooseDateTime>, Option<LooseDateTime>), &'static str> {
    let start = parse_datetime(start)?;

    if end.is_empty() {
        Ok((start, None))
    } else if let Ok(dt) = NaiveDateTime::parse_from_str(end, "%Y-%m-%d %H:%M") {
        Ok((start, Some(local_from_datetime(dt))))
    } else if let Ok(time) = NaiveTime::parse_from_str(end, "%H:%M") {
        // If the input is just a time, we assume it's the same/next day as start or today if start is None
        let end = match start {
            Some(LooseDateTime::DateOnly(date)) => NaiveDateTime::new(date, time).into(),
            Some(LooseDateTime::Floating(dt)) => {
                let delta = if dt.time() <= time {
                    TimeDelta::zero()
                } else {
                    TimeDelta::days(1)
                };
                NaiveDateTime::new(dt.date() + delta, time).into()
            }
            Some(LooseDateTime::Local(dt)) => {
                let delta = if dt.time() <= time {
                    TimeDelta::zero()
                } else {
                    TimeDelta::days(1)
                };
                (local_with_time(dt, time)? + delta).into()
            }
            None => local_with_time(Local::now(), time)?.into(),
        };
        Ok((start, Some(end)))
    } else if let Ok(date) = NaiveDate::parse_from_str(end, "%Y-%m-%d") {
        Ok((start, Some(date.into())))
    } else {
        Err("Invalid date format. Expected format: YYYY-MM-DD, HH:MM and YYYY-MM-DD HH:MM")
    }
}

pub fn parse_timedelta(
    timedelta: &str,
    now: DateTime<Local>,
) -> Result<LooseDateTime, Box<dyn Error>> {
    // Handle "tomorrow" keyword (tomorrow at 9:00 AM)
    if timedelta == "tomorrow" {
        let tomorrow = now.date_naive() + TimeDelta::days(1);
        let time = NaiveTime::from_hms_opt(9, 0, 0).ok_or("Failed to create time")?;
        let dt = NaiveDateTime::new(tomorrow, time);
        return Ok(local_from_datetime(dt));
    }

    // Try to parse as datetime
    if let Ok(dt) = NaiveDateTime::parse_from_str(timedelta, "%Y-%m-%d %H:%M") {
        return Ok(local_from_datetime(dt));
    }

    // Try to parse as time only
    if let Ok(time) = NaiveTime::parse_from_str(timedelta, "%H:%M") {
        let date = now.date_naive();

        // If the time has already passed today, use tomorrow
        let delta = if now.time() <= time {
            TimeDelta::zero()
        } else {
            TimeDelta::days(1)
        };
        let dt = NaiveDateTime::new(date, time) + delta;
        return Ok(local_from_datetime(dt));
    }

    Err(format!("Invalid timedelta format: {timedelta}").into())
}

fn local_from_datetime(dt: NaiveDateTime) -> LooseDateTime {
    match Local.from_local_datetime(&dt) {
        LocalResult::Single(dt) => dt.into(),
        LocalResult::Ambiguous(dt1, _) => {
            tracing::warn!(?dt, "ambiguous local time in local, picking earliest");
            dt1.into()
        }
        LocalResult::None => {
            tracing::warn!(?dt, "invalid local time in local, falling back to floating");
            dt.into()
        }
    }
}

fn local_with_time<Tz: TimeZone>(
    dt: DateTime<Tz>,
    time: NaiveTime,
) -> Result<DateTime<Local>, &'static str> {
    match dt.with_time(time) {
        LocalResult::Single(dt) => Ok(dt.with_timezone(&Local)),
        LocalResult::Ambiguous(dt1, _) => {
            tracing::warn!(?dt, "ambiguous local time in local, picking earliest");
            Ok(dt1.with_timezone(&Local))
        }
        LocalResult::None => Err("Invalid local time"),
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

/// Return the byte range of the grapheme cluster at index `g_idx` in `s`.
/// If out of bounds, returns None.
pub fn byte_range_of_grapheme_at(s: &str, g_idx: usize) -> Option<std::ops::Range<usize>> {
    for (i, (byte_start, g)) in s.grapheme_indices(true).enumerate() {
        if i == g_idx {
            let byte_end = byte_start + g.len();
            return Some(byte_start..byte_end);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_parse_datetime_empty() {
        assert_eq!(parse_datetime("").unwrap(), None);
    }

    #[test]
    fn test_parse_datetime_date_only() {
        let result = parse_datetime("2023-12-25").unwrap().unwrap();
        match result {
            LooseDateTime::DateOnly(date) => {
                assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
            }
            _ => panic!("Expected DateOnly variant"),
        }
    }

    #[test]
    fn test_parse_datetime_date_time() {
        let result = parse_datetime("2023-12-25 14:30").unwrap().unwrap();
        match result {
            LooseDateTime::Local(dt) => {
                assert_eq!(
                    dt.date_naive(),
                    NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()
                );
                assert_eq!(dt.time(), NaiveTime::from_hms_opt(14, 30, 0).unwrap());
            }
            _ => panic!("Expected Local variant"),
        }
    }

    #[test]
    fn test_parse_datetime_time_only() {
        let now = Local::now();
        let result = parse_datetime("14:30").unwrap().unwrap();
        match result {
            LooseDateTime::Local(dt) => {
                assert_eq!(dt.date_naive(), now.date_naive());
                assert_eq!(dt.time(), NaiveTime::from_hms_opt(14, 30, 0).unwrap());
            }
            _ => panic!("Expected Local variant"),
        }
    }

    #[test]
    fn test_parse_datetime_invalid() {
        assert!(parse_datetime("invalid").is_err());
        assert!(parse_datetime("25:00").is_err());
        assert!(parse_datetime("2023-13-01").is_err());
    }

    #[test]
    fn test_parse_datetime_range_both_empty() {
        let (start, end) = parse_datetime_range("", "").unwrap();
        assert_eq!(start, None);
        assert_eq!(end, None);
    }

    #[test]
    fn test_parse_datetime_range_start_only() {
        let (start, end) = parse_datetime_range("2023-12-25", "").unwrap();
        assert!(start.is_some());
        assert_eq!(end, None);
    }

    #[test]
    fn test_parse_datetime_range_both_dates() {
        let (start, end) = parse_datetime_range("2023-12-25", "2023-12-26").unwrap();
        assert!(start.is_some());
        assert!(end.is_some());
    }

    #[test]
    fn test_parse_datetime_range_date_and_time() {
        let (start, end) = parse_datetime_range("2023-12-25", "14:30").unwrap();
        assert!(start.is_some());
        assert!(end.is_some());

        match start.unwrap() {
            LooseDateTime::DateOnly(date) => {
                assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
            }
            _ => panic!("Expected DateOnly variant for start"),
        }

        match end.unwrap() {
            LooseDateTime::Floating(dt) => {
                assert_eq!(dt.date(), NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
                assert_eq!(dt.time(), NaiveTime::from_hms_opt(14, 30, 0).unwrap());
            }
            _ => panic!("Expected Floating variant for end"),
        }
    }

    #[test]
    fn test_parse_datetime_range_datetime_and_time() {
        let (start, end) = parse_datetime_range("2023-12-25 14:00", "14:30").unwrap();
        assert!(start.is_some());
        assert!(end.is_some());

        match start.unwrap() {
            LooseDateTime::Local(date) => {
                assert_eq!(
                    date.date_naive(),
                    NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()
                );
                assert_eq!(date.time(), NaiveTime::from_hms_opt(14, 00, 00).unwrap());
            }
            _ => panic!("Expected Local variant for start"),
        }

        match end.unwrap() {
            LooseDateTime::Local(dt) => {
                assert_eq!(
                    dt.date_naive(),
                    NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()
                );
                assert_eq!(dt.time(), NaiveTime::from_hms_opt(14, 30, 0).unwrap());
            }
            _ => panic!("Expected Local variant for end"),
        }
    }

    #[test]
    fn test_parse_datetime_range_datetime_and_earlier_time() {
        let (start, end) = parse_datetime_range("2023-12-25 14:00", "13:30").unwrap();
        assert!(start.is_some());
        assert!(end.is_some());

        match start.unwrap() {
            LooseDateTime::Local(date) => {
                assert_eq!(
                    date.date_naive(),
                    NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()
                );
                assert_eq!(date.time(), NaiveTime::from_hms_opt(14, 00, 00).unwrap());
            }
            _ => panic!("Expected Local variant for start"),
        }

        match end.unwrap() {
            LooseDateTime::Local(dt) => {
                assert_eq!(
                    dt.date_naive(),
                    NaiveDate::from_ymd_opt(2023, 12, 26).unwrap()
                );
                assert_eq!(dt.time(), NaiveTime::from_hms_opt(13, 30, 0).unwrap());
            }
            _ => panic!("Expected Local variant for end"),
        }
    }

    #[test]
    fn test_parse_delay_timedelta() {
        let now = Local.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();

        // Test "tomorrow"
        let expected_tomorrow = Local.with_ymd_and_hms(2025, 1, 2, 9, 0, 0).unwrap();
        let result = parse_timedelta("tomorrow", now).unwrap();
        match result {
            LooseDateTime::Local(dt) => assert_eq!(dt, expected_tomorrow),
            _ => panic!("Expected LooseDateTime::Local for 'tomorrow'"),
        }

        // Test "YYYY-MM-DD HH:MM"
        let expected_datetime = Local.with_ymd_and_hms(2025, 1, 3, 15, 0, 0).unwrap();
        let result = parse_timedelta("2025-01-3 15:00", now).unwrap();
        match result {
            LooseDateTime::Local(dt) => assert_eq!(dt, expected_datetime),
            _ => panic!("Expected LooseDateTime::Local for datetime string"),
        }

        // Test "HH:MM" (before now, should be tomorrow)
        let expected_time_before = Local.with_ymd_and_hms(2025, 1, 2, 10, 0, 0).unwrap();
        let result = parse_timedelta("10:00", now).unwrap();
        match result {
            LooseDateTime::Local(dt) => assert_eq!(dt, expected_time_before),
            _ => panic!("Expected LooseDateTime::Local for time before now"),
        }

        // Test "HH:MM" (after now, should be today)
        let expected_time_after = Local.with_ymd_and_hms(2025, 1, 1, 14, 0, 0).unwrap();
        let result = parse_timedelta("14:00", now).unwrap();
        match result {
            LooseDateTime::Local(dt) => assert_eq!(dt, expected_time_after),
            _ => panic!("Expected LooseDateTime::Local for time after now"),
        }

        // Test invalid format
        let result = parse_timedelta("invalid", now);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_datetime_date_only() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        let formatted = format_datetime(LooseDateTime::DateOnly(date));
        assert_eq!(formatted, "2023-12-25");
    }

    #[test]
    fn test_format_datetime_floating() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
            NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
        );
        let formatted = format_datetime(LooseDateTime::Floating(dt));
        assert_eq!(formatted, "2023-12-25 14:30");
    }

    #[test]
    fn test_format_datetime_local() {
        let dt: DateTime<Local> = Local::now();
        let formatted = format_datetime(LooseDateTime::Local(dt));
        // Just check that it produces a non-empty string with expected format
        assert!(formatted.contains("202") && formatted.contains(":"));
    }

    #[test]
    fn test_byte_range_ascii_basic() {
        let s = "hello";
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..1)); // 'h'
        assert_eq!(byte_range_of_grapheme_at(s, 4), Some(4..5)); // 'o'
        assert_eq!(byte_range_of_grapheme_at(s, 5), None); // out of bounds
    }

    #[test]
    fn test_byte_range_chinese_multibyte() {
        let s = "a中b";
        // UTF-8: 'a' = 1 byte, '中' = 3 bytes, 'b' = 1 byte
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..1)); // 'a'
        assert_eq!(byte_range_of_grapheme_at(s, 1), Some(1..4)); // '中'
        assert_eq!(byte_range_of_grapheme_at(s, 2), Some(4..5)); // 'b'
        assert_eq!(byte_range_of_grapheme_at(s, 3), None); // out of bounds
    }

    #[test]
    fn test_byte_range_emoji_with_skin_tone() {
        let s = "👍🏻a";
        // "👍🏻" is 1 grapheme cluster, composed of two code points (8 bytes)
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..8));
        assert_eq!(byte_range_of_grapheme_at(s, 1), Some(8..9)); // 'a'
    }

    #[test]
    fn test_byte_range_emoji_family_zwj() {
        let s = "👨‍👩‍👧"; // ZWJ sequence, treated as 1 grapheme cluster
        let len = s.len();
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..len));
        assert_eq!(byte_range_of_grapheme_at(s, 1), None); // out of bounds
    }

    #[test]
    fn test_byte_range_combining_mark() {
        // 'e' + combining acute accent = 1 grapheme cluster,
        // then 'b' UTF-8: 'e' (1 byte) + U+0301 (2 bytes) = 3 bytes total
        let s = "e\u{0301}b";
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..3));
        assert_eq!(byte_range_of_grapheme_at(s, 1), Some(3..4)); // 'b'
        assert_eq!(byte_range_of_grapheme_at(s, 2), None); // out of bounds
    }

    #[test]
    fn test_byte_range_empty_string() {
        let s = "";
        assert_eq!(byte_range_of_grapheme_at(s, 0), None);
        assert_eq!(byte_range_of_grapheme_at(s, 1), None);
    }
}
