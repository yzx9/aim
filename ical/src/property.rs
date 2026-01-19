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
mod common;

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

pub use alarm::{Action, ActionValue, Repeat, Trigger, TriggerValue};
pub use calendar::{
    CalendarScale, CalendarScaleValue, Method, MethodValue, ProductId, Version, VersionValue,
};
pub use changemgmt::{Created, DtStamp, LastModified, Sequence};
pub use common::{Text, TextOnly, TextWithLanguage, UriProperty};
pub use datetime::{
    Completed, Date, DateTime, DateTimeProperty, DateTimeUtc, DtEnd, DtStart, Due, Duration,
    FreeBusy, Period, Time, TimeTransparency, TimeTransparencyValue,
};
pub use descriptive::{
    Attachment, AttachmentValue, Categories, Classification, ClassificationValue, Comment,
    Description, Geo, Location, PercentComplete, Priority, Resources, Status, StatusValue, Summary,
};
pub use kind::PropertyKind;
pub use miscellaneous::RequestStatus;
pub use recurrence::{ExDate, ExDateValue, RDate, RDateValue, RRule};
pub use relationship::{Attendee, Contact, Organizer, RecurrenceId, RelatedTo, Uid, Url};
pub use timezone::{TzId, TzName, TzOffsetFrom, TzOffsetTo, TzUrl};

use crate::parameter::Parameter;
use crate::string_storage::{Segments, StringStorage};
use crate::typed::{ParsedProperty, TypedError};
use crate::value::Value;

/// Unified property enum with one variant per `PropertyKind`.
///
/// Each variant holds the corresponding semantic type from the property modules,
/// providing type-safe access to parsed property values.
///
/// # Example
///
/// ```ignore
/// match property {
///     Property::Summary(text) => println!("Summary: {}", text.content),
///     Property::DtStart(dt) => println!("Starts at: {:?}", dt),
///     Property::Attendee(attendee) => println!("Attendee: {:?}", attendee.cal_address),
/// }
/// ```
#[derive(Debug, Clone)]
pub enum Property<S: StringStorage> {
    // Section 3.7 - Calendar Properties
    /// 3.7.1 Calendar Scale
    CalScale(CalendarScale<S>),

    /// 3.7.2 Method
    Method(Method<S>),

    /// 3.7.3 Product Identifier
    ProdId(ProductId<S>),

    /// 3.7.4 Version
    Version(Version<S>),

    // Section 3.8.1 - Descriptive Component Properties
    /// 3.8.1.1 Attachment
    Attach(Attachment<S>),

    /// 3.8.1.2 Categories (multi-valued text)
    Categories(Categories<S>),

    /// 3.8.1.3 Classification
    Class(Classification<S>),

    /// 3.8.1.4 Comment
    Comment(Comment<S>),

    /// 3.8.1.5 Description
    Description(Description<S>),

    /// 3.8.1.6 Geographic Position
    Geo(Geo<S>),

    /// 3.8.1.7 Location
    Location(Location<S>),

    /// 3.8.1.8 Percent Complete
    PercentComplete(PercentComplete<S>),

    /// 3.8.1.9 Priority
    Priority(Priority<S>),

    /// 3.8.1.10 Resources (multi-valued text)
    Resources(Resources<S>),

    /// 3.8.1.11 Status
    Status(Status<S>),

    /// 3.8.1.12 Summary
    Summary(Summary<S>),

    // Section 3.8.2 - Date and Time Properties
    /// 3.8.2.1 Date-Time Completed
    Completed(Completed<S>),

    /// 3.8.2.2 Date-Time End
    DtEnd(DtEnd<S>),

    /// 3.8.2.3 Date-Time Due
    Due(Due<S>),

    /// 3.8.2.4 Date-Time Start
    DtStart(DtStart<S>),

    /// 3.8.2.5 Duration
    Duration(Duration<S>),

    /// 3.8.2.6 Free/Busy Time
    FreeBusy(FreeBusy<S>),

    /// 3.8.2.7 Time Transparency
    Transp(TimeTransparency<S>),

