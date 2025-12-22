// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use std::borrow::Cow;

use chumsky::Parser;
use chumsky::error::RichPattern;
use chumsky::extra::ParserExtra;
use chumsky::input::{Input, Stream};
use chumsky::label::LabelError;
use chumsky::prelude::*;
use chumsky::span::SimpleSpan;

use crate::syntax::SpannedSegments;
use crate::typed::parameter_types::ValueType;
use crate::typed::value_datetime::{value_utc_offset, values_date, values_date_time, values_time};
use crate::typed::value_numeric::{values_float, values_integer};
use crate::typed::value_text::values_text;
use crate::typed::{ValueDate, ValueDateTime, ValueText, ValueTime, ValueUtcOffset};

/// The properties in an iCalendar object are strongly typed.  The definition
/// of each property restricts the value to be one of the value data types, or
/// simply value types, defined in this section. The value type for a property
/// will either be specified implicitly as the default value type or will be
/// explicitly specified with the "VALUE" parameter.  If the value type of a
/// property is one of the alternate valid types, then it MUST be explicitly
/// specified with the "VALUE" parameter.
///
/// See RFC 5545 Section 3.3 for more details.
#[derive(Debug, Clone)]
pub enum Value<'src> {
    /// This value type is used to identify properties that contain a character
    /// encoding of inline binary data.  For example, an inline attachment of a
    /// document might be included in an iCalendar object.
    ///
    /// See RFC 5545 Section 3.3.1 for more details.
    Binary(SpannedSegments<'src>),

    /// This value type is used to identify properties that contain either a
    /// "TRUE" or "FALSE" Boolean value.
    ///
    /// See RFC 5545 Section 3.3.2 for more details.
    Boolean(bool),

    // TODO: 3.3.3. Calendar User Address
    //
    /// This value type is used to identify values that contain a calendar date.
    ///
    /// See RFC 5545 Section 3.3.4 for more details.
    Date(ValueDate),

    /// This value type is used to identify properties that contain a date with
    ///
    /// See RFC 5545 Section 3.3.5 for more details.
    DateTime(ValueDateTime),

    /// This value type is used to identify properties that contain a duration
    /// of time.
    ///
    /// See RFC 5545 Section 3.3.6 for more details.
    Duration(ValueDuration),

    /// This value type is used to identify properties that contain a real-
    /// number value.
    ///
    /// See RFC 5545 Section 3.3.7 for more details.
    Float(f64),

    /// This value type is used to identify properties that contain a signed
    /// integer value.
    ///
    /// See RFC 5545 Section 3.3.8 for more details.
    Integer(i32),

    // TODO: 3.3.9. Period of Time
    // TODO: 3.3.10. Recurrence Rule
    //
    /// This value type is used to identify values that contain human-readable
    /// text.
    ///
    /// See RFC 5545 Section 3.3.11 for more details.
    Text(ValueText<'src>),

    /// This value type is used to identify values that contain a time of day.
    Time(ValueTime),

    // TODO: 3.3.13. URI
    //
    /// This value type is used to identify properties that contain an offset
    /// from UTC to local time.
    ///
    /// See RFC 5545 Section 3.3.14 for more details.
    UtcOffset(ValueUtcOffset),
}

pub fn parse_values(
    kind: ValueType,
    value: SpannedSegments<'_>,
) -> Result<Vec<Value<'_>>, Vec<Rich<'_, char>>> {
    use ValueType::{
        Binary, Boolean, Date, DateTime, Duration, Float, Integer, Text, Time, UtcOffset,
    };

    match kind {
        Binary => {
            value_binary::<'_, _, extra::Err<_>>()
                .check(make_input(value.clone())) // PERF: avoid clone
                .into_result()?;
            Ok(vec![Value::Binary(value)])
        }

        Boolean => value_boolean::<'_, _, extra::Err<_>>()
            .map(|a| vec![Value::Boolean(a)])
            .parse(make_input(value))
            .into_result(),

        Date => values_date::<'_, _, extra::Err<_>>()
            .map(|a| a.into_iter().map(Value::Date).collect())
            .parse(make_input(value))
            .into_result(),

        DateTime => values_date_time::<'_, _, extra::Err<_>>()
            .map(|a| a.into_iter().map(Value::DateTime).collect())
            .parse(make_input(value))
            .into_result(),

        Duration => values_duration::<'_, _, extra::Err<_>>()
            .map(|a| a.into_iter().map(Value::Duration).collect())
            .parse(make_input(value))
            .into_result(),

        Float => values_float::<'_, _, extra::Err<_>>()
            .map(|a| a.into_iter().map(Value::Float).collect())
            .parse(make_input(value))
            .into_result(),

        Integer => values_integer::<'_, _, extra::Err<_>>()
            .map(|a| a.into_iter().map(Value::Integer).collect())
            .parse(make_input(value))
            .into_result(),

        Text => values_text::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone())) // PERF: avoid clone
            .into_result()
            .map(|texts| {
                texts
                    .into_iter()
                    .map(|a| Value::Text(a.build(&value)))
                    .collect()
            }),

        Time => values_time::<'_, _, extra::Err<_>>()
            .map(|a| a.into_iter().map(Value::Time).collect())
            .parse(make_input(value))
            .into_result(),

        UtcOffset => value_utc_offset::<'_, _, extra::Err<_>>()
            .map(|a| vec![Value::UtcOffset(a)])
            .parse(make_input(value))
            .into_result(),

        _ => unimplemented!("Parser for {kind} is not implemented"),
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueExpected {
    Date,
    F64,
    I32,
    U32,
}

impl From<ValueExpected> for RichPattern<'_, char> {
    fn from(expected: ValueExpected) -> Self {
        match expected {
            ValueExpected::Date => Self::Label(Cow::Borrowed("invalid date")),
            ValueExpected::F64 => Self::Label(Cow::Borrowed("f64 out of range")),
            ValueExpected::I32 => Self::Label(Cow::Borrowed("i32 out of range")),
            ValueExpected::U32 => Self::Label(Cow::Borrowed("u32 out of range")),
        }
    }
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// binary     = *(4b-char) [b-end]
/// ; A "BASE64" encoded character string, as defined by [RFC4648].
///
/// b-end      = (2b-char "==") / (3b-char "=")
///
/// b-char = ALPHA / DIGIT / "+" / "/"
/// ```
pub fn value_binary<'src, I, E>() -> impl Parser<'src, I, (), E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    // b-char = ALPHA / DIGIT / "+" / "/"
    let b_char = select! {
        'A'..='Z' => (),
        'a'..='z' => (),
        '0'..='9' => (),
        '+' => (),
        '/' => (),
    };

    // 4b-char
    let quartet = b_char.repeated().exactly(4).ignored();

    // b-end
    let two_eq = just('=').then_ignore(just('='));
    let one_eq = just('=');

    // b-end = (2b-char "==") / (3b-char "=")
    let b_end = b_char
        .repeated()
        .exactly(2)
        .ignored()
        .then_ignore(two_eq)
        .or(b_char.repeated().exactly(3).ignored().then_ignore(one_eq))
        .ignored();

    // *(4b-char) [b-end]
    quartet
        .repeated() // allow zero quartets
        .ignore_then(b_end.or_not())
        .ignored()
        .then_ignore(end())
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// boolean    = "TRUE" / "FALSE"
/// ```
///
/// Description:  These values are case-insensitive text.  No additional
///    content value encoding (i.e., BACKSLASH character encoding, see
///    Section 3.3.11) is defined for this value type.
pub fn value_boolean<'src, I, E>() -> impl Parser<'src, I, bool, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    // case-insensitive
    let t = choice((just('T'), just('t')))
        .ignore_then(choice((just('R'), just('r'))))
        .ignore_then(choice((just('U'), just('u'))))
        .ignore_then(choice((just('E'), just('e'))))
        .ignored()
        .to(true);

    let f = choice((just('F'), just('f')))
        .ignore_then(choice((just('A'), just('a'))))
        .ignore_then(choice((just('L'), just('l'))))
        .ignore_then(choice((just('S'), just('s'))))
        .ignore_then(choice((just('E'), just('e'))))
        .ignored()
        .to(false);

    choice((t, f))
}

