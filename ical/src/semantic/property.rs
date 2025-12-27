// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property and value types for iCalendar semantic components.

use crate::typed::{
    AlarmTriggerRelationship, CalendarUserType, Encoding, ParticipationRole, ParticipationStatus,
    ValueDate, ValueDuration, ValueTime,
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

/// Geographic position
#[derive(Debug, Clone, Copy)]
pub struct Geo {
    /// Latitude
    pub lat: f64,

    /// Longitude
    pub lon: f64,
}

/// URI representation
#[derive(Debug, Clone)]
pub struct Uri {
    /// The URI string
    pub uri: String,
}

/// Text with language and encoding information
#[derive(Debug, Clone)]
pub struct Text {
    /// The actual text content
    pub content: String,

    /// Language code (optional)
    pub language: Option<String>,
}

/// Product identifier that identifies the software that created the iCalendar data
#[derive(Debug, Clone, Default)]
pub struct ProductId {
    /// Company identifier
    pub company: String,

    /// Product identifier
    pub product: String,

    /// Language of the text (optional)
    pub language: Option<String>,
}

/// Organizer information
#[derive(Debug, Clone)]
pub struct Organizer {
    /// Calendar user address (mailto: or other URI)
    pub cal_address: Uri,

    /// Common name (optional)
    pub cn: Option<String>,

    /// Directory entry reference (optional)
    pub dir: Option<Uri>,

    /// Sent by (optional)
    pub sent_by: Option<Uri>,

    /// Language (optional)
    pub language: Option<String>,
}

/// Attendee information
#[derive(Debug, Clone)]
pub struct Attendee {
    /// Calendar user address (mailto: or other URI)
    pub cal_address: Uri,

    /// Common name (optional)
    pub cn: Option<String>,

    /// Participation role
    pub role: ParticipationRole,

    /// Participation status
    pub part_stat: ParticipationStatus,

    /// RSVP expectation
    pub rsvp: Option<bool>,

    /// Whether the attendee is required
    pub cutype: CalendarUserType,

    /// Member of a group (optional)
    pub member: Option<Uri>,

    /// Delegated to (optional)
    pub delegated_to: Option<Uri>,

    /// Delegated from (optional)
    pub delegated_from: Option<Uri>,

    /// Directory entry reference (optional)
    pub dir: Option<Uri>,

    /// Sent by (optional)
    pub sent_by: Option<Uri>,

    /// Language (optional)
    pub language: Option<String>,
}

/// Attachment information
#[derive(Debug, Clone)]
pub struct Attachment {
    /// URI or binary data
    pub value: AttachmentValue,

    /// Format type (optional)
    pub fmt_type: Option<String>,

    /// Encoding (optional)
    pub encoding: Option<Encoding>,
}

/// Trigger for alarms
#[derive(Debug, Clone)]
pub struct Trigger {
    /// When to trigger (relative or absolute)
    pub value: TriggerValue,

    /// Related parameter for relative triggers
    pub related: Option<AlarmTriggerRelationship>,
}

/// Trigger value (relative duration or absolute date/time)
#[derive(Debug, Clone)]
pub enum TriggerValue {
    /// Relative duration before/after the event
    Duration(ValueDuration),

    /// Absolute date/time
    DateTime(DateTime),
}

/// Timezone offset
#[derive(Debug, Clone, Copy)]
pub struct TimeZoneOffset {
    /// Whether the offset is positive
    pub positive: bool,

    /// Hours
    pub hours: u8,

    /// Minutes
    pub minutes: u8,
}

/// Classification of calendar data
#[derive(Debug, Clone, Copy)]
pub enum Classification {
    /// Public classification
    Public,

    /// Private classification
    Private,

    /// Confidential classification
    Confidential,
    // /// Custom classification
    // Custom(String),
}

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

#[cfg(feature = "jiff")]
fn apply_duration(start: jiff::civil::DateTime, duration: &ValueDuration) -> jiff::civil::DateTime {
    use crate::typed::ValueDuration;

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
        self.end().civil_date_time().unwrap_or_default() // SAFETY: end() never returns Date-only
    }
}

/// Attachment value (URI or binary)
#[derive(Debug, Clone)]
pub enum AttachmentValue {
    /// URI reference
    Uri(Uri),

    /// Binary data
    Binary(Vec<u8>),
}
