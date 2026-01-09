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
use std::fmt::Display;

use crate::keyword::{KW_TRANSP_OPAQUE, KW_TRANSP_TRANSPARENT};
use crate::parameter::{FreeBusyType, Parameter, ValueType};
use crate::property::PropertyKind;
use crate::property::util::{take_single_text, take_single_value};
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueDate, ValueDuration, ValuePeriod, ValueTime};

/// Date and time representation
#[derive(Debug, Clone)]
pub enum DateTime<S: Clone + Display> {
    /// Date and time without timezone (floating time)
    Floating {
        /// Date part
        date: ValueDate,
        /// Time part
        time: Time,
        /// X-name parameters (custom experimental parameters)
        x_parameters: Vec<Parameter<S>>,
        /// Unrecognized parameters (IANA tokens not recognized by this implementation)
        unrecognized_parameters: Vec<Parameter<S>>,
    },

    /// Date and time with specific timezone
    Zoned {
        /// Date part
        date: ValueDate,
        /// Time part
        time: Time,
        /// Timezone identifier
        tz_id: S,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
        /// X-name parameters (custom experimental parameters)
        x_parameters: Vec<Parameter<S>>,
        /// Unrecognized parameters (IANA tokens not recognized by this implementation)
        unrecognized_parameters: Vec<Parameter<S>>,
    },

    /// Date and time in UTC
    Utc {
        /// Date part
        date: ValueDate,
        /// Time part
        time: Time,
        /// X-name parameters (custom experimental parameters)
        x_parameters: Vec<Parameter<S>>,
        /// Unrecognized parameters (IANA tokens not recognized by this implementation)
        unrecognized_parameters: Vec<Parameter<S>>,
    },

    /// Date-only value
    Date {
        /// Date part
        date: ValueDate,
        /// X-name parameters (custom experimental parameters)
        x_parameters: Vec<Parameter<S>>,
        /// Unrecognized parameters (IANA tokens not recognized by this implementation)
        unrecognized_parameters: Vec<Parameter<S>>,
    },
}

impl<S: Clone + Display> DateTime<S> {
    /// Get the date part of this `DateTime`
    #[must_use]
    pub fn date(&self) -> ValueDate {
        match self {
            DateTime::Floating { date, .. }
            | DateTime::Zoned { date, .. }
            | DateTime::Utc { date, .. }
            | DateTime::Date { date, .. } => *date,
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
    pub fn tz_id(&self) -> Option<&S> {
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
            DateTime::Floating { date, time, .. }
            | DateTime::Zoned { date, time, .. }
            | DateTime::Utc { date, time, .. } => Some(jiff::civil::DateTime::from_parts(
                date.civil_date(),
                time.civil_time(),
            )),
            DateTime::Date { .. } => None,
        }
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for DateTime<SpannedSegments<'src>> {
    type Error = Vec<TypedError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        let value = take_single_value(&prop.kind, prop.value)?;

        let mut errors: Vec<TypedError<'src>> = Vec::new();

        // Get TZID parameter
        let mut tz_id = None;
        #[cfg(feature = "jiff")]
        let mut tz_jiff = None;
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::TimeZoneIdentifier { .. } if tz_id.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
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
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
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
                        Ok(DateTime::Utc {
                            date: dt.date,
                            time: dt.time.into(),
                            x_parameters,
                            unrecognized_parameters,
                        })
                    } else {
                        Ok(DateTime::Zoned {
                            date: dt.date,
                            time: dt.time.into(),
                            tz_id: tz_id_value,
                            #[cfg(feature = "jiff")]
                            tz_jiff: tz_jiff.unwrap(), // SAFETY: set above
                            x_parameters,
                            unrecognized_parameters,
                        })
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
                    Ok(DateTime::Date {
                        date,
                        x_parameters,
                        unrecognized_parameters,
                    })
                }
                Value::DateTime { mut values, .. } if values.len() == 1 => {
                    let dt = values.pop().unwrap();
                    if dt.time.utc {
                        Ok(DateTime::Utc {
                            date: dt.date,
                            time: dt.time.into(),
                            x_parameters,
                            unrecognized_parameters,
                        })
                    } else {
                        Ok(DateTime::Floating {
                            date: dt.date,
                            time: dt.time.into(),
                            x_parameters,
                            unrecognized_parameters,
                        })
                    }
                }
                _ => Err(vec![TypedError::PropertyInvalidValue {
                    property: PropertyKind::DtStart, // Default fallback
                    value: format!("Expected date or date-time value, got {value:?}"),
                    span: value.span(),
                }]),
            }
        }
    }
}

