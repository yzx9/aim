// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Duration value type parser as defined in RFC 5545 Section 3.3.6.

use chumsky::extra::ParserExtra;
use chumsky::input::Input;
use chumsky::label::LabelError;
use chumsky::prelude::*;
use chumsky::span::SimpleSpan;

use crate::value::miscellaneous::ValueExpected;

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
pub fn value_duration<'src, I, E>() -> impl Parser<'src, I, ValueDuration, E>
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

    // Base parsers for each time component
    let second_val = int.then_ignore(just('S'));
    let minute_val = int.then_ignore(just('M'));
    let hour_val = int.then_ignore(just('H'));

    // dur-second = 1*DIGIT "S"
    let second_only = second_val.map(|s| (0, 0, s));

    // dur-minute = 1*DIGIT "M" [dur-second]
    let minute_with_second = minute_val
        .then(second_val.or_not())
        .map(|(m, s)| (0, m, s.unwrap_or(0)));

    // dur-hour = 1*DIGIT "H" [dur-minute]
    let hour_with_minute = hour_val
        .then(minute_val.then(second_val.or_not()).or_not())
        .map(|(h, opt_ms)| match opt_ms {
            Some((m, opt_s)) => (h, m, opt_s.unwrap_or(0)),
            None => (h, 0, 0),
        });

    // dur-time = "T" (dur-hour / dur-minute / dur-second)
    let time = just('T').ignore_then(choice((hour_with_minute, minute_with_second, second_only)));

    let day = int.then_ignore(just('D'));
    let date = day.then(time.or_not());

    let sign = select! { c @ ('+' | '-') => c }
        .or_not()
        .map(|sign| !matches!(sign, Some('-')));
    let prefix = sign.then_ignore(just('P'));
    choice((
        prefix.then(date).map(|(positive, (day, time))| {
            let (hour, minute, second) = time.unwrap_or((0, 0, 0));
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
            .map(|(positive, (h, m, s))| ValueDuration::DateTime {
                positive,
                day: 0,
                hour: h,
                minute: m,
                second: s,
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

#[cfg(test)]
mod tests {
    use chumsky::extra;
    use chumsky::input::Stream;

    use super::*;

    #[test]
    fn parses_duration() {
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
            ("P2W",  Week { positive: true,  week: 2 }),
            // extra tests
            ("+P3W", Week { positive: true,  week: 3 }),
            ("-P1W", Week { positive: false, week: 1 }),
            ("+P3DT4H5M6S",  DateTime { positive:  true, day: 3, hour:  4, minute:  5, second:  6 }),
            ("-PT10H11M12S", DateTime { positive: false, day: 0, hour: 10, minute: 11, second: 12 }),
            ("PT15M",        DateTime { positive: true,  day: 0, hour:  0, minute: 15, second:  0 }),
            ("PT30S",        DateTime { positive: true,  day: 0, hour:  0, minute:  0, second: 30 }),
            ("PT1H30M",      DateTime { positive: true,  day: 0, hour:  1, minute: 30, second:  0 }),
            ("-PT15M",       DateTime { positive: false, day: 0, hour:  0, minute: 15, second:  0 }),
        ];
        for (src, expected) in success_cases {
            assert_eq!(parse(src).unwrap(), expected, "Failed to parse: {src}");
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
