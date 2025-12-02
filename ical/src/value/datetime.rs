// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use chumsky::Parser;
use chumsky::extra::ParserExtra;
use chumsky::label::LabelError;
use chumsky::prelude::*;

use crate::value::types::ValueExpected;

/// Date value in the iCalendar format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueDate {
    /// Year component.
    pub year: i16,

    /// Month component, 1-12.
    pub month: i8,

    /// Day component, 1-31.
    pub day: i8,
}

impl ValueDate {
    /// Convert to `jiff::civil::Date`.
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn civil_date(self) -> jiff::civil::Date {
        self.into()
    }
}

#[cfg(feature = "jiff")]
impl From<ValueDate> for jiff::civil::Date {
    fn from(value: ValueDate) -> Self {
        jiff::civil::date(value.year, value.month, value.day)
    }
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// date               = date-value
///
/// date-value         = date-fullyear date-month date-mday
/// date-fullyear      = 4DIGIT
/// date-month         = 2DIGIT        ;01-12
/// date-mday          = 2DIGIT        ;01-28, 01-29, 01-30, 01-31
///                                    ;based on month/year
/// ```
fn value_date<'src, I, E>() -> impl Parser<'src, I, ValueDate, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    let year = i16_0_9()
        .then(i16_0_9())
        .then(i16_0_9())
        .then(i16_0_9())
        .map(|(((a, b), c), d)| 1000 * a + 100 * b + 10 * c + d);

    let month = choice((
        just('0').ignore_then(i8_1_9()),
        just('1').ignore_then(i8_0_2()).map(|b| 10 + b),
    ));

    // TODO: validate day based on month/year
    let day = choice((
        just('0').ignore_then(i8_1_9()),
        i8_1_2().then(i8_0_9()).map(|(a, b)| 10 * a + b),
        just('3').ignore_then(i8_0_1()).map(|b| 30 + b),
    ));

    year.then(month)
        .then(day)
        .try_map(|((year, month), day), span| {
            if cfg!(feature = "jiff") && jiff::civil::Date::new(year, month, day).is_err() {
                Err(E::Error::expected_found([ValueExpected::Date], None, span))
            } else {
                Ok(ValueDate { year, month, day })
            }
        })
}

/// Date multiple values parser.
///
/// If the property permits, multiple "date" values are specified as a
/// COMMA-separated list of values.
pub fn values_date<'src, I, E>() -> impl Parser<'src, I, Vec<ValueDate>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    value_date().separated_by(just(',')).collect()
}

/// Date-Time value defined in the RFC 5545 Section 3.3.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueDateTime {
    /// Date component.
    pub date: ValueDate,

    /// Time component.
    pub time: ValueTime,
}

impl ValueDateTime {
    /// Convert to `jiff::civil::DateTime`, contracting leap second 60 to 59.
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn civil_date_time(self) -> jiff::civil::DateTime {
        self.into()
    }
}

#[cfg(feature = "jiff")]
impl From<ValueDateTime> for jiff::civil::DateTime {
    fn from(value: ValueDateTime) -> Self {
        // NOTE: We contract leap second 60 to 59 for simplicity
        let second = if value.time.second == 60 {
            59
        } else {
            value.time.second
        };

        jiff::civil::datetime(
            value.date.year,
            value.date.month,
            value.date.day,
            value.time.hour,
            value.time.minute,
            second,
            0,
        )
    }
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// date-time  = date "T" time ;As specified in the DATE and TIME
/// ```
fn value_date_time<'src, I, E>() -> impl Parser<'src, I, ValueDateTime, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    value_date()
        .then_ignore(just('T'))
        .then(value_time())
        .map(|(date, time)| ValueDateTime { date, time })
}

/// Date-Time multiple values parser.
///
/// If the property permits, multiple "DATE-TIME" values are specified as a
/// COMMA-separated list of values.
pub fn values_date_time<'src, I, E>() -> impl Parser<'src, I, Vec<ValueDateTime>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    value_date_time().separated_by(just(',')).collect()
}

/// Time value defined in the RFC 5545 Section 3.3.12.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueTime {
    /// Hour component, 0-23.
    pub hour: i8,

    /// Minute component, 0-59.
    pub minute: i8,

    /// Second component, 0-60 (60 for leap second).
    pub second: i8,

    /// Whether the time is in UTC (indicated by a trailing 'Z').
    pub utc: bool,
}

impl ValueTime {
    /// Convert to `jiff::civil::Time`, contracting leap second 60 to 59.
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn civil_time(self) -> jiff::civil::Time {
        self.into()
    }
}

