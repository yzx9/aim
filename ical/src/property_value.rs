// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use std::fmt::Display;
use std::str::FromStr;

use chumsky::extra::ParserExtra;
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use chumsky::{Parser, input::Stream};

use crate::keyword::{
    KW_BINARY, KW_BOOLEAN, KW_DATE, KW_DATETIME, KW_DURATION, KW_INTEGER, KW_PERIOD, KW_RRULE,
    KW_TEXT, KW_URI,
};
use crate::lexer::{SpannedToken, SpannedTokens, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyValueKind {
    Binary,
    Boolean,
    Text,
    Date,
    DateTime,
    Duration,
    Integer,
    Uri,
    Period,
    Rrule,
}

impl FromStr for PropertyValueKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            KW_BINARY => Ok(PropertyValueKind::Binary),
            KW_BOOLEAN => Ok(PropertyValueKind::Boolean),
            KW_TEXT => Ok(PropertyValueKind::Text),
            KW_DATE => Ok(PropertyValueKind::Date),
            KW_DATETIME => Ok(PropertyValueKind::DateTime),
            KW_DURATION => Ok(PropertyValueKind::Duration),
            KW_INTEGER => Ok(PropertyValueKind::Integer),
            KW_URI => Ok(PropertyValueKind::Uri),
            KW_PERIOD => Ok(PropertyValueKind::Period),
            KW_RRULE => Ok(PropertyValueKind::Rrule),
            _ => Err(()),
        }
    }
}

impl AsRef<str> for PropertyValueKind {
    fn as_ref(&self) -> &str {
        match self {
            PropertyValueKind::Binary => KW_BINARY,
            PropertyValueKind::Boolean => KW_BOOLEAN,
            PropertyValueKind::Text => KW_TEXT,
            PropertyValueKind::Date => KW_DATE,
            PropertyValueKind::DateTime => KW_DATETIME,
            PropertyValueKind::Duration => KW_DURATION,
            PropertyValueKind::Integer => KW_INTEGER,
            PropertyValueKind::Uri => KW_URI,
            PropertyValueKind::Period => KW_PERIOD,
            PropertyValueKind::Rrule => KW_RRULE,
        }
    }
}

impl Display for PropertyValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug, Clone)]
pub enum PropertyValue<'src> {
    Binary(SpannedTokens<'src>),
    Boolean(bool),
    Duration(PropertyValueDuration),
    Integer(i32),
    Text(SpannedTokens<'src>),
}

pub fn parse_property_value<'src, 'b: 'src>(
    kind: PropertyValueKind,
    tokens: SpannedTokens<'src>,
) -> Result<PropertyValue<'src>, Vec<Rich<'src, char>>> {
    use PropertyValueKind::{Binary, Boolean, Duration, Integer, Text};

    match kind {
        Binary => return Ok(PropertyValue::Binary(tokens.clone())),
        Text => return Ok(property_value_text(tokens.clone())),
        _ => {}
    }

    // TODO: this is slow than u8, and maybe buggy in span
    // TODO: map span
    let stream = Stream::from_iter(tokens.into_iter_chars());
    match kind {
        Boolean => property_value_boolean::<'_, _, extra::Err<_>>().parse(stream),
        Duration => property_value_duration::<'_, _, extra::Err<_>>().parse(stream),
        Integer => property_value_integer::<'_, _, extra::Err<_>>().parse(stream),
        _ => unimplemented!("Parser for {kind} is not implemented"),
    }
    .into_result()
}

// TODO: 3.3.1. Binary

/// 3.3.2. Boolean
fn property_value_boolean<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
where
    I: ValueInput<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    // case-sensitive
    let t = just('T')
        .ignore_then(just('R'))
        .ignore_then(just('U'))
        .ignore_then(just('E'))
        .ignored()
        .map(|_| PropertyValue::Boolean(true));

    let f = just('F')
        .ignore_then(just('A'))
        .ignore_then(just('L'))
        .ignore_then(just('S'))
        .ignore_then(just('E'))
        .ignored()
        .map(|_| PropertyValue::Boolean(false));

    choice((t, f))
}

// TODO: 3.3.3. Calendar User Address

// TODO: 3.3.4. Date

// TODO: 3.3.5. Date-Time

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyValueDuration {
    DateTime {
        positive: bool,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    },
    Week {
        positive: bool,
        week: u32,
    },
}

/// 3.3.6. Duration
fn property_value_duration<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
where
    I: ValueInput<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    // case-sensitive
    let int = one_of("0123456789")
        .repeated()
        .at_least(1)
        .at_most(10) // u32 max is 10 digits: 4_294_967_295
        .collect::<String>()
        .map(|str| str.parse::<u32>().unwrap()); // TODO: handle parse error

    let week = int
        .then_ignore(just('W'))
        .map(|week| PropertyValueDuration::Week {
            positive: false,
            week,
        });

    let second = int.then_ignore(just('S'));
    let minute = int.then_ignore(just('M')).then(second.or_not());
    let hour = int
        .then_ignore(just('H'))
        .then(minute.or_not())
        .map(|(h, ms)| PropertyValueDuration::DateTime {
            positive: true,
            day: 0,
            hour: h,
            minute: ms.map(|(m, _s)| m).unwrap_or(0),
            second: ms.map(|(_m, s)| s.unwrap_or(0)).unwrap_or(0),
        });
    let time = just('T').ignore_then(hour);

    let day = int.then_ignore(just('D'));
    let date = day.then(time.or_not()).map(|(day, time)| {
        let (hour, minute, second) = match time {
            Some(PropertyValueDuration::DateTime {
                hour,
                minute,
                second,
                ..
            }) => (hour, minute, second),
            None => (0, 0, 0),
            _ => unreachable!(),
        };
        PropertyValueDuration::DateTime {
            positive: false,
            day,
            hour,
            minute,
            second,
        }
    });

    let sign = one_of("+-").or_not();
    sign.then_ignore(just("P"))
        .then(choice((date, time, week)))
        .map(|(sign, dur)| {
            let positive = !matches!(sign, Some('-'));
            let v = match dur {
                // TODO: avoid clone
                PropertyValueDuration::Week { week, .. } => {
                    PropertyValueDuration::Week { positive, week }
                }
                PropertyValueDuration::DateTime {
                    day,
                    hour,
                    minute,
                    second,
                    ..
                } => PropertyValueDuration::DateTime {
                    positive,
                    day,
                    hour,
                    minute,
                    second,
                },
            };
            PropertyValue::Duration(v)
        })
}

