// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Lexer for iCalendar files as defined in RFC 5545

use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Range;

use chumsky::input::{Input, MapExtra, Stream, ValueInput};
use chumsky::{extra::ParserExtra, span::SimpleSpan};
use logos::Logos;

/// Create a lexer for iCalendar source code
pub fn lex_analysis(src: &'_ str) -> impl ValueInput<'_, Token = Token<'_>, Span = SimpleSpan> {
    // Create a logos lexer over the source code
    let token_iter = Token::lexer(src)
        .spanned()
        // Convert logos errors into tokens. We want parsing to be recoverable and not fail at the lexing stage, so
        // we have a dedicated `Token::Error` variant that represents a token error that was previously encountered
        .map(|(tok, span)| match tok {
            // Turn the `Range<usize>` spans logos gives us into chumsky's `SimpleSpan` via `Into`, because it's easier
            // to work with
            Ok(tok) => (tok, span.into()),
            Err(()) => (Token::Error, span.into()),
        });

    // Turn the token iterator into a stream that chumsky can use for things like backtracking
    Stream::from_iter(token_iter)
        // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
        // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
        .map((0..src.len()).into(), |(t, s): (_, _)| (t, s))
}

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

    /// CONTROL = %x00-08 / %x0A-1F / %x7F
    ///    ; All the controls except HTAB
    /// NOTE: Only matches single control characters to avoid conflict with `Folding`
    #[regex(r"[\x00-\x08\x0A-\x1F\x7F]")]
    Control(&'a str),

    /// ASCII symbols: sequences of printable ASCII characters excluding
    /// NOTE: only matches single symbol to avoid conflict with `Escape`
    #[regex(r#"[\t !#$%&'()*+./<>?@\[\\\]\^`\{|\}~]"#)]
    Symbol(&'a str),

    /// Carriage Return (\r, decimal codepoint 13) followed by Line Feed (\n, decimal codepoint 10)
    #[token("\r\n")]
    Newline,

    /// ESCAPED-CHAR = ("\\" / "\;" / "\," / "\N" / "\n")
    ///    ; \\ encodes \, \N or \n encodes newline
    ///    ; \; encodes ;, \, encodes ,
    #[regex(r"\\[\\;,Nn]")]
    Escape(&'a str),

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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DQuote => write!(f, "DQuote"),
            Self::Comma => write!(f, "Comma"),
            Self::Colon => write!(f, "Colon"),
            Self::Semicolon => write!(f, "Semicolon"),
            Self::Equal => write!(f, "Equal"),
            Self::Control(s) => match s.as_bytes().first() {
                Some(i) => write!(f, "Control(U+{i:02X})"),
                None => write!(f, "Control(<empty>)"),
            },
            Self::Symbol(s) => write!(f, "Symbol({s})"),
            Self::Newline => write!(f, "Newline"),
            Self::Escape(s) => write!(f, "Escape({s})"),
            Self::Word(s) => write!(f, "Word({s})"),
            Self::UnicodeText(s) => write!(f, "UnicodeText({s})"),
            Self::Error => write!(f, "Error"),
        }
    }
}

impl Debug for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

pub type Span = Range<usize>;

#[derive(Debug, Clone, PartialEq, Eq)]
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
        let range = span.start..span.end;
        SpannedToken(token, range)
    }
}

impl Display for SpannedToken<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{:?}", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::Token::*;
    use super::*;

    #[test]
    fn test_ascii() {
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

        test_ascii_range!(test_ascii_chars_00_08, 0x00..=0x08, Control, true);
        test_ascii_range!(test_ascii_chars_09_09, 0x09..=0x09, Symbol, true);
        test_ascii_range!(test_ascii_chars_0a_1f, 0x0A..=0x1F, Control, true);
        test_ascii_range!(test_ascii_chars_20_21, 0x20..=0x21, Symbol, true);
        // 0x22 is Quote
        test_ascii_range!(test_ascii_chars_23_2b, 0x23..=0x2B, Symbol, true);
        // 0x2C is Comma
        test_ascii_range!(test_ascii_chars_2e_2f, 0x2E..=0x2F, Symbol, true);
        test_ascii_range!(test_ascii_chars_30_39, 0x30..=0x39, Word, false);
        // 0x3A is Colon
        // 0x3B is Semi
        test_ascii_range!(test_ascii_chars_3c_3c, 0x3C..=0x3C, Symbol, true);
        // 0x3D is Eq
        test_ascii_range!(test_ascii_chars_3e_40, 0x3E..=0x40, Symbol, true);
        test_ascii_range!(test_ascii_chars_41_5a, 0x41..=0x5A, Word, false);
        test_ascii_range!(test_ascii_chars_5b_5b, 0x5B..=0x5B, Symbol, true);
        // 0x5C is Backslash, double backslash is Escape
        test_ascii_range!(test_ascii_chars_5d_5e, 0x5D..=0x5E, Symbol, true);
        // 0x5F is Underscore, part of word
        test_ascii_range!(test_ascii_chars_60_60, 0x60..=0x60, Symbol, true);
        test_ascii_range!(test_ascii_chars_61_7a, 0x61..=0x7A, Word, false);
        test_ascii_range!(test_ascii_chars_7b_7e, 0x7B..=0x7E, Symbol, true);
        test_ascii_range!(test_ascii_chars_7f_7f, 0x7F..=0x7F, Control, true);
    }

    fn test_tokenize(src: &str, expected: &[Token]) {
        let tokens: Vec<_> = Token::lexer(src).map(|t| t.unwrap()).collect();
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_ascii_special() {
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
        test_tokenize(src, &expected);
    }

    #[test]
    fn test_folding() {
        let src = "\r\n \r\n\t\r \n \r\n";
        let expected = [
            Control("\r"),
            Symbol(" "),
            Control("\n"),
            Symbol(" "),
            Newline,
        ];
        test_tokenize(src, &expected);
    }

    #[test]
    fn test_escape_characters() {
        let src = r"\\\;\,\N\n\r";
        let expected = [
            Escape(r"\\"),
            Escape(r"\;"),
            Escape(r"\,"),
            Escape(r"\N"),
            Escape(r"\n"),
            Symbol(r"\"),
            Word("r"),
        ];
        test_tokenize(src, &expected);
    }

    #[test]
    fn test_word_parsing() {
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
        test_tokenize(src, &expected);
    }

    #[test]
    fn test_mixed_quotes_and_folding() {
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
        test_tokenize(src, &expected);
    }
}
