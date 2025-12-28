// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides strongly-typed structures that represent the semantic
//! meaning of iCalendar data parsed from the raw syntax. These types follow
//! the RFC 5545 specification and provide a convenient API for working with
//! calendar data without dealing with string parsing and validation.

mod analysis;
mod icalendar;
mod property_attendee;
mod property_common;
mod property_datetime;
mod property_period;
mod property_util;
mod valarm;
mod vevent;
mod vfreebusy;
mod vjournal;
mod vtimezone;
mod vtodo;

// Re-export public types from the analysis submodule
pub use analysis::{SemanticError, semantic_analysis};
pub use icalendar::{
    CalendarComponent, CalendarScaleType, ICalendar, MethodType, ProductId, VersionType,
};
pub use property_attendee::Attendee;
pub use property_common::{
    Attachment, AttachmentValue, Classification, Geo, Organizer, Text, Trigger, TriggerValue,
    Uri,
};
pub use property_datetime::DateTime;
pub use property_period::Period;
pub use valarm::VAlarm;
pub use vevent::{EventStatus, TimeTransparency, VEvent};
pub use vfreebusy::VFreeBusy;
pub use vjournal::{JournalStatus, VJournal};
pub use vtimezone::{TimeZoneObservance, TimeZoneOffset, VTimeZone};
pub use vtodo::{TodoStatus, VTodo};
