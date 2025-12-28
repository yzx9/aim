// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Period of time type for iCalendar semantic components.

use std::convert::TryFrom;

use crate::semantic::{DateTime, SemanticError};
use crate::typed::{Value, ValueDate, ValueDuration, ValuePeriod, ValueTime, ValueType};

/// Period of time (start-end or start-duration)
///
/// This type separates UTC, floating, and zoned time periods at the type level,
/// ensuring that start and end times always have consistent timezone semantics.
///
/// Per RFC 5545 Section 3.3.9, periods have two forms:
/// - Explicit: start and end date-times
/// - Duration: start date-time and duration
#[derive(Debug, Clone)]
pub enum Period {
    /// Start and end date/time in UTC
    ExplicitUtc {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: ValueTime,
        /// End date
        end_date: ValueDate,
        /// End time
        end_time: ValueTime,
    },

    /// Start and end date/time in floating time (no timezone)
    ExplicitFloating {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: ValueTime,
        /// End date
        end_date: ValueDate,
        /// End time
        end_time: ValueTime,
    },

    /// Start and end date/time with timezone reference
    ExplicitZoned {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: ValueTime,
        /// End date
        end_date: ValueDate,
        /// End time
        end_time: ValueTime,
        /// Timezone ID (same for both start and end)
        tz_id: String,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
    },

    /// Start date/time in UTC with a duration
    DurationUtc {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: ValueTime,
        /// Duration from the start
        duration: ValueDuration,
    },

    /// Start date/time in floating time with a duration
    DurationFloating {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: ValueTime,
        /// Duration from the start
        duration: ValueDuration,
    },

    /// Start date/time with timezone reference and a duration
    DurationZoned {
        /// Start date
        start_date: ValueDate,
        /// Start time
        start_time: ValueTime,
        /// Duration from the start
        duration: ValueDuration,
        /// Start timezone ID
        tz_id: String,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
    },
}

impl TryFrom<&Value<'_>> for Period {
    type Error = SemanticError;

    fn try_from(value: &Value<'_>) -> Result<Self, Self::Error> {
        match value {
            Value::Period(value_period) => match value_period {
                ValuePeriod::Explicit { start, end } => {
                    // Both start and end have the same UTC flag (guaranteed by parser)
                    if start.time.utc {
                        Ok(Period::ExplicitUtc {
                            start_date: start.date,
                            start_time: start.time,
                            end_date: end.date,
                            end_time: end.time,
                        })
                    } else {
                        Ok(Period::ExplicitFloating {
                            start_date: start.date,
                            start_time: start.time,
                            end_date: end.date,
                            end_time: end.time,
                        })
                    }
                }
                ValuePeriod::Duration { start, duration } => {
                    // Only positive durations are valid for periods
                    if !matches!(duration, ValueDuration::DateTime { positive: true, .. })
                        && !matches!(duration, ValueDuration::Week { positive: true, .. })
                    {
                        return Err(SemanticError::InvalidValue {
                            property: crate::typed::PropertyKind::FreeBusy,
                            value: "Duration must be positive for periods".to_string(),
                        });
                    }

                    if start.time.utc {
                        Ok(Period::DurationUtc {
                            start_date: start.date,
                            start_time: start.time,
                            duration: *duration,
                        })
                    } else {
                        Ok(Period::DurationFloating {
                            start_date: start.date,
                            start_time: start.time,
                            duration: *duration,
                        })
                    }
                }
            },
            _ => Err(SemanticError::ExpectedType {
                property: crate::typed::PropertyKind::FreeBusy,
                expected: ValueType::Period,
            }),
        }
    }
}

#[cfg(feature = "jiff")]
fn apply_duration(start: jiff::civil::DateTime, duration: &ValueDuration) -> jiff::civil::DateTime {
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

impl Period {
    /// Get the timezone ID if this is a zoned period
    #[must_use]
    pub fn tz_id(&self) -> Option<&str> {
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
    pub fn timezone(&self) -> Option<&jiff::tz::TimeZone> {
        match self {
            Period::ExplicitZoned { tz_jiff: tz, .. }
            | Period::DurationZoned { tz_jiff: tz, .. } => Some(tz),
            _ => None,
        }
    }

    /// Get the start as a `DateTime` (when jiff feature is enabled).
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn start(&self) -> DateTime {
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
                tz_jiff,
                ..
            }
            | Period::DurationZoned {
                start_date,
                start_time,
                tz_id,
                tz_jiff,
                ..
            } => DateTime::Zoned {
                date: *start_date,
                time: *start_time,
                tz_id: tz_id.clone(),
                tz_jiff: tz_jiff.clone(),
            },
        }
    }

    /// Get the end as a `DateTime` (when jiff feature is enabled).
    ///
    /// For duration-based periods, calculates the end by adding the duration to the start.
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn end(&self) -> DateTime {
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
                let end = apply_duration(start, duration);
                DateTime::Utc {
                    date: ValueDate {
                        year: end.year(),
                        month: end.month(),
                        day: end.day(),
                    },
                    time: ValueTime::new(end.hour(), end.minute(), end.second(), true),
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
                let end = apply_duration(start, duration);
                DateTime::Floating {
                    date: ValueDate {
                        year: end.year(),
                        month: end.month(),
                        day: end.day(),
                    },
                    time: ValueTime::new(end.hour(), end.minute(), end.second(), false),
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
                let end = apply_duration(start, duration);
                DateTime::Zoned {
                    date: ValueDate {
                        year: end.year(),
                        month: end.month(),
                        day: end.day(),
                    },
                    time: ValueTime::new(end.hour(), end.minute(), end.second(), false),
                    tz_id: tz_id.clone(),
                    tz_jiff: tz_jiff.clone(),
                }
            }
        }
    }

    /// Get the start date and time as `jiff::civil::DateTime` (when jiff feature is enabled).
    #[cfg(feature = "jiff")]
    #[must_use]
    pub fn start_civil(&self) -> jiff::civil::DateTime {
        match self {
            Period::ExplicitUtc {
                start_date,
                start_time,
                ..
            }
            | Period::ExplicitFloating {
                start_date,
                start_time,
                ..
            }
            | Period::ExplicitZoned {
                start_date,
                start_time,
                ..
            }
            | Period::DurationUtc {
                start_date,
                start_time,
                ..
            }
            | Period::DurationFloating {
                start_date,
                start_time,
                ..
            }
            | Period::DurationZoned {
                start_date,
                start_time,
                ..
            } => {
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