    // Section 3.8.3 - Time Zone Component Properties
    /// 3.8.3.1 Time Zone Identifier
    TzId(TzId<S>),

    /// 3.8.3.2 Time Zone Name
    TzName(TzName<S>),

    /// 3.8.3.3 Time Zone Offset From
    TzOffsetFrom(TzOffsetFrom<S>),

    /// 3.8.3.4 Time Zone Offset To
    TzOffsetTo(TzOffsetTo<S>),

    /// 3.8.3.5 Time Zone URL
    TzUrl(TzUrl<S>),

    // Section 3.8.4 - Component Relationship Properties
    /// 3.8.4.1 Attendee
    Attendee(Attendee<S>),

    /// 3.8.4.2 Contact
    Contact(Contact<S>),

    /// 3.8.4.3 Organizer
    Organizer(Organizer<S>),

    /// 3.8.4.4 Recurrence ID
    RecurrenceId(RecurrenceId<S>),

    /// 3.8.4.5 Related To
    RelatedTo(RelatedTo<S>),

    /// 3.8.4.6 URL
    Url(Url<S>),

    /// 3.8.4.7 Unique Identifier
    Uid(Uid<S>),

    // Section 3.8.5 - Recurrence Properties
    /// 3.8.5.1 Exception Date-Times
    ExDate(ExDate<S>),

    /// 3.8.5.2 Recurrence Date-Times
    RDate(RDate<S>),

    /// 3.8.5.3 Recurrence Rule
    RRule(RRule<S>),

    // Section 3.8.6 - Alarm Component Properties
    /// 3.8.6.1 Action
    Action(Action<S>),

    /// 3.8.6.2 Repeat Count
    Repeat(Repeat<S>),

    /// 3.8.6.3 Trigger
    Trigger(Trigger<S>),

    // Section 3.8.7 - Change Management Properties
    /// 3.8.7.1 Date-Time Created
    Created(Created<S>),

    /// 3.8.7.2 Date-Time Stamp
    DtStamp(DtStamp<S>),

    /// 3.8.7.3 Last Modified
    LastModified(LastModified<S>),

    /// 3.8.7.4 Sequence Number
    Sequence(Sequence<S>),

    // Section 3.8.8 - Miscellaneous Properties
    /// 3.8.8.3 Request Status
    RequestStatus(RequestStatus<S>),

    /// Custom experimental x-name property (must start with "X-" or "x-").
    ///
    /// Per RFC 5545: All property names and parameter names are case-insensitive.
    /// Names starting with "X-" and "x-" are reserved for experimental use.
    ///
    /// This variant preserves the original data for round-trip compatibility.
    XName(XNameProperty<S>),

    /// Unrecognized property (not a known standard property).
    ///
    /// Per RFC 5545: Compliant applications are expected to be able to parse
    /// these other IANA-registered properties but can ignore them.
    ///
    /// This variant preserves the original data for round-trip compatibility.
    Unrecognized(UnrecognizedProperty<S>),
}

