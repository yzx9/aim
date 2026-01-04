// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Date and Time Properties (RFC 5545 Section 3.8.2)
//!
//! This module contains property types for the "Date and Time Properties"
//! section of RFC 5545, including:
//!
//! ## Base Types
//!
//! - `DateTime`: Core date/time representation (floating, UTC, or timezone-aware)
//! - `Period`: Time period with start/end or start/duration
//! - `Time`: Time of day representation
//!
//! ## Property Wrapper Types
//!
//! Each wrapper type implements `Deref` and `DerefMut` to `DateTime` for convenient
//! access to the underlying date/time data, and includes a `kind()` method for
//! property validation:
//!
//! - 3.8.2.1: `Completed` - Date-Time Completed
//! - 3.8.2.2: `DtEnd` - Date-Time End
//! - 3.8.2.3: `Due` - Date-Time Due
//! - 3.8.2.4: `DtStart` - Date-Time Start
//! - 3.8.2.5: `Duration` - Duration of time
//! - 3.8.2.6: `FreeBusy` - Free/busy time information
//! - 3.8.2.7: `TimeTransparency` - Time transparency (OPAQUE/TRANSPARENT)
//!
//! - 3.8.7.1: `Created` - Date-Time Created
//! - 3.8.7.2: `DtStamp` - Date-Time Stamp
//! - 3.8.7.3: `LastModified` - Last Modified
//!
//! All wrapper types validate their property kind during conversion from
//! `ParsedProperty`, ensuring type safety throughout the parsing pipeline.

use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use crate::keyword::{KW_TRANSP_OPAQUE, KW_TRANSP_TRANSPARENT};
use crate::parameter::{FreeBusyType, Parameter, ValueKind};
use crate::property::PropertyKind;
use crate::property::util::take_single_value;
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueDate, ValueDuration, ValuePeriod, ValueTime};

/// Date and time representation
#[derive(Debug, Clone)]
pub enum DateTime<'src> {
    /// Date and time without timezone (floating time)
    Floating {
        /// Date part
        date: ValueDate,
        /// Time part
        time: Time,
    },

    /// Date and time with specific timezone
    Zoned {
        /// Date part
        date: ValueDate,
        /// Time part
        time: Time,
        /// Timezone identifier
        tz_id: SpannedSegments<'src>,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
    },

    /// Date and time in UTC
    Utc {
        /// Date part
        date: ValueDate,
        /// Time part
        time: Time,
    },

    /// Date-only value
    Date {
        /// Date part
        date: ValueDate,
    },
}

impl<'src> DateTime<'src> {
    /// Get the date part of this `DateTime`
    #[must_use]
    pub fn date(&self) -> ValueDate {
        match self {
            DateTime::Floating { date, .. }
            | DateTime::Zoned { date, .. }
            | DateTime::Utc { date, .. }
            | DateTime::Date { date } => *date,
        }
    }

    /// Get the time part if this is not a date-only value
    #[must_use]
    pub fn time(&self) -> Option<Time> {
        match self {
            DateTime::Floating { time, .. }
            | DateTime::Zoned { time, .. }
            | DateTime::Utc { time, .. } => Some(*time),
            DateTime::Date { .. } => None,
        }
    }

