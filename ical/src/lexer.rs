// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Lexer for iCalendar files as defined in RFC 5545

use std::fmt::{Debug, Display};

use logos::Logos;

pub fn lex<'a>(src: &'a str) -> logos::Lexer<'a, Token<'a>> {
    Token::lexer(src)
}

#[derive(PartialEq, Eq, Clone, Copy, logos::Logos)]
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

    /// Control characters: ASCII 0x00..0x1F and 0x7F
    /// NOTE: Only matches single control characters to avoid conflict with `Folding`
    #[regex(r"[\x00-\x1F\x7F]")]
    Control(&'a str),

    /// ASCII symbols: sequences of printable ASCII characters excluding symbols
    /// NOTE: only matches single symbol to avoid conflict with `Escape`
    #[regex(r#"[ !#$%&'()*+./<>?@\[\\\]\^`\{|\}~]"#)]
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
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        }
    }
}

impl Debug for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::Token::*;
    use super::*;

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
        let tokens: Vec<_> = lex(src).map(|t| t.unwrap()).collect();
        assert_eq!(tokens, expected);
    }

    macro_rules! test_ascii_range {
        ($name:ident, $from:expr, $to:expr, $token:ident, $single_char:expr) => {
            #[test]
            fn $name() {
                for i in $from..=$to {
                    let c = i as u8 as char;
                    let src = c.to_string();
                    let mut lexer = lex(&src);
                    assert_eq!(lexer.next(), Some(Ok(Token::$token(&src))), "U+{i:02X}",);
                    assert_eq!(lexer.next(), None, "U+{i:02X}");

                    let src2 = format!("{c}{c}");
                    let mut lexer = lex(&src2);
                    if $single_char {
                        // Ensure it does not match as part of a longer token
                        assert_eq!(lexer.next(), Some(Ok(Token::$token(&src))), "U+{i:02X}");
                        assert_eq!(lexer.next(), Some(Ok(Token::$token(&src))), "U+{i:02X}");
                    } else {
                        assert_eq!(lexer.next(), Some(Ok(Token::$token(&src2))), "U+{i:02X}");
                    }
                    assert_eq!(lexer.next(), None, "U+{i:02X}");
                }
            }
        };
    }

    test_ascii_range!(test_ascii_chars_00_1f, 0x00, 0x1F, Control, true);
    test_ascii_range!(test_ascii_chars_20_21, 0x20, 0x21, Symbol, true);
    // 0x22 is Quote
    test_ascii_range!(test_ascii_chars_23_2b, 0x23, 0x2B, Symbol, true);
    // 0x2C is Comma
    test_ascii_range!(test_ascii_chars_2e_2f, 0x2E, 0x2F, Symbol, true);
    test_ascii_range!(test_ascii_chars_30_39, 0x30, 0x39, Word, false);
    // 0x3A is Colon
    // 0x3B is Semi
    test_ascii_range!(test_ascii_chars_3c_3c, 0x3C, 0x3C, Symbol, true);
    // 0x3D is Eq
    test_ascii_range!(test_ascii_chars_3e_40, 0x3E, 0x40, Symbol, true);
    test_ascii_range!(test_ascii_chars_41_5a, 0x41, 0x5A, Word, false);
    test_ascii_range!(test_ascii_chars_5b_5b, 0x5B, 0x5B, Symbol, true);
    // 0x5C is Backslash, double backslash is Escape
    test_ascii_range!(test_ascii_chars_5d_5e, 0x5D, 0x5E, Symbol, true);
    // 0x5F is Underscore, part of word
    test_ascii_range!(test_ascii_chars_60_60, 0x60, 0x60, Symbol, true);
    test_ascii_range!(test_ascii_chars_61_7a, 0x61, 0x7A, Word, false);
    test_ascii_range!(test_ascii_chars_7b_7e, 0x7B, 0x7E, Symbol, true);
    test_ascii_range!(test_ascii_chars_7f_7f, 0x7F, 0x7F, Control, true);

    #[test]
    fn test_ascii_chars_special() {
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
        let tokens: Vec<_> = lex(src).map(|t| t.unwrap()).collect();
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_escape_characters() {
        let src = r#"\\\;\,\N\n\r"#;
        let expected = [
            Escape(r"\\"),
            Escape(r"\;"),
            Escape(r"\,"),
            Escape(r"\N"),
            Escape(r"\n"),
            Symbol(r"\"),
            Word("r"),
        ];
        let tokens: Vec<_> = lex(src).map(|t| t.unwrap()).collect();
        assert_eq!(tokens, expected);
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
        let tokens: Vec<_> = lex(src).map(|t| t.unwrap()).collect();
        assert_eq!(tokens, expected);
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
        let tokens: Vec<_> = lex(src).map(|t| t.unwrap()).collect();
        assert_eq!(tokens, expected);
    }
}
