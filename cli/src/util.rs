// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use unicode_width::UnicodeWidthStr;

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