impl<'src> TryFrom<ParsedProperty<'src>> for Property<Segments<'src>> {
    type Error = Vec<TypedError<'src>>;

    #[rustfmt::skip]
    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        match prop.kind {
            // Section 3.7 - Calendar Properties
            PropertyKind::CalScale      => prop.try_into().map(Property::CalScale),
            PropertyKind::Method        => prop.try_into().map(Property::Method),
            PropertyKind::ProdId        => prop.try_into().map(Property::ProdId),
            PropertyKind::Version       => prop.try_into().map(Property::Version),

            // Section 3.8.1 - Descriptive Component Properties
            PropertyKind::Attach        => prop.try_into().map(Property::Attach),
            PropertyKind::Categories    => prop.try_into().map(Property::Categories),
            PropertyKind::Class         => prop.try_into().map(Property::Class),
            PropertyKind::Comment       => prop.try_into().map(Property::Comment),
            PropertyKind::Description   => prop.try_into().map(Property::Description),
            PropertyKind::Geo           => prop.try_into().map(Property::Geo),
            PropertyKind::Location      => prop.try_into().map(Property::Location),
            PropertyKind::PercentComplete => prop.try_into().map(Property::PercentComplete),
            PropertyKind::Priority      => prop.try_into().map(Property::Priority),
            PropertyKind::Resources     => prop.try_into().map(Property::Resources),
            PropertyKind::Status        => prop.try_into().map(Property::Status),
            PropertyKind::Summary       => prop.try_into().map(Property::Summary),

            // Section 3.8.2 - Date and Time Properties
            PropertyKind::Completed     => prop.try_into().map(Property::Completed),
            PropertyKind::DtEnd         => prop.try_into().map(Property::DtEnd),
            PropertyKind::Due           => prop.try_into().map(Property::Due),
            PropertyKind::DtStart       => prop.try_into().map(Property::DtStart),
            PropertyKind::Duration      => prop.try_into().map(Property::Duration),
            PropertyKind::FreeBusy      => prop.try_into().map(Property::FreeBusy),
            PropertyKind::Transp        => prop.try_into().map(Property::Transp),

            // Section 3.8.3 - Time Zone Component Properties
            PropertyKind::TzId          => prop.try_into().map(Property::TzId),
            PropertyKind::TzName        => prop.try_into().map(Property::TzName),
            PropertyKind::TzOffsetFrom  => prop.try_into().map(Property::TzOffsetFrom),
            PropertyKind::TzOffsetTo    => prop.try_into().map(Property::TzOffsetTo),
            PropertyKind::TzUrl         => prop.try_into().map(Property::TzUrl),

            // Section 3.8.4 - Component Relationship Properties
            PropertyKind::Attendee      => prop.try_into().map(Property::Attendee),
            PropertyKind::Contact       => prop.try_into().map(Property::Contact),
            PropertyKind::Organizer     => prop.try_into().map(Property::Organizer),
            PropertyKind::RecurrenceId  => prop.try_into().map(Property::RecurrenceId),
            PropertyKind::RelatedTo     => prop.try_into().map(Property::RelatedTo),
            PropertyKind::Url           => prop.try_into().map(Property::Url),
            PropertyKind::Uid           => prop.try_into().map(Property::Uid),

            // Section 3.8.5 - Recurrence Properties
            PropertyKind::ExDate        => prop.try_into().map(Property::ExDate),
            PropertyKind::RDate         => prop.try_into().map(Property::RDate),
            PropertyKind::RRule         => prop.try_into().map(Property::RRule),

            // Section 3.8.6 - Alarm Component Properties
            PropertyKind::Action        => prop.try_into().map(Property::Action),
            PropertyKind::Repeat        => prop.try_into().map(Property::Repeat),
            PropertyKind::Trigger       => prop.try_into().map(Property::Trigger),

            // Section 3.8.7 - Change Management Properties
            PropertyKind::Created       => prop.try_into().map(Property::Created),
            PropertyKind::DtStamp       => prop.try_into().map(Property::DtStamp),
            PropertyKind::LastModified  => prop.try_into().map(Property::LastModified),
            PropertyKind::Sequence      => prop.try_into().map(Property::Sequence),

            // Section 3.8.8 - Miscellaneous Properties
            PropertyKind::RequestStatus => prop.try_into().map(Property::RequestStatus),

            // XName properties (experimental x-name properties)
            PropertyKind::XName(_)      => Ok(Property::XName(prop.into())),

            // Unrecognized properties (not a known standard property)
            PropertyKind::Unrecognized(_) => Ok(Property::Unrecognized(prop.into())),
        }
    }
}

