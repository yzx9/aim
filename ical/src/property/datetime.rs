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
//! All wrapper types validate their property kind during conversion from
//! `ParsedProperty`, ensuring type safety throughout the parsing pipeline.

use std::convert::TryFrom;

use crate::keyword::{KW_TRANSP_OPAQUE, KW_TRANSP_TRANSPARENT};
use crate::parameter::{FreeBusyType, Parameter, ValueType};
use crate::property::PropertyKind;
use crate::property::common::{take_single_text, take_single_value};
use crate::string_storage::{Segments, StringStorage};
use crate::syntax::RawParameter;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueDate, ValueDuration, ValuePeriod, ValueTime};

/// Core date-time value variants without parameters or span
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTime {
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
    Date(ValueDate),
}

impl DateTime {
    /// Get the date part of this `DateTime`
    #[must_use]
    pub const fn date(&self) -> ValueDate {
        match self {
            DateTime::Floating { date, .. }
            | DateTime::Zoned { date, .. }
            | DateTime::Utc { date, .. }
            | DateTime::Date(date) => *date,
        }
    }

    /// Get the time part if this is not a date-only value
    #[must_use]
    pub const fn time(&self) -> Option<Time> {
        match self {
            DateTime::Floating { time, .. }
            | DateTime::Zoned { time, .. }
            | DateTime::Utc { time, .. } => Some(*time),
            DateTime::Date(_) => None,
        }
    }

    /// Check if this is a date-only value
    #[must_use]
    pub const fn is_date_only(&self) -> bool {
        matches!(self, DateTime::Date(_))
    }

    /// Check if this is a UTC value
    #[must_use]
    pub const fn is_utc(&self) -> bool {
        matches!(self, DateTime::Utc { .. })
    }

    /// Check if this is a floating (no timezone) value
    #[must_use]
    pub const fn is_floating(&self) -> bool {
        matches!(self, DateTime::Floating { .. })
    }

    /// Check if this is a zoned value
    #[must_use]
    pub const fn is_zoned(&self) -> bool {
        matches!(self, DateTime::Zoned { .. })
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
            DateTime::Date(_) => None,
        }
    }
}

