// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use std::borrow::Cow;

use chumsky::Parser;
use chumsky::error::RichPattern;
use chumsky::extra::ParserExtra;
use chumsky::input::{Input, Stream};
use chumsky::prelude::*;
use chumsky::span::SimpleSpan;

use crate::syntax::SpannedSegments;
use crate::typed::parameter_type::ValueType;
use crate::typed::value_datetime::{value_utc_offset, values_date, values_date_time, values_time};
use crate::typed::value_duration::{ValueDuration, values_duration};
use crate::typed::value_numeric::{values_float, values_integer};
use crate::typed::value_period::{ValuePeriod, values_period};
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
    Binary(SpannedSegments<'src>), // TODO: implement

    /// This value type is used to identify properties that contain either a
    /// "TRUE" or "FALSE" Boolean value.
    ///
    /// See RFC 5545 Section 3.3.2 for more details.
    Boolean(bool), // TODO: implement

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

    // TODO: 3.3.10. Recurrence Rule
    //
    /// This value type is used to identify values that contain a precise
    /// period of time.
    ///
    /// See RFC 5545 Section 3.3.9 for more details.
    Period(ValuePeriod),

    /// This value type is used to identify values that contain human-readable
    /// text.
    ///
    /// See RFC 5545 Section 3.3.11 for more details.
    Text(ValueText<'src>),

    /// This value type is used to identify values that contain a time of day.
    Time(ValueTime), // TODO: implement

    // TODO: 3.3.13. URI
    //
    /// This value type is used to identify properties that contain an offset
    /// from UTC to local time.
    ///
    /// See RFC 5545 Section 3.3.14 for more details.
    UtcOffset(ValueUtcOffset), // TODO: implement
}

/// Parse property values, attempting each allowed value type until one succeeds.
///
/// When multiple value types are allowed (e.g., DATE or DATE-TIME), this function
/// will try each type in order, returning the first successful parse. This enables
/// type inference based on the format of the value.
///
/// # Arguments
///
/// * `kinds` - Slice of allowed value types to try, in order of preference
/// * `value` - The property value to parse
///
/// # Returns
///
/// * `Ok(Vec<Value>)` - Successfully parsed values
/// * `Err(Vec<Rich>)` - Parse errors from all attempted types
#[allow(clippy::too_many_lines)]
pub fn parse_values<'src>(
    kinds: &[ValueType],
    value: &SpannedSegments<'src>,
) -> Result<Vec<Value<'src>>, Vec<Rich<'src, char>>> {
    use ValueType::{
        Binary, Boolean, CalendarUserAddress, Date, DateTime, Duration, Float, Integer, Period,
        RecurrenceRule, Text, Time, Uri, UtcOffset,
    };

    // Collect errors from all attempted types
    let mut all_errors = Vec::new();

    // PERF: provide fast path for common groups of value types
    // - DATE / DATE-TIME: DTSTART, DTEND, DUE, EXDATE, RECURRENCE-ID, RDATE
    // - DATE-TIME / DATE / PERIOD: RDATE
    // - DURATION / DATE-TIME: TRIGGER
    //
    // Try each value type in order
    for kind in kinds {
        match kind {
            Binary => {
                let result: Result<(), Vec<Rich<char>>> = value_binary::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();
                if result.is_ok() {
                    return Ok(vec![Value::Binary(value.clone())]);
                }
            }

            Boolean => {
                let result = value_boolean::<'_, _, extra::Err<_>>()
                    .map(|a| vec![Value::Boolean(a)])
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            Date => {
                let result = values_date::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Date).collect())
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            DateTime => {
                let result = values_date_time::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::DateTime).collect())
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            Duration => {
                let result = values_duration::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Duration).collect())
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            Float => {
                let result = values_float::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Float).collect())
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            Integer => {
                let result = values_integer::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Integer).collect())
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            // URI and CAL-ADDRESS are parsed as text per RFC 5545
            // (cal-address = uri, and URI values are essentially text strings)
            CalendarUserAddress | Text | Uri => {
                let result = values_text::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result()
                    .map(|texts| {
                        texts
                            .into_iter()
                            .map(|a| Value::Text(a.build(value)))
                            .collect()
                    });
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            Time => {
                let result = values_time::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Time).collect())
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            UtcOffset => {
                let result = value_utc_offset::<'_, _, extra::Err<_>>()
                    .map(|a| vec![Value::UtcOffset(a)])
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            Period => {
                let result = values_period::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Period).collect())
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(values);
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            // TODO: implement other value types
            RecurrenceRule => {
                // Return an error for unimplemented types
                let span = value.span();
                return Err(vec![Rich::custom(
                    SimpleSpan::new((), span),
                    format!("Parser for {kind} is not implemented"),
                )]);
            }
        }
    }

    // All types failed - return all collected errors
    Err(all_errors)
}

/// Expected value types for parser error reporting.
///
/// This enum is used to provide descriptive error messages when parsing
/// fails. Each variant represents a type that was expected during parsing.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueExpected {
    /// A date value was expected
    Date,
    /// A 64-bit floating-point value was expected
    F64,
    /// A 32-bit signed integer value was expected
    I32,
    /// A 32-bit unsigned integer value was expected
    U32,
    /// Period date-times must have consistent timezone (both UTC or both floating)
    MismatchedTimezone,
}

impl From<ValueExpected> for RichPattern<'_, char> {
    fn from(expected: ValueExpected) -> Self {
        match expected {
            ValueExpected::Date => Self::Label(Cow::Borrowed("invalid date")),
            ValueExpected::F64 => Self::Label(Cow::Borrowed("f64 out of range")),
            ValueExpected::I32 => Self::Label(Cow::Borrowed("i32 out of range")),
            ValueExpected::U32 => Self::Label(Cow::Borrowed("u32 out of range")),
            ValueExpected::MismatchedTimezone => Self::Label(Cow::Borrowed(
                "period date-times must have consistent timezone",
            )),
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
    fn parses_binary() {
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
    fn parses_boolean() {
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
}
