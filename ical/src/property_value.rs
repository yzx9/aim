// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chumsky::extra::ParserExtra;
use chumsky::prelude::*;
use chumsky::{Parser, input::Stream};

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PropertyValue<'src> {
    Binary(&'src str),
    Boolean(bool),
    Duration(PropertyValueDuration),
    Integer(i32),
    Text(String), // TODO: zero-copy
}

#[allow(dead_code)]
pub fn property_value<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, PropertyValue<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
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
fn value_boolean<'tokens, 'src: 'tokens, I, E>()
-> impl Parser<'tokens, I, PropertyValue<'src>, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
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
    Date {
        negative: bool,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    },
    Time {
        negative: bool,
        hour: u32,
        minute: u32,
        second: u32,
    },
    Week {
        negative: bool,
        week: u32,
    },
}

/// 3.3.6. Duration
fn value_duration<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, PropertyValue<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    // case-sensitive

    let int = any::<_, extra::Err<Rich<'tokens, char>>>()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .try_map(|chars, span| {
            String::from_iter(chars.iter())
                .parse::<u32>()
                .map_err(|e| Rich::custom(span, format!("Invalid integer: {e}")))
        });

    let week = int
        .then_ignore(just('W'))
        .map(|week| PropertyValueDuration::Week {
            negative: false,
            week,
        });

    let second = int.then_ignore(just('S'));
    let minute = int.then_ignore(just('M')).then(second.or_not());
    let hour = int
        .then_ignore(just('H'))
        .then(minute.or_not())
        .map(|(h, ms)| PropertyValueDuration::Time {
            negative: false,
            hour: h,
            minute: ms.map(|(m, _s)| m).unwrap_or(0),
            second: ms.map(|(_m, s)| s.unwrap_or(0)).unwrap_or(0),
        });
    let time = just('T').ignore_then(hour);

    let day = int.then_ignore(just('D'));
    let date = day.then(time.or_not()).map(|(day, time)| {
        let (hour, minute, second) = match time {
            Some(PropertyValueDuration::Time {
                hour,
                minute,
                second,
                ..
            }) => (hour, minute, second),
            None => (0, 0, 0),
            _ => unreachable!(),
        };
        PropertyValueDuration::Date {
            negative: false,
            day,
            hour,
            minute,
            second,
        }
    });

    let sign = one_of("+-").or_not();
    let dur = sign
        .then_ignore(just("P"))
        .then(choice((date, time, week)))
        .map(|(sign, dur)| {
            let negative = matches!(sign, Some('-'));
            match dur {
                PropertyValueDuration::Week { week, .. } => {
                    PropertyValueDuration::Week { negative, week }
                }
                PropertyValueDuration::Time {
                    hour,
                    minute,
                    second,
                    ..
                } => PropertyValueDuration::Time {
                    negative,
                    hour,
                    minute,
                    second,
                },
                PropertyValueDuration::Date {
                    day,
                    hour,
                    minute,
                    second,
                    ..
                } => PropertyValueDuration::Date {
                    negative,
                    day,
                    hour,
                    minute,
                    second,
                },
            }
        });

    select! { Token::Word(s) => s }
        .repeated()
        .collect::<Vec<_>>()
        .try_map(move |chunks: Vec<&'src str>, span| {
            let iter = chunks.into_iter().flat_map(|s| s.chars());
            let stream = Stream::from_iter(iter); // TODO: map span
            match dur.parse(stream).into_result() {
                Ok(v) => Ok(PropertyValue::Duration(v)),
                Err(_e) => Err(Rich::custom(span, "Invalid duration")), // TODO: e
            }
        })
}

// TODO: 3.3.7. Float

/// 3.3.8. Integer
fn value_integer<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, PropertyValue<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
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
fn value_text<'tokens, 'src: 'tokens, I, E>()
-> impl Parser<'tokens, I, PropertyValue<'src>, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
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
            value_boolean::<'_, '_, _, extra::Err<_>>()
                .parse(token_stream)
                .into_result()
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
            PropertyValue::Duration(PropertyValueDuration::Date {
                negative: true,
                day: 3,
                hour: 4,
                minute: 5,
                second: 6
            })
        );

        let src = "-PT10H11M12S";
        let result = parse_duration(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        assert_eq!(
            val,
            PropertyValue::Duration(PropertyValueDuration::Time {
                negative: true,
                hour: 10,
                minute: 11,
                second: 12
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
            value_text::<'_, '_, _, extra::Err<_>>()
                .parse(token_stream)
                .into_result()
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
