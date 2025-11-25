// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

use chumsky::error::{Error, RichPattern};
use chumsky::extra::ParserExtra;
use chumsky::label::LabelError;
use chumsky::prelude::*;
use chumsky::{Parser, input::Stream};
use jiff::civil::{Date, Time, date, time};

use crate::keyword::{
    KW_BINARY, KW_BOOLEAN, KW_CAL_ADDRESS, KW_DATE, KW_DATETIME, KW_DURATION, KW_FLOAT, KW_INTEGER,
    KW_PERIOD, KW_RRULE, KW_TEXT, KW_TIME, KW_URI, KW_UTC_OFFSET,
};
use crate::syntax::{SegmentedChars, SpannedSegments};

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
pub enum PropertyValue<'src> {
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
    Date(Date),

    // TODO: 3.3.5. Date-Time
    //
    /// This value type is used to identify properties that contain a duration
    /// of time.
    ///
    /// See RFC 5545 Section 3.3.6 for more details.
    Duration(PropertyValueDuration),

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
    Text(SpannedSegments<'src>),

    /// This value type is used to identify values that contain a time of day.
    Time(PropertyValueTime),
    //
    // TODO: 3.3.13. URI
    //
    /// This value type is used to identify properties that contain an offset
    /// from UTC to local time.
    ///
    /// See RFC 5545 Section 3.3.14 for more details.
    UtcOffset(PropertyValueUtcOffset),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyValueKind {
    Binary,
    Boolean,
    CalendarUserAddress,
    Date,
    DateTime,
    Duration,
    Float,
    Integer,
    Period,
    RecurrenceRule,
    Text,
    Time,
    Uri,
    UtcOffset,
}

impl FromStr for PropertyValueKind {
    type Err = ();

    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            KW_BINARY      => Ok(PropertyValueKind::Binary),
            KW_BOOLEAN     => Ok(PropertyValueKind::Boolean),
            KW_CAL_ADDRESS => Ok(PropertyValueKind::CalendarUserAddress),
            KW_DATE        => Ok(PropertyValueKind::Date),
            KW_DATETIME    => Ok(PropertyValueKind::DateTime),
            KW_DURATION    => Ok(PropertyValueKind::Duration),
            KW_FLOAT       => Ok(PropertyValueKind::Float),
            KW_INTEGER     => Ok(PropertyValueKind::Integer),
            KW_PERIOD      => Ok(PropertyValueKind::Period),
            KW_RRULE       => Ok(PropertyValueKind::RecurrenceRule),
            KW_TEXT        => Ok(PropertyValueKind::Text),
            KW_URI         => Ok(PropertyValueKind::Uri),
            KW_TIME        => Ok(PropertyValueKind::Time),
            KW_UTC_OFFSET  => Ok(PropertyValueKind::UtcOffset),
            _ => Err(()),
        }
    }
}

impl AsRef<str> for PropertyValueKind {
    #[rustfmt::skip]
    fn as_ref(&self) -> &str {
        match self {
            PropertyValueKind::Binary              => KW_BINARY,
            PropertyValueKind::Boolean             => KW_BOOLEAN,
            PropertyValueKind::CalendarUserAddress => KW_CAL_ADDRESS,
            PropertyValueKind::Date                => KW_DATE,
            PropertyValueKind::DateTime            => KW_DATETIME,
            PropertyValueKind::Duration            => KW_DURATION,
            PropertyValueKind::Float               => KW_FLOAT,
            PropertyValueKind::Integer             => KW_INTEGER,
            PropertyValueKind::Period              => KW_PERIOD,
            PropertyValueKind::RecurrenceRule      => KW_RRULE,
            PropertyValueKind::Text                => KW_TEXT,
            PropertyValueKind::Time                => KW_TIME,
            PropertyValueKind::Uri                 => KW_URI,
            PropertyValueKind::UtcOffset           => KW_UTC_OFFSET,
        }
    }
}

impl Display for PropertyValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyValueExpected {
    Float,
    Integer,
}

