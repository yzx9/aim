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
pub use miscellaneous::ValueExpected;
pub(crate) use numeric::values_float_semicolon;
pub use period::ValuePeriod;
pub use rrule::{Day, RecurrenceFrequency, RecurrenceRule, WeekDay};
pub use text::ValueText;

use chumsky::input::Stream;
use chumsky::prelude::*;

use crate::lexer::Span;
use crate::parameter::ValueType;
use crate::syntax::SpannedSegments;
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
pub enum Value<'src> {
    /// This value type is used to identify properties that contain a character
    /// encoding of inline binary data.  For example, an inline attachment of a
    /// document might be included in an iCalendar object.
    ///
    /// See RFC 5545 Section 3.3.1 for more details.
    ///
    /// Note: This is a single-value type (comma-separated values not allowed).
    Binary {
        /// The binary data
        raw: SpannedSegments<'src>,
        /// The span of the value
        span: Span,
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
        span: Span,
    },

    // TODO: 3.3.3. Calendar User Address
    //
    /// This value type is used to identify values that contain a calendar date.
    ///
    /// See RFC 5545 Section 3.3.4 for more details.
    ///
    /// Note: This type supports multiple comma-separated values.
    Date {
        /// The date values
        values: Vec<ValueDate>,
        /// The span of the values
        span: Span,
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
        span: Span,
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
        span: Span,
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
        span: Span,
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
        span: Span,
    },

    // TODO: 3.3.10. Recurrence Rule
    //
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
        span: Span,
    },

    /// This value type is used to identify values that contain human-readable
    /// text.
    ///
    /// See RFC 5545 Section 3.3.11 for more details.
    ///
    /// Note: This type supports multiple comma-separated values.
    Text {
        /// The text values
        values: Vec<ValueText<'src>>,
        /// The span of the values
        span: Span,
    },

    /// This value type is used to identify values that contain a time of day.
    ///
    /// Note: This type supports multiple comma-separated values.
    Time {
        /// The time values
        values: Vec<ValueTime>,
        /// The span of the values
        span: Span,
    },

    // TODO: 3.3.13. URI
    //
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
        span: Span,
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
        raw: SpannedSegments<'src>,
        /// The value type that was specified
        kind: ValueType<'src>,
        /// The span of the value
        span: Span,
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
        raw: SpannedSegments<'src>,
        /// The value type that was specified
        kind: ValueType<'src>,
        /// The span of the value
        span: Span,
    },
}

