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
#[derive(Debug, Clone)]
pub enum Period {
    /// Start and end date/time
    DateTimeRange {
        /// Start of the period
        start: DateTime,
        /// End of the period
        end: DateTime,
    },

    /// Start date/time and duration
    Duration {
        /// Start of the period
        start: DateTime,
        /// Duration from the start
        duration: ValueDuration,
    },
}

/// Attachment value (URI or binary)
#[derive(Debug, Clone)]
pub enum AttachmentValue {
    /// URI reference
    Uri(Uri),

    /// Binary data
    Binary(Vec<u8>),
}
