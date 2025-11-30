// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use chumsky::Parser;
use chumsky::extra::ParserExtra;
use chumsky::prelude::*;

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
pub fn value_boolean<'src, I, E>() -> impl Parser<'src, I, bool, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    // case-sensitive
    let t = just('T')
        .ignore_then(just('R'))
        .ignore_then(just('U'))
        .ignore_then(just('E'))
        .ignored()
        .to(true);

    let f = just('F')
        .ignore_then(just('A'))
        .ignore_then(just('L'))
        .ignore_then(just('S'))
        .ignore_then(just('E'))
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
{
    // case-sensitive
    let int = select! { c @ '0'..='9' => c }
        .repeated()
        .at_least(1)
        .at_most(10) // u32 max is 10 digits: 4_294_967_295
        .collect::<String>()
        .map(|str| str.parse::<u32>().unwrap()); // TODO: handle parse error

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
{
    value_duration().separated_by(just(',')).collect()
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

        for (src, expected) in [("TRUE", true), ("FALSE", false)] {
            assert_eq!(parse(src).unwrap(), expected);
        }

        let fail_cases = ["True", "False", "true", "false", "1", "0", "YES", "NO", ""];
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