impl From<PropertyValueExpected> for RichPattern<'_, char> {
    fn from(expected: PropertyValueExpected) -> Self {
        match expected {
            PropertyValueExpected::Float => Self::Label(Cow::Borrowed("float out of range")),
            PropertyValueExpected::Integer => Self::Label(Cow::Borrowed("integer out of range")),
        }
    }
}

type BoxedParser<'src, Err> =
    Boxed<'src, 'src, Stream<SegmentedChars<'src>>, PropertyValue<'src>, extra::Err<Err>>;

#[rustfmt::skip]
#[derive(Clone)]
pub struct PropertyValueParser<'src, Err>
where
    Err: Error<'src, Stream<SegmentedChars<'src>>> 
        + LabelError<'src, Stream<SegmentedChars<'src>>, PropertyValueExpected> + 'src
{
    boolean:    BoxedParser<'src, Err>,
    date:       BoxedParser<'src, Err>,
    duration:   BoxedParser<'src, Err>,
    float:      BoxedParser<'src, Err>,
    integer:    BoxedParser<'src, Err>,
    time:       BoxedParser<'src, Err>,
    utc_offset: BoxedParser<'src, Err>,
}

impl<'src, Err> PropertyValueParser<'src, Err>
where
    Err: Error<'src, Stream<SegmentedChars<'src>>>
        + LabelError<'src, Stream<SegmentedChars<'src>>, PropertyValueExpected>,
{
    #[rustfmt::skip]
    pub fn new() -> Self {
        Self {
            boolean:    property_value_boolean().boxed(),
            date:       property_value_date().boxed(),
            duration:   property_value_duration().boxed(),
            float:      property_value_float().boxed(),
            integer:    property_value_integer().boxed(),
            time:       property_value_time().boxed(),
            utc_offset: property_value_utc_offset().boxed(),
        }
    }

    pub fn parse(
        &self,
        kind: PropertyValueKind,
        strs: SpannedSegments<'src>,
    ) -> Result<PropertyValue<'src>, Vec<Err>> {
        use PropertyValueKind::{
            Binary, Boolean, Date, Duration, Float, Integer, Text, Time, UtcOffset,
        };

        match kind {
            Binary => return Ok(property_value_binary(strs)),
            Text => return Ok(property_value_text(strs)),
            _ => {}
        }

        // TODO: map span
        let stream = Stream::from_iter(strs.into_chars());

        match kind {
            Boolean => self.boolean.parse(stream),
            Date => self.date.parse(stream),
            Duration => self.duration.parse(stream),
            Float => self.float.parse(stream),
            Integer => self.integer.parse(stream),
            Time => self.time.parse(stream),
            UtcOffset => self.utc_offset.parse(stream),
            _ => unimplemented!("Parser for {kind} is not implemented"),
        }
        .into_result()
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
fn property_value_binary(strs: SpannedSegments<'_>) -> PropertyValue<'_> {
    // TODO: check is it a valid BASE64, this is easy to implement but currently
    // we will have to collect the chars as fragmented tokens, which may cause bad
    // performance.
    PropertyValue::Binary(strs)
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// boolean    = "TRUE" / "FALSE"
/// ```
fn property_value_boolean<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
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
        .to(PropertyValue::Boolean(true));

    let f = just('F')
        .ignore_then(just('A'))
        .ignore_then(just('L'))
        .ignore_then(just('S'))
        .ignore_then(just('E'))
        .ignored()
        .to(PropertyValue::Boolean(false));

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
fn property_value_duration<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
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

    let sign = sign().or_not().map(|sign| !matches!(sign, Some('-')));
    let prefix = sign.then_ignore(just('P'));
    choice((
        prefix.then(date).map(|(positive, (day, time))| {
            let hour = time.map_or(0, |t| t.0);
            let minute = time.and_then(|t| t.1).map_or(0, |m_s| m_s.0);
            let second = time.and_then(|t| t.1).and_then(|m_s| m_s.1).unwrap_or(0);
            PropertyValue::Duration(PropertyValueDuration::DateTime {
                positive,
                day,
                hour,
                minute,
                second,
            })
        }),
        prefix.then(time).map(|(positive, (h, ms))| {
            PropertyValue::Duration(PropertyValueDuration::DateTime {
                positive,
                day: 0,
                hour: h,
                minute: ms.map_or(0, |(m, _)| m),
                second: ms.map_or(0, |(_, s)| s.unwrap_or(0)),
            })
        }),
        prefix.then(week).map(|(positive, week)| {
            PropertyValue::Duration(PropertyValueDuration::Week { positive, week })
        }),
    ))
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
fn property_value_date<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    let i16 = select! { c @ '0'..='9' => c }.map(into_digit10::<i16>);
    let year = i16
        .then(i16)
        .then(i16)
        .then(i16)
        .map(|(((a, b), c), d)| 1000 * a + 100 * b + 10 * c + d);

    let i8 = select! { c @ '0'..='9' => c }.map(into_digit10::<i8>);
    let month = i8.then(i8).map(|(a, b)| 10 * a + b);
    let day = i8.then(i8).map(|(a, b)| 10 * a + b);
    year.then(month)
        .then(day)
        .map(|((y, m), d)| PropertyValue::Date(date(y, m, d)))
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// float      = (["+"] / "-") 1*DIGIT ["." 1*DIGIT]
/// ```
fn property_value_float<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, PropertyValueExpected>,
{
    let sign = sign().or_not();
    let integer_part = select! { c @ '0'..='9' => c }
        .repeated()
        .at_least(1)
        .collect::<String>();

    let fractional_part = just('.').ignore_then(integer_part);

    sign.then(integer_part)
        .then(fractional_part.or_not())
        .try_map_with(|((sign, int_part), frac_part), e| {
            let capacity = sign.map_or(0, |_| 1)
                + int_part.len()
                + frac_part.as_ref().map_or(0, |f| 1 + f.len());

            let mut s = String::with_capacity(capacity);
            if let Some(sign) = sign {
                s.push(sign);
            }
            s.push_str(&int_part);
            if let Some(frac) = frac_part {
                s.push('.');
                s.push_str(&frac);
            }

            let n = match lexical::parse_partial::<f64, _>(&s) {
                Ok((f, n)) => {
                    if n < s.len() {
                        n
                    } else if f.is_infinite() || f.is_nan() {
                        0
                    } else {
                        return Ok(PropertyValue::Float(f));
                    }
                }
                Err(_) => 0,
            };

            Err(E::Error::expected_found(
                [PropertyValueExpected::Float],
                Some(s.chars().nth(n).unwrap().into()), // safe unwrap since n < len
                e.span(),
            ))
        })
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// integer    = (["+"] / "-") 1*DIGIT
/// ```
fn property_value_integer<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, PropertyValueExpected>,
{
    sign()
        .or_not()
        .then(
            select! { c @ '0'..='9' => c }
                .repeated()
                .at_least(1)
                .collect::<String>(),
        )
        .try_map_with(|(sign, digits), e| {
            let capacity = sign.map_or(0, |_| 1) + digits.len();
            let mut int_str = String::with_capacity(capacity);
            if let Some(s) = sign {
                int_str.push(s);
            }
            int_str.push_str(&digits);

            match lexical::parse_partial::<i32, _>(&int_str) {
                Ok((v, n)) if n == int_str.len() => Ok(PropertyValue::Integer(v)),
                Ok((_, n)) => Err(E::Error::expected_found(
                    [PropertyValueExpected::Integer],
                    Some(int_str.chars().nth(n).unwrap().into()), // safe unwrap since n < len
                    e.span(),
                )),
                Err(_) => Err(E::Error::expected_found(
                    [PropertyValueExpected::Integer],
                    Some(int_str.chars().next().unwrap().into()), // safe unwrap since at least 1 digit
                    e.span(),
                )),
            }
        })
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// text       = *(TSAFE-CHAR / ":" / DQUOTE / ESCAPED-CHAR)
/// ; Folded according to description above
///
/// ESCAPED-CHAR = ("\\" / "\;" / "\," / "\N" / "\n")
/// ; \\ encodes \, \N or \n encodes newline
/// ; \; encodes ;, \, encodes ,
///
/// TSAFE-CHAR = WSP / %x21 / %x23-2B / %x2D-39 / %x3C-5B / %x5D-7E / NON-US-
/// ASCII
/// ; Any character except CONTROLs not needed by the current
/// ; character set, DQUOTE, ";", ":", "\", ","
/// ```
fn property_value_text(strs: SpannedSegments<'_>) -> PropertyValue<'_> {
    // TODO: handle escape sequences
    PropertyValue::Text(strs)
    // let strs = strs
    //     .into_iter()
    //     .map(|t| match t {
    //         SpannedToken(Token::Escape(c), span) => {
    //             let s = match c {
    //                 r"\n" | r"\N" => "\n",
    //                 r"\;" => ";",
    //                 r"\," => ",",
    //                 r"\\" => "\\",
    //                 _ => unreachable!("Invalid escape sequence: {c}"),
    //             };
    //             SpannedToken(Token::Word(s), span)
    //         }
    //         other => other,
    //     })
    //     .collect();
    //
    // PropertyValue::Text(strs)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyValueTime {
    pub time: Time,
    pub utc: bool,
}

// TODO: 3.3.10. Recurrence Rule

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
fn property_value_time<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    time_hour()
        .then(time_minute())
        .then(time_second())
        .then(just('Z').or_not())
        .map(|(((hour, minute), second), utc)| {
            PropertyValue::Time(PropertyValueTime {
                time: time(hour, minute, second, 0),
                utc: utc.is_some(),
            })
        })
}

// TODO: 3.3.13. URI

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyValueUtcOffset {
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
fn property_value_utc_offset<'src, I, E>() -> impl Parser<'src, I, PropertyValue<'src>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    sign()
        .then(time_hour())
        .then(time_minute())
        .then(time_second().or_not())
        .map(|(((sign, hour), minute), second)| {
            PropertyValue::UtcOffset(PropertyValueUtcOffset {
                positive: !matches!(sign, '-'),
                hour,
                minute,
                second: second.unwrap_or(0),
            })
        })
}

const fn sign<'src, I, E>() -> impl Parser<'src, I, char, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    select! { '+' => '+', '-' => '-' }
}

fn time_hour<'src, I, E>() -> impl Parser<'src, I, i8, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        select! { c @ '0'..='1' => c }
            .then(select! { c @ '0'..='9' => c })
            .map(|(a, b)| 10 * into_digit10::<i8>(a) + into_digit10::<i8>(b)),
        just('2')
            .ignore_then(select! { c @ '0'..='3' => c })
            .map(|b| 20 + into_digit10::<i8>(b)),
    ))
}

fn time_minute<'src, I, E>() -> impl Parser<'src, I, i8, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    select! { c @ '0'..='5' => c }
        .then(select! { c @ '0'..='9' => c })
        .map(|(a, b)| 10 * into_digit10::<i8>(a) + into_digit10::<i8>(b))
}

