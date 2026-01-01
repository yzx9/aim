// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Period value type parser as defined in RFC 5545 Section 3.3.9.

use chumsky::extra::ParserExtra;
use chumsky::input::Input;
use chumsky::label::LabelError;
use chumsky::prelude::*;
use chumsky::span::SimpleSpan;

use crate::value::ast::ValueExpected;
use crate::value::datetime::{ValueDateTime, value_date_time};
use crate::value::duration::{ValueDuration, value_duration};

/// Period of Time value defined in RFC 5545 Section 3.3.9
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValuePeriod {
    /// Explicit period with start and end date-time
    ///
    /// Format: `date-time "/" date-time`
    Explicit {
        /// Start date-time
        start: ValueDateTime,

        /// End date-time
        end: ValueDateTime,
    },

    /// Period with start date-time and duration
    ///
    /// Format: `date-time "/" dur-value`
    Duration {
        /// Start date-time
        start: ValueDateTime,

        /// Duration
        duration: ValueDuration,
    },
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// period     = period-explicit / period-start
///
/// period-explicit = date-time "/" date-time
/// ; [ISO.8601.2004] complete representation basic format for a
/// ; period of time consisting of a start and end.  The start MUST
/// ; be before the end.
///
/// period-start = date-time "/" dur-value
/// ; [ISO.8601.2004] complete representation basic format for a
/// ; period of time consisting of a start and positive duration
/// ; of time.
/// ```
pub fn value_period<'src, I, E>() -> impl Parser<'src, I, ValuePeriod, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    // period-explicit = date-time "/" date-time
    // Both date-times must have the same UTC flag (both UTC or both floating)
    let explicit = value_date_time()
        .then_ignore(just('/'))
        .then(value_date_time())
        .try_map(|(start, end), span| {
            // Validate that both date-times have the same UTC flag
            if start.time.utc == end.time.utc {
                Ok(ValuePeriod::Explicit { start, end })
            } else {
                Err(E::Error::expected_found(
                    [ValueExpected::MismatchedTimezone],
                    None,
                    span,
                ))
            }
        });

    // period-start = date-time "/" dur-value
    let start = value_date_time()
        .then_ignore(just('/'))
        .then(value_duration())
        .map(|(start, duration)| ValuePeriod::Duration { start, duration });

    choice((explicit, start))
}

/// Period multiple values parser.
///
/// If the property permits, multiple "period" values are specified by a
/// COMMA-separated list of values.
pub fn values_period<'src, I, E>() -> impl Parser<'src, I, Vec<ValuePeriod>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    value_period().separated_by(just(',')).collect()
}

#[cfg(test)]
mod tests {
    use chumsky::extra;
    use chumsky::input::Stream;

    use crate::value::{ValueDate, ValueDuration, ValueTime};

    use super::*;

