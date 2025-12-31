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
mod parameter;
mod parser;
mod property;
pub mod semantic;
pub mod syntax;
pub mod typed;
mod value;

pub use crate::parser::{ParseError, parse};
pub use crate::semantic::{
    Action, Attachment, AttachmentValue, Attendee, CalendarComponent, CalendarScale,
    Classification, DateTime, EventStatus, Geo, ICalendar, Method, Organizer, Period, ProductId,
    Text, Time, TimeTransparency, TimeZoneOffset, Trigger, TriggerValue, VAlarm, VEvent, VFreeBusy,
    VJournal, VTimeZone, VTodo, Version,
};
pub use crate::value::{
    Day, RecurrenceFrequency, RecurrenceRule, ValueDuration as Duration, WeekDay,
};
