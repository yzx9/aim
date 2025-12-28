// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Date and time representation for iCalendar semantic components.

use std::convert::TryFrom;

use crate::semantic::SemanticError;
use crate::semantic::property_util::find_parameter;
use crate::typed::{
    TypedParameter, TypedParameterKind, TypedProperty, Value, ValueDate, ValueTime,
};

/// Date and time representation
#[derive(Debug, Clone)]
pub enum DateTime {
    /// Date and time without timezone (floating time)
    Floating {
        /// Date part
        date: ValueDate,
        /// Time part
        time: ValueTime,
    },

    /// Date and time with specific timezone
    Zoned {
        /// Date part
        date: ValueDate,
        /// Time part
        time: ValueTime,
        /// Timezone identifier
        tz_id: String,
        /// Cached parsed timezone (available with jiff feature)
        #[cfg(feature = "jiff")]
        tz_jiff: jiff::tz::TimeZone,
    },

    /// Date and time in UTC
    Utc {
        /// Date part
        date: ValueDate,
        /// Time part
        time: ValueTime,
    },

    /// Date-only value
    Date {
        /// Date part
        date: ValueDate,
    },
}

impl DateTime {
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
    pub fn time(&self) -> Option<ValueTime> {
        match self {
            DateTime::Floating { time, .. }
            | DateTime::Zoned { time, .. }
            | DateTime::Utc { time, .. } => Some(*time),
            DateTime::Date { .. } => None,
        }
    }

    /// Get the timezone ID if this is a zoned value
    #[must_use]
    pub fn tz_id(&self) -> Option<&str> {
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

impl TryFrom<&TypedProperty<'_>> for DateTime {
    type Error = SemanticError;

    fn try_from(prop: &TypedProperty<'_>) -> Result<Self, Self::Error> {
        let value = prop.values.first().ok_or(SemanticError::MissingValue {
            property: prop.kind,
        })?;

        // Get TZID parameter
        let tz = find_parameter(&prop.parameters, TypedParameterKind::TimeZoneIdentifier);

        // Try with timezone if available, otherwise fallback to basic conversion
        match tz {
            Some(TypedParameter::TimeZoneIdentifier {
                value: tz_id,
                #[cfg(feature = "jiff")]
                tz,
                ..
            }) => match value {
                Value::DateTime(dt) if dt.time.utc => Ok(DateTime::Utc {
                    date: dt.date,
                    time: dt.time,
                }),

                Value::DateTime(dt) => Ok(DateTime::Zoned {
                    date: dt.date,
                    time: dt.time,
                    tz_id: tz_id.to_string(),
                    #[cfg(feature = "jiff")]
                    tz_jiff: tz.clone(),
                }),

                _ => Err(SemanticError::InvalidValue {
                    property: prop.kind,
                    value: "Expected date-time value".to_string(),
                }),
            },

            _ => match value {
                Value::Date(date) => Ok(DateTime::Date { date: *date }),
                Value::DateTime(dt) if dt.time.utc => Ok(DateTime::Utc {
                    date: dt.date,
                    time: dt.time,
                }),
                Value::DateTime(dt) => Ok(DateTime::Floating {
                    date: dt.date,
                    time: dt.time,
                }),
                _ => Err(SemanticError::InvalidValue {
                    property: crate::typed::PropertyKind::DtStart, // Default fallback
                    value: format!("Expected date or date-time value, got {value:?}"),
                }),
            },
        }
    }
}
