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

use std::ops::{Deref, DerefMut};

pub use datetime::{ValueDate, ValueDateTime, ValueTime, ValueUtcOffset};
pub use duration::ValueDuration;
pub use miscellaneous::ValueExpected;
pub use numeric::values_float_semicolon;
pub use period::ValuePeriod;
pub use rrule::{Day, RecurrenceFrequency, RecurrenceRule, WeekDay};
pub use text::ValueText;

use chumsky::input::Stream;
use chumsky::prelude::*;

use crate::lexer::Span;
use crate::parameter::ValueKind;
use crate::syntax::SpannedSegments;
use crate::value::datetime::{value_utc_offset, values_date, values_date_time, values_time};
use crate::value::duration::values_duration;
use crate::value::miscellaneous::{value_binary, value_boolean};
use crate::value::numeric::{values_float, values_integer};
use crate::value::period::values_period;
use crate::value::text::values_text;

/// Represents multiple property values with their source span.
///
/// This type wraps a vector of parsed values with span information,
/// enabling error reporting that references the original source location.
#[derive(Debug, Clone)]
pub struct Values<'src> {
    /// The parsed values
    pub values: Vec<Value<'src>>,
    /// The span covering all values in the source
    pub span: Span,
}

impl<'src> Deref for Values<'src> {
    type Target = Vec<Value<'src>>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl DerefMut for Values<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl<'src> IntoIterator for Values<'src> {
    type Item = Value<'src>;
    type IntoIter = std::vec::IntoIter<Value<'src>>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a, 'src> IntoIterator for &'a Values<'src> {
    type Item = &'a Value<'src>;
    type IntoIter = std::slice::Iter<'a, Value<'src>>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

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
    Time(ValueTime),

    // TODO: 3.3.13. URI
    //
    /// This value type is used to identify properties that contain an offset
    /// from UTC to local time.
    ///
    /// See RFC 5545 Section 3.3.14 for more details.
    UtcOffset(ValueUtcOffset),
}

impl Value<'_> {
    /// Get the kind of this value.
    #[must_use]
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Binary(_) => ValueKind::Binary,
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Date(_) => ValueKind::Date,
            Value::DateTime(_) => ValueKind::DateTime,
            Value::Duration(_) => ValueKind::Duration,
            Value::Float(_) => ValueKind::Float,
            Value::Integer(_) => ValueKind::Integer,
            Value::Period(_) => ValueKind::Period,
            Value::Text(_) => ValueKind::Text,
            Value::Time(_) => ValueKind::Time,
            Value::UtcOffset(_) => ValueKind::UtcOffset,
        }
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
    kinds: &[ValueKind],
    value: &SpannedSegments<'src>,
) -> Result<Values<'src>, Vec<Rich<'src, char>>> {
    use ValueKind::{
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
                    return Ok(Values {
                        values: vec![Value::Binary(value.clone())],
                        span: value.span(),
                    });
                }
            }

            Boolean => {
                let result = value_boolean::<'_, _, extra::Err<_>>()
                    .map(|a| vec![Value::Boolean(a)])
                    .parse(make_input(value.clone()))
                    .into_result();
                if let Ok(values) = result {
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
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
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
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
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
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
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
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
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
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
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
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
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
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
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
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
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
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
                    return Ok(Values {
                        values,
                        span: value.span(),
                    });
                } else if let Err(errs) = result {
                    all_errors.extend(errs);
                }
            }

            // TODO: implement other value types
            RecurrenceRule => {
                // Return an error for unimplemented types
                let span = value.span();
                return Err(vec![Rich::custom(
                    span.into(),
                    format!("Parser for {kind} is not implemented"),
                )]);
            }
        }
    }

    // All types failed - return all collected errors
    // TODO: map span to the entire value span
    Err(all_errors)
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