impl<S: StringStorage> Property<S> {
    /// Gets the kind of this property
    #[must_use]
    pub fn kind(&self) -> PropertyKind<&S> {
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

            // XName and unknown properties
            Self::XName(v) => PropertyKind::XName(&v.name),
            Self::Unrecognized(v) => PropertyKind::Unrecognized(&v.name),
        }
    }

    /// Returns the span of this property
    #[must_use]
    pub fn span(&self) -> S::Span {
        match self {
            // Section 3.7 - Calendar Properties
            Self::CalScale(v) => v.span(),
            Self::Method(v) => v.span(),
            Self::ProdId(v) => v.span(),
            Self::Version(v) => v.span(),

            // Section 3.8.1 - Descriptive Component Properties
            Self::Attach(v) => v.span(),
            Self::Categories(v) => v.span(),
            Self::Class(v) => v.span(),
            Self::Comment(v) => v.span(),
            Self::Description(v) => v.span(),
            Self::Geo(v) => v.span(),
            Self::Location(v) => v.span(),
            Self::PercentComplete(v) => v.span(),
            Self::Priority(v) => v.span(),
            Self::Resources(v) => v.span(),
            Self::Status(v) => v.span(),
            Self::Summary(v) => v.span(),

            // Section 3.8.2 - Date and Time Properties
            Self::Completed(v) => v.span(),
            Self::DtEnd(v) => v.span(),
            Self::Due(v) => v.span(),
            Self::DtStart(v) => v.span(),
            Self::Duration(v) => v.span(),
            Self::FreeBusy(v) => v.span(),
            Self::Transp(v) => v.span(),

            // Section 3.8.3 - Time Zone Component Properties
            Self::TzId(v) => v.span(),
            Self::TzName(v) => v.span(),
            Self::TzOffsetFrom(v) => v.span(),
            Self::TzOffsetTo(v) => v.span(),
            Self::TzUrl(v) => v.span(),

            // Section 3.8.4 - Component Relationship Properties
            Self::Attendee(v) => v.span(),
            Self::Contact(v) => v.span(),
            Self::Organizer(v) => v.span(),
            Self::RecurrenceId(v) => v.span(),
            Self::RelatedTo(v) => v.span(),
            Self::Url(v) => v.span(),
            Self::Uid(v) => v.span(),

            // Section 3.8.5 - Recurrence Properties
            Self::ExDate(v) => v.span(),
            Self::RDate(v) => v.span(),
            Self::RRule(v) => v.span(),

            // Section 3.8.6 - Alarm Component Properties
            Self::Action(v) => v.span(),
            Self::Repeat(v) => v.span(),
            Self::Trigger(v) => v.span(),

            // Section 3.8.7 - Change Management Properties
            Self::Created(v) => v.span(),
            Self::DtStamp(v) => v.span(),
            Self::LastModified(v) => v.span(),
            Self::Sequence(v) => v.span(),

            // Section 3.8.8 - Miscellaneous Properties
            Self::RequestStatus(v) => v.span(),

            // XName and unknown properties
            Self::XName(v) => v.span(),
            Self::Unrecognized(v) => v.span(),
        }
    }
}

