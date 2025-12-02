// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use std::{borrow::Cow, fmt::Display, str::FromStr};

use chumsky::Parser;
use chumsky::error::RichPattern;
use chumsky::input::Stream;
use chumsky::prelude::*;

use crate::keyword::{
    KW_BINARY, KW_BOOLEAN, KW_CAL_ADDRESS, KW_DATE, KW_DATETIME, KW_DURATION, KW_FLOAT, KW_INTEGER,
    KW_PERIOD, KW_RRULE, KW_TEXT, KW_TIME, KW_URI, KW_UTC_OFFSET,
};
use crate::syntax::SpannedSegments;
use crate::value::datetime::{value_utc_offset, values_date, values_date_time, values_time};
use crate::value::mics::{value_binary, value_boolean, values_duration};
use crate::value::numeric::{values_float, values_integer};
use crate::value::text::values_text;
use crate::{ValueDate, ValueDateTime, ValueDuration, ValueText, ValueTime, ValueUtcOffset};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
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

impl TryFrom<&SpannedSegments<'_>> for ValueKind {
    type Error = ();

    fn try_from(segs: &SpannedSegments<'_>) -> Result<Self, Self::Error> {
        // TODO: check quote: Property parameter values that are not in quoted-strings are case-insensitive.
        // TODO: avoid allocation
        segs.resolve().parse()
    }
}

impl FromStr for ValueKind {
    type Err = ();

    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            KW_BINARY      => Ok(ValueKind::Binary),
            KW_BOOLEAN     => Ok(ValueKind::Boolean),
            KW_CAL_ADDRESS => Ok(ValueKind::CalendarUserAddress),
            KW_DATE        => Ok(ValueKind::Date),
            KW_DATETIME    => Ok(ValueKind::DateTime),
            KW_DURATION    => Ok(ValueKind::Duration),
            KW_FLOAT       => Ok(ValueKind::Float),
            KW_INTEGER     => Ok(ValueKind::Integer),
            KW_PERIOD      => Ok(ValueKind::Period),
            KW_RRULE       => Ok(ValueKind::RecurrenceRule),
            KW_TEXT        => Ok(ValueKind::Text),
            KW_URI         => Ok(ValueKind::Uri),
            KW_TIME        => Ok(ValueKind::Time),
            KW_UTC_OFFSET  => Ok(ValueKind::UtcOffset),
            _ => Err(()),
        }
    }
}

impl AsRef<str> for ValueKind {
    #[rustfmt::skip]
    fn as_ref(&self) -> &str {
        match self {
            ValueKind::Binary              => KW_BINARY,
            ValueKind::Boolean             => KW_BOOLEAN,
            ValueKind::CalendarUserAddress => KW_CAL_ADDRESS,
            ValueKind::Date                => KW_DATE,
            ValueKind::DateTime            => KW_DATETIME,
            ValueKind::Duration            => KW_DURATION,
            ValueKind::Float               => KW_FLOAT,
            ValueKind::Integer             => KW_INTEGER,
            ValueKind::Period              => KW_PERIOD,
            ValueKind::RecurrenceRule      => KW_RRULE,
            ValueKind::Text                => KW_TEXT,
            ValueKind::Time                => KW_TIME,
            ValueKind::Uri                 => KW_URI,
            ValueKind::UtcOffset           => KW_UTC_OFFSET,
        }
    }
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

pub fn values(
    kind: ValueKind,
    value: SpannedSegments<'_>,
) -> Result<Vec<Value<'_>>, Vec<Rich<'_, char>>> {
    use ValueKind::{
        Binary, Boolean, Date, DateTime, Duration, Float, Integer, Text, Time, UtcOffset,
    };

    match kind {
        Binary => {
            let stream = make_input(value.clone()); // PERF: avoid clone
            value_binary::<'_, _, extra::Err<_>>()
                .check(stream)
                .into_result()?;
            Ok(vec![Value::Binary(value)])
        }
        Text => {
            let stream = make_input(value.clone()); // PERF: avoid clone
            values_text::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
                .map(|texts| {
                    texts
                        .into_iter()
                        .map(|a| Value::Text(a.build(&value)))
                        .collect()
                })
        }
        _ => {
            let stream = make_input(value);
            match kind {
                Boolean => value_boolean::<'_, _, extra::Err<_>>()
                    .map(|a| vec![Value::Boolean(a)])
                    .parse(stream),

                Date => values_date::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Date).collect())
                    .parse(stream),

                DateTime => values_date_time::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::DateTime).collect())
                    .parse(stream),

                Duration => values_duration::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Duration).collect())
                    .parse(stream),

                Float => values_float::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Float).collect())
                    .parse(stream),

                Integer => values_integer::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Integer).collect())
                    .parse(stream),

                Time => values_time::<'_, _, extra::Err<_>>()
                    .map(|a| a.into_iter().map(Value::Time).collect())
                    .parse(stream),

                UtcOffset => value_utc_offset::<'_, _, extra::Err<_>>()
                    .map(|a| vec![Value::UtcOffset(a)])
                    .parse(stream),

                _ => unimplemented!("Parser for {kind} is not implemented"),
            }
            .into_result()
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueExpected {
    Date,
    Float,
    Integer,
}

impl From<ValueExpected> for RichPattern<'_, char> {
    fn from(expected: ValueExpected) -> Self {
        match expected {
            ValueExpected::Date => Self::Label(Cow::Borrowed("invalid date")),
            ValueExpected::Float => Self::Label(Cow::Borrowed("float out of range")),
            ValueExpected::Integer => Self::Label(Cow::Borrowed("integer out of range")),
        }
    }
}

fn make_input(segs: SpannedSegments<'_>) -> impl Input<'_, Token = char, Span = SimpleSpan> {
    let eoi = match (segs.segments.first(), segs.segments.last()) {
        (Some(first), Some(last)) => first.1.start..last.1.end, // is it ..?
        _ => 0..0,
    };
    Stream::from_iter(segs.into_spanned_chars()).map(eoi.into(), |(t, s)| (t, s.into()))
}
