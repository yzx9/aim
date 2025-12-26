// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property and value types for iCalendar semantic components.

use crate::semantic::enums::AttachmentValue;
use crate::typed::{
    AlarmTriggerRelationship, CalendarUserType, Encoding, ParticipationRole, ParticipationStatus,
    ValueDate, ValueTime,
};

/// Date and time representation
#[derive(Debug, Clone)]
pub struct DateTime {
    /// Date part
    pub date: ValueDate,

    /// Time part (optional for DATE values)
    pub time: Option<ValueTime>,

    /// Timezone identifier (optional for local time)
    pub tz_id: Option<String>,

    /// Whether this is a DATE-only value
    pub date_only: bool,
}

/// Duration representation
#[derive(Debug, Clone, Copy)]
pub struct Duration {
    /// Whether the duration is positive
    pub positive: bool,

    /// Weeks component
    pub weeks: Option<u32>,

    /// Days component
    pub days: Option<u32>,

    /// Hours component
    pub hours: Option<u32>,

    /// Minutes component
    pub minutes: Option<u32>,

    /// Seconds component
    pub seconds: Option<u32>,
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
    Duration(Duration),

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