/// Date and time representation
#[derive(Debug, Clone)]
pub struct DateTimeProperty<S: StringStorage> {
    /// Core date-time value
    pub value: DateTime,
    /// Timezone ID (only for Zoned variant)
    pub tz_id: Option<S>,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<S: StringStorage> DateTimeProperty<S> {
    /// Create a new floating `DateTimeProperty` (no timezone)
    #[must_use]
    pub fn floating(
        date: ValueDate,
        time: Time,
        x_parameters: Vec<RawParameter<S>>,
        retained_parameters: Vec<Parameter<S>>,
        span: S::Span,
    ) -> Self {
        Self {
            value: DateTime::Floating { date, time },
            tz_id: None,
            x_parameters,
            retained_parameters,
            span,
        }
    }

    /// Create a new zoned `DateTimeProperty` (with timezone)
    #[must_use]
    pub fn zoned(
        date: ValueDate,
        time: Time,
        tz_id: S,
        #[cfg(feature = "jiff")] tz_jiff: jiff::tz::TimeZone,
        x_parameters: Vec<RawParameter<S>>,
        retained_parameters: Vec<Parameter<S>>,
        span: S::Span,
    ) -> Self {
        Self {
            value: DateTime::Zoned {
                date,
                time,
                #[cfg(feature = "jiff")]
                tz_jiff,
            },
            tz_id: Some(tz_id),
            x_parameters,
            retained_parameters,
            span,
        }
    }

    /// Create a new UTC `DateTimeProperty`
    #[must_use]
    pub fn utc(
        date: ValueDate,
        time: Time,
        x_parameters: Vec<RawParameter<S>>,
        retained_parameters: Vec<Parameter<S>>,
        span: S::Span,
    ) -> Self {
        Self {
            value: DateTime::Utc { date, time },
            tz_id: None,
            x_parameters,
            retained_parameters,
            span,
        }
    }

    /// Create a new date-only `DateTimeProperty`
    #[must_use]
    pub fn date_only(
        date: ValueDate,
        x_parameters: Vec<RawParameter<S>>,
        retained_parameters: Vec<Parameter<S>>,
        span: S::Span,
    ) -> Self {
        Self {
            value: DateTime::Date(date),
            tz_id: None,
            x_parameters,
            retained_parameters,
            span,
        }
    }

    /// Get the date part of this `DateTimeProperty`
    #[must_use]
    pub fn date(&self) -> ValueDate {
        self.value.date()
    }

    /// Get the time part if this is not a date-only value
    #[must_use]
    pub fn time(&self) -> Option<Time> {
        self.value.time()
    }

    /// Get the timezone ID if this is a zoned value
    #[must_use]
    pub fn tz_id(&self) -> Option<&S> {
        self.tz_id.as_ref()
    }

    /// Get the timezone if this is a zoned value (when jiff feature is enabled)
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn timezone(&self) -> Option<&jiff::tz::TimeZone> {
        match self.value {
            DateTime::Zoned { ref tz_jiff, .. } => Some(tz_jiff),
            _ => None,
        }
    }

    /// Check if this is a date-only value
    #[must_use]
    pub fn is_date_only(&self) -> bool {
        self.value.is_date_only()
    }

    /// Check if this is a UTC value
    #[must_use]
    pub fn is_utc(&self) -> bool {
        self.value.is_utc()
    }

    /// Check if this is a floating (no timezone) value
    #[must_use]
    pub fn is_floating(&self) -> bool {
        self.value.is_floating()
    }

    /// Check if this is a zoned value
    #[must_use]
    pub fn is_zoned(&self) -> bool {
        self.value.is_zoned()
    }

    /// Get the combined date and time as `jiff::civil::DateTime` (when jiff feature is enabled).
    ///
    /// Returns `None` for date-only values.
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn civil_date_time(&self) -> Option<jiff::civil::DateTime> {
        self.value.civil_date_time()
    }

    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for DateTimeProperty<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let span = prop.span;
        let mut errors: Vec<TypedError<'src>> = Vec::new();

        let value = match take_single_value(&prop.kind, prop.value) {
            Ok(v) => v,
            Err(mut e) => {
                errors.append(&mut e);
                return Err(errors);
            }
        };

        // Get TZID parameter
        let mut tz_id = None;
        #[cfg(feature = "jiff")]
        let mut tz_jiff = None;
        let mut x_parameters = Vec::new();
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::TimeZoneIdentifier { .. } if tz_id.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.kind().into(),
                    });
                }
                Parameter::TimeZoneIdentifier {
                    value,
                    #[cfg(feature = "jiff")]
                    tz,
                    ..
                } => {
                    tz_id = Some(value);
                    #[cfg(feature = "jiff")]
                    {
                        tz_jiff = Some(tz);
                    }
                }

                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        // Try with timezone if available, otherwise fallback to basic conversion
        if let Some(tz_id_value) = tz_id {
            match value {
                Value::DateTime { mut values, .. } if values.len() == 1 => {
                    let dt = values.pop().unwrap();
                    if dt.time.utc {
                        Ok(DateTimeProperty::utc(
                            dt.date,
                            dt.time.into(),
                            x_parameters,
                            retained_parameters,
                            span,
                        ))
                    } else {
                        Ok(DateTimeProperty::zoned(
                            dt.date,
                            dt.time.into(),
                            tz_id_value,
                            #[cfg(feature = "jiff")]
                            tz_jiff.unwrap(), // SAFETY: set above
                            x_parameters,
                            retained_parameters,
                            span,
                        ))
                    }
                }

                _ => Err(vec![TypedError::PropertyInvalidValue {
                    property: prop.kind,
                    value: "Expected date-time value".to_string(),
                    span: value.span(),
                }]),
            }
        } else {
            match value {
                Value::Date { mut values, .. } if values.len() == 1 => {
                    let date = values.pop().unwrap();
                    Ok(DateTimeProperty::date_only(
                        date,
                        x_parameters,
                        retained_parameters,
                        span,
                    ))
                }
                Value::DateTime { mut values, .. } if values.len() == 1 => {
                    let dt = values.pop().unwrap();
                    if dt.time.utc {
                        Ok(DateTimeProperty::utc(
                            dt.date,
                            dt.time.into(),
                            x_parameters,
                            retained_parameters,
                            span,
                        ))
                    } else {
                        Ok(DateTimeProperty::floating(
                            dt.date,
                            dt.time.into(),
                            x_parameters,
                            retained_parameters,
                            span,
                        ))
                    }
                }
                _ => {
                    const EXPECTED: &[ValueType<String>] = &[ValueType::Date, ValueType::DateTime];
                    Err(vec![TypedError::PropertyUnexpectedValue {
                        property: PropertyKind::DtStart, // Default fallback
                        expected: EXPECTED,
                        found: value.kind().into(),
                        span: value.span(),
                    }])
                }
            }
        }
    }
}

