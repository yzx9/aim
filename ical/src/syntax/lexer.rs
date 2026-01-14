// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Lexer for iCalendar files as defined in RFC 5545

use std::fmt::{self, Display};

use logos::Logos;

use crate::string_storage::Span;

/// Tokenize iCalendar source code into a vector of `SpannedToken`
///
/// This is a simpler alternative to `lex_analysis` that returns a Vec directly,
/// which is useful for the new scanner-based parser.
#[must_use]
pub fn tokenize<'src>(src: &'src str) -> impl IntoIterator<Item = SpannedToken<'src>> {
    Token::lexer(src).spanned().map(|(tok, span)| match tok {
        Ok(tok) => SpannedToken(tok, Span::new(span.start, span.end)),
        Err(()) => SpannedToken(Token::Error, Span::new(span.start, span.end)),
    })
}

/// Token emitted by the iCalendar lexer
#[derive(PartialEq, Eq, Clone, Copy, Logos)]
#[logos(skip r#"\r\n[ \t]"#)] // skip folding
pub enum Token<'a> {
    /// Double Quote ("), decimal codepoint 22
    #[token(r#"""#)]
    DQuote,

    /// Comma (,), decimal codepoint 44
    #[token(",")]
    Comma,

    /// Colon (:), decimal codepoint 58
    #[token(":")]
    Colon,

    /// Semicolon (;), decimal codepoint 59
    #[token(";")]
    Semicolon,

    /// Equal sign (=), decimal codepoint 61
    #[token("=")]
    Equal,

    /// ASCII symbols: sequences of printable ASCII characters
    #[regex(r#"[\t !#$%&'()*+./<>?@\[\\\]\^`\{|\}~]+"#)]
    Symbol(&'a str),

    /// Carriage Return (\r, decimal codepoint 13) followed by Line Feed (\n, decimal codepoint 10)
    #[token("\r\n")]
    Newline,

    /// ASCII word characters: 0-9, A-Z, a-z, underscore
    #[regex("[0-9A-Za-z_-]+")]
    Word(&'a str),

    /// NON-US-ASCII  = UTF8-2 / UTF8-3 / UTF8-4
    ///    ; UTF8-2, UTF8-3, and UTF8-4 are defined in [RFC3629]
    #[regex(r#"[^\x00-\x7F]+"#)]
    UnicodeText(&'a str),

    /// Error token for lexing errors
    Error,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DQuote => write!(f, "DQuote"),
            Self::Comma => write!(f, "Comma"),
            Self::Colon => write!(f, "Colon"),
            Self::Semicolon => write!(f, "Semicolon"),
            Self::Equal => write!(f, "Equal"),
            Self::Symbol(s) => write!(f, "Symbol({s})"),
            Self::Newline => write!(f, "Newline"),
            Self::Word(s) => write!(f, "Word({s})"),
            Self::UnicodeText(s) => write!(f, "UnicodeText({s})"),
            Self::Error => write!(f, "Error"),
        }
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

/// A token with its associated span in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpannedToken<'src>(pub Token<'src>, pub Span);

impl Display for SpannedToken<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{:?}", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::indexing_slicing)]

    use super::Token::*;
    use super::*;

    #[test]
    fn tokenizes_ascii_range() {
        macro_rules! test_ascii_range {
            ($name:ident, $range:expr, $token:ident, $single_char:expr) => {
                for i in $range {
                    let c = u8::try_from(i).unwrap_or_default() as char;
                    let src = c.to_string();
                    let mut lexer = Token::lexer(&src);
                    assert_eq!(lexer.next(), Some(Ok(Token::$token(&src))), "U+{i:02X}",);
                    assert_eq!(lexer.next(), None, "U+{i:02X}");

                    let src2 = format!("{c}{c}");
                    let mut lexer = Token::lexer(&src2);
                    if $single_char {
                        // Ensure it does not match as part of a longer token
                        assert_eq!(lexer.next(), Some(Ok(Token::$token(&src))), "U+{i:02X}");
                        assert_eq!(lexer.next(), Some(Ok(Token::$token(&src))), "U+{i:02X}");
                    } else {
                        assert_eq!(lexer.next(), Some(Ok(Token::$token(&src2))), "U+{i:02X}");
                    }
                    assert_eq!(lexer.next(), None, "U+{i:02X}");
                }
            };
        }

        // Control characters (0x00-0x08, 0x0A-0x1F, 0x7F) are not explicitly tokenized
        // They will be matched as Error tokens
        // Symbol now matches multiple consecutive characters
        test_ascii_range!(test_ascii_chars_09_09, 0x09..=0x09, Symbol, false);
        test_ascii_range!(test_ascii_chars_20_21, 0x20..=0x21, Symbol, false);
        // 0x22 is Quote
        test_ascii_range!(test_ascii_chars_23_2b, 0x23..=0x2B, Symbol, false);
        // 0x2C is Comma
        test_ascii_range!(test_ascii_chars_2e_2f, 0x2E..=0x2F, Symbol, false);
        test_ascii_range!(test_ascii_chars_30_39, 0x30..=0x39, Word, false);
        // 0x3A is Colon
        // 0x3B is Semi
        test_ascii_range!(test_ascii_chars_3c_3c, 0x3C..=0x3C, Symbol, false);
        // 0x3D is Eq
        test_ascii_range!(test_ascii_chars_3e_40, 0x3E..=0x40, Symbol, false);
        test_ascii_range!(test_ascii_chars_41_5a, 0x41..=0x5A, Word, false);
        test_ascii_range!(test_ascii_chars_5b_5b, 0x5B..=0x5B, Symbol, false);
        // 0x5C is Backslash
        test_ascii_range!(test_ascii_chars_5d_5e, 0x5D..=0x5E, Symbol, false);
        // 0x5F is Underscore, part of word
        test_ascii_range!(test_ascii_chars_60_60, 0x60..=0x60, Symbol, false);
        test_ascii_range!(test_ascii_chars_61_7a, 0x61..=0x7A, Word, false);
        test_ascii_range!(test_ascii_chars_7b_7e, 0x7B..=0x7E, Symbol, false);
    }

    fn assert_tokenize(src: &str, expected: &[Token]) {
        let tokens: Vec<_> = Token::lexer(src).map(|t| t.unwrap()).collect();
        assert_eq!(tokens, expected);
    }

    /// Tokenize and include Error tokens (converts logos errors to `Token::Error`)
    fn tokenize_with_errors(src: &str) -> Vec<Token<'_>> {
        Token::lexer(src)
            .map(|t| match t {
                Ok(tok) => tok,
                Err(()) => Error,
            })
            .collect()
    }

    #[test]
    fn tokenizes_special_ascii_chars() {
        let src = r#";:=,"\_"#;
        let expected = [
            Semicolon,
            Colon,
            Equal,
            Comma,
            DQuote,
            Symbol(r"\"),
            Word("_"),
        ];
        assert_tokenize(src, &expected);
    }

    #[test]
    fn handles_line_folding() {
        // Line folding (CRLF + space/tab) is skipped by logos
        // The tokens should only include non-folded newlines
        let src = "WORD1\r\n WORD2\r\n\tWORD3\r\nWORD4";
        let expected = [
            Word("WORD1"),
            Word("WORD2"),
            Word("WORD3"),
            Newline,
            Word("WORD4"),
        ];
        assert_tokenize(src, &expected);
    }

    #[test]
    fn tokenizes_escape_sequences() {
        let src = r"\\\;\,\N\n\r";
        let expected = [
            Symbol(r"\\\"), // Three backslashes merged into one Symbol
            Semicolon,
            Symbol(r"\"),
            Comma,
            Symbol(r"\"),
            Word("N"),
            Symbol(r"\"),
            Word("n"),
            Symbol(r"\"),
            Word("r"),
        ];
        assert_tokenize(src, &expected);
    }

    #[test]
    fn tokenizes_words_and_unicode() {
        let src = "ABC_foo-123 456 你好🎉🎊Hello世界";
        let expected = [
            Word("ABC_foo-123"),
            Symbol(" "),
            Word("456"),
            Symbol(" "),
            UnicodeText("你好🎉🎊"),
            Word("Hello"),
            UnicodeText("世界"),
        ];
        assert_tokenize(src, &expected);
    }

    #[test]
    fn handles_quotes_and_folding() {
        let src = "SUMMARY:\"Test\" description\r\n with folding";
        let expected = [
            Word("SUMMARY"),
            Colon,
            DQuote,
            Word("Test"),
            DQuote,
            Symbol(" "),
            Word("description"),
            Word("with"),
            Symbol(" "),
            Word("folding"),
        ];
        assert_tokenize(src, &expected);
    }

    #[test]
    fn tokenizes_control_chars_as_error() {
        // Control characters (except HTAB) should produce Error tokens
        // CONTROL = %x00-08 / %x0A-1F / %x7F

        // Test NULL (0x00)
        let tokens = tokenize_with_errors("\x00");
        assert_eq!(tokens, vec![Error]);

        // Test Bell (0x07)
        let tokens = tokenize_with_errors("\x07");
        assert_eq!(tokens, vec![Error]);

        // Test Line Feed alone (0x0A) - not part of CRLF
        let tokens = tokenize_with_errors("\n");
        assert_eq!(tokens, vec![Error]);

        // Test Escape (0x1B)
        let tokens = tokenize_with_errors("\x1B");
        assert_eq!(tokens, vec![Error]);

        // Test DEL (0x7F)
        let tokens = tokenize_with_errors("\x7F");
        assert_eq!(tokens, vec![Error]);
    }

    #[test]
    fn tokenizes_mixed_valid_and_invalid_chars() {
        // Mix of valid tokens and control characters
        let tokens = tokenize_with_errors("WORD\x01WORD2");
        assert_eq!(tokens, vec![Word("WORD"), Error, Word("WORD2")]);
    }

    #[test]
    fn tokenizes_bare_cr_as_error() {
        // Bare CR (not followed by LF) should be an error
        let tokens = tokenize_with_errors("WORD1\rWORD2");
        assert_eq!(tokens, vec![Word("WORD1"), Error, Word("WORD2")]);
    }

    #[test]
    fn tokenizes_lf_without_cr_as_error() {
        // LF without preceding CR should be an error
        let tokens = tokenize_with_errors("WORD1\nWORD2");
        assert_eq!(tokens, vec![Word("WORD1"), Error, Word("WORD2")]);
    }

    #[test]
    fn tokenizes_multiple_consecutive_control_chars() {
        // Multiple consecutive control characters should each produce an Error
        let tokens = tokenize_with_errors("WORD\x01\x02\x03WORD2");
        assert_eq!(
            tokens,
            vec![Word("WORD"), Error, Error, Error, Word("WORD2")]
        );
    }

    #[test]
    fn tokenizes_valid_crlf_sequence() {
        // Valid CRLF sequence should produce Newline token
        let tokens = tokenize_with_errors("WORD1\r\nWORD2");
        assert_eq!(tokens, vec![Word("WORD1"), Newline, Word("WORD2")]);
    }

    #[test]
    fn tokenizes_multiple_crlf_sequences() {
        // Multiple consecutive CRLF sequences
        let tokens = tokenize_with_errors("WORD1\r\n\r\nWORD2");
        assert_eq!(tokens, vec![Word("WORD1"), Newline, Newline, Word("WORD2")]);
    }

    #[test]
    fn tokenizes_htab_as_symbol() {
        // HTAB (0x09) should be tokenized as Symbol, not Error
        let tokens = tokenize_with_errors("WORD1\tWORD2");
        assert_eq!(tokens, vec![Word("WORD1"), Symbol("\t"), Word("WORD2")]);
    }

    // Tests for realistic iCalendar scenarios
    #[test]
    fn lexer_tokenizes_complete_minimal_icalendar() {
        let src = "\
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp.//CalDAV Client//EN
BEGIN:VEVENT
UID:12345@example.com
DTSTAMP:20250101T120000Z
DTSTART:20250615T133000Z
DTEND:20250615T143000Z
SUMMARY:Team Meeting
END:VEVENT
END:VCALENDAR
";
        let tokens = tokenize_with_errors(src);

        // Verify key tokens are present
        assert!(tokens.contains(&Word("BEGIN")));
        assert!(tokens.contains(&Word("VCALENDAR")));
        assert!(tokens.contains(&Word("VERSION")));
        assert!(tokens.contains(&Word("VEVENT")));
        assert!(tokens.contains(&Word("SUMMARY")));
        assert!(tokens.contains(&Word("Team")));
        assert!(tokens.contains(&Word("Meeting")));
    }

    #[test]
    fn lexer_handles_multiline_unicode_description() {
        let src = "\
DESCRIPTION:Important meeting with team members from 中国🇨🇳 and Japan 🇯🇵\
\r\n to discuss Q1 2025 strategy and planning.\r\n Please prepare your reports.\
";
        let tokens = tokenize_with_errors(src);

        // Verify unicode tokens are properly recognized
        assert!(tokens.iter().any(|t| matches!(t, UnicodeText(_))));
        assert!(tokens.contains(&Word("DESCRIPTION")));
        assert!(tokens.contains(&Word("Important")));
    }

    #[test]
    fn lexer_returns_token_positions() {
        let src = "BEGIN:VCALENDAR";
        let tokens: Vec<_> = tokenize(src).into_iter().collect();

        // Check that we get proper span information
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].0, Word("BEGIN"));
        assert_eq!(tokens[0].1, Span::new(0, 5));
        assert_eq!(tokens[1].0, Colon);
        assert_eq!(tokens[1].1, Span::new(5, 6));
        assert_eq!(tokens[2].0, Word("VCALENDAR"));
        assert_eq!(tokens[2].1, Span::new(6, 15));
    }

    #[test]
    fn lexer_handles_complex_nested_components() {
        // Test a realistic calendar with nested VTIMEZONE and VEVENT
        let src = "\
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
BEGIN:VTIMEZONE
TZID:America/New_York
BEGIN:DAYLIGHT
DTSTART:20070311T020000
TZOFFSETFROM:-0500
TZOFFSETTO:-0400
END:DAYLIGHT
END:VTIMEZONE
BEGIN:VEVENT
DTSTART;TZID=America/New_York:20250101T090000
DURATION:PT1H
SUMMARY:Test Event with 参数
END:VEVENT
END:VCALENDAR
";
        let tokens = tokenize_with_errors(src);

        // Verify structure is recognized
        assert!(tokens.contains(&Word("VTIMEZONE")));
        assert!(tokens.contains(&Word("DAYLIGHT")));
        assert!(tokens.contains(&Word("VEVENT")));
        assert!(tokens.contains(&Word("TZID")));
    }

    #[test]
    fn lexer_handles_real_world_attendee_list() {
        // Test realistic attendee list with parameters
        let src = "\
BEGIN:VEVENT
UID:meeting123@example.com
DTSTART:20250115T140000Z
ATTENDEE;RSVP=TRUE;CUTYPE=INDIVIDUAL;PARTSTAT=NEEDS-ACTION:mailto:alice@example.com
ATTENDEE;RSVP=FALSE;CUTYPE=ROOM;PARTSTAT=ACCEPTED:mailto:conf-room@example.com
ATTENDEE;RSVP=TRUE;CUTYPE=GROUP;PARTSTAT=TENTATIVE:mailto:team@example.com
END:VEVENT
";
        let tokens = tokenize_with_errors(src);

        // Verify all attendees are tokenized
        assert!(tokens.contains(&Word("ATTENDEE")));
        // Check that parameters are recognized
        assert!(tokens.contains(&Word("RSVP")));
        assert!(tokens.contains(&Word("CUTYPE")));
        assert!(tokens.contains(&Word("PARTSTAT")));
    }
}