/// Duration Value defined in RFC 5545 Section 3.3.6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueDuration {
    /// Date and Time Duration
    DateTime {
        /// Whether the duration is positive
        positive: bool,

        /// Day Duration
        day: u32,

        /// Hour Duration
        hour: u32,

        /// Minute Duration
        minute: u32,

        /// Second Duration
        second: u32,
    },

    /// Week Duration
    Week {
        /// Whether the duration is positive
        positive: bool,

        /// Week Duration
        week: u32,
    },
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// dur-value  = (["+"] / "-") "P" (dur-date / dur-time / dur-week)
///
/// dur-date   = dur-day [dur-time]
/// dur-time   = "T" (dur-hour / dur-minute / dur-second)
/// dur-week   = 1*DIGIT "W"
/// dur-hour   = 1*DIGIT "H" [dur-minute]
/// dur-minute = 1*DIGIT "M" [dur-second]
/// dur-second = 1*DIGIT "S"
/// dur-day    = 1*DIGIT "D"
/// ```
#[allow(clippy::doc_link_with_quotes)]
fn value_duration<'src, I, E>() -> impl Parser<'src, I, ValueDuration, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    // case-sensitive
    let int = select! { c @ '0'..='9' => c }
        .repeated()
        .at_least(1)
        .at_most(10) // u32 max is 10 digits: 4_294_967_295
        .collect::<String>()
        .try_map_with(|str, e| match lexical::parse_partial::<u32, _>(&str) {
            Ok((v, n)) if n == str.len() => Ok(v),
            Ok((_, n)) => Err(E::Error::expected_found(
                [ValueExpected::U32],
                Some(str.chars().nth(n).unwrap().into()), // SAFETY: since n < len
                e.span(),
            )),
            Err(_) => Err(E::Error::expected_found(
                [ValueExpected::U32],
                Some(str.chars().next().unwrap().into()), // SAFETY: since at least 1 digit
                e.span(),
            )),
        });

    let week = int.then_ignore(just('W'));

    let second = int.then_ignore(just('S'));
    let minute = int.then_ignore(just('M')).then(second.or_not());
    let hour = int.then_ignore(just('H')).then(minute.or_not());
    let time = just('T').ignore_then(hour);

    let day = int.then_ignore(just('D'));
    let date = day.then(time.or_not());

    let sign = select! { c @ ('+' | '-') => c }
        .or_not()
        .map(|sign| !matches!(sign, Some('-')));
    let prefix = sign.then_ignore(just('P'));
    choice((
        prefix.then(date).map(|(positive, (day, time))| {
            let hour = time.map_or(0, |t| t.0);
            let minute = time.and_then(|t| t.1).map_or(0, |m_s| m_s.0);
            let second = time.and_then(|t| t.1).and_then(|m_s| m_s.1).unwrap_or(0);
            ValueDuration::DateTime {
                positive,
                day,
                hour,
                minute,
                second,
            }
        }),
        prefix
            .then(time)
            .map(|(positive, (h, ms))| ValueDuration::DateTime {
                positive,
                day: 0,
                hour: h,
                minute: ms.map_or(0, |(m, _)| m),
                second: ms.map_or(0, |(_, s)| s.unwrap_or(0)),
            }),
        prefix
            .then(week)
            .map(|(positive, week)| ValueDuration::Week { positive, week }),
    ))
}

