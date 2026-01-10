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
pub mod string_storage;
pub mod syntax;
pub mod typed;
pub mod value;

pub use crate::parameter::{
    AlarmTriggerRelationship, CalendarUserType, CalendarUserTypeOwned, CalendarUserTypeRef,
    Encoding, FreeBusyType, FreeBusyTypeOwned, FreeBusyTypeRef, Parameter, ParameterKind,
    ParameterKindOwned, ParameterKindRef, ParameterOwned, ParameterRef, ParticipationRole,
    ParticipationRoleOwned, ParticipationRoleRef, ParticipationStatus, ParticipationStatusOwned,
    ParticipationStatusRef, RecurrenceIdRange, RelationshipType, RelationshipTypeOwned,
    RelationshipTypeRef, ValueType, ValueTypeOwned, ValueTypeRef,
};
pub use crate::parser::{ParseError, parse};
pub use crate::property::{
    Action, ActionValue, Attachment, AttachmentValue, AttachmentValueOwned, AttachmentValueRef,
    Attendee, CalendarScale, CalendarScaleValue, Categories, CategoriesOwned, CategoriesRef,
    Classification, ClassificationValue, Comment, CommentOwned, CommentRef, Completed,
    CompletedOwned, CompletedRef, Contact, ContactOwned, ContactRef, Created, CreatedOwned,
    CreatedRef, DateTime, Description, DescriptionOwned, DescriptionRef, DtEnd, DtEndOwned,
    DtEndRef, DtStamp, DtStampOwned, DtStampRef, DtStart, DtStartOwned, DtStartRef, Due, DueOwned,
    DueRef, Duration, ExDate, ExDateValueOwned, ExDateValueRef, FreeBusy, Geo, LastModified,
    LastModifiedOwned, LastModifiedRef, Location, LocationOwned, LocationRef, Method, MethodValue,
    Organizer, PercentComplete, Period, Priority, ProductId, Property, PropertyKind,
    PropertyKindOwned, PropertyKindRef, PropertyOwned, PropertyRef, RDateValueOwned, RDateValueRef,
    RecurrenceId, RecurrenceIdOwned, RecurrenceIdRef, RelatedTo, RelatedToOwned, RelatedToRef,
    Repeat, RequestStatus, RequestStatusOwned, RequestStatusRef, Resources, ResourcesOwned,
    ResourcesRef, Sequence, Status, StatusValue, Summary, SummaryOwned, SummaryRef, Text, TextOnly,
    TextWithLanguage, Time, TimeTransparency, TimeTransparencyValue, Trigger, TriggerValueOwned,
    TriggerValueRef, TzId, TzIdOwned, TzIdRef, TzName, TzNameOwned, TzNameRef, TzOffsetFrom,
    TzOffsetFromOwned, TzOffsetFromRef, TzOffsetTo, TzOffsetToOwned, TzOffsetToRef, TzUrl,
    TzUrlOwned, TzUrlRef, Uid, UidOwned, UidRef, UriProperty, Url, UrlOwned, UrlRef, Version,
    VersionValue,
};
pub use crate::semantic::{
    CalendarComponent, EventStatus, EventStatusOwned, EventStatusRef, ICalendar, ICalendarOwned,
    ICalendarRef, JournalStatus, TimeZoneObservance, TodoStatus, VAlarm, VAlarmOwned, VAlarmRef,
    VEvent, VEventOwned, VEventRef, VFreeBusy, VFreeBusyOwned, VFreeBusyRef, VJournal,
    VJournalOwned, VJournalRef, VTimeZone, VTimeZoneOwned, VTimeZoneRef, VTodo, VTodoOwned,
    VTodoRef,
};
pub use crate::string_storage::{SpannedSegments, StringStorage};
pub use crate::value::{
    RecurrenceFrequency, Value, ValueDate, ValueDateTime, ValueDuration, ValueOwned, ValuePeriod,
    ValueRecurrenceRule, ValueRef, ValueText, ValueTextOwned, ValueTextRef, ValueTime,
    ValueUtcOffset, WeekDay, WeekDayNum,
};