    #[test]
    fn parses_period() {
        use ValueDuration::{DateTime, Week};

        fn parse(src: &str) -> Result<ValuePeriod, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_period::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        // Test explicit periods (start/end)
        {
            #[rustfmt::skip]
            let test_cases = [
                ("19970101T180000Z/19970102T070000Z",
                 (ValueDate { year: 1997, month: 1, day: 1 },
                  ValueTime::new(18, 0, 0, true),
                  ValueDate { year: 1997, month: 1, day: 2 },
                  ValueTime::new(7, 0, 0, true))),
                ("20240101T000000Z/20240101T235959Z",
                 (ValueDate { year: 2024, month: 1, day: 1 },
                  ValueTime::new(0, 0, 0, true),
                  ValueDate { year: 2024, month: 1, day: 1 },
                  ValueTime::new(23, 59, 59, true))),
            ];

            for (src, (start_date, start_time, end_date, end_time)) in test_cases {
                let result = parse(src).unwrap();
                match result {
                    ValuePeriod::Explicit { ref start, ref end } => {
                        assert_eq!(start.date, start_date, "Failed start date for {src}");
                        assert_eq!(start.time, start_time, "Failed start time for {src}");
                        assert_eq!(end.date, end_date, "Failed end date for {src}");
                        assert_eq!(end.time, end_time, "Failed end time for {src}");
                    }
                    ValuePeriod::Duration { .. } => panic!("Expected Explicit period for {src}"),
                }
            }
        }

        // Test duration periods (start + duration)
        {
            #[rustfmt::skip]
            let test_cases: [(&str, (ValueDate, ValueTime, ValueDuration)); 4] = [
                ("19970101T180000Z/PT5H30M",
                 (ValueDate { year: 1997, month: 1, day: 1 },
                  ValueTime::new(18, 0, 0, true),
                  DateTime { positive: true, day: 0, hour: 5, minute: 30, second: 0 })),
                ("19970101T180000Z/P1D",
                 (ValueDate { year: 1997, month: 1, day: 1 },
                  ValueTime::new(18, 0, 0, true),
                  DateTime { positive: true, day: 1, hour: 0, minute: 0, second: 0 })),
                ("20240101T120000/PT2H30M",
                 (ValueDate { year: 2024, month: 1, day: 1 },
                  ValueTime::new(12, 0, 0, false),
                  DateTime { positive: true, day: 0, hour: 2, minute: 30, second: 0 })),
                ("20240101T000000Z/P2W",
                 (ValueDate { year: 2024, month: 1, day: 1 },
                  ValueTime::new(0, 0, 0, true),
                  Week { positive: true, week: 2 })),
            ];

            for (src, (start_date, start_time, duration)) in test_cases {
                let result = parse(src).unwrap();
                match result {
                    ValuePeriod::Duration {
                        ref start,
                        duration: d,
                    } => {
                        assert_eq!(start.date, start_date, "Failed start date for {src}");
                        assert_eq!(start.time, start_time, "Failed start time for {src}");
                        assert_eq!(d, duration, "Failed duration for {src}");
                    }
                    ValuePeriod::Explicit { .. } => panic!("Expected Duration period for {src}"),
                }
            }
        }

        let fail_cases = [
            "",                                 // empty string
            "19970101T180000Z",                 // missing / and duration/end
            "/19970102T070000Z",                // missing start
            "19970101T180000Z/",                // missing end/duration
            "19970101T180000Z/P",               // invalid duration
            "invalid/19970102T070000Z",         // invalid start
            "19970101T180000Z/invalid",         // invalid end
            "19970101T180000Z/19970102T070000", // mixed UTC and non-UTC
            "19970101T180000/19970102T070000Z", // mixed non-UTC and UTC
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn parses_periods() {
        use ValueDuration::DateTime;

        fn parse(src: &str) -> Result<Vec<ValuePeriod>, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            values_period::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        // Test: multiple periods separated by commas
        let src = "19970101T180000Z/19970102T070000Z,19970102T180000Z/19970103T070000Z";
        let result = parse(src).unwrap();
        assert_eq!(result.len(), 2);
        let Some(ValuePeriod::Explicit { start, end }) = result.first() else {
            panic!("Expected Explicit period at index 0");
        };
        assert_eq!(start.date.year, 1997);
        assert_eq!(start.time.hour, 18);
        assert!(start.time.utc);
        assert_eq!(end.date.year, 1997);
        assert_eq!(end.time.hour, 7);
        assert!(end.time.utc);

        let Some(ValuePeriod::Explicit { start, end }) = result.get(1) else {
            panic!("Expected Explicit period at index 1");
        };
        assert_eq!(start.date.day, 2);
        assert_eq!(start.time.hour, 18);
        assert!(start.time.utc);
        assert_eq!(end.date.day, 3);
        assert_eq!(end.time.hour, 7);
        assert!(end.time.utc);

        // Test: mixed period types
        let src = "19970101T180000Z/19970102T070000Z,19970102T180000Z/PT5H30M";
        let result = parse(src).unwrap();
        assert_eq!(result.len(), 2);
        let Some(ValuePeriod::Explicit { start, end }) = result.first() else {
            panic!("Expected Explicit period at index 0");
        };
        assert_eq!(start.date.year, 1997);
        assert_eq!(start.time.hour, 18);
        assert!(start.time.utc);
        assert_eq!(end.date.day, 2);
        assert_eq!(end.time.hour, 7);
        assert!(end.time.utc);

        let Some(ValuePeriod::Duration { start, duration }) = result.get(1) else {
            panic!("Expected Duration period at index 1");
        };
        assert_eq!(start.date.day, 2);
        assert_eq!(start.time.hour, 18);
        assert!(start.time.utc);
        match duration {
            DateTime {
                hour: 5,
                minute: 30,
                ..
            } => {}
            _ => panic!("Expected duration 5h30m"),
        }
    }
}
