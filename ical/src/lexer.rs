// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use logos::Logos;

#[derive(Debug, PartialEq, Eq, Clone, Copy, logos::Logos)]
pub enum Token<'a> {
    /// Regular "word" segments:
    ///   - Match alphanumeric characters, underscores, hyphens, etc.;
    ///   - Parser distinguishes NAME / VALUE later;
    ///   - Note to exclude syntax symbols like newlines, semicolons, colons, equals, commas.
    #[regex(r#"[^;:,\r\n\t ="]+"#)]
    Word(&'a str),

    // Delimiters
    //
    /// Semicolon (;)
    #[token(";")]
    Semi,

    /// Colon (:)
    #[token(":")]
    Colon,

    /// Equal sign (=)
    #[token("=")]
    Eq,

    /// Comma (,)
    #[token(",")]
    Comma,

    // Newlines, Carriage Return + Line Feed (\r\n)
    #[token("\r\n")]
    Newline,

    // Whitespace (mainly for folding judgment)
    //
    /// Space ( )
    #[token(" ")]
    Space,

    /// Tab (\t)
    #[token("\t")]
    Tab,

    /// Quoted strings (including quotes)
    ///   - Support for \" escape left to post-processing;
    ///   - Does not cross newlines;
    // FIXME: folding inside quoted strings?
    #[regex(r#""([^"\\\r\n]|\\.)*""#)]
    Quoted(&'a str),
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Word(s) => write!(f, "Word({s})"),
            Token::Semi => write!(f, "Semi"),
            Token::Colon => write!(f, "Colon"),
            Token::Eq => write!(f, "Eq"),
            Token::Comma => write!(f, "Comma"),
            Token::Newline => write!(f, "Newline"),
            Token::Space => write!(f, "Space"),
            Token::Tab => write!(f, "Tab"),
            Token::Quoted(s) => write!(f, "Quoted({s})"),
        }
    }
}

pub fn lex<'a>(src: &'a str) -> logos::Lexer<'a, Token<'a>> {
    Token::lexer(src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let src = "SUMMARY:Hello World";
        let mut lexer = lex(src);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Word("SUMMARY"));

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Colon);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Word("Hello"));
    }

    #[test]
    fn test_quoted_string() {
        let src = r#""Hello, World""#;
        let mut lexer = lex(src);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Quoted(r#""Hello, World""#));
    }

    #[test]
    fn test_whitespace() {
        let src = " \t";
        let mut lexer = lex(src);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Space);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Tab);
    }

    #[test]
    fn test_separators() {
        let src = ";:=,";
        let mut lexer = lex(src);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Semi);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Colon);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Eq);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Comma);
    }

    #[test]
    fn test_newlines() {
        let src = "\r\n\n";
        let mut lexer = lex(src);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Newline);

        let token = lexer.next().unwrap();
        assert_eq!(token, Err(())); // The second newline is not recognized (just \n)
    }

    #[test]
    fn test_word_parsing() {
        let src = "ABC-123";
        let mut lexer = lex(src);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Word("ABC-123"));
    }

    #[test]
    fn test_mod_operator() {
        // Test MOD as a word token in a context where it might appear
        // In iCalendar, MOD might appear in recurrence rules
        let src = "FREQ=MONTHLY;BYMONTHDAY=MOD";
        let mut lexer = lex(src);

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Word("FREQ"));

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Eq); // =

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Word("MONTHLY"));

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Semi); // ;

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Word("BYMONTHDAY"));

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Eq); // =

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token, Token::Word("MOD"));
    }
}