// TODO: 3.3.7. Float
// FIXME: it only supports abc.xyz format, not scientific notation.

/// 3.3.8. Integer
fn property_value_integer<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
where
    I: ValueInput<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    // FIXME: RFC 5545 allows leading "+" sign, but logos lexer does not handle it well.
    let sign = one_of("+-").or_not();
    sign.then(
        one_of("0123456789")
            .repeated()
            .at_least(1)
            .at_most(10) // i32 max is 10 digits: 2_147_483_647
            .collect::<String>(),
    )
    .map(|(sign, digits)| {
        let i = digits.parse::<i32>().unwrap(); // TODO: handle parse error
        match sign {
            Some('-') => PropertyValue::Integer(-i),
            _ => PropertyValue::Integer(i),
        }
    })
}

// TODO: 3.3.9. Period of Time

// TODO: 3.3.10. Recurrence Rule

/// 3.3.11. Text
fn property_value_text<'src>(tokens: SpannedTokens<'src>) -> PropertyValue<'src> {
    let tokens = tokens
        .into_iter()
        .map(|t| match t {
            SpannedToken(Token::Escape(c), span) => {
                let s = match c {
                    r"\n" | r"\N" => "\n",
                    r"\;" => ";",
                    r"\," => ",",
                    r"\\" => "\\",
                    _ => unreachable!("Invalid escape sequence: {c}"),
                };
                SpannedToken(Token::Word(s), span)
            }
            other => other,
        })
        .collect();

    PropertyValue::Text(tokens)
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
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, char>>> {
            let stream = Stream::from_iter(src.chars());
            property_value_boolean::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        let src = "TRUE";
        let result = parse_bool(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        match val {
            PropertyValue::Boolean(b) => assert!(b),
            _ => panic!("Expected PropertyValue::Boolean"),
        }

        let src = "FALSE";
        let result = parse_bool(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        match val {
            PropertyValue::Boolean(b) => assert!(!b),
            _ => panic!("Expected PropertyValue::Boolean"),
        }
    }

    #[test]
    fn test_integer() {
        fn parse_integer<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, char>>> {
            let stream = Stream::from_iter(src.chars());
            property_value_integer::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        let src = "12345";
        let result = parse_integer(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        match val {
            PropertyValue::Integer(i) => assert_eq!(i, 12345),
            _ => panic!("Expected PropertyValue::Integer"),
        }

        let src = "-6789";
        let result = parse_integer(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        match val {
            PropertyValue::Integer(i) => assert_eq!(i, -6789),
            _ => panic!("Expected PropertyValue::Integer"),
        }
    }

    #[test]
    fn test_duration() {
        fn parse_duration<'src>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, char>>> {
            let stream = Stream::from_iter(src.chars());
            property_value_duration::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        let src = "P2W";
        let result = parse_duration(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        let expected = PropertyValueDuration::Week {
            positive: true,
            week: 2,
        };
        match val {
            PropertyValue::Duration(d) => assert_eq!(d, expected),
            _ => panic!("Expected PropertyValue::Duration::Week"),
        }

        let src = "+P3DT4H5M6S";
        let result = parse_duration(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        match val {
            PropertyValue::Duration(d) => assert_eq!(
                d,
                PropertyValueDuration::DateTime {
                    positive: true,
                    day: 3,
                    hour: 4,
                    minute: 5,
                    second: 6
                }
            ),
            _ => panic!("Expected PropertyValue::Duration::Date"),
        }

        let src = "-PT10H11M12S";
        let result = parse_duration(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let val = result.unwrap();
        let expected = PropertyValueDuration::DateTime {
            positive: false,
            day: 0,
            hour: 10,
            minute: 11,
            second: 12,
        };
        match val {
            PropertyValue::Duration(d) => assert_eq!(d, expected),
            _ => panic!("Expected PropertyValue::Duration::Time"),
        }
    }

    #[test]
    fn test_text() {
        fn parse_text(src: &'_ str) -> PropertyValue<'_> {
            let tokens = lex(src)
                .spanned()
                .map(|(token, span)| match token {
                    Ok(tok) => SpannedToken(tok, span),
                    Err(()) => panic!("lex error"),
                })
                .collect();

            property_value_text(tokens)
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
            match parse_text(src) {
                PropertyValue::Text(ref segments) => assert_eq!(segments.to_string(), expected),
                _ => panic!("Expected PropertyValue::Text"),
            }
        }
    }
}