impl DateTime<SpannedSegments<'_>> {
    /// Convert borrowed `DateTime` to owned `DateTime`
    #[must_use]
    pub fn to_owned(&self) -> DateTime<String> {
        match self {
            DateTime::Floating {
                date,
                time,
                x_parameters,
                unrecognized_parameters,
            } => DateTime::Floating {
                date: *date,
                time: *time,
                x_parameters: x_parameters.iter().map(Parameter::to_owned).collect(),
                unrecognized_parameters: unrecognized_parameters
                    .iter()
                    .map(Parameter::to_owned)
                    .collect(),
            },
            #[cfg(feature = "jiff")]
            DateTime::Zoned {
                date,
                time,
                tz_id,
                tz_jiff,
                x_parameters,
                unrecognized_parameters,
            } => DateTime::Zoned {
                date: *date,
                time: *time,
                tz_id: tz_id.to_owned(),
                tz_jiff: tz_jiff.clone(),
                x_parameters: x_parameters.iter().map(Parameter::to_owned).collect(),
                unrecognized_parameters: unrecognized_parameters
                    .iter()
                    .map(Parameter::to_owned)
                    .collect(),
            },
            #[cfg(not(feature = "jiff"))]
            DateTime::Zoned {
                date,
                time,
                tz_id,
                x_parameters,
                unrecognized_parameters,
            } => DateTime::Zoned {
                date: *date,
                time: *time,
                tz_id: tz_id.to_string(),
                x_parameters: x_parameters.iter().map(Parameter::to_owned).collect(),
                unrecognized_parameters: unrecognized_parameters
                    .iter()
                    .map(Parameter::to_owned)
                    .collect(),
            },
            DateTime::Utc {
                date,
                time,
                x_parameters,
                unrecognized_parameters,
            } => DateTime::Utc {
                date: *date,
                time: *time,
                x_parameters: x_parameters.iter().map(Parameter::to_owned).collect(),
                unrecognized_parameters: unrecognized_parameters
                    .iter()
                    .map(Parameter::to_owned)
                    .collect(),
            },
            DateTime::Date {
                date,
                x_parameters,
                unrecognized_parameters,
            } => DateTime::Date {
                date: *date,
                x_parameters: x_parameters.iter().map(Parameter::to_owned).collect(),
                unrecognized_parameters: unrecognized_parameters
                    .iter()
                    .map(Parameter::to_owned)
                    .collect(),
            },
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
pub enum Period<S: Clone + Display> {
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
        tz_id: S,
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
        tz_id: S,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
    },
}

impl<S: Clone + Display> Period<S> {
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
    pub fn start(&self) -> DateTime<S> {
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
                x_parameters: Vec::new(),
                unrecognized_parameters: Vec::new(),
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
                x_parameters: Vec::new(),
                unrecognized_parameters: Vec::new(),
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
                x_parameters: Vec::new(),
                unrecognized_parameters: Vec::new(),
            },
        }
    }

    /// Get the end as a `DateTime` (when jiff feature is enabled).
    ///
    /// For duration-based periods, calculates the end by adding the duration to the start.
    #[cfg(feature = "jiff")]
    #[expect(clippy::missing_panics_doc, clippy::too_many_lines)]
    #[must_use]
    pub fn end(&self) -> DateTime<S> {
        match self {
            Period::ExplicitUtc {
                end_date, end_time, ..
            } => DateTime::Utc {
                date: *end_date,
                time: *end_time,
                x_parameters: Vec::new(),
                unrecognized_parameters: Vec::new(),
            },
            Period::ExplicitFloating {
                end_date, end_time, ..
            } => DateTime::Floating {
                date: *end_date,
                time: *end_time,
                x_parameters: Vec::new(),
                unrecognized_parameters: Vec::new(),
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
                x_parameters: Vec::new(),
                unrecognized_parameters: Vec::new(),
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
                    x_parameters: Vec::new(),
                    unrecognized_parameters: Vec::new(),
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
                    x_parameters: Vec::new(),
                    unrecognized_parameters: Vec::new(),
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
                    x_parameters: Vec::new(),
                    unrecognized_parameters: Vec::new(),
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

impl<'src> TryFrom<Value<SpannedSegments<'src>>> for Period<SpannedSegments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(value: Value<SpannedSegments<'src>>) -> Result<Self, Self::Error> {
        let span = value.span();
        match value {
            Value::Period { mut values, .. } if values.len() == 1 => {
                let value_period = values.pop().unwrap();
                match value_period {
                    // Both start and end have the same UTC flag (guaranteed by parser)
                    ValuePeriod::Explicit { start, end } if start.time.utc => {
                        Ok(Period::ExplicitUtc {
                            start_date: start.date,
                            start_time: start.time.into(),
                            end_date: end.date,
                            end_time: end.time.into(),
                        })
                    }
                    ValuePeriod::Explicit { start, end } => Ok(Period::ExplicitFloating {
                        start_date: start.date,
                        start_time: start.time.into(),
                        end_date: end.date,
                        end_time: end.time.into(),
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
                }
            }
            _ => Err(vec![TypedError::ValueTypeDisallowed {
                property: PropertyKind::FreeBusy,
                value_type: value.into_kind(),
                expected_types: &[ValueType::Period],
                span,
            }]),
        }
    }
}

impl Period<SpannedSegments<'_>> {
    /// Convert borrowed `Period` to owned `Period`
    #[must_use]
    pub fn to_owned(&self) -> Period<String> {
        match self {
            Period::ExplicitUtc {
                start_date,
                start_time,
                end_date,
                end_time,
            } => Period::ExplicitUtc {
                start_date: *start_date,
                start_time: *start_time,
                end_date: *end_date,
                end_time: *end_time,
            },
            Period::ExplicitFloating {
                start_date,
                start_time,
                end_date,
                end_time,
            } => Period::ExplicitFloating {
                start_date: *start_date,
                start_time: *start_time,
                end_date: *end_date,
                end_time: *end_time,
            },
            #[cfg(feature = "jiff")]
            Period::ExplicitZoned {
                start_date,
                start_time,
                end_date,
                end_time,
                tz_id,
                tz_jiff,
            } => Period::ExplicitZoned {
                start_date: *start_date,
                start_time: *start_time,
                end_date: *end_date,
                end_time: *end_time,
                tz_id: tz_id.to_owned(),
                tz_jiff: tz_jiff.clone(),
            },
            #[cfg(not(feature = "jiff"))]
            Period::ExplicitZoned {
                start_date,
                start_time,
                end_date,
                end_time,
                tz_id,
            } => Period::ExplicitZoned {
                start_date: *start_date,
                start_time: *start_time,
                end_date: *end_date,
                end_time: *end_time,
                tz_id: tz_id.to_string(),
            },
            Period::DurationUtc {
                start_date,
                start_time,
                duration,
            } => Period::DurationUtc {
                start_date: *start_date,
                start_time: *start_time,
                duration: *duration,
            },
            Period::DurationFloating {
                start_date,
                start_time,
                duration,
            } => Period::DurationFloating {
                start_date: *start_date,
                start_time: *start_time,
                duration: *duration,
            },
            #[cfg(feature = "jiff")]
            Period::DurationZoned {
                start_date,
                start_time,
                duration,
                tz_id,
                tz_jiff,
            } => Period::DurationZoned {
                start_date: *start_date,
                start_time: *start_time,
                duration: *duration,
                tz_id: tz_id.to_owned(),
                tz_jiff: tz_jiff.clone(),
            },
            #[cfg(not(feature = "jiff"))]
            Period::DurationZoned {
                start_date,
                start_time,
                duration,
                tz_id,
            } => Period::DurationZoned {
                start_date: *start_date,
                start_time: *start_time,
                duration: *duration,
                tz_id: tz_id.concatnate(),
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

// DateTime wrapper types for specific properties

simple_property_wrapper!(
    /// Date-Time Completed property wrapper (RFC 5545 Section 3.8.2.1)
    pub Completed<S> => DateTime

    ref   = pub type CompletedRef;
    owned = pub type CompletedOwned;
);

simple_property_wrapper!(
    /// Date-Time End property wrapper (RFC 5545 Section 3.8.2.2)
    pub DtEnd<S> => DateTime

    ref   = pub type DtEndRef;
    owned = pub type DtEndOwned;
);

simple_property_wrapper!(
    /// Time Transparency property wrapper (RFC 5545 Section 3.8.2.3)
    pub Due<S> => DateTime

    ref   = pub type DueRef;
    owned = pub type DueOwned;
);

simple_property_wrapper!(
    /// Date-Time Start property wrapper (RFC 5545 Section 3.8.2.4)
    pub DtStart<S> => DateTime

    ref   = pub type DtStartRef;
    owned = pub type DtStartOwned;
);

/// Duration (RFC 5545 Section 3.8.2.5)
///
/// This property specifies a duration of time.
#[derive(Debug, Clone)]
pub struct Duration<S: Clone + Display> {
    /// Duration value
    pub value: ValueDuration,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Duration<SpannedSegments<'src>> {
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
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
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
                unrecognized_parameters,
            }),
            Ok(v) => {
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueType::Duration,
                    found: v.into_kind(),
                    span,
                }])
            }
            Err(e) => Err(e),
        }
    }
}

