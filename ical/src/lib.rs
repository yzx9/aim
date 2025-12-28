// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parse and represent iCalendar components and properties.

#![warn(
    trivial_casts,
    trivial_numeric_casts,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    clippy::dbg_macro,
    clippy::indexing_slicing,
    clippy::pedantic
)]
// Allow certain clippy lints that are too restrictive for this crate
#![allow(
    clippy::option_option,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::match_bool
)]

pub mod keyword;
pub mod lexer;
mod parser;
pub mod semantic;
pub mod syntax;
pub mod typed;

pub use crate::parser::{ParseError, parse};
pub use crate::semantic::{
    Attachment, AttachmentValue, Attendee, CalendarComponent, CalendarScaleType, Classification,
    DateTime, EventStatus, Geo, ICalendar, MethodType, Organizer, Period, ProductId, Text,
    TimeTransparency, TimeZoneOffset, Trigger, TriggerValue, Uri, VAlarm, VEvent, VFreeBusy,
    VJournal, VTimeZone, VTodo, VersionType,
};
pub use crate::typed::{
    Day, RecurrenceFrequency, RecurrenceRule, ValueDuration as Duration, ValueTime as Time, WeekDay,
};