impl Property<Segments<'_>> {
    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> Property<String> {
        match self {
            // Section 3.7 - Calendar Properties
            Property::CalScale(v) => Property::CalScale(v.to_owned()),
            Property::Method(v) => Property::Method(v.to_owned()),
            Property::ProdId(v) => Property::ProdId(v.to_owned()),
            Property::Version(v) => Property::Version(v.to_owned()),

            // Section 3.8.1 - Descriptive Component Properties
            Property::Attach(v) => Property::Attach(v.to_owned()),
            Property::Categories(v) => Property::Categories(v.to_owned()),
            Property::Class(v) => Property::Class(v.to_owned()),
            Property::Comment(v) => Property::Comment(v.to_owned()),
            Property::Description(v) => Property::Description(v.to_owned()),
            Property::Geo(v) => Property::Geo(v.to_owned()),
            Property::Location(v) => Property::Location(v.to_owned()),
            Property::PercentComplete(v) => Property::PercentComplete(v.to_owned()),
            Property::Priority(v) => Property::Priority(v.to_owned()),
            Property::Resources(v) => Property::Resources(v.to_owned()),
            Property::Status(v) => Property::Status(v.to_owned()),
            Property::Summary(v) => Property::Summary(v.to_owned()),

            // Section 3.8.2 - Date and Time Properties
            Property::Completed(v) => Property::Completed(v.to_owned()),
            Property::DtEnd(v) => Property::DtEnd(v.to_owned()),
            Property::Due(v) => Property::Due(v.to_owned()),
            Property::DtStart(v) => Property::DtStart(v.to_owned()),
            Property::Duration(v) => Property::Duration(v.to_owned()),
            Property::FreeBusy(v) => Property::FreeBusy(v.to_owned()),
            Property::Transp(v) => Property::Transp(v.to_owned()),

            // Section 3.8.3 - Time Zone Component Properties
            Property::TzId(v) => Property::TzId(v.to_owned()),
            Property::TzName(v) => Property::TzName(v.to_owned()),
            Property::TzOffsetFrom(v) => Property::TzOffsetFrom(v.to_owned()),
            Property::TzOffsetTo(v) => Property::TzOffsetTo(v.to_owned()),
            Property::TzUrl(v) => Property::TzUrl(v.to_owned()),

            // Section 3.8.4 - Component Relationship Properties
            Property::Attendee(v) => Property::Attendee(v.to_owned()),
            Property::Contact(v) => Property::Contact(v.to_owned()),
            Property::Organizer(v) => Property::Organizer(v.to_owned()),
            Property::RecurrenceId(v) => Property::RecurrenceId(v.to_owned()),
            Property::RelatedTo(v) => Property::RelatedTo(v.to_owned()),
            Property::Url(v) => Property::Url(v.to_owned()),
            Property::Uid(v) => Property::Uid(v.to_owned()),

            // Section 3.8.5 - Recurrence Properties
            Property::ExDate(v) => Property::ExDate(v.to_owned()),
            Property::RDate(v) => Property::RDate(v.to_owned()),
            Property::RRule(v) => Property::RRule(v.to_owned()),

            // Section 3.8.6 - Alarm Component Properties
            Property::Action(v) => Property::Action(v.to_owned()),
            Property::Repeat(v) => Property::Repeat(v.to_owned()),
            Property::Trigger(v) => Property::Trigger(v.to_owned()),

            // Section 3.8.7 - Change Management Properties
            Property::Created(v) => Property::Created(v.to_owned()),
            Property::DtStamp(v) => Property::DtStamp(v.to_owned()),
            Property::LastModified(v) => Property::LastModified(v.to_owned()),
            Property::Sequence(v) => Property::Sequence(v.to_owned()),

            // Section 3.8.8 - Miscellaneous Properties
            Property::RequestStatus(v) => Property::RequestStatus(v.to_owned()),

            // XName and Unknown properties
            Property::XName(v) => Property::XName(v.to_owned()),
            Property::Unrecognized(v) => Property::Unrecognized(v.to_owned()),
        }
    }
}

macro_rules! define_nonstandard_property {
    (
        struct $ty:ident;
    ) => {
        /// Non-standard property structure
        #[derive(Debug, Clone)]
        pub struct $ty<S: StringStorage> {
            /// Property name
            pub name: S,
            /// Parsed parameters (may include Unknown parameters)
            pub parameters: Vec<Parameter<S>>,
            /// Parsed value(s)
            pub value: Value<S>,
            /// The span of the property
            pub span: S::Span,
        }

        impl $ty<Segments<'_>> {
            /// Convert borrowed type to owned type
            pub fn to_owned(&self) -> $ty<String> {
                $ty {
                    name: self.name.to_owned(),
                    parameters: self.parameters.iter().map(Parameter::to_owned).collect(),
                    value: self.value.to_owned(),
                    span: (),
                }
            }
        }

        impl<S: StringStorage> $ty<S> {
            /// Get the span of this property
            #[must_use]
            pub const fn span(&self) -> S::Span {
                self.span
            }
        }

        impl<'src> From<ParsedProperty<'src>> for $ty<Segments<'src>> {
            fn from(prop: ParsedProperty<'src>) -> Self {
                Self {
                    name: prop.name,
                    parameters: prop.parameters,
                    value: prop.value,
                    span: prop.span,
                }
            }
        }
    };
}

define_nonstandard_property! {
    struct XNameProperty;
}

define_nonstandard_property! {
    struct UnrecognizedProperty;
}