    /// Get the timezone ID if this is a zoned value
    #[must_use]
    pub fn tz_id(&self) -> Option<&SpannedSegments<'src>> {
        match self {
            DateTime::Zoned { tz_id, .. } => Some(tz_id),
            _ => None,
        }
    }

    /// Get the timezone if this is a zoned value (when jiff feature is enabled)
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn timezone(&self) -> Option<&jiff::tz::TimeZone> {
        match self {
            DateTime::Zoned { tz_jiff, .. } => Some(tz_jiff),
            _ => None,
        }
    }

    /// Check if this is a date-only value
    #[must_use]
    pub fn is_date_only(&self) -> bool {
        matches!(self, DateTime::Date { .. })
    }

    /// Check if this is a UTC value
    #[must_use]
    pub fn is_utc(&self) -> bool {
        matches!(self, DateTime::Utc { .. })
    }

    /// Check if this is a floating (no timezone) value
    #[must_use]
    pub fn is_floating(&self) -> bool {
        matches!(self, DateTime::Floating { .. })
    }

    /// Get the combined date and time as `jiff::civil::DateTime` (when jiff feature is enabled).
    ///
    /// Returns `None` for date-only values.
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn civil_date_time(&self) -> Option<jiff::civil::DateTime> {
        match self {
            DateTime::Floating { date, time }
            | DateTime::Zoned { date, time, .. }
            | DateTime::Utc { date, time } => Some(jiff::civil::DateTime::from_parts(
                date.civil_date(),
                time.civil_time(),
            )),
            DateTime::Date { .. } => None,
        }
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for DateTime<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        let value = match take_single_value(prop.kind, prop.values) {
            Ok(v) => v,
            Err(e) => return Err(vec![e]),
        };

        // Get TZID parameter
        let mut tz_id = None;
        #[cfg(feature = "jiff")]
        let mut tz_jiff = None;
        for param in prop.parameters {
            let kind_name = param.kind().name();
            let param_span = param.span();

            #[expect(clippy::single_match)]
            match param {
                Parameter::TimeZoneIdentifier {
                    value,
                    #[cfg(feature = "jiff")]
                    tz,
                    ..
                } => match tz_id {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => {
                        tz_id = Some(value);
                        #[cfg(feature = "jiff")]
                        {
                            tz_jiff = Some(tz);
                        }
                    }
                },
                _ => {}
            }
        }

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        // Try with timezone if available, otherwise fallback to basic conversion
        if let Some(tz_id_value) = tz_id {
            match value {
                Value::DateTime(dt) if dt.time.utc => Ok(DateTime::Utc {
                    date: dt.date,
                    time: dt.time.into(),
                }),

                Value::DateTime(dt) => Ok(DateTime::Zoned {
                    date: dt.date,
                    time: dt.time.into(),
                    tz_id: tz_id_value,
                    #[cfg(feature = "jiff")]
                    tz_jiff: tz_jiff.unwrap(), // SAFETY: set above
                }),

                _ => Err(vec![TypedError::PropertyInvalidValue {
                    property: prop.kind,
                    value: "Expected date-time value".to_string(),
                    span: prop.span,
                }]),
            }
        } else {
            match value {
                Value::Date(date) => Ok(DateTime::Date { date }),
                Value::DateTime(dt) if dt.time.utc => Ok(DateTime::Utc {
                    date: dt.date,
                    time: dt.time.into(),
                }),
                Value::DateTime(dt) => Ok(DateTime::Floating {
                    date: dt.date,
                    time: dt.time.into(),
                }),
                _ => Err(vec![TypedError::PropertyInvalidValue {
                    property: PropertyKind::DtStart, // Default fallback
                    value: format!("Expected date or date-time value, got {value:?}"),
                    span: prop.span,
                }]),
            }
        }
    }
}

/// Time representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    /// Hour component (0-23)
    pub hour: u8,

    /// Minute component (0-59)
    pub minute: u8,

    /// Second component (0-60)
    pub second: u8,

    /// Cached `jiff::civil::Time` representation
    #[cfg(feature = "jiff")]
    pub(crate) jiff: jiff::civil::Time,
}

impl Time {
    /// Create a new `Time` instance.
    ///
    /// # Errors
    /// If hour, minute, or second are out of valid ranges.
    #[expect(clippy::cast_possible_wrap)]
    pub fn new(hour: u8, minute: u8, second: u8) -> Result<Self, String> {
        Ok(Time {
            hour,
            minute,
            second,
            #[cfg(feature = "jiff")]
            jiff: jiff::civil::Time::new(
                hour as i8,
                minute as i8,
                second.min(59) as i8, // NOTE: we clamp second to 59 here because jiff does not support leap seconds
                0,
            )
            .map_err(|e| e.to_string())?,
        })
    }

    /// Get reference to cached `jiff::civil::Time`.
    #[cfg(feature = "jiff")]
    #[must_use]
    pub const fn civil_time(&self) -> jiff::civil::Time {
        self.jiff
    }
}

impl From<ValueTime> for Time {
    fn from(value: ValueTime) -> Self {
        Time {
            hour: value.hour,
            minute: value.minute,
            second: value.second,
            #[cfg(feature = "jiff")]
            jiff: value.jiff,
        }
    }
}

