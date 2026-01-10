// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Value type parsing module for iCalendar property values.
//!
//! This module handles the parsing and validation of iCalendar value types
//! as defined in RFC 5545 Section 3.3.

mod datetime;
mod duration;
mod miscellaneous;
pub(crate) mod numeric;
mod period;
mod rrule;
mod text;

pub use datetime::{ValueDate, ValueDateTime, ValueTime, ValueUtcOffset};
pub use duration::ValueDuration;
pub(crate) use numeric::values_float_semicolon;
pub use period::ValuePeriod;
pub use rrule::{RecurrenceFrequency, ValueRecurrenceRule, WeekDay, WeekDayNum};
pub use text::{ValueText, ValueTextOwned, ValueTextRef};

use chumsky::input::Stream;
use chumsky::prelude::*;

use crate::parameter::{ValueType, ValueTypeRef};
use crate::string_storage::{Span, SpannedSegments, StringStorage};
use crate::value::datetime::{value_utc_offset, values_date, values_date_time, values_time};
use crate::value::duration::values_duration;
use crate::value::miscellaneous::{value_binary, value_boolean};
use crate::value::numeric::{values_float, values_integer};
use crate::value::period::values_period;
use crate::value::text::values_text;

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
pub enum Value<S: StringStorage> {
    /// This value type is used to identify properties that contain a character
    /// encoding of inline binary data.  For example, an inline attachment of a
    /// document might be included in an iCalendar object.
    ///
    /// See RFC 5545 Section 3.3.1 for more details.
    ///
    /// Note: This is a single-value type (comma-separated values not allowed).
    Binary {
        /// The binary data
        value: S,
        /// The span of the value
        span: S::Span,
    },

    /// This value type is used to identify properties that contain either a
    /// "TRUE" or "FALSE" Boolean value.
    ///
    /// See RFC 5545 Section 3.3.2 for more details.
    ///
    /// Note: This is a single-value type (comma-separated values not allowed).
    Boolean {
        /// The boolean value
        value: bool,
        /// The span of the value
        span: S::Span,
    },

    /// This value type is used to identify properties that contain a calendar
    /// user address.
    ///
    /// See RFC 5545 Section 3.3.3 for more details.
    ///
    /// Note: This is a single-value type (comma-separated values not allowed).
    /// Per RFC 5545, no additional content value encoding (e.g., BACKSLASH
    /// character encoding) is defined for this value type.
    CalAddress {
        /// The calendar user address value (a URI)
        value: S,
        /// The span of the value
        span: S::Span,
    },

    /// This value type is used to identify values that contain a calendar date.
    ///
    /// See RFC 5545 Section 3.3.4 for more details.
    ///
    /// Note: This type supports multiple comma-separated values.
    Date {
        /// The date values
        values: Vec<ValueDate>,
        /// The span of the values
        span: S::Span,
    },

    /// This value type is used to identify properties that contain a date with
    ///
    /// See RFC 5545 Section 3.3.5 for more details.
    ///
    /// Note: This type supports multiple comma-separated values.
    DateTime {
        /// The date-time values
        values: Vec<ValueDateTime>,
        /// The span of the values
        span: S::Span,
    },

    /// This value type is used to identify properties that contain a duration
    /// of time.
    ///
    /// See RFC 5545 Section 3.3.6 for more details.
    ///
    /// Note: This type supports multiple comma-separated values.
    Duration {
        /// The duration values
        values: Vec<ValueDuration>,
        /// The span of the values
        span: S::Span,
    },

    /// This value type is used to identify properties that contain a real-
    /// number value.
    ///
    /// See RFC 5545 Section 3.3.7 for more details.
    ///
    /// Note: This type supports multiple comma-separated values.
    Float {
        /// The float values
        values: Vec<f64>,
        /// The span of the values
        span: S::Span,
    },

    /// This value type is used to identify properties that contain a signed
    /// integer value.
    ///
    /// See RFC 5545 Section 3.3.8 for more details.
    ///
    /// Note: This type supports multiple comma-separated values.
    Integer {
        /// The integer values
        values: Vec<i32>,
        /// The span of the values
        span: S::Span,
    },

    /// This value type is used to identify properties that contain a
    /// recurrence rule specification.
    ///
    /// See RFC 5545 Section 3.3.10 for more details.
    ///
    /// Note: This is a single-value type (comma-separated values not allowed).
    RecurrenceRule {
        /// The recurrence rule value
        value: Box<ValueRecurrenceRule>,
        /// The span of the value
        span: S::Span,
    },

