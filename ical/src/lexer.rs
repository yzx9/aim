// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Lexer for iCalendar files as defined in RFC 5545

use std::fmt;
use std::ops::Range;

use chumsky::input::{Input, MapExtra, Stream, ValueInput};
use chumsky::{extra::ParserExtra, span::SimpleSpan};
use logos::Logos;

/// Create a lexer for iCalendar source code
#[must_use]
pub fn lex_analysis(src: &'_ str) -> impl ValueInput<'_, Token = Token<'_>, Span = SimpleSpan> {
    // Create a logos lexer over the source code
    let token_iter = Token::lexer(src)
        .spanned()
        // Convert logos errors into tokens. We want parsing to be recoverable and not fail at the lexing stage, so
        // we have a dedicated `Token::Error` variant that represents a token error that was previously encountered
        .map(|(tok, span)| match tok {
            // Convert logos' `Range<usize>` spans to our custom `Span` type, then to chumsky's `SimpleSpan`
            Ok(tok) => (tok, span.into()),
            Err(()) => (Token::Error, span.into()),
        });

    // Turn the token iterator into a stream that chumsky can use for things like backtracking
    Stream::from_iter(token_iter)
        // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
        // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
        .map((0..src.len()).into(), |(t, s): (_, _)| (t, s))
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

impl fmt::Display for Token<'_> {
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
        fmt::Display::fmt(self, f)
    }
}

/// A span representing a range in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start position of the span
    pub start: usize,
    /// End position of the span
    pub end: usize,
}

impl Span {
    /// Create a new span from start and end positions
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Convert to a standard range
    #[must_use]
    pub const fn into_range(self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<SimpleSpan<usize>> for Span {
    fn from(span: SimpleSpan<usize>) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<Span> for SimpleSpan<usize> {
    fn from(span: Span) -> Self {
        use chumsky::span::Span;
        SimpleSpan::new((), span.start..span.end)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A token with its associated span in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpannedToken<'src>(pub Token<'src>, pub Span);

impl<'src> SpannedToken<'src> {
    pub(crate) fn from_map_extra<'tokens, I, E>(
        token: Token<'src>,
        e: &mut MapExtra<'tokens, '_, I, E>,
    ) -> Self
    where
        'src: 'tokens,
        I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
        E: ParserExtra<'tokens, I>,
    {
        let span = e.span();
        SpannedToken(token, span.into())
    }
}

impl fmt::Display for SpannedToken<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{:?}", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
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

    fn tokenize(src: &str, expected: &[Token]) {
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
        tokenize(src, &expected);
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
        tokenize(src, &expected);
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
        tokenize(src, &expected);
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
        tokenize(src, &expected);
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
        tokenize(src, &expected);
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
}