#[cfg(feature = "jiff")]
impl From<ValueTime> for jiff::civil::Time {
    fn from(value: ValueTime) -> Self {
        // NOTE: We contract leap second 60 to 59 for simplicity
        let second = if value.second == 60 { 59 } else { value.second };
        jiff::civil::time(value.hour, value.minute, second, 0)
    }
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// time         = time-hour time-minute time-second [time-utc]
///
/// time-hour    = 2DIGIT        ;00-23
/// time-minute  = 2DIGIT        ;00-59
/// time-second  = 2DIGIT        ;00-60
/// ;The "60" value is used to account for positive "leap" seconds.
///
/// time-utc     = "Z"
/// ```
fn value_time<'src, I, E>() -> impl Parser<'src, I, ValueTime, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    time_hour()
        .then(time_minute())
        .then(time_second_with_leap())
        .then(just('Z').or_not())
        .map(|(((hour, minute), second), utc)| ValueTime {
            hour,
            minute,
            second,
            utc: utc.is_some(),
        })
}

/// Time multiple values parser.
///
/// If the property permits, multiple "time" values are specified by a
/// COMMA-separated list of values.
pub fn values_time<'src, I, E>() -> impl Parser<'src, I, Vec<ValueTime>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    value_time().separated_by(just(',')).collect()
}

/// UTC Offset Value defined in RFC 5545 Section 3.3.14
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueUtcOffset {
    /// Whether the offset is positive
    pub positive: bool,

    /// Hour, 0-23
    pub hour: i8,

    /// Minute, 0-59
    pub minute: i8,

    /// Second, 0-60, optional
    pub second: i8,
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// utc-offset = time-numzone
///
/// time-numzone = ("+" / "-") time-hour time-minute [time-second]
/// ```
pub fn value_utc_offset<'src, I, E>() -> impl Parser<'src, I, ValueUtcOffset, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    select! { c @ ('+' | '-') => c }
        .then(time_hour())
        .then(time_minute())
        .then(time_second_with_leap().or_not())
        .map(|(((sign, hour), minute), second)| ValueUtcOffset {
            positive: !matches!(sign, '-'),
            hour,
            minute,
            second: second.unwrap_or(0),
        })
}

fn time_hour<'src, I, E>() -> impl Parser<'src, I, i8, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        i8_0_1().then(i8_0_9()).map(|(a, b)| 10 * a + b),
        just('2').ignore_then(i8_0_3()).map(|b| 20 + b),
    ))
}

fn time_minute<'src, I, E>() -> impl Parser<'src, I, i8, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    i8_0_5().then(i8_0_9()).map(|(a, b)| 10 * a + b)
}

fn time_second<'src, I, E>() -> impl Parser<'src, I, i8, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    i8_0_5().then(i8_0_9()).map(|(a, b)| 10 * a + b)
}

fn time_second_with_leap<'src, I, E>() -> impl Parser<'src, I, i8, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        time_second(),
        just('6').ignore_then(just('0').ignored().to(60)),
    ))
}

macro_rules! define_digit_select {
    ($fname:ident : $ty:ty => { $($ch:literal),+ $(,)? }) => {
        #[allow(clippy::cast_lossless, clippy::char_lit_as_u8, clippy::cast_possible_wrap)]
        const fn $fname<'src, I, E>() -> impl Parser<'src, I, $ty, E> + Copy
        where
            I: Input<'src, Token = char, Span = SimpleSpan>,
            E: ParserExtra<'src, I>,
        {
            select! {
                $(
                    $ch => (($ch as u8 - b'0') as $ty),
                )+
            }
        }
    };
}

define_digit_select!(i8_0_1 : i8 => { '0', '1' });
define_digit_select!(i8_0_2 : i8 => { '0', '1', '2' });
define_digit_select!(i8_0_3 : i8 => { '0', '1', '2', '3' });
define_digit_select!(i8_0_5 : i8 => { '0', '1', '2', '3', '4', '5' });
define_digit_select!(i8_0_9 : i8 => { '0', '1', '2', '3', '4', '5', '6', '7', '8', '9' });
define_digit_select!(i8_1_2 : i8 => { '1', '2' });
define_digit_select!(i8_1_9 : i8 => { '1', '2', '3', '4', '5', '6', '7', '8', '9' });
define_digit_select!(i16_0_9 : i16 => { '0', '1', '2', '3', '4', '5', '6', '7', '8', '9' });

#[cfg(test)]
mod tests {
    use chumsky::input::Stream;

    use super::*;