    /// This value type is used to identify values that contain a precise
    /// period of time.
    ///
    /// See RFC 5545 Section 3.3.9 for more details.
    ///
    /// Note: This type supports multiple comma-separated values.
    Period {
        /// The period values
        values: Vec<ValuePeriod>,
        /// The span of the values
        span: S::Span,
    },

    /// This value type is used to identify values that contain human-readable
    /// text.
    ///
    /// See RFC 5545 Section 3.3.11 for more details.
    ///
    /// Note: This type supports multiple comma-separated values.
    Text {
        /// The text values
        values: Vec<ValueText<S>>,
        /// The span of the values
        span: S::Span,
    },

    /// This value type is used to identify values that contain a time of day.
    ///
    /// Note: This type supports multiple comma-separated values.
    Time {
        /// The time values
        values: Vec<ValueTime>,
        /// The span of the values
        span: S::Span,
    },

    /// This value type is used to identify properties that contain a Uniform
    /// Resource Identifier (URI).
    ///
    /// See RFC 5545 Section 3.3.13 for more details.
    ///
    /// Note: This is a single-value type (comma-separated values not allowed).
    /// Per RFC 5545, no additional content value encoding (e.g., BACKSLASH
    /// character encoding) is defined for this value type.
    Uri {
        /// The URI value
        value: S,
        /// The span of the value
        span: S::Span,
    },

    /// This value type is used to identify properties that contain an offset
    /// from UTC to local time.
    ///
    /// See RFC 5545 Section 3.3.14 for more details.
    ///
    /// Note: This is a single-value type (comma-separated values not allowed).
    UtcOffset {
        /// The UTC offset value
        value: ValueUtcOffset,
        /// The span of the value
        span: S::Span,
    },

    /// Custom experimental x-name value type (must start with "X-" or "x-").
    ///
    /// Per RFC 5545 Section 3.2.20: Applications MUST preserve the value data
    /// for x-name value types that they don't recognize without attempting to
    /// interpret or parse the value data.
    ///
    /// See also: RFC 5545 Section 3.2.20 (Value Data Types)
    XName {
        /// The raw value string (unparsed)
        raw: S,
        /// The value type that was specified
        kind: S,
        /// The span of the value
        span: S::Span,
    },

    /// Unrecognized value type (not a known standard value type).
    ///
    /// Per RFC 5545 Section 3.2.20: Applications MUST preserve the value data
    /// for iana-token value types that they don't recognize without attempting to
    /// interpret or parse the value data.
    ///
    /// See also: RFC 5545 Section 3.2.20 (Value Data Types)
    Unrecognized {
        /// The raw value string (unparsed)
        raw: S,
        /// The value type that was specified
        kind: S,
        /// The span of the value
        span: S::Span,
    },
}

/// Type alias for borrowed value
pub type ValueRef<'src> = Value<SpannedSegments<'src>>;

/// Type alias for owned value
pub type ValueOwned = Value<String>;

impl<S: StringStorage> Value<S> {
    /// Get the kind of this value.
    #[must_use]
    pub fn kind(&self) -> ValueType<&S> {
        match self {
            Value::Binary { .. } => ValueType::Binary,
            Value::Boolean { .. } => ValueType::Boolean,
            Value::CalAddress { .. } => ValueType::CalendarUserAddress,
            Value::Date { .. } => ValueType::Date,
            Value::DateTime { .. } => ValueType::DateTime,
            Value::Duration { .. } => ValueType::Duration,
            Value::Float { .. } => ValueType::Float,
            Value::Integer { .. } => ValueType::Integer,
            Value::RecurrenceRule { .. } => ValueType::RecurrenceRule,
            Value::Period { .. } => ValueType::Period,
            Value::Text { .. } => ValueType::Text,
            Value::Time { .. } => ValueType::Time,
            Value::Uri { .. } => ValueType::Uri,
            Value::UtcOffset { .. } => ValueType::UtcOffset,
            Value::XName { kind, .. } => ValueType::XName(kind),
            Value::Unrecognized { kind, .. } => ValueType::Unrecognized(kind),
        }
    }