fn time_second<'src, I, E>() -> impl Parser<'src, I, i8, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        select! { c @ '0'..='5' => c }
            .then(select! { c @ '0'..='9' => c })
            .map(|(a, b)| 10 * into_digit10::<i8>(a) + into_digit10::<i8>(b)),
        // NOTE: We contract leap second 60 to 59 for simplicity
        just('6').ignore_then(just('0').ignored().to(59)),
    ))
}

fn into_digit10<I: TryFrom<u32> + Default>(c: char) -> I {
    let i = c.to_digit(10).unwrap_or_default();
    I::try_from(i).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use chumsky::input::Stream;

    use super::*;

    #[test]
    fn test_boolean() {
        fn parse<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, char>>> {
            let stream = Stream::from_iter(src.chars());
            property_value_boolean::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        for (src, expected) in [("TRUE", true), ("FALSE", false)] {
            match parse(src) {
                Ok(PropertyValue::Boolean(b)) => assert_eq!(b, expected),
                e => panic!("Expected Ok(PropertyValue::Boolean({expected})), got {e:?}"),
            }
        }

        let fail_cases = ["True", "False", "true", "false", "1", "0", "YES", "NO", ""];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn test_duration() {
        use PropertyValueDuration::{DateTime, Week};

        fn parse(src: &'_ str) -> Result<PropertyValue<'_>, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            property_value_duration::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        #[rustfmt::skip]
        let success_cases = [
            // in RFC 5545 Section 3.3.6 examples
            ("P15DT5H0M20S", DateTime { positive: true, day: 15, hour: 5, minute: 0, second: 20 }),
            ("P2W", Week { positive: true, week: 2 }),
            // extra tests
            ("+P3W", Week { positive: true, week: 3 }),
            ("-P1W", Week { positive: false, week: 1 }),
            ("+P3DT4H5M6S",  DateTime { positive:  true, day: 3, hour:  4, minute:  5, second:  6 }),
            ("-PT10H11M12S", DateTime { positive: false, day: 0, hour: 10, minute: 11, second: 12 }),
        ];
        for (src, expected) in success_cases {
            match parse(src) {
                Ok(PropertyValue::Duration(d)) => assert_eq!(d, expected),
                e => panic!("Expected Ok(PropertyValue::Duration({expected:?})), got {e:?}"),
            }
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

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_float() {
        fn parse<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, char>>> {
            let stream = Stream::from_iter(src.chars());
            property_value_float::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        let success_cases = [
            // Examples from RFC 5545 Section 3.3.7
            ("1000000.0000001", 1_000_000.000_000_1),
            ("1.333", 1.333),
            ("-3.14", -3.14),
            // extra tests
            ("123.456", 123.456),
            ("-987.654", -987.654),
            ("+0.001", 0.001),
            ("42", 42.0),
            ("+3.14", 3.14),
            ("-0.0", -0.0),
            ("0", 0.0),
            ("+0", 0.0),
            ("-1234567890.0987654321", -1_234_567_890.098_765_4), // precision limit, last digit rounded
        ];
        for (src, expected) in success_cases {
            match parse(src) {
                Ok(PropertyValue::Float(f)) => assert!((f - expected).abs() < 1e-5),
                e => panic!("Expected Ok(PropertyValue::Float({expected})), got {e:?}"),
            }
        }

        let infinity = (0..=f64::MAX_10_EXP).map(|_| '9').collect::<String>();
        let fail_cases = [
            &infinity,  // infinity
            "nan",      // RFC5545 does not allow non-numeric values
            "infinity", // RFC5545 does not allow non-numeric values
            "+.",       // missing digits
            "-.",       // missing digits
            ".",        // missing digits
            "",         // empty string
            "12a34",    // invalid character
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn test_integer() {
        fn parse<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Cheap>> {
            let stream = Stream::from_iter(src.chars());
            property_value_integer::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        #[rustfmt::skip]
        let success_cases = [
            // Examples from RFC 5545 Section 3.3.8
            ("1234567890", 1_234_567_890),
            ("-1234567890", -1_234_567_890),
            ("+1234567890", 1_234_567_890),
            ("432109876", 432_109_876),
            // extra tests
            ( "0", 0),
            ("+0", 0),
            ("-0", 0),
            ("+0000000000000000000000", 0), // long zero
            ("12345", 12345),
            ("-6789", -6789),
            ("+2147483647",  2_147_483_647), // i32 max
            ("-2147483648", -2_147_483_648), // i32 min
        ];
        for (src, expected) in success_cases {
            match parse(src) {
                Ok(PropertyValue::Integer(i)) => assert_eq!(i, expected),
                e => panic!("Expected Ok(PropertyValue::Integer({expected})), got {e:?}"),
            }
        }

        let fail_cases = [
            "nan",                   // RFC5545 does not allow non-numeric values
            "infinity",              // RFC5545 does not allow non-numeric values
            "+2147483648",           // i32 max + 1
            "-2147483649",           // i32 min - 1
            "12345678901234567890",  // overflow, too long
            "-12345678901234567890", // underflow, too long
            "+",                     // missing digits
            "-",                     // missing digits
            "",                      // empty string
            "12a34",                 // invalid character
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }

    // TODO: enable this test after implementing escape sequence handling
    //
    // #[test]
    // fn test_text() {
    //     use logos::Logos;
    //
    //     fn parse(src: &'_ str) -> PropertyValue<'_> {
    //         let strs = SpannedStrs::from_tokens(
    //         );
    //         property_value_text(tokens)
    //     }
    //
    //     #[rustfmt::skip]
    //     let success_cases = [
    //         // Examples from RFC 5545 Section 3.3.11
    //         (r"Project XYZ Final Review\nConference Room - 3B\nCome Prepared.",
    //           "Project XYZ Final Review\nConference Room - 3B\nCome Prepared."),
    //         // extra tests
    //         (r"Hello\, World\; \N", "Hello, World; \n"),
    //         ( r#""Quoted Text" and more text"#, r#""Quoted Text" and more text"#,),
    //         ("Unicode 字符串 🎉", "Unicode 字符串 🎉"),
    //         ("123\r\n 456\r\n\t789", "123456789"),
    //     ];
    //     for (src, expected) in success_cases {
    //         match parse(src) {
    //             PropertyValue::Text(ref segments) => assert_eq!(segments.to_string(), expected),
    //             _ => panic!("Expected PropertyValue::Text"),
    //         }
    //     }
    // }

    #[test]
    fn test_time() {
        fn parse<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, char>>> {
            let stream = Stream::from_iter(src.chars());
            property_value_time::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        #[rustfmt::skip]
        let success_cases = [
            // Examples from RFC 5545 Section 3.3.12
            ("135501",  PropertyValueTime { time: time(13, 55,  1, 0), utc: false }),
            ("135501Z", PropertyValueTime { time: time(13, 55,  1, 0), utc:  true }),
            // extra tests
            ("000000",  PropertyValueTime { time: time( 0,  0,  0, 0), utc: false }),
            ("235959",  PropertyValueTime { time: time(23, 59, 59, 0), utc: false }),
            ("120000Z", PropertyValueTime { time: time(12,  0,  0, 0), utc:  true }),
            ("000060",  PropertyValueTime { time: time( 0,  0, 59, 0), utc: false }), // leap second
        ];
        for (src, expected) in success_cases {
            match parse(src) {
                Ok(PropertyValue::Time(t)) => assert_eq!(t, expected),
                e => panic!("Expected Ok(PropertyValue::Time({expected:?})), got {e:?}"),
            }
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
        fn parse<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<PropertyValue<'src>, Vec<Rich<'src, char>>> {
            let stream = Stream::from_iter(src.chars());
            property_value_utc_offset::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }
        #[rustfmt::skip]
        let success_cases = [
            // Examples from RFC 5545 Section 3.3.14
            (  "-0500", PropertyValueUtcOffset{positive: false, hour: 5, minute:  0, second:  0}),
            (  "+0100", PropertyValueUtcOffset{positive:  true, hour: 1, minute:  0, second:  0}),
            // extra tests
            (  "+0000", PropertyValueUtcOffset{positive:  true, hour: 0, minute:  0, second:  0}),
            ("-123456", PropertyValueUtcOffset{positive: false, hour:12, minute: 34, second: 56}),
        ];
        for (src, expected) in success_cases {
            match parse(src) {
                Ok(PropertyValue::UtcOffset(v)) => assert_eq!(v, expected),
                e => panic!("Expected Ok(PropertyValue::UtcOffset({expected:?})), got {e:?}"),
            }
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
