// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property module for iCalendar properties organized by RFC 5545 sections.
//!
//! This module provides property specifications and typed property structures
//! as defined in RFC 5545. Property types are organized by their corresponding
//! RFC 5545 sections for better code organization and maintainability.
//!
//! ## Property Organization
//!
//! - 3.7. Calendar Properties (calendar.rs)
//! - 3.8.1. Descriptive Component Properties (descriptive.rs)
//! - 3.8.2. Date and Time Properties (datetime.rs)
//! - 3.8.3. Time Zone Component Properties (timezone.rs)
//! - 3.8.4. Relationship Component Properties (relationship.rs)
//! - 3.8.5. Recurrence Properties (recurrence.rs)
//! - 3.8.6. Alarm Component Properties (alarm.rs)
//! - 3.8.7. Change Management Component Properties (changemgmt.rs)
//! - 3.8.8. Miscellaneous Properties (miscellaneous.rs)
//!
//! ## Type Safety
//!
//! All property types implement kind validation through:
//! - A `kind()` method returning the corresponding `PropertyKind`
//! - Type checking in `TryFrom<ParsedProperty>` implementations that verify
//!   the property kind matches the expected type
//! - Dedicated wrapper types for specific properties (e.g., `Created`, `DtStart`, `Summary`)
//!
//! This ensures that properties are correctly typed during parsing and prevents
//! invalid property assignments.

#[macro_use]
mod util;

// Property type modules organized by RFC 5545 sections
mod alarm;
mod calendar;
mod changemgmt;
mod datetime;
mod descriptive;
mod kind;
mod miscellaneous;
mod recurrence;
mod relationship;
mod timezone;

pub use alarm::{Action, Repeat, Trigger, TriggerValue};
pub use calendar::{CalendarScale, Method, ProductId, Version};
pub use changemgmt::{Created, DtStamp, LastModified, Sequence};
pub use datetime::{
    Completed, DateTime, DtEnd, DtStart, Due, Duration, FreeBusy, Period, Time, TimeTransparency,
};
pub use descriptive::{
    Attachment, AttachmentValue, Categories, Classification, Comment, Description, Geo, Location,
    PercentComplete, Priority, Resources, Status, Summary,
};
pub use kind::PropertyKind;
pub use miscellaneous::RequestStatus;
pub use recurrence::{ExDate, ExDateValue, RDate, RDateValue};
pub use relationship::{Attendee, Contact, Organizer, RecurrenceId, RelatedTo, Uid, Url};
pub use timezone::{TzId, TzName, TzOffsetFrom, TzOffsetTo, TzUrl};
pub use util::{Text, Texts};

use crate::typed::{ParsedProperty, TypedError};
use crate::value::RecurrenceRule;