    /// Get the span of this value.
    #[must_use]
    pub const fn span(&self) -> S::Span {
        match self {
            Value::Binary { span, .. }
            | Value::Boolean { span, .. }
            | Value::CalAddress { span, .. }
            | Value::Date { span, .. }
            | Value::DateTime { span, .. }
            | Value::Duration { span, .. }
            | Value::Float { span, .. }
            | Value::Integer { span, .. }
            | Value::RecurrenceRule { span, .. }
            | Value::Period { span, .. }
            | Value::Text { span, .. }
            | Value::Time { span, .. }
            | Value::Uri { span, .. }
            | Value::UtcOffset { span, .. }
            | Value::XName { span, .. }
            | Value::Unrecognized { span, .. } => *span,
        }
    }

    /// Get the number of values in this value variant.
    ///
    /// Single-value types return 1, multi-value types return the length of the vector.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Value::Date { values, .. } => values.len(),
            Value::DateTime { values, .. } => values.len(),
            Value::Duration { values, .. } => values.len(),
            Value::Float { values, .. } => values.len(),
            Value::Integer { values, .. } => values.len(),
            Value::Period { values, .. } => values.len(),
            Value::Text { values, .. } => values.len(),
            Value::Time { values, .. } => values.len(),
            Value::Binary { .. }
            | Value::Boolean { .. }
            | Value::CalAddress { .. }
            | Value::RecurrenceRule { .. }
            | Value::Uri { .. }
            | Value::UtcOffset { .. }
            | Value::XName { .. }
            | Value::Unrecognized { .. } => 1,
        }
    }

    /// Check if this value is empty (has 0 values).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl ValueRef<'_> {
    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> ValueOwned {
        match self {
            Value::Binary { value: raw, .. } => ValueOwned::Binary {
                value: raw.to_owned(),
                span: (),
            },
            Value::Boolean { value, .. } => ValueOwned::Boolean {
                value: *value,
                span: (),
            },
            Value::CalAddress { value, .. } => ValueOwned::CalAddress {
                value: value.to_owned(),
                span: (),
            },
            Value::Date { values, .. } => ValueOwned::Date {
                values: values.clone(),
                span: (),
            },
            Value::DateTime { values, .. } => ValueOwned::DateTime {
                values: values.clone(),
                span: (),
            },
            Value::Duration { values, .. } => ValueOwned::Duration {
                values: values.clone(),
                span: (),
            },
            Value::Float { values, .. } => ValueOwned::Float {
                values: values.clone(),
                span: (),
            },
            Value::Integer { values, .. } => ValueOwned::Integer {
                values: values.clone(),
                span: (),
            },
            Value::RecurrenceRule { value, .. } => ValueOwned::RecurrenceRule {
                value: value.clone(),
                span: (),
            },
            Value::Period { values, .. } => ValueOwned::Period {
                values: values.clone(),
                span: (),
            },
            Value::Text { values, .. } => ValueOwned::Text {
                values: values.iter().map(ValueText::to_owned).collect(),
                span: (),
            },
            Value::Time { values, .. } => ValueOwned::Time {
                values: values.clone(),
                span: (),
            },
            Value::Uri { value, .. } => ValueOwned::Uri {
                value: value.to_owned(),
                span: (),
            },
            Value::UtcOffset { value, .. } => ValueOwned::UtcOffset {
                value: *value,
                span: (),
            },
            Value::XName { raw, kind, .. } => ValueOwned::XName {
                raw: raw.to_owned(),
                kind: kind.to_owned(),
                span: (),
            },
            Value::Unrecognized { raw, kind, .. } => ValueOwned::Unrecognized {
                raw: raw.to_owned(),
                kind: kind.to_owned(),
                span: (),
            },
        }
    }
}

/// Parse property values, attempting each allowed value type until one succeeds.
///
/// When multiple value types are allowed (e.g., DATE or DATE-TIME), this function
/// will try each type in order, returning the first successful parse. This enables
/// type inference based on the format of the value.
///
/// # Errors
///
/// Parse errors from all attempted types
pub fn parse_value<'src>(
    value_types: &Vec<ValueTypeRef<'src>>,
    value: &SpannedSegments<'src>,
) -> Result<ValueRef<'src>, Vec<Rich<'src, char>>> {
    // Collect errors from all attempted types
    let mut all_errors: Vec<Rich<'src, char>> = Vec::new();

    // PERF: provide fast path for common groups of value types
    // - DATE / DATE-TIME: DTSTART, DTEND, DUE, EXDATE, RECURRENCE-ID, RDATE
    // - DATE-TIME / DATE / PERIOD: RDATE
    // - DURATION / DATE-TIME: TRIGGER
    //
    // Try each value type in order
    for value_type in value_types {
        match parse_value_single_type(value_type, value) {
            Ok(v) => return Ok(v),
            Err(errs) => all_errors.extend(errs),
        }
    }

    Err(all_errors)
}

