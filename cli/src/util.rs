// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{DateTimeAnchor, LooseDateTime};
use chrono::{DateTime, TimeZone};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// The output format for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
}

pub fn parse_datetime<Tz: TimeZone>(
    now: &DateTime<Tz>,
    anchor: &str,
) -> Result<Option<LooseDateTime>, Box<dyn Error>> {
    if anchor.is_empty() {
        Ok(None)
    } else {
        let anchor: DateTimeAnchor = anchor.parse()?;
        Ok(Some(anchor.resolve_since_datetime(now)))
    }
}

/// Parses a date range from two strings, where the first is the start date and the second is the end date.
///
/// NOTE: Don't assert that the start date is before the end date, as this function does not enforce that.
pub fn parse_datetime_range<Tz: TimeZone>(
    now: &DateTime<Tz>,
    start: &str,
    end: &str,
) -> Result<(Option<LooseDateTime>, Option<LooseDateTime>), Box<dyn Error>> {
    let start = parse_datetime(now, start)?;

    if end.is_empty() {
        Ok((start, None))
    } else {
        let anchor: DateTimeAnchor = end.parse()?;
        let end = match start {
            Some(s) => anchor.resolve_since(&s),
            None => anchor.resolve_since_datetime(now),
        };
        Ok((start, Some(end)))
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

/// The byte range of the grapheme cluster at index `g_idx` in `s`.
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
    use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime};

    use super::*;

    #[test]
    fn calculates_width_for_ascii_only() {
        let s = "hello world";
        assert_eq!(unicode_width_of_slice(s, 100), 11);
        assert_eq!(unicode_width_of_slice(s, 5), 5);
        assert_eq!(unicode_width_of_slice(s, 0), 0);
    }

    #[test]
    fn calculates_width_for_mixed_english_chinese() {
        let s = "abc中文def";
        // "abc" + "中"
        assert_eq!(unicode_width_of_slice(s, 4), "abc中".width());
        // Full string
        assert_eq!(unicode_width_of_slice(s, 8), s.width());
        assert_eq!(unicode_width_of_slice(s, 9), s.width());
    }

    #[test]
    fn calculates_width_for_emoji() {
        let s = "a😀b";
        // "a😀" => 1 (a) + 2 (😀)
        assert_eq!(unicode_width_of_slice(s, 2), "a😀".width());
    }

    #[test]
    fn calculates_width_for_out_of_bounds_char_index() {
        let s = "hi";
        assert_eq!(unicode_width_of_slice(s, 10), s.width());
    }

    #[test]
    fn calculates_width_for_empty_string() {
        let s = "";
        assert_eq!(unicode_width_of_slice(s, 0), 0);
    }

    #[test]
    fn calculates_width_for_full_width_characters() {
        let s = "ＡＢＣ"; // Full-width Latin letters
        assert_eq!(unicode_width_of_slice(s, 2), "ＡＢ".width());
    }

    fn default_datetime() -> DateTime<Local> {
        Local.from_utc_datetime(&NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        ))
    }

    #[test]
    fn parses_empty_datetime() {
        let now = default_datetime();
        assert_eq!(parse_datetime(&now, "").unwrap(), None);
    }

    #[test]
    fn parses_datetime_date_only() {
        let now = default_datetime();
        let result = parse_datetime(&now, "2023-12-25").unwrap().unwrap();
        match result {
            LooseDateTime::DateOnly(date) => {
                assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
            }
            _ => panic!("Expected DateOnly variant"),
        }
    }

    #[test]
    fn parses_datetime_date_and_time() {
        let now = default_datetime();
        let result = parse_datetime(&now, "2023-12-25 14:30").unwrap().unwrap();
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
    fn parses_datetime_time_only() {
        let now = default_datetime();
        let result = parse_datetime(&now, "20:30").unwrap().unwrap();
        match result {
            LooseDateTime::Local(dt) => {
                assert_eq!(dt.date_naive(), now.date_naive());
                assert_eq!(dt.time(), NaiveTime::from_hms_opt(20, 30, 0).unwrap());
            }
            _ => panic!("Expected Local variant"),
        }
    }

    #[test]
    fn returns_error_for_invalid_datetime() {
        let now = default_datetime();
        assert!(parse_datetime(&now, "invalid").is_err());
        assert!(parse_datetime(&now, "25:00").is_err());
        assert!(parse_datetime(&now, "2023-13-01").is_err());
    }

    #[test]
    fn parses_datetime_range_with_both_empty() {
        let now = default_datetime();
        let (start, end) = parse_datetime_range(&now, "", "").unwrap();
        assert_eq!(start, None);
        assert_eq!(end, None);
    }

    #[test]
    fn parses_datetime_range_with_start_only() {
        let now = default_datetime();
        let (start, end) = parse_datetime_range(&now, "2023-12-25", "").unwrap();
        assert!(start.is_some());
        assert_eq!(end, None);
    }

    #[test]
    fn parses_datetime_range_with_both_dates() {
        let now = default_datetime();
        let (start, end) = parse_datetime_range(&now, "2023-12-25", "2023-12-26").unwrap();
        assert!(start.is_some());
        assert!(end.is_some());
    }

    #[test]
    fn parses_datetime_range_with_date_and_time() {
        let now = default_datetime();
        let (start, end) = parse_datetime_range(&now, "2023-12-25", "14:30").unwrap();
        assert!(start.is_some());
        assert!(end.is_some());

        match start.unwrap() {
            LooseDateTime::DateOnly(date) => {
                assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
            }
            _ => panic!("Expected DateOnly variant for start"),
        }

        match end.unwrap() {
            LooseDateTime::Local(dt) => {
                assert_eq!(
                    dt.date_naive(),
                    NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()
                );
                assert_eq!(dt.time(), NaiveTime::from_hms_opt(14, 30, 0).unwrap());
            }
            _ => panic!("Expected Floating variant for end"),
        }
    }

    #[test]
    fn parses_datetime_range_with_datetime_and_time() {
        let now = default_datetime();
        let (start, end) = parse_datetime_range(&now, "2023-12-25 14:00", "14:30").unwrap();
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
    fn parses_datetime_range_with_datetime_and_earlier_time() {
        let now = default_datetime();
        let (start, end) = parse_datetime_range(&now, "2023-12-25 14:00", "13:30").unwrap();
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
    fn formats_datetime_date_only() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        let formatted = format_datetime(LooseDateTime::DateOnly(date));
        assert_eq!(formatted, "2023-12-25");
    }

    #[test]
    fn formats_datetime_floating() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
            NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
        );
        let formatted = format_datetime(LooseDateTime::Floating(dt));
        assert_eq!(formatted, "2023-12-25 14:30");
    }

    #[test]
    fn formats_datetime_local() {
        let dt = Local
            .from_local_datetime(&NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
                NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
            ))
            .unwrap();
        let formatted = format_datetime(LooseDateTime::Local(dt));
        assert_eq!(formatted, "2023-12-25 14:30");
    }

    #[test]
    fn finds_byte_range_for_ascii_basic() {
        let s = "hello";
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..1)); // 'h'
        assert_eq!(byte_range_of_grapheme_at(s, 4), Some(4..5)); // 'o'
        assert_eq!(byte_range_of_grapheme_at(s, 5), None); // out of bounds
    }

    #[test]
    fn finds_byte_range_for_chinese_multibyte() {
        let s = "a中b";
        // UTF-8: 'a' = 1 byte, '中' = 3 bytes, 'b' = 1 byte
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..1)); // 'a'
        assert_eq!(byte_range_of_grapheme_at(s, 1), Some(1..4)); // '中'
        assert_eq!(byte_range_of_grapheme_at(s, 2), Some(4..5)); // 'b'
        assert_eq!(byte_range_of_grapheme_at(s, 3), None); // out of bounds
    }

    #[test]
    fn finds_byte_range_for_emoji_with_skin_tone() {
        let s = "👍🏻a";
        // "👍🏻" is 1 grapheme cluster, composed of two code points (8 bytes)
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..8));
        assert_eq!(byte_range_of_grapheme_at(s, 1), Some(8..9)); // 'a'
    }

    #[test]
    fn finds_byte_range_for_emoji_family_zwj() {
        let s = "👨‍👩‍👧"; // ZWJ sequence, treated as 1 grapheme cluster
        let len = s.len();
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..len));
        assert_eq!(byte_range_of_grapheme_at(s, 1), None); // out of bounds
    }

    #[test]
    fn finds_byte_range_for_combining_mark() {
        // 'e' + combining acute accent = 1 grapheme cluster,
        // then 'b' UTF-8: 'e' (1 byte) + U+0301 (2 bytes) = 3 bytes total
        let s = "e\u{0301}b";
        assert_eq!(byte_range_of_grapheme_at(s, 0), Some(0..3));
        assert_eq!(byte_range_of_grapheme_at(s, 1), Some(3..4)); // 'b'
        assert_eq!(byte_range_of_grapheme_at(s, 2), None); // out of bounds
    }

    #[test]
    fn finds_byte_range_for_empty_string() {
        let s = "";
        assert_eq!(byte_range_of_grapheme_at(s, 0), None);
        assert_eq!(byte_range_of_grapheme_at(s, 1), None);
    }
}