/// Duration multiple values parser.
///
/// If the property permits, multiple "duration" values are specified by a
/// COMMA-separated list of values.
pub fn values_duration<'src, I, E>() -> impl Parser<'src, I, Vec<ValueDuration>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    value_duration().separated_by(just(',')).collect()
}

fn make_input(segs: SpannedSegments<'_>) -> impl Input<'_, Token = char, Span = SimpleSpan> {
    let eoi = match (segs.segments.first(), segs.segments.last()) {
        (Some(first), Some(last)) => first.1.start..last.1.end,
        _ => 0..0,
    };
    Stream::from_iter(segs.into_spanned_chars()).map(eoi.into(), |(t, s)| (t, s.into()))
}

#[cfg(test)]
mod tests {
    use chumsky::input::Stream;

    use super::*;

    #[test]
    fn test_binary() {
        fn check(src: &str) -> Result<(), Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_binary::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }
        let success_cases = [
            // examples from RFC 5545 Section 3.1.3
            // Original text include a typo (ignore the padding): https://www.rfc-editor.org/errata/eid5602
            "VGhlIHF1aWNrIGJyb3duIGZveCBqdW1wcyBvdmVyIHRoZSBsYXp5IGRvZy4=",
            // examples from RFC 5545 Section 3.3.1
            "\
AAABAAEAEBAQAAEABAAoAQAAFgAAACgAAAAQAAAAIAAAAAEABAAA\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAgIAAAICAgADAwMAA////AAAA\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
AAAAAAAAAAAAAAAAAAAAAAMwAAAAAAABNEMQAAAAAAAkQgAAAAAAJEREQgAA\
ACECQ0QgEgAAQxQzM0E0AABERCRCREQAADRDJEJEQwAAAhA0QwEQAAAAAERE\
AAAAAAAAREQAAAAAAAAkQgAAAAAAAAMgAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
AAAAAAAAAAAA\
",
            // extra tests
            "TWFu",     // "Man"
            "QUJDREVG", // "ABCDEF"
            "AAAA",     // all zero bytes
            "+/9a",     // bytes with high bits set
            "ZgZg",     // "ff"
            "TQ==",     // "M"
            "TWE=",     // "Ma"
            "SGVsbG8=", // "Hello"
        ];
        for src in success_cases {
            assert!(check(src).is_ok(), "Parse {src} should succeed");
        }

        let fail_cases = [
            "VGhlIHF1aWNrIGJyb3duIGZveCBqdW1wcyBvdmVyIHRoZSBsYXp5IGRvZy4",
            "TQ===",   // invalid length
            "TWFu=",   // invalid length
            "TWFuA",   // invalid length
            "TWFu===", // invalid length
            "T@Fu",    // invalid character
        ];
        for src in fail_cases {
            assert!(check(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn test_boolean() {
        fn parse(src: &str) -> Result<bool, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_boolean::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        for (src, expected) in [
            ("TRUE", true),
            ("True", true),
            ("true", true),
            ("FALSE", false),
            ("False", false),
            ("false", false),
        ] {
            assert_eq!(parse(src).unwrap(), expected);
        }

        let fail_cases = [
            "True ", " FALSE", "T RUE", "FA LSE", "1", "0", "YES", "NO", "",
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn test_duration() {
        use ValueDuration::{DateTime, Week};

        fn parse(src: &str) -> Result<ValueDuration, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_duration::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        #[rustfmt::skip]
        let success_cases = [
            // examples from RFC 5545 Section 3.3.6
            ("P15DT5H0M20S", DateTime { positive: true, day: 15, hour: 5, minute: 0, second: 20 }),
            ("P2W", Week { positive: true, week: 2 }),
            // extra tests
            ("+P3W", Week { positive: true, week: 3 }),
            ("-P1W", Week { positive: false, week: 1 }),
            ("+P3DT4H5M6S",  DateTime { positive:  true, day: 3, hour:  4, minute:  5, second:  6 }),
            ("-PT10H11M12S", DateTime { positive: false, day: 0, hour: 10, minute: 11, second: 12 }),
        ];
        for (src, expected) in success_cases {
            assert_eq!(parse(src).unwrap(), expected);
        }

        let fail_cases = [
            "P",           // missing duration value
            "PT",          // missing time value
            "P3X",         // invalid designator
            "P-3W",        // invalid negative sign position
            "P3DT4H5M6",   // missing 'S' designator
            "3W",          // missing 'P' designator
            "P10H11M12S3", // missing 'T' designator
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }
}
