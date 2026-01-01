// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
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
pub mod parameter;
mod parser;
pub mod property;
pub mod semantic;
pub mod syntax;
pub mod typed;
pub mod value;

pub use crate::parser::{ParseError, parse};
pub use crate::property::{
    Action, Attachment, AttachmentValue, Attendee, CalendarScale, Categories, Classification,
    Comment, Completed, Contact, Created, DateTime, Description, DtEnd, DtStamp, DtStart, Due,
    Duration, ExDate, ExDateValue, FreeBusy, Geo, LastModified, Location, Method, Organizer,
    PercentComplete, Period, Priority, ProductId, RDateValue, RecurrenceId, RelatedTo, Repeat,
    RequestStatus, Resources, Sequence, Status, Summary, Text, Time, TimeTransparency, Trigger,
    TriggerValue, TzId, TzName, TzOffsetFrom, TzOffsetTo, TzUrl, Uid, Url, Version,
};
pub use crate::semantic::{CalendarComponent, ICalendar, VEvent, VFreeBusy, VJournal, VTodo};
pub use crate::value::{
    Day, RecurrenceFrequency, RecurrenceRule, ValueDate, ValueDateTime, ValueDuration,
    ValueExpected, ValuePeriod, ValueText, ValueTime, ValueUtcOffset, WeekDay,
};