/// Period of time (RFC 5545 Section 3.8.2.6)
///
/// This type separates UTC, floating, and zoned time periods at the type level,
/// ensuring that start and end times always have consistent timezone semantics.
///
/// Per RFC 5545 Section 3.3.9, periods have two forms:
/// - Explicit: start and end date-times
/// - Duration: start date-time and duration
#[derive(Debug, Clone)]
pub enum Period<'src> {
    /// Start and end date/time in UTC
    ExplicitUtc {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: Time,
        /// End date
        end_date: ValueDate,
        /// End time
        end_time: Time,
    },

    /// Start and end date/time in floating time (no timezone)
    ExplicitFloating {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: Time,
        /// End date
        end_date: ValueDate,
        /// End time
        end_time: Time,
    },

    /// Start and end date/time with timezone reference
    ExplicitZoned {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: Time,
        /// End date
        end_date: ValueDate,
        /// End time
        end_time: Time,
        /// Timezone ID (same for both start and end)
        tz_id: SpannedSegments<'src>,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
    },

    /// Start date/time in UTC with a duration
    DurationUtc {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: Time,
        /// Duration from the start
        duration: ValueDuration,
    },

    /// Start date/time in floating time with a duration
    DurationFloating {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: Time,
        /// Duration from the start
        duration: ValueDuration,
    },

    /// Start date/time with timezone reference and a duration
    DurationZoned {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: Time,
        /// Duration from the start
        duration: ValueDuration,
        /// Start timezone ID
        tz_id: SpannedSegments<'src>,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
    },
}

impl<'src> Period<'src> {
    /// Get the timezone ID if this is a zoned period
    #[must_use]
    pub fn tz_id(&self) -> Option<&SpannedSegments<'src>> {
        match self {
            Period::ExplicitZoned { tz_id, .. } | Period::DurationZoned { tz_id, .. } => {
                Some(tz_id)
            }
            _ => None,
        }
    }

    /// Get the timezone if this is a zoned period (when jiff feature is enabled)
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn jiff_timezone(&self) -> Option<&jiff::tz::TimeZone> {
        match self {
            Period::ExplicitZoned { tz_jiff: tz, .. }
            | Period::DurationZoned { tz_jiff: tz, .. } => Some(tz),
            _ => None,
        }
    }

    /// Get the start as a `DateTime`.
    #[must_use]
    pub fn start(&self) -> DateTime<'src> {
        match self {
            Period::ExplicitUtc {
                start_date,
                start_time,
                ..
            }
            | Period::DurationUtc {
                start_date,
                start_time,
                ..
            } => DateTime::Utc {
                date: *start_date,
                time: *start_time,
            },
            Period::ExplicitFloating {
                start_date,
                start_time,
                ..
            }
            | Period::DurationFloating {
                start_date,
                start_time,
                ..
            } => DateTime::Floating {
                date: *start_date,
                time: *start_time,
            },
            Period::ExplicitZoned {
                start_date,
                start_time,
                tz_id,
                #[cfg(feature = "jiff")]
                tz_jiff,
                ..
            }
            | Period::DurationZoned {
                start_date,
                start_time,
                tz_id,
                #[cfg(feature = "jiff")]
                tz_jiff,
                ..
            } => DateTime::Zoned {
                date: *start_date,
                time: *start_time,
                tz_id: tz_id.clone(),
                #[cfg(feature = "jiff")]
                tz_jiff: tz_jiff.clone(),
            },
        }
    }

    /// Get the end as a `DateTime` (when jiff feature is enabled).
    ///
    /// For duration-based periods, calculates the end by adding the duration to the start.
    #[cfg(feature = "jiff")]
    #[expect(clippy::missing_panics_doc)]
    #[must_use]
    pub fn end(&self) -> DateTime<'src> {
        match self {
            Period::ExplicitUtc {
                end_date, end_time, ..
            } => DateTime::Utc {
                date: *end_date,
                time: *end_time,
            },
            Period::ExplicitFloating {
                end_date, end_time, ..
            } => DateTime::Floating {
                date: *end_date,
                time: *end_time,
            },
            Period::ExplicitZoned {
                end_date,
                end_time,
                tz_id,
                tz_jiff,
                ..
            } => DateTime::Zoned {
                date: *end_date,
                time: *end_time,
                tz_id: tz_id.clone(),
                tz_jiff: tz_jiff.clone(),
            },
            Period::DurationUtc {
                start_date,
                start_time,
                duration,
                ..
            } => {
                let start = jiff::civil::DateTime::from_parts(
                    start_date.civil_date(),
                    start_time.civil_time(),
                );
                let end = add_duration(start, duration);
                DateTime::Utc {
                    date: ValueDate {
                        year: end.year(),
                        month: end.month(),
                        day: end.day(),
                    },
                    #[expect(clippy::cast_sign_loss)]
                    time: Time::new(end.hour() as u8, end.minute() as u8, end.second() as u8)
                        .map_err(|e| format!("invalid time: {e}"))
                        .unwrap(), // SAFETY: hour, minute, second are within valid ranges
                }
            }
            Period::DurationFloating {
                start_date,
                start_time,
                duration,
                ..
            } => {
                let start = jiff::civil::DateTime::from_parts(
                    start_date.civil_date(),
                    start_time.civil_time(),
                );
                let end = add_duration(start, duration);
                DateTime::Floating {
                    date: ValueDate {
                        year: end.year(),
                        month: end.month(),
                        day: end.day(),
                    },
                    #[expect(clippy::cast_sign_loss)]
                    time: Time::new(end.hour() as u8, end.minute() as u8, end.second() as u8)
                        .map_err(|e| format!("invalid time: {e}"))
                        .unwrap(), // SAFETY: hour, minute, second are within valid ranges
                }
            }
            Period::DurationZoned {
                start_date,
                start_time,
                tz_id,
                tz_jiff,
                duration,
            } => {
                let start = jiff::civil::DateTime::from_parts(
                    start_date.civil_date(),
                    start_time.civil_time(),
                );
                let end = add_duration(start, duration);
                DateTime::Zoned {
                    date: ValueDate {
                        year: end.year(),
                        month: end.month(),
                        day: end.day(),
                    },
                    #[expect(clippy::cast_sign_loss)]
                    time: Time::new(end.hour() as u8, end.minute() as u8, end.second() as u8)
                        .expect("invalid time"),
                    tz_id: tz_id.clone(),
                    tz_jiff: tz_jiff.clone(),
                }
            }
        }
    }

    /// Get the start date and time as `jiff::civil::DateTime` (when jiff feature is enabled).
    #[cfg(feature = "jiff")]
    #[must_use]
    #[rustfmt::skip]
    pub fn start_civil(&self) -> jiff::civil::DateTime {
        match self {
            Period::ExplicitUtc { start_date, start_time, .. }
            | Period::ExplicitFloating { start_date, start_time, .. }
            | Period::ExplicitZoned { start_date, start_time, .. }
            | Period::DurationUtc { start_date, start_time, .. }
            | Period::DurationFloating { start_date, start_time, .. }
            | Period::DurationZoned { start_date, start_time, .. } => {
                jiff::civil::DateTime::from_parts(start_date.civil_date(), start_time.civil_time())
            }
        }
    }

    /// Get the end date and time as `jiff::civil::DateTime` (when jiff feature is enabled).
    ///
    /// For duration-based periods, calculates the end by adding the duration to the start.
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn end_civil(&self) -> jiff::civil::DateTime {
        self.start().civil_date_time().unwrap_or_default() // SAFETY: start() never returns Date-only
    }
}