impl<'src> Value<'src> {
    /// Get the span of this value.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Value::Binary { span, .. }
            | Value::Boolean { span, .. }
            | Value::Date { span, .. }
            | Value::DateTime { span, .. }
            | Value::Duration { span, .. }
            | Value::Float { span, .. }
            | Value::Integer { span, .. }
            | Value::Period { span, .. }
            | Value::Text { span, .. }
            | Value::Time { span, .. }
            | Value::UtcOffset { span, .. }
            | Value::XName { span, .. }
            | Value::Unrecognized { span, .. } => *span,
        }
    }

    /// Get the kind of this value, consuming the value in the process.
    ///
    /// This is useful when you need to move the kind out of a value that will
    /// be dropped anyway (e.g., in error handling).
    #[must_use]
    pub fn into_kind(self) -> ValueType<'src> {
        match self {
            Value::Binary { .. } => ValueType::Binary,
            Value::Boolean { .. } => ValueType::Boolean,
            Value::Date { .. } => ValueType::Date,
            Value::DateTime { .. } => ValueType::DateTime,
            Value::Duration { .. } => ValueType::Duration,
            Value::Float { .. } => ValueType::Float,
            Value::Integer { .. } => ValueType::Integer,
            Value::Period { .. } => ValueType::Period,
            Value::Text { .. } => ValueType::Text,
            Value::Time { .. } => ValueType::Time,
            Value::UtcOffset { .. } => ValueType::UtcOffset,
            Value::XName { kind, .. } | Value::Unrecognized { kind, .. } => kind,
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
/// # Errors
///
/// Parse errors from all attempted types
#[expect(clippy::too_many_lines)]
pub fn parse_values<'src>(
    kinds: &[ValueType<'src>],
    value: &SpannedSegments<'src>,
) -> Result<Value<'src>, Vec<Rich<'src, char>>> {
    // Collect errors from all attempted types
    let mut all_errors: Vec<Rich<'src, char>> = Vec::new();

    // PERF: provide fast path for common groups of value types
    // - DATE / DATE-TIME: DTSTART, DTEND, DUE, EXDATE, RECURRENCE-ID, RDATE
    // - DATE-TIME / DATE / PERIOD: RDATE
    // - DURATION / DATE-TIME: TRIGGER
    //
    // Try each value type in order
    for kind in kinds {
        match kind {
            ValueType::Binary => {
                let result: Result<(), Vec<Rich<char>>> = value_binary::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();
                if result.is_ok() {
                    let span = value.span();
                    return Ok(Value::Binary {
                        raw: value.clone(),
                        span,
                    });
                }
            }

            ValueType::Boolean => {
                let result = value_boolean::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();

                match result {
                    Ok(bool_value) => {
                        let span = value.span();
                        return Ok(Value::Boolean {
                            value: bool_value,
                            span,
                        });
                    }
                    Err(errs) => all_errors.extend(errs),
                }
            }

            ValueType::Date => {
                let result = values_date::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();

                match result {
                    Ok(dates) => {
                        let span = value.span();
                        return Ok(Value::Date {
                            values: dates,
                            span,
                        });
                    }
                    Err(errs) => all_errors.extend(errs),
                }
            }

            ValueType::DateTime => {
                let result = values_date_time::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();

                match result {
                    Ok(date_times) => {
                        let span = value.span();
                        return Ok(Value::DateTime {
                            values: date_times,
                            span,
                        });
                    }
                    Err(errs) => all_errors.extend(errs),
                }
            }

            ValueType::Duration => {
                let result = values_duration::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();

                match result {
                    Ok(durations) => {
                        let span = value.span();
                        return Ok(Value::Duration {
                            values: durations,
                            span,
                        });
                    }
                    Err(errs) => all_errors.extend(errs),
                }
            }

            ValueType::Float => {
                let result = values_float::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(floats) = result {
                    let span = value.span();
                    return Ok(Value::Float {
                        values: floats,
                        span,
                    });
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            ValueType::Integer => {
                let result = values_integer::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(integers) = result {
                    let span = value.span();
                    return Ok(Value::Integer {
                        values: integers,
                        span,
                    });
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            // URI and CAL-ADDRESS are parsed as text per RFC 5545
            // (cal-address = uri, and URI values are essentially text strings)
            ValueType::CalendarUserAddress | ValueType::Text | ValueType::Uri => {
                let result = values_text::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result()
                    .map(|texts| texts.into_iter().map(|a| a.build(value)).collect());

                match result {
                    Ok(texts) => {
                        let span = value.span();
                        return Ok(Value::Text {
                            values: texts,
                            span,
                        });
                    }
                    Err(errs) => all_errors.extend(errs),
                }
            }

            ValueType::Time => {
                let result = values_time::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();

                match result {
                    Ok(times) => {
                        let span = value.span();
                        return Ok(Value::Time {
                            values: times,
                            span,
                        });
                    }
                    Err(errs) => all_errors.extend(errs),
                }
            }

            ValueType::UtcOffset => {
                let result = value_utc_offset::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();

                match result {
                    Ok(offset) => {
                        let span = value.span();
                        return Ok(Value::UtcOffset {
                            value: offset,
                            span,
                        });
                    }
                    Err(errs) => all_errors.extend(errs),
                }
            }

            ValueType::Period => {
                let result = values_period::<'_, _, extra::Err<_>>()
                    .parse(make_input(value.clone()))
                    .into_result();

                match result {
                    Ok(periods) => {
                        let span = value.span();
                        return Ok(Value::Period {
                            values: periods,
                            span,
                        });
                    }
                    Err(errs) => all_errors.extend(errs),
                }
            }

            // TODO: implement other value types
            ValueType::RecurrenceRule => {
                // Return an error for unimplemented types
                let span = value.span();
                return Err(vec![Rich::custom(
                    span.into(),
                    format!("Parser for {kind} is not implemented"),
                )]);
            }

            ValueType::XName(_) | ValueType::Unrecognized(_) => {
                // For unknown value types, skip parsing and fall through to the fallback
                // at the end of this function
            }
        }
    }

    // All types failed - preserve raw data as XName or Unrecognized value per RFC 5545 Section 3.2.20
    // TODO: handle X-Name / Unrecognized gracefully
    // TODO: emit warning for unknown value type
    let kind = kinds.first().cloned().unwrap_or(ValueType::Text);
    let span = value.span();

    // Determine if this is an x-name or unrecognized based on the ValueType itself
    let value_variant = match &kind {
        ValueType::XName(_) => Value::XName {
            raw: value.clone(),
            kind,
            span,
        },
        _ => Value::Unrecognized {
            raw: value.clone(),
            kind,
            span,
        },
    };

    Ok(value_variant)
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