/// Parse property value for a single specified value type.
#[expect(clippy::too_many_lines)]
fn parse_value_single_type<'src>(
    value_type: &ValueTypeRef<'src>,
    value: &SpannedSegments<'src>,
) -> Result<ValueRef<'src>, Vec<Rich<'src, char>>> {
    // Try the specified value type
    match value_type {
        ValueType::Binary => value_binary::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|()| Value::Binary {
                span: value.span(),
                value: value.clone(),
            }),

        ValueType::Boolean => value_boolean::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|bool_value| Value::Boolean {
                span: value.span(),
                value: bool_value,
            }),

        // CAL-ADDRESS: No additional content encoding, store raw string
        ValueType::CalendarUserAddress => Ok(Value::CalAddress {
            value: value.clone(),
            span: value.span(),
        }),

        ValueType::Date => values_date::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|values| Value::Date {
                span: value.span(),
                values,
            }),

        ValueType::DateTime => values_date_time::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|values| Value::DateTime {
                span: value.span(),
                values,
            }),

        ValueType::Duration => values_duration::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|values| Value::Duration {
                values,
                span: value.span(),
            }),

        ValueType::Float => values_float::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|values| Value::Float {
                values,
                span: value.span(),
            }),

        ValueType::Integer => values_integer::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|values| Value::Integer {
                values,
                span: value.span(),
            }),

        ValueType::RecurrenceRule => rrule::value_rrule::<'_, _, extra::Err<_>>()
            .parse(make_uppercase_input(value.clone())) // case-insensitive
            .into_result()
            .map(|rrule| Value::RecurrenceRule {
                value: Box::new(rrule),
                span: value.span(),
            }),

        ValueType::Period => values_period::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|values| Value::Period {
                values,
                span: value.span(),
            }),

        ValueType::Text => values_text::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|texts| Value::Text {
                values: texts.into_iter().map(|a| a.build(value)).collect(),
                span: value.span(),
            }),

        ValueType::Time => values_time::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|values| Value::Time {
                values,
                span: value.span(),
            }),

        // URI: No additional content encoding, store raw string
        ValueType::Uri => Ok(Value::Uri {
            value: value.clone(),
            span: value.span(),
        }),

        ValueType::UtcOffset => value_utc_offset::<'_, _, extra::Err<_>>()
            .parse(make_input(value.clone()))
            .into_result()
            .map(|offset| Value::UtcOffset {
                value: offset,
                span: value.span(),
            }),

        // For unknown value types, preserve raw data as XName or Unrecognized
        // value per RFC 5545 Section 3.2.20
        ValueType::XName(kind) => Ok(Value::XName {
            raw: value.clone(),
            kind: kind.clone(),
            span: value.span(),
        }),
        ValueType::Unrecognized(kind) => Ok(Value::Unrecognized {
            raw: value.clone(),
            kind: kind.clone(),
            span: value.span(),
        }),
    }
}

fn make_input(segs: SpannedSegments<'_>) -> impl Input<'_, Token = char, Span = SimpleSpan> {
    let eoi = match (segs.segments.first(), segs.segments.last()) {
        (Some(first), Some(last)) => Span {
            start: first.1.start,
            end: last.1.end,
        },
        _ => Span { start: 0, end: 0 },
    };
    Stream::from_iter(segs.into_spanned_chars()).map(eoi.into(), |(t, s)| (t, s.into()))
}

fn make_uppercase_input(
    segs: SpannedSegments<'_>,
) -> impl Input<'_, Token = char, Span = SimpleSpan> {
    let eoi = match (segs.segments.first(), segs.segments.last()) {
        (Some(first), Some(last)) => Span {
            start: first.1.start,
            end: last.1.end,
        },
        _ => Span { start: 0, end: 0 },
    };
    Stream::from_iter(segs.into_spanned_chars())
        .map(eoi.into(), |(t, s)| (t.to_ascii_uppercase(), s.into()))
}