impl<'src> TryFrom<Value<'src>> for Period<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(value: Value<'src>) -> Result<Self, Self::Error> {
        match value {
            Value::Period(value_period) => match value_period {
                ValuePeriod::Explicit { start, end } => {
                    // Both start and end have the same UTC flag (guaranteed by parser)
                    if start.time.utc {
                        Ok(Period::ExplicitUtc {
                            start_date: start.date,
                            start_time: start.time.into(),
                            end_date: end.date,
                            end_time: end.time.into(),
                        })
                    } else {
                        Ok(Period::ExplicitFloating {
                            start_date: start.date,
                            start_time: start.time.into(),
                            end_date: end.date,
                            end_time: end.time.into(),
                        })
                    }
                }
                ValuePeriod::Duration { start, duration } => {
                    // Only positive durations are valid for periods
                    if !matches!(duration, ValueDuration::DateTime { positive: true, .. })
                        && !matches!(duration, ValueDuration::Week { positive: true, .. })
                    {
                        return Err(vec![TypedError::PropertyInvalidValue {
                            property: PropertyKind::FreeBusy,
                            value: "Duration must be positive for periods".to_string(),
                            span: (0..0).into(), // TODO: provide actual span
                        }]);
                    }

                    if start.time.utc {
                        Ok(Period::DurationUtc {
                            start_date: start.date,
                            start_time: start.time.into(),
                            duration,
                        })
                    } else {
                        Ok(Period::DurationFloating {
                            start_date: start.date,
                            start_time: start.time.into(),
                            duration,
                        })
                    }
                }
            },
            _ => Err(vec![TypedError::ValueTypeDisallowed {
                property: PropertyKind::FreeBusy.as_str(),
                value_type: value.kind(),
                expected_types: &[ValueKind::Period],
                span: (0..0).into(), // TODO: provide actual span
            }]),
        }
    }
}