    #[test]
    fn test_date() {
        fn parse(src: &str) -> Result<ValueDate, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_date::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        #[rustfmt::skip]
        let mut success_cases = vec![
            // examples from RFC 5545 Section 3.3.4
            ("19970714", ValueDate { year: 1997, month: 7, day: 14 }),
            // extra tests
            ("20240101", ValueDate { year: 2024, month: 1, day: 1 }),
            ("20000229", ValueDate { year: 2000, month: 2, day: 29 }), // leap year
            ("19000101", ValueDate { year: 1900, month: 1, day: 1 }),
        ];

        let mut fail_cases = vec![
            "20241301",  // invalid month
            "20240001",  // invalid month
            "abcd1234",  // invalid characters
            "2024011",   // invalid length
            "202401011", // invalid length
        ];

        #[rustfmt::skip]
        let need_validate = [
            ("19970230", ValueDate { year: 1997, month: 2, day: 30 }), // invalid date
            ("20240230", ValueDate { year: 2024, month: 2, day: 30 }), // invalid date
        ];
        if cfg!(feature = "jiff") {
            fail_cases.extend(need_validate.into_iter().map(|(src, _)| src));
        } else {
            success_cases.extend(need_validate);
        }

        for (src, expected) in success_cases {
            assert_eq!(parse(src).unwrap(), expected);
        }

        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn test_date_time() {
        fn parse(src: &str) -> Result<ValueDateTime, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_date_time::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        #[rustfmt::skip]
        let success_cases = [
            // examples from RFC 5545 Section 3.3.5
            ("19980118T230000", ValueDateTime {
                date: ValueDate { year: 1998, month: 1, day: 18 },
                time: ValueTime { hour: 23, minute: 0, second: 0, utc: false },
            }),
            ("19980119T070000Z", ValueDateTime {
                date: ValueDate { year: 1998, month: 1, day: 19 },
                time: ValueTime { hour: 7, minute: 0, second: 0, utc: true },
            }),
            ("19980119T020000", ValueDateTime { // TODO: TZID=America/New_York:19980119T020000
                date: ValueDate { year: 1998, month: 1, day: 19 },
                time: ValueTime { hour: 2, minute: 0, second: 0, utc: false },
            }),
            ("19970630T235960Z", ValueDateTime {
                date: ValueDate { year: 1997, month: 6, day: 30 },
                time: ValueTime { hour: 23, minute: 59, second: 60, utc: true },
            }),
            ("19970714T133000", ValueDateTime { // Local time
                date: ValueDate { year: 1997, month: 7, day: 14 },
                time: ValueTime { hour: 13, minute: 30, second: 0, utc: false },
            }),
            ("19970714T173000Z", ValueDateTime { // UTC time
                date: ValueDate { year: 1997, month: 7, day: 14 },
                time: ValueTime { hour: 17, minute: 30, second: 0, utc: true },
            }),
            // TODO: TZID=America/New_York:19970714T133000
            //
            // extra tests
            ("19970714T133000", ValueDateTime {
                date: ValueDate { year: 1997, month: 7, day: 14 },
                time: ValueTime { hour: 13, minute: 30, second: 0, utc: false },
            }),
            ("19970714T133000Z", ValueDateTime {
                date: ValueDate { year: 1997, month: 7, day: 14 },
                time: ValueTime { hour: 13, minute: 30, second: 0, utc: true },
            }),
        ];
        for (src, expected) in success_cases {
            assert_eq!(parse(src).unwrap(), expected);
        }

        let fail_cases = [
            // examples from RFC 5545 Section 3.3.5
            "19980119T230000-0800", // invalid time format
            // extra tests
            "19970714 133000", // missing 'T'
            "19970714T250000", // invalid hour
            "19970714T126000", // invalid minute
            "19970714T123461", // invalid second
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn test_time() {
        fn parse(src: &str) -> Result<ValueTime, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_time::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        #[rustfmt::skip]
        let success_cases = [
            // examples from RFC 5545 Section 3.3.12
            ("135501",  ValueTime { hour: 13, minute: 55, second:  1, utc: false }),
            ("135501Z", ValueTime { hour: 13, minute: 55, second:  1, utc:  true }),
            // extra tests
            ("000000",  ValueTime { hour:  0, minute:  0, second:  0, utc: false }),
            ("235959",  ValueTime { hour: 23, minute: 59, second: 59, utc: false }),
            ("120000Z", ValueTime { hour: 12, minute:  0, second:  0, utc:  true }),
            ("000060",  ValueTime { hour:  0, minute:  0, second: 60, utc: false }), // leap second
        ];
        for (src, expected) in success_cases {
            assert_eq!(parse(src).unwrap(), expected);
        }

        let fail_cases = [
            // examples from RFC 5545 Section 3.3.12
            "230000-0800", // invalid time format
            // extra tests
            "240000",   // invalid hour
            "126060",   // invalid minute
            "123461",   // invalid second
            "12000",    // missing digit
            "120000ZZ", // extra character
            "",         // empty string
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn test_utc_offset() {
        fn parse(src: &str) -> Result<ValueUtcOffset, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_utc_offset::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }
        #[rustfmt::skip]
        let success_cases = [
            // examples from RFC 5545 Section 3.3.14
            (  "-0500", ValueUtcOffset{positive: false, hour: 5, minute:  0, second:  0}),
            (  "+0100", ValueUtcOffset{positive:  true, hour: 1, minute:  0, second:  0}),
            // extra tests
            (  "+0000", ValueUtcOffset{positive:  true, hour: 0, minute:  0, second:  0}),
            ("-123456", ValueUtcOffset{positive: false, hour:12, minute: 34, second: 56}),
        ];
        for (src, expected) in success_cases {
            assert_eq!(parse(src).unwrap(), expected);
        }

        let fail_cases = [
            "0500",     // missing sign
            "+2400",    // invalid hour
            "-1260",    // invalid minute
            "+123461",  // invalid second
            "+120",     // missing digit
            "+120000Z", // extra character
            "",         // empty string
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }
}
