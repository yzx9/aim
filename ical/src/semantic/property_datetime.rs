// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Date and time representation for iCalendar semantic components.

use std::convert::TryFrom;

use crate::semantic::SemanticError;
use crate::syntax::SpannedSegments;
use crate::parameter::{TypedParameter, TypedParameterKind};
use crate::typed::TypedProperty;
use crate::typed::Value;
use crate::value::{ValueDate, ValueTime};

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

impl<'src> TryFrom<TypedProperty<'src>> for DateTime<'src> {
    type Error = Vec<SemanticError>;

    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        let Some(value) = prop.values.first() else {
            return Err(vec![SemanticError::MissingValue {
                property: prop.kind,
            }]);
        };

        // Get TZID parameter
        let mut tz_id = None;
        #[cfg(feature = "jiff")]
        let mut tz_jiff = None;
        for param in prop.parameters {
            #[allow(clippy::single_match)]
            match param {
                TypedParameter::TimeZoneIdentifier {
                    value,
                    #[cfg(feature = "jiff")]
                    tz,
                    ..
                } => match tz_id {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::TimeZoneIdentifier,
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

                _ => Err(vec![SemanticError::InvalidValue {
                    property: prop.kind,
                    value: "Expected date-time value".to_string(),
                }]),
            }
        } else {
            match value {
                Value::Date(date) => Ok(DateTime::Date { date: *date }),
                Value::DateTime(dt) if dt.time.utc => Ok(DateTime::Utc {
                    date: dt.date,
                    time: dt.time.into(),
                }),
                Value::DateTime(dt) => Ok(DateTime::Floating {
                    date: dt.date,
                    time: dt.time.into(),
                }),
                _ => Err(vec![SemanticError::InvalidValue {
                    property: crate::typed::PropertyKind::DtStart, // Default fallback
                    value: format!("Expected date or date-time value, got {value:?}"),
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
    #[allow(clippy::cast_possible_wrap)]
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
