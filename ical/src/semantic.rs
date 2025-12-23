// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides strongly-typed structures that represent the semantic
//! meaning of iCalendar data parsed from the raw syntax. These types follow
//! the RFC 5545 specification and provide a convenient API for working with
//! calendar data without dealing with string parsing and validation.

mod analysis;
mod enums;
pub mod icalendar;
mod properties;
mod valarm;
mod vevent;
mod vfreebusy;
mod vjournal;
mod vtimezone;
mod vtodo;

// Re-export public types from the analysis submodule
pub use analysis::{
    SemanticError, find_parameter, find_properties, find_property, get_language, get_single_value,
    get_tzid, parse_cal_address, semantic_analysis, value_to_date_time, value_to_duration,
    value_to_int, value_to_string,
};
pub use enums::{AttachmentValue, Classification, Period};
pub use icalendar::{ICalendar, MethodType, parse_icalendar};
pub use properties::{
    Attachment, Attendee, DateTime, Duration, Geo, Organizer, ProductId, Text, TimeZoneOffset,
    Trigger, TriggerValue, Uri,
};
pub use valarm::VAlarm;
pub use vevent::{EventStatus, TimeTransparency, VEvent};
pub use vfreebusy::VFreeBusy;
pub use vjournal::VJournal;
pub use vtimezone::VTimeZone;
pub use vtodo::VTodo;