impl Duration<SpannedSegments<'_>> {
    /// Convert borrowed `Duration` to owned `Duration`
    #[must_use]
    pub fn to_owned(&self) -> Duration<String> {
        Duration {
            value: self.value,
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
        }
    }
}

/// Free/Busy Time (RFC 5545 Section 3.8.2.6)
///
/// This property defines one or more free or busy time intervals.
#[derive(Debug, Clone)]
pub struct FreeBusy<S: Clone + Display> {
    /// Free/Busy type parameter
    pub fb_type: FreeBusyType<S>,
    /// List of free/busy time periods
    pub values: Vec<Period<S>>,
    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,
    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for FreeBusy<SpannedSegments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::FreeBusy) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::FreeBusy,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        // Extract FBTYPE parameter (defaults to BUSY)
        let mut fb_type: FreeBusyType<SpannedSegments<'src>> = FreeBusyType::Busy;
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                Parameter::FreeBusyType { value, .. } => {
                    fb_type = value.clone();
                }
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
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
                let span = v.span();
                return Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueType::Period,
                    found: v.into_kind(),
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
                    start_date: start.date,
                    start_time: start.time.into(),
                    end_date: end.date,
                    end_time: end.time.into(),
                },
                ValuePeriod::Explicit { start, end } => Period::ExplicitFloating {
                    start_date: start.date,
                    start_time: start.time.into(),
                    end_date: end.date,
                    end_time: end.time.into(),
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
                            start_date: start.date,
                            start_time: start.time.into(),
                            duration,
                        }
                    } else {
                        Period::DurationFloating {
                            start_date: start.date,
                            start_time: start.time.into(),
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
            unrecognized_parameters,
        })
    }
}

impl FreeBusy<SpannedSegments<'_>> {
    /// Convert borrowed `FreeBusy` to owned `FreeBusy`
    #[must_use]
    pub fn to_owned(&self) -> FreeBusy<String> {
        FreeBusy {
            fb_type: self.fb_type.to_owned(),
            values: self.values.iter().map(Period::to_owned).collect(),
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
        }
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
pub struct TimeTransparency<S: Clone + Display> {
    /// Transparency value
    pub value: TimeTransparencyValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for TimeTransparency<SpannedSegments<'src>> {
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
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
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
            unrecognized_parameters,
        })
    }
}

impl TimeTransparency<SpannedSegments<'_>> {
    /// Convert borrowed `TimeTransparency` to owned `TimeTransparency`
    #[must_use]
    pub fn to_owned(&self) -> TimeTransparency<String> {
        TimeTransparency {
            value: self.value,
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
        }
    }
}
