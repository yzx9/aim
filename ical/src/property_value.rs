// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::sync::OnceLock;

use chumsky::prelude::*;
use chumsky::{Parser, input::ValueInput};
use regex::Regex;

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValue<'src> {
    Binary(&'src str),
    Boolean(bool),
    Duration(PropertyValueDuration),
    Integer(i32),
    Text(String), // TODO: zero-copy
}

pub fn property_value<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, PropertyValue<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    choice((
        value_boolean(),
        value_duration(),
        value_integer(),
        value_text(),
    ))
}

// TODO: 3.3.1. Binary

/// 3.3.2. Boolean
fn value_boolean<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, PropertyValue<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    // case-sensitive
    select! {
        Token::Word("TRUE") => "TRUE",
        Token::Word("FALSE") => "FALSE",
    }
    .map(|a| match a {
        "TRUE" => PropertyValue::Boolean(true),
        "FALSE" => PropertyValue::Boolean(false),
        _ => unreachable!(),
    })
}

// TODO: 3.3.3. Calendar User Address

// TODO: 3.3.4. Date

// TODO: 3.3.5. Date-Time

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyValueDuration {
    Week {
        negative: bool,
        week: u32,
    },
    DateTime {
        negative: bool,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    },
}

/// 3.3.6. Duration
fn value_duration<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, PropertyValue<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    // case-sensitive
    const RE: &str = r"^(?<sign>[+-]?)P((?<week>\d+)W|((?<day>\d+)D)?T((?<hour>\d)+H)?((?<minute>\d)+M)?((?<second>\d)+S))$";
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let re = REGEX.get_or_init(|| Regex::new(RE).unwrap());

    select! { Token::Word(s) => s }.try_map(|s, span| {
        let caps = re
            .captures(s)
            .ok_or(Rich::custom(span, "Invalid duration format"))?;

        let negative = matches!(caps.name("sign").map_or("", |m| m.as_str()), "-");

        let parse_group = |name| {
            caps.name(name).map_or(Ok(0), |m| {
                m.as_str()
                    .parse()
                    .map_err(|e| Rich::custom(span, format!("Invalid {name} value: {e}")))
            })
        };

        let dur = match caps.name("week") {
            Some(_) => PropertyValueDuration::Week {
                negative,
                week: parse_group("week")?,
            },
            None => PropertyValueDuration::DateTime {
                negative,
                day: parse_group("day")?,
                hour: parse_group("hour")?,
                minute: parse_group("minute")?,
                second: parse_group("second")?,
            },
        };
        Ok(PropertyValue::Duration(dur))
    })
}

// TODO: 3.3.7. Float

/// 3.3.8. Integer
fn value_integer<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, PropertyValue<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    // FIXME: RFC 5545 allows leading "+" sign, but logos lexer does not handle it well.
    // FIXME: Also, it only supports abc.xyz format, not scientific notation.
    select! { Token::Word(s) => s }
        .repeated() // folding can happen here
        .collect::<Vec<_>>()
        .try_map(|s, span| {
            s.concat()
                .parse::<i32>()
                .map(PropertyValue::Integer)
                .map_err(|e| Rich::custom(span, format!("Invalid integer: {e}")))
        })
}

// TODO: 3.3.9. Period of Time

// TODO: 3.3.10. Recurrence Rule

/// 3.3.11. Text
fn value_text<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, PropertyValue<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    select! {
        // Token::Semi => ";",
        Token::Colon => ":",
        Token::Eq => "=",
        // Token::Comma => ",",
        Token::Quote => "\"",
        // Token::Control(s) => s,
        Token::Symbol(s) if s != r"\" => s,
        Token::Escape(s) => match s {
            r"\n" | r"\N" => "\n",
            r"\;" => ";",
            r"\," => ",",
            r"\\" => "\\",
            _ => unreachable!("Invalid escape sequence: {s}"),
        },
        Token::Word(s) => s,
        Token::UnicodeText(s) => s,
    }
    .repeated()
    .at_least(1)
    .collect::<Vec<_>>()
    .map(|tokens| PropertyValue::Text(tokens.into_iter().collect::<String>()))
}

// TODO: 3.3.12. Time

// TODO: 3.3.13. URI

// TODO: 3.3.14. UTC Offset

#[cfg(test)]
mod tests {
    use chumsky::input::Stream;

    use crate::lexer::lex;

    use super::*;

    #[test]
    fn test_boolean() {
        fn parse_bool<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, Token<'tokens>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });
            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));
            value_boolean().parse(token_stream).into_result()
        }

        let src = "TRUE";
        let result = parse_bool(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        assert_eq!(val, PropertyValue::Boolean(true));

        let src = "FALSE";
        let result = parse_bool(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        assert_eq!(val, PropertyValue::Boolean(false));
    }

    #[test]
    fn test_integer() {
        fn parse_integer<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, Token<'tokens>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });
            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));
            value_integer().parse(token_stream).into_result()
        }

        let src = "12345";
        let result = parse_integer(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        assert_eq!(val, PropertyValue::Integer(12345));

        let src = "-6789";
        let result = parse_integer(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        assert_eq!(val, PropertyValue::Integer(-6789));
    }

    #[test]
    fn test_duration() {
        fn parse_duration<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, Token<'tokens>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });
            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));
            value_duration().parse(token_stream).into_result()
        }

        let src = "P2W";
        let result = parse_duration(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        assert_eq!(
            val,
            PropertyValue::Duration(PropertyValueDuration::Week {
                negative: false,
                week: 2
            })
        );

        let src = "-P3DT4H5M6S";
        let result = parse_duration(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        assert_eq!(
            val,
            PropertyValue::Duration(PropertyValueDuration::DateTime {
                negative: true,
                day: 3,
                hour: 4,
                minute: 5,
                second: 6
            })
        );
    }

    #[test]
    fn test_text() {
        fn parse_text<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, Token<'tokens>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });
            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));
            value_text().parse(token_stream).into_result()
        }

        for (src, expected) in [
            (r#"Hello\, World\; \N"#, "Hello, World; \n"),
            (
                r#""Quoted Text" and more text"#,
                r#""Quoted Text" and more text"#,
            ),
            ("Unicode 字符串 🎉", "Unicode 字符串 🎉"),
            ("123\r\n 456\r\n\t789", "123456789"),
        ] {
            let result = parse_text(src);
            assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
            let val = result.unwrap();
            assert_eq!(val, PropertyValue::Text(expected.to_string()));
        }
    }
}
