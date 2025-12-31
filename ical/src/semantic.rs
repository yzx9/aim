// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides strongly-typed structures that represent the semantic
//! meaning of iCalendar data parsed from the raw syntax. These types follow
//! the RFC 5545 specification and provide a convenient API for working with
//! calendar data without dealing with string parsing and validation.

mod analysis;
mod icalendar;
mod property_util;
mod valarm;
mod vevent;
mod vfreebusy;
mod vjournal;
mod vtimezone;
mod vtodo;

// Re-export public types from the analysis submodule
pub use analysis::{SemanticError, semantic_analysis};

// Re-export from property module (types moved during reorganization)
pub use crate::property::{
    Action, Attachment, AttachmentValue, Attendee, CalendarScale, Classification, DateTime,
    EventStatus, Geo, JournalStatus, Method, Organizer, Period, ProductId, Text, Time,
    TimeTransparency, TimeZoneOffset, TodoStatus, Trigger, TriggerValue, Version,
};

// Re-export component types
pub use icalendar::{CalendarComponent, ICalendar};
pub use valarm::VAlarm;
pub use vevent::VEvent;
pub use vfreebusy::VFreeBusy;
pub use vjournal::VJournal;
pub use vtimezone::{TimeZoneObservance, VTimeZone};
pub use vtodo::VTodo;

// Re-export helper functions from property_common
pub use property_util::{
    take_single_floating_date_time, take_single_int, take_single_text, take_single_value,
    take_single_value_string, value_to_floating_date_time,
};
