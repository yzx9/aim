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

pub mod fmt;

// TODO: Remove deprecated `formatter` module alias in version 0.15.0 (3 versions after 0.12.0)
/// Deprecated alias for [`fmt`] module.
#[deprecated(
    since = "0.12.0",
    note = "Use `fmt` module instead. Will be removed in 0.15.0"
)]
pub mod formatter {
    #[doc(hidden)]
    pub use crate::fmt::*;
}
pub mod keyword;
pub mod ops;
pub mod parameter;
mod parser;
pub mod property;
pub mod semantic;
pub mod string_storage;
pub mod syntax;
pub mod typed;
pub mod value;

pub use crate::parameter::{
    AlarmTriggerRelationship, CalendarUserType, Encoding, FreeBusyType, Parameter, ParameterKind,
    ParticipationRole, ParticipationStatus, RecurrenceIdRange, RelationshipType, ValueType,
};
pub use crate::parser::{ParseError, parse, parse_with_options};
pub use crate::property::{
    Action, ActionValue, Attachment, AttachmentValue, Attendee, CalendarScale, CalendarScaleValue,
    Categories, Classification, ClassificationValue, Comment, Completed, Contact, Created, Date,
    DateTime, DateTimeProperty, DateTimeUtc, Description, DtEnd, DtStamp, DtStart, Due, Duration,
    ExDate, FreeBusy, Geo, LastModified, Location, Method, MethodValue, Organizer, PercentComplete,
    Period, Priority, ProductId, Property, PropertyKind, RDate, RDateValue, RecurrenceId,
    RelatedTo, Repeat, RequestStatus, Resources, Sequence, Status, StatusValue, Summary, Text,
    TextOnly, TextWithLanguage, Time, TimeTransparency, TimeTransparencyValue, Trigger,
    TriggerValue, TzId, TzName, TzOffsetFrom, TzOffsetTo, TzUrl, Uid, UnrecognizedProperty,
    UriProperty, Url, Version, VersionValue, XNameProperty,
};
pub use crate::semantic::{
    CalendarComponent, EventStatus, EventStatusValue, ICalendar, JournalStatus, TimeZoneObservance,
    TodoStatus, TodoStatusValue, VAlarm, VEvent, VFreeBusy, VJournal, VTimeZone, VTodo,
};
pub use crate::string_storage::{Segments, StringStorage};
pub use crate::syntax::ParseOptions;
pub use crate::value::{
    RecurrenceFrequency, Value, ValueDate, ValueDateTime, ValueDuration, ValuePeriod,
    ValueRecurrenceRule, ValueText, ValueTime, ValueUtcOffset, WeekDay, WeekDayNum,
};