/// Unified property enum with one variant per `PropertyKind`.
///
/// Each variant holds the corresponding semantic type from the property modules,
/// providing type-safe access to parsed property values.
///
/// # Example
///
/// ```ignore
/// match property {
///     Property::Summary(text) => println!("Summary: {}", text.content.resolve()),
///     Property::DtStart(dt) => println!("Starts at: {:?}", dt),
///     Property::Attendee(attendee) => println!("Attendee: {:?}", attendee.cal_address),
/// }
/// ```
#[derive(Debug, Clone)]
pub enum Property<'src> {
    // Section 3.7 - Calendar Properties
    /// 3.7.1 Calendar Scale
    CalScale(CalendarScale),

    /// 3.7.2 Method
    Method(Method),

    /// 3.7.3 Product Identifier
    ProdId(ProductId),

    /// 3.7.4 Version
    Version(Version),

    // Section 3.8.1 - Descriptive Component Properties
    /// 3.8.1.1 Attachment
    Attach(Attachment<'src>),

    /// 3.8.1.2 Categories (multi-valued text)
    Categories(Categories<'src>),

    /// 3.8.1.3 Classification
    Class(Classification),

    /// 3.8.1.4 Comment
    Comment(Comment<'src>),

    /// 3.8.1.5 Description
    Description(Description<'src>),

    /// 3.8.1.6 Geographic Position
    Geo(Geo),

    /// 3.8.1.7 Location
    Location(Location<'src>),

    /// 3.8.1.8 Percent Complete
    PercentComplete(PercentComplete),

    /// 3.8.1.9 Priority
    Priority(Priority),

    /// 3.8.1.10 Resources (multi-valued text)
    Resources(Resources<'src>),

    /// 3.8.1.11 Status
    Status(Status),

    /// 3.8.1.12 Summary
    Summary(Summary<'src>),

    // Section 3.8.2 - Date and Time Properties
    /// 3.8.2.1 Date-Time Completed
    Completed(Completed<'src>),

    /// 3.8.2.2 Date-Time End
    DtEnd(DtEnd<'src>),

    /// 3.8.2.3 Date-Time Due
    Due(Due<'src>),

    /// 3.8.2.4 Date-Time Start
    DtStart(DtStart<'src>),

    /// 3.8.2.5 Duration
    Duration(Duration),

    /// 3.8.2.6 Free/Busy Time
    FreeBusy(FreeBusy<'src>),

    /// 3.8.2.7 Time Transparency
    Transp(TimeTransparency),

    // Section 3.8.3 - Time Zone Component Properties
    /// 3.8.3.1 Time Zone Identifier
    TzId(TzId<'src>),

    /// 3.8.3.2 Time Zone Name
    TzName(TzName<'src>),

    /// 3.8.3.3 Time Zone Offset From
    TzOffsetFrom(TzOffsetFrom),

    /// 3.8.3.4 Time Zone Offset To
    TzOffsetTo(TzOffsetTo),

    /// 3.8.3.5 Time Zone URL
    TzUrl(TzUrl<'src>),

    // Section 3.8.4 - Component Relationship Properties
    /// 3.8.4.1 Attendee
    Attendee(Attendee<'src>),

    /// 3.8.4.2 Contact
    Contact(Contact<'src>),

    /// 3.8.4.3 Organizer
    Organizer(Organizer<'src>),

    /// 3.8.4.4 Recurrence ID
    RecurrenceId(RecurrenceId<'src>),

    /// 3.8.4.5 Related To
    RelatedTo(RelatedTo<'src>),

    /// 3.8.4.6 URL
    Url(Url<'src>),

    /// 3.8.4.7 Unique Identifier
    Uid(Uid<'src>),

    // Section 3.8.5 - Recurrence Properties
    /// 3.8.5.1 Exception Date-Times
    ExDate(ExDate<'src>),

    /// 3.8.5.2 Recurrence Date-Times
    RDate(RDate<'src>),

    /// 3.8.5.3 Recurrence Rule
    RRule(RecurrenceRule),

    // Section 3.8.6 - Alarm Component Properties
    /// 3.8.6.1 Action
    Action(Action),

    /// 3.8.6.2 Repeat Count
    Repeat(Repeat),

    /// 3.8.6.3 Trigger
    Trigger(Trigger<'src>),

    // Section 3.8.7 - Change Management Properties
    /// 3.8.7.1 Date-Time Created
    Created(Created<'src>),

    /// 3.8.7.2 Date-Time Stamp
    DtStamp(DtStamp<'src>),

    /// 3.8.7.3 Last Modified
    LastModified(LastModified<'src>),

    /// 3.8.7.4 Sequence Number
    Sequence(Sequence),

    // Section 3.8.8 - Miscellaneous Properties
    /// 3.8.8.3 Request Status
    RequestStatus(RequestStatus<'src>),
}

impl<'src> TryFrom<ParsedProperty<'src>> for Property<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        match prop.kind {
            // Section 3.7 - Calendar Properties
            PropertyKind::CalScale => CalendarScale::try_from(prop).map(Property::CalScale),
            PropertyKind::Method => Method::try_from(prop).map(Property::Method),
            PropertyKind::ProdId => ProductId::try_from(prop).map(Property::ProdId),
            PropertyKind::Version => Version::try_from(prop).map(Property::Version),

            // Section 3.8.1 - Descriptive Component Properties
            PropertyKind::Attach => Attachment::try_from(prop).map(Property::Attach),
            PropertyKind::Categories => Categories::try_from(prop).map(Property::Categories),
            PropertyKind::Class => Classification::try_from(prop).map(Property::Class),
            PropertyKind::Comment => Comment::try_from(prop).map(Property::Comment),
            PropertyKind::Description => Description::try_from(prop).map(Property::Description),
            PropertyKind::Geo => Geo::try_from(prop).map(Property::Geo),
            PropertyKind::Location => Location::try_from(prop).map(Property::Location),
            PropertyKind::PercentComplete => {
                PercentComplete::try_from(prop).map(Property::PercentComplete)
            }
            PropertyKind::Priority => Priority::try_from(prop).map(Property::Priority),
            PropertyKind::Resources => Resources::try_from(prop).map(Property::Resources),
            PropertyKind::Status => Status::try_from(prop).map(Property::Status),
            PropertyKind::Summary => Summary::try_from(prop).map(Property::Summary),

            // Section 3.8.2 - Date and Time Properties
            PropertyKind::Completed => Completed::try_from(prop).map(Property::Completed),
            PropertyKind::DtEnd => DtEnd::try_from(prop).map(Property::DtEnd),
            PropertyKind::Due => Due::try_from(prop).map(Property::Due),
            PropertyKind::DtStart => DtStart::try_from(prop).map(Property::DtStart),
            PropertyKind::Duration => Duration::try_from(prop).map(Property::Duration),
            PropertyKind::FreeBusy => FreeBusy::try_from(prop).map(Property::FreeBusy),
            PropertyKind::Transp => TimeTransparency::try_from(prop).map(Property::Transp),

            // Section 3.8.3 - Time Zone Component Properties
            PropertyKind::TzId => TzId::try_from(prop).map(Property::TzId),
            PropertyKind::TzName => TzName::try_from(prop).map(Property::TzName),
            PropertyKind::TzOffsetFrom => TzOffsetFrom::try_from(prop).map(Property::TzOffsetFrom),
            PropertyKind::TzOffsetTo => TzOffsetTo::try_from(prop).map(Property::TzOffsetTo),
            PropertyKind::TzUrl => TzUrl::try_from(prop).map(Property::TzUrl),

            // Section 3.8.4 - Component Relationship Properties
            PropertyKind::Attendee => Attendee::try_from(prop).map(Property::Attendee),
            PropertyKind::Contact => Contact::try_from(prop).map(Property::Contact),
            PropertyKind::Organizer => Organizer::try_from(prop).map(Property::Organizer),
            PropertyKind::RecurrenceId => RecurrenceId::try_from(prop).map(Property::RecurrenceId),
            PropertyKind::RelatedTo => RelatedTo::try_from(prop).map(Property::RelatedTo),
            PropertyKind::Url => Url::try_from(prop).map(Property::Url),
            PropertyKind::Uid => Uid::try_from(prop).map(Property::Uid),

            // Section 3.8.5 - Recurrence Properties
            PropertyKind::ExDate => ExDate::try_from(prop).map(Property::ExDate),
            PropertyKind::RDate => RDate::try_from(prop).map(Property::RDate),
            PropertyKind::RRule => {
                // TODO: Parse RRULE from text (Value::Text)
                // For now, return an error since RecurrenceRule parsing is not yet implemented
                Err(vec![TypedError::PropertyInvalidValue {
                    property: prop.kind,
                    value: "RRULE parsing not yet implemented".to_string(),
                    span: prop.span,
                }])
            }

            // Section 3.8.6 - Alarm Component Properties
            PropertyKind::Action => Action::try_from(prop).map(Property::Action),
            PropertyKind::Repeat => Repeat::try_from(prop).map(Property::Repeat),
            PropertyKind::Trigger => Trigger::try_from(prop).map(Property::Trigger),

            // Section 3.8.7 - Change Management Properties
            PropertyKind::Created => Created::try_from(prop).map(Property::Created),
            PropertyKind::DtStamp => DtStamp::try_from(prop).map(Property::DtStamp),
            PropertyKind::LastModified => LastModified::try_from(prop).map(Property::LastModified),
            PropertyKind::Sequence => Sequence::try_from(prop).map(Property::Sequence),

            // Section 3.8.8 - Miscellaneous Properties
            PropertyKind::RequestStatus => {
                RequestStatus::try_from(prop).map(Property::RequestStatus)
            }
        }
    }
}

impl Property<'_> {
    /// Returns the `PropertyKind` for this property
    #[must_use]
    pub const fn kind(&self) -> PropertyKind {
        match self {
            // Section 3.7 - Calendar Properties
            Self::CalScale(_) => PropertyKind::CalScale,
            Self::Method(_) => PropertyKind::Method,
            Self::ProdId(_) => PropertyKind::ProdId,
            Self::Version(_) => PropertyKind::Version,

            // Section 3.8.1 - Descriptive Component Properties
            Self::Attach(_) => PropertyKind::Attach,
            Self::Categories(_) => PropertyKind::Categories,
            Self::Class(_) => PropertyKind::Class,
            Self::Comment(_) => PropertyKind::Comment,
            Self::Description(_) => PropertyKind::Description,
            Self::Geo(_) => PropertyKind::Geo,
            Self::Location(_) => PropertyKind::Location,
            Self::PercentComplete(_) => PropertyKind::PercentComplete,
            Self::Priority(_) => PropertyKind::Priority,
            Self::Resources(_) => PropertyKind::Resources,
            Self::Status(_) => PropertyKind::Status,
            Self::Summary(_) => PropertyKind::Summary,

            // Section 3.8.2 - Date and Time Properties
            Self::Completed(_) => PropertyKind::Completed,
            Self::DtEnd(_) => PropertyKind::DtEnd,
            Self::Due(_) => PropertyKind::Due,
            Self::DtStart(_) => PropertyKind::DtStart,
            Self::Duration(_) => PropertyKind::Duration,
            Self::FreeBusy(_) => PropertyKind::FreeBusy,
            Self::Transp(_) => PropertyKind::Transp,

            // Section 3.8.3 - Time Zone Component Properties
            Self::TzId(_) => PropertyKind::TzId,
            Self::TzName(_) => PropertyKind::TzName,
            Self::TzOffsetFrom(_) => PropertyKind::TzOffsetFrom,
            Self::TzOffsetTo(_) => PropertyKind::TzOffsetTo,
            Self::TzUrl(_) => PropertyKind::TzUrl,

            // Section 3.8.4 - Component Relationship Properties
            Self::Attendee(_) => PropertyKind::Attendee,
            Self::Contact(_) => PropertyKind::Contact,
            Self::Organizer(_) => PropertyKind::Organizer,
            Self::RecurrenceId(_) => PropertyKind::RecurrenceId,
            Self::RelatedTo(_) => PropertyKind::RelatedTo,
            Self::Url(_) => PropertyKind::Url,
            Self::Uid(_) => PropertyKind::Uid,

            // Section 3.8.5 - Recurrence Properties
            Self::ExDate(_) => PropertyKind::ExDate,
            Self::RDate(_) => PropertyKind::RDate,
            Self::RRule(_) => PropertyKind::RRule,

            // Section 3.8.6 - Alarm Component Properties
            Self::Action(_) => PropertyKind::Action,
            Self::Repeat(_) => PropertyKind::Repeat,
            Self::Trigger(_) => PropertyKind::Trigger,

            // Section 3.8.7 - Change Management Properties
            Self::Created(_) => PropertyKind::Created,
            Self::DtStamp(_) => PropertyKind::DtStamp,
            Self::LastModified(_) => PropertyKind::LastModified,
            Self::Sequence(_) => PropertyKind::Sequence,

            // Section 3.8.8 - Miscellaneous Properties
            Self::RequestStatus(_) => PropertyKind::RequestStatus,
        }
    }
}