#[cfg(feature = "jiff")]
fn add_duration(start: jiff::civil::DateTime, duration: &ValueDuration) -> jiff::civil::DateTime {
    match duration {
        ValueDuration::DateTime {
            positive,
            day,
            hour,
            minute,
            second,
        } => {
            let span = jiff::Span::new()
                .try_days(i64::from(*day))
                .unwrap()
                .try_hours(i64::from(*hour))
                .unwrap()
                .try_minutes(i64::from(*minute))
                .unwrap()
                .try_seconds(i64::from(*second))
                .unwrap();

            if *positive {
                start.checked_add(span).unwrap()
            } else {
                start.checked_sub(span).unwrap()
            }
        }
        ValueDuration::Week { positive, week } => {
            let span = jiff::Span::new().try_weeks(i64::from(*week)).unwrap();
            if *positive {
                start.checked_add(span).unwrap()
            } else {
                start.checked_sub(span).unwrap()
            }
        }
    }
}

// DateTime wrapper types for specific properties

simple_property_wrapper!(
    /// Date-Time Completed property wrapper (RFC 5545 Section 3.8.2.1)
    Completed<'src>: DateTime<'src> => Completed
);

simple_property_wrapper!(
    /// Date-Time End property wrapper (RFC 5545 Section 3.8.2.2)
    DtEnd<'src>: DateTime<'src> => DtEnd
);

simple_property_wrapper!(
    /// Time Transparency property wrapper (RFC 5545 Section 3.8.2.3)
    Due<'src>: DateTime<'src> => Due
);

simple_property_wrapper!(
    /// Date-Time Start property wrapper (RFC 5545 Section 3.8.2.4)
    DtStart<'src>: DateTime<'src> => DtStart
);

/// Duration (RFC 5545 Section 3.8.2.5)
///
/// This property specifies a duration of time.
#[derive(Debug, Clone, Copy)]
pub struct Duration {
    /// Duration value
    pub value: ValueDuration,
}

impl Duration {
    /// Get the property kind for `Duration`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Duration
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Duration {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        match take_single_value(Self::kind(), prop.values) {
            Ok(Value::Duration(d)) => Ok(Self { value: d }),
            Ok(v) => Err(vec![TypedError::PropertyUnexpectedValue {
                property: prop.kind,
                expected: ValueKind::Duration,
                found: v.kind(),
                span: prop.span,
            }]),
            Err(e) => Err(vec![e]),
        }
    }
}

/// Free/Busy Time (RFC 5545 Section 3.8.2.6)
///
/// This property defines one or more free or busy time intervals.
#[derive(Debug, Clone)]
pub struct FreeBusy<'src> {
    /// Free/Busy type parameter
    pub fb_type: FreeBusyType,
    /// List of free/busy time periods
    pub values: Vec<Period<'src>>,
}

impl FreeBusy<'_> {
    /// Get the property kind for `FreeBusy`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::FreeBusy
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for FreeBusy<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        // Extract FBTYPE parameter (defaults to BUSY)
        let mut fb_type = FreeBusyType::Busy;
        for param in &prop.parameters {
            if let Parameter::FreeBusyType { value, .. } = param {
                fb_type = *value;
                break; // Found it, no need to continue
            }
        }

        let mut errors = Vec::new();
        if prop.values.is_empty() {
            errors.push(TypedError::PropertyMissingValue {
                property: prop.kind,
                span: prop.span,
            });
        }

        let mut values = Vec::with_capacity(prop.values.len());
        for value in prop.values {
            match Period::try_from(value) {
                Ok(period) => values.push(period),
                Err(e) => errors.extend(e),
            }
        }

        Ok(FreeBusy { fb_type, values })
    }
}

/// Time transparency for events (RFC 5545 Section 3.8.2.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeTransparency {
    /// Event blocks time
    #[default]
    Opaque,

    /// Event does not block time
    Transparent,
}

impl TimeTransparency {
    /// Get the property kind for `TimeTransparency`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Transp
    }
}

impl FromStr for TimeTransparency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_TRANSP_OPAQUE => Ok(Self::Opaque),
            KW_TRANSP_TRANSPARENT => Ok(Self::Transparent),
            _ => Err(format!("Invalid time transparency: {s}")),
        }
    }
}

impl AsRef<str> for TimeTransparency {
    fn as_ref(&self) -> &str {
        match self {
            Self::Opaque => KW_TRANSP_OPAQUE,
            Self::Transparent => KW_TRANSP_TRANSPARENT,
        }
    }
}

impl fmt::Display for TimeTransparency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TimeTransparency {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let text = match take_single_value(Self::kind(), prop.values) {
            Ok(Value::Text(t)) => t.resolve().to_string(),
            Ok(v) => {
                return Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueKind::Text,
                    found: v.kind(),
                    span: prop.span,
                }]);
            }
            Err(e) => return Err(vec![e]),
        };

        text.parse().map_err(|e| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Transp,
                value: e,
                span: prop.span,
            }]
        })
    }
}