impl DateTimeProperty<Segments<'_>> {
    /// Convert borrowed `DateTimeProperty` to owned `DateTimeProperty`
    #[must_use]
    pub fn to_owned(&self) -> DateTimeProperty<String> {
        DateTimeProperty {
            value: self.value.clone(),
            tz_id: self.tz_id.as_ref().map(ToString::to_string),
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

/// Time representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    /// Hour component (0-23)
    pub hour: i8,
    /// Minute component (0-59)
    pub minute: i8,
    /// Second component (0-60)
    pub second: i8,
    /// Cached `jiff::civil::Time` representation
    #[cfg(feature = "jiff")]
    pub(crate) jiff: jiff::civil::Time,
}

impl Time {
    /// Create a new `Time` instance.
    ///
    /// # Errors
    /// If hour, minute, or second are out of valid ranges.
    pub fn new(hour: i8, minute: i8, second: i8) -> Result<Self, String> {
        Ok(Time {
            hour,
            minute,
            second,
            #[cfg(feature = "jiff")]
            jiff: jiff::civil::Time::new(
                hour,
                minute,
                second.min(59), // NOTE: we clamp second to 59 here because jiff does not support leap seconds
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

#[cfg(feature = "jiff")]
impl From<jiff::civil::Time> for Time {
    fn from(value: jiff::civil::Time) -> Self {
        Time {
            hour: value.hour(),
            minute: value.minute(),
            second: value.second(),
            jiff: value,
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
pub enum Period<S: StringStorage> {
    /// Start and end date/time in UTC
    ExplicitUtc {
        /// Start date/time
        start: DateTime,
        /// End date/time
        end: DateTime,
    },

    /// Start and end date/time in floating time (no timezone)
    ExplicitFloating {
        /// Start date/time
        start: DateTime,
        /// End date/time
        end: DateTime,
    },

    /// Start and end date/time with timezone reference
    ExplicitZoned {
        /// Start date/time
        start: DateTime,
        /// End date/time
        end: DateTime,
        /// Timezone ID (same for both start and end)
        tz_id: S,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
    },

    /// Start date/time in UTC with a duration
    DurationUtc {
        /// Start date/time
        start: DateTime,
        /// Duration from the start
        duration: ValueDuration,
    },

    /// Start date/time in floating time with a duration
    DurationFloating {
        /// Start date/time
        start: DateTime,
        /// Duration from the start
        duration: ValueDuration,
    },

    /// Start date/time with timezone reference and a duration
    DurationZoned {
        /// Start date/time
        start: DateTime,
        /// Duration from the start
        duration: ValueDuration,
        /// Start timezone ID
        tz_id: S,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
    },
}

impl<S: StringStorage> Period<S> {
    /// Get the timezone ID if this is a zoned period
    #[must_use]
    pub fn tz_id(&self) -> Option<&S> {
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
    pub fn start(&self) -> DateTime {
        match self {
            Period::ExplicitUtc { start, .. }
            | Period::ExplicitFloating { start, .. }
            | Period::ExplicitZoned { start, .. }
            | Period::DurationUtc { start, .. }
            | Period::DurationFloating { start, .. }
            | Period::DurationZoned { start, .. } => start.clone(),
        }
    }

    /// Get the end as a `DateTime` (when jiff feature is enabled).
    ///
    /// For duration-based periods, calculates the end by adding the duration to the start.
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn end(&self) -> DateTime {
        match self {
            Period::ExplicitUtc { end, .. }
            | Period::ExplicitFloating { end, .. }
            | Period::ExplicitZoned { end, .. } => end.clone(),
            Period::DurationUtc {
                start, duration, ..
            } => {
                let start_dt = start.civil_date_time().unwrap_or_default();
                let end_dt = add_duration(start_dt, duration);
                DateTime::Utc {
                    date: end_dt.date().into(),
                    time: end_dt.time().into(),
                }
            }
            Period::DurationFloating {
                start, duration, ..
            } => {
                let start_dt = start.civil_date_time().unwrap_or_default();
                let end_dt = add_duration(start_dt, duration);
                DateTime::Floating {
                    date: end_dt.date().into(),
                    time: end_dt.time().into(),
                }
            }
            Period::DurationZoned {
                start,
                duration,
                #[cfg(feature = "jiff")]
                tz_jiff,
                ..
            } => {
                let start_dt = start.civil_date_time().unwrap_or_default();
                let end_dt = add_duration(start_dt, duration);
                DateTime::Zoned {
                    date: end_dt.date().into(),
                    time: end_dt.time().into(),
                    #[cfg(feature = "jiff")]
                    tz_jiff: tz_jiff.clone(),
                }
            }
        }
    }

    /// Get the start date and time as `jiff::civil::DateTime` (when jiff feature is enabled).
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn start_civil(&self) -> jiff::civil::DateTime {
        // SAFETY: Period never contains Date-only DateTimeValue
        self.start().civil_date_time().unwrap_or_default()
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

impl<'src> TryFrom<Value<Segments<'src>>> for Period<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(value: Value<Segments<'src>>) -> Result<Self, Self::Error> {
        let span = value.span();
        match value {
            Value::Period { mut values, .. } if values.len() == 1 => {
                let value_period = values.pop().unwrap();
                match value_period {
                    // Both start and end have the same UTC flag (guaranteed by parser)
                    ValuePeriod::Explicit { start, end } if start.time.utc => {
                        Ok(Period::ExplicitUtc {
                            start: DateTime::Utc {
                                date: start.date,
                                time: start.time.into(),
                            },
                            end: DateTime::Utc {
                                date: end.date,
                                time: end.time.into(),
                            },
                        })
                    }
                    ValuePeriod::Explicit { start, end } => Ok(Period::ExplicitFloating {
                        start: DateTime::Floating {
                            date: start.date,
                            time: start.time.into(),
                        },
                        end: DateTime::Floating {
                            date: end.date,
                            time: end.time.into(),
                        },
                    }),
                    ValuePeriod::Duration { start, duration } => {
                        // Only positive durations are valid for periods
                        if !matches!(duration, ValueDuration::DateTime { positive: true, .. })
                            && !matches!(duration, ValueDuration::Week { positive: true, .. })
                        {
                            return Err(vec![TypedError::PropertyInvalidValue {
                                property: PropertyKind::FreeBusy,
                                value: "Duration must be positive for periods".to_string(),
                                span,
                            }]);
                        }

                        if start.time.utc {
                            Ok(Period::DurationUtc {
                                start: DateTime::Utc {
                                    date: start.date,
                                    time: start.time.into(),
                                },
                                duration,
                            })
                        } else {
                            Ok(Period::DurationFloating {
                                start: DateTime::Floating {
                                    date: start.date,
                                    time: start.time.into(),
                                },
                                duration,
                            })
                        }
                    }
                }
            }
            _ => Err(vec![TypedError::ValueTypeDisallowed {
                property: PropertyKind::FreeBusy,
                value_type: value.kind().into(),
                expected_types: &[ValueType::Period],
                span,
            }]),
        }
    }
}

impl Period<Segments<'_>> {
    /// Convert borrowed `Period` to owned `Period`
    #[must_use]
    pub fn to_owned(&self) -> Period<String> {
        match self {
            Period::ExplicitUtc { start, end } => Period::ExplicitUtc {
                start: start.clone(),
                end: end.clone(),
            },
            Period::ExplicitFloating { start, end } => Period::ExplicitFloating {
                start: start.clone(),
                end: end.clone(),
            },
            Period::ExplicitZoned {
                start,
                end,
                tz_id,
                #[cfg(feature = "jiff")]
                tz_jiff,
            } => Period::ExplicitZoned {
                start: start.clone(),
                end: end.clone(),
                tz_id: tz_id.to_owned(),
                #[cfg(feature = "jiff")]
                tz_jiff: tz_jiff.clone(),
            },
            Period::DurationUtc { start, duration } => Period::DurationUtc {
                start: start.clone(),
                duration: *duration,
            },
            Period::DurationFloating { start, duration } => Period::DurationFloating {
                start: start.clone(),
                duration: *duration,
            },
            Period::DurationZoned {
                start,
                duration,
                tz_id,
                #[cfg(feature = "jiff")]
                tz_jiff,
            } => Period::DurationZoned {
                start: start.clone(),
                duration: *duration,
                tz_id: tz_id.to_owned(),
                #[cfg(feature = "jiff")]
                tz_jiff: tz_jiff.clone(),
            },
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

/// UTC-only date and time representation.
///
/// This type is used for properties that MUST be specified in UTC time format
/// per RFC 5545: COMPLETED, CREATED, DTSTAMP, LAST-MODIFIED.
#[derive(Debug, Clone)]
pub struct DateTimeUtc<S: StringStorage> {
    /// Date part
    pub date: ValueDate,
    /// Time part
    pub time: Time,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<S: StringStorage> DateTimeUtc<S> {
    /// Get the date part of this `DateTimeUtc`
    #[must_use]
    pub const fn date(&self) -> ValueDate {
        self.date
    }

    /// Get the time part of this `DateTimeUtc`
    #[must_use]
    pub const fn time(&self) -> Time {
        self.time
    }

    /// Get the combined date and time as `jiff::civil::DateTime` (when jiff feature is enabled).
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn civil_date_time(&self) -> jiff::civil::DateTime {
        jiff::civil::DateTime::from_parts(self.date.civil_date(), self.time.civil_time())
    }

    /// Get the combined date and time as `jiff::Zoned` in UTC (when jiff feature is enabled).
    #[cfg(feature = "jiff")]
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn zoned(&self) -> jiff::Zoned {
        self.civil_date_time()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .expect("UTC timezone should always be valid")
    }

    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

impl DateTimeUtc<Segments<'_>> {
    /// Convert borrowed `DateTimeUtc` to owned `DateTimeUtc`
    #[must_use]
    pub fn to_owned(&self) -> DateTimeUtc<String> {
        DateTimeUtc {
            date: self.date,
            time: self.time,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for DateTimeUtc<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        // Save the span before moving prop
        let span = prop.span;

        // First convert to DateTimeProperty
        let dt = DateTimeProperty::try_from(prop)?;

        // Validate that it's UTC only and extract components
        if !dt.is_utc() {
            return Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::DtStamp, // fallback kind
                value: if dt.is_date_only() {
                    "DATE value not allowed, must be DATE-TIME with UTC time".to_string()
                } else if dt.is_floating() {
                    "Floating time not allowed, must be UTC time (ends with 'Z')".to_string()
                } else {
                    "Timezone reference not allowed, must be UTC time (ends with 'Z')".to_string()
                },
                span,
            }]);
        }

        Ok(Self {
            date: dt.date(),
            time: dt.time().unwrap(), // SAFETY: is_utc() guarantees Some(time)
            x_parameters: dt.x_parameters,
            retained_parameters: dt.retained_parameters,
            span,
        })
    }
}

#[cfg(feature = "jiff")]
impl From<jiff::civil::DateTime> for DateTimeUtc<String> {
    fn from(value: jiff::civil::DateTime) -> Self {
        DateTimeUtc {
            date: Date::from(value.date()),
            time: Time::from(value.time()),
            x_parameters: Vec::new(),
            retained_parameters: Vec::new(),
            span: (),
        }
    }
}

// DateTime wrapper types for specific properties

simple_property_wrapper!(
    /// Date-Time Completed property wrapper (RFC 5545 Section 3.8.2.1)
    ///
    /// This property MUST be specified in UTC time format.
    pub Completed<S> => DateTimeUtc
);

simple_property_wrapper!(
    /// Date-Time End property wrapper (RFC 5545 Section 3.8.2.2)
    pub DtEnd<S> => DateTimeProperty
);

simple_property_wrapper!(
    /// Time Transparency property wrapper (RFC 5545 Section 3.8.2.3)
    pub Due<S> => DateTimeProperty
);

simple_property_wrapper!(
    /// Date-Time Start property wrapper (RFC 5545 Section 3.8.2.4)
    pub DtStart<S> => DateTimeProperty
);

/// Duration (RFC 5545 Section 3.8.2.5)
///
/// This property specifies a duration of time.
#[derive(Debug, Clone)]
pub struct Duration<S: StringStorage> {
    /// Duration value
    pub value: ValueDuration,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Duration<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Duration) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Duration,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        match take_single_value(&PropertyKind::Duration, prop.value) {
            Ok(Value::Duration { values, .. }) if values.is_empty() => {
                Err(vec![TypedError::PropertyMissingValue {
                    property: prop.kind,
                    span: prop.span,
                }])
            }
            Ok(Value::Duration { values, .. }) if values.len() != 1 => {
                Err(vec![TypedError::PropertyInvalidValueCount {
                    property: prop.kind,
                    expected: 1,
                    found: values.len(),
                    span: prop.span,
                }])
            }
            Ok(Value::Duration { mut values, .. }) => Ok(Self {
                value: values.pop().unwrap(), // SAFETY: checked above
                x_parameters,
                retained_parameters,
                span: prop.span,
            }),
            Ok(v) => {
                const EXPECTED: &[ValueType<String>] = &[ValueType::Duration];
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: EXPECTED,
                    found: v.kind().into(),
                    span,
                }])
            }
            Err(e) => Err(e),
        }
    }
}

impl Duration<Segments<'_>> {
    /// Convert borrowed `Duration` to owned `Duration`
    #[must_use]
    pub fn to_owned(&self) -> Duration<String> {
        Duration {
            value: self.value,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> Duration<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

/// Free/Busy Time (RFC 5545 Section 3.8.2.6)
///
/// This property defines one or more free or busy time intervals.
#[derive(Debug, Clone)]
pub struct FreeBusy<S: StringStorage> {
    /// Free/Busy type parameter
    pub fb_type: FreeBusyType<S>,
    /// List of free/busy time periods
    pub values: Vec<Period<S>>,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for FreeBusy<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::FreeBusy) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::FreeBusy,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        // Extract FBTYPE parameter (defaults to BUSY)
        let mut fb_type: FreeBusyType<Segments<'src>> = FreeBusyType::Busy;
        let mut x_parameters = Vec::new();
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::FreeBusyType { value, .. } => {
                    fb_type = value.clone();
                }
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        let (periods, value_span) = match prop.value {
            Value::Period { values, span: _ } if values.is_empty() => {
                return Err(vec![TypedError::PropertyMissingValue {
                    property: prop.kind,
                    span: prop.span,
                }]);
            }
            Value::Period { values, span } => (values, span),
            v => {
                const EXPECTED: &[ValueType<String>] = &[ValueType::Period];
                let span = v.span();
                return Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: EXPECTED,
                    found: v.kind().into(),
                    span,
                }]);
            }
        };

        let mut values = Vec::with_capacity(periods.len());
        let mut errors: Vec<TypedError<'src>> = Vec::new();
        for value_period in periods {
            let period = match value_period {
                // Both start and end have the same UTC flag (guaranteed by parser)
                ValuePeriod::Explicit { start, end } if start.time.utc => Period::ExplicitUtc {
                    start: DateTime::Utc {
                        date: start.date,
                        time: start.time.into(),
                    },
                    end: DateTime::Utc {
                        date: end.date,
                        time: end.time.into(),
                    },
                },
                ValuePeriod::Explicit { start, end } => Period::ExplicitFloating {
                    start: DateTime::Floating {
                        date: start.date,
                        time: start.time.into(),
                    },
                    end: DateTime::Floating {
                        date: end.date,
                        time: end.time.into(),
                    },
                },
                ValuePeriod::Duration { start, duration } => {
                    // Only positive durations are valid for periods
                    if !matches!(duration, ValueDuration::DateTime { positive: true, .. })
                        && !matches!(duration, ValueDuration::Week { positive: true, .. })
                    {
                        // Use the overall value span since individual periods don't have spans
                        errors.push(TypedError::PropertyInvalidValue {
                            property: prop.kind.clone(),
                            value: "Duration must be positive for periods".to_string(),
                            span: value_span,
                        });
                        continue;
                    }

                    if start.time.utc {
                        Period::DurationUtc {
                            start: DateTime::Utc {
                                date: start.date,
                                time: start.time.into(),
                            },
                            duration,
                        }
                    } else {
                        Period::DurationFloating {
                            start: DateTime::Floating {
                                date: start.date,
                                time: start.time.into(),
                            },
                            duration,
                        }
                    }
                }
            };

            values.push(period);
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(FreeBusy {
            fb_type,
            values,
            x_parameters,
            retained_parameters,
            span: prop.span,
        })
    }
}

impl FreeBusy<Segments<'_>> {
    /// Convert borrowed `FreeBusy` to owned `FreeBusy`
    #[must_use]
    pub fn to_owned(&self) -> FreeBusy<String> {
        FreeBusy {
            fb_type: self.fb_type.to_owned(),
            values: self.values.iter().map(Period::to_owned).collect(),
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> FreeBusy<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}

define_prop_value_enum! {
    /// Time transparency value (RFC 5545 Section 3.8.2.7)
    #[derive(Default)]
    pub enum TimeTransparencyValue {
        /// Event blocks time
        #[default]
        Opaque => KW_TRANSP_OPAQUE,
        /// Event does not block time
        Transparent => KW_TRANSP_TRANSPARENT,
    }
}

/// Time transparency for events (RFC 5545 Section 3.8.2.7)
#[derive(Debug, Clone, Default)]
pub struct TimeTransparency<S: StringStorage> {
    /// Transparency value
    pub value: TimeTransparencyValue,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unrecognized / Non-standard parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for TimeTransparency<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Transp) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Transp,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut retained_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::XName(raw) => x_parameters.push(raw),
                p @ Parameter::Unrecognized { .. } => retained_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    retained_parameters.push(p);
                }
            }
        }

        let value_span = prop.value.span();
        let text = take_single_text(&PropertyKind::Transp, prop.value)?;
        let value = text.try_into().map_err(|value| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Transp,
                value: format!("Invalid time transparency value: {value}"),
                span: value_span,
            }]
        })?;

        Ok(TimeTransparency {
            value,
            x_parameters,
            retained_parameters,
            span: prop.span,
        })
    }
}

impl TimeTransparency<Segments<'_>> {
    /// Convert borrowed `TimeTransparency` to owned `TimeTransparency`
    #[must_use]
    pub fn to_owned(&self) -> TimeTransparency<String> {
        TimeTransparency {
            value: self.value,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl<S: StringStorage> TimeTransparency<S> {
    /// Get the span of this property
    #[must_use]
    pub const fn span(&self) -> S::Span {
        self.span
    }
}
