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

pub use alarm::{Action, ActionValue, Repeat, Trigger, TriggerValueOwned, TriggerValueRef};
pub use calendar::{
    CalendarScale, CalendarScaleValue, Method, MethodValue, ProductId, Version, VersionValue,
};
pub use changemgmt::{
    Created, CreatedOwned, CreatedRef, DtStamp, DtStampOwned, DtStampRef, LastModified,
    LastModifiedOwned, LastModifiedRef, Sequence,
};
pub use datetime::{
    Completed, CompletedOwned, CompletedRef, DateTime, DtEnd, DtEndOwned, DtEndRef, DtStart,
    DtStartOwned, DtStartRef, Due, DueOwned, DueRef, Duration, FreeBusy, Period, Time,
    TimeTransparency, TimeTransparencyValue,
};
pub use descriptive::{
    Attachment, AttachmentValue, AttachmentValueOwned, AttachmentValueRef, Categories,
    CategoriesOwned, CategoriesRef, Classification, ClassificationValue, Comment, CommentOwned,
    CommentRef, Description, DescriptionOwned, DescriptionRef, Geo, Location, LocationOwned,
    LocationRef, PercentComplete, Priority, Resources, ResourcesOwned, ResourcesRef, Status,
    StatusValue, Summary, SummaryOwned, SummaryRef,
};
pub use kind::{PropertyKind, PropertyKindOwned, PropertyKindRef};
pub use miscellaneous::{RequestStatus, RequestStatusOwned, RequestStatusRef};
pub use recurrence::{
    ExDate, ExDateValueOwned, ExDateValueRef, RDate, RDateValueOwned, RDateValueRef,
};
pub use relationship::{
    Attendee, Contact, ContactOwned, ContactRef, Organizer, RecurrenceId, RecurrenceIdOwned,
    RecurrenceIdRef, RelatedTo, RelatedToOwned, RelatedToRef, Uid, UidOwned, UidRef, Url, UrlOwned,
    UrlRef,
};
pub use timezone::{
    TzId, TzIdOwned, TzIdRef, TzName, TzNameOwned, TzNameRef, TzOffsetFrom, TzOffsetFromOwned,
    TzOffsetFromRef, TzOffsetTo, TzOffsetToOwned, TzOffsetToRef, TzUrl, TzUrlOwned, TzUrlRef,
};
pub use util::{Text, Texts};

use std::fmt::Display;

use crate::lexer::Span;
use crate::parameter::Parameter;
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{RecurrenceRule, Value};

/// Type alias for borrowed property
pub type PropertyRef<'src> = Property<SpannedSegments<'src>>;

/// Type alias for owned property (not yet implemented, would require owned semantic types)
pub type PropertyOwned = Property<String>;

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
pub enum Property<S: Clone + Display> {
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
    RRule(RecurrenceRule),

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
    XName {
        /// Property name (e.g., "X-CUSTOM", "x-custom")
        name: S,
        /// Parsed parameters (may include Unknown parameters)
        parameters: Vec<Parameter<S>>,
        /// Parsed value(s)
        value: Value<S>,
        /// The span of the property
        span: Span,
    },

    /// Unrecognized property (not a known standard property).
    ///
    /// Per RFC 5545: Compliant applications are expected to be able to parse
    /// these other IANA-registered properties but can ignore them.
    ///
    /// This variant preserves the original data for round-trip compatibility.
    Unrecognized {
        /// Property name (e.g., "SOME-IANA-PROP")
        name: S,
        /// Parsed parameters (may include Unknown parameters)
        parameters: Vec<Parameter<S>>,
        /// Parsed value(s)
        value: Value<S>,
        /// The span of the property
        span: Span,
    },
}

impl<'src> TryFrom<ParsedProperty<'src>> for PropertyRef<'src> {
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

            // TODO: Parse RRULE from text (Value::Text)
            // For now, return an error since RecurrenceRule parsing is not yet implemented
            PropertyKind::RRule => Err(vec![TypedError::PropertyInvalidValue {
                property: prop.kind,
                value: "RRULE parsing not yet implemented".to_string(),
                span: prop.span,
            }]),

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

            // XName properties (experimental x-name properties)
            PropertyKind::XName(_name) => Ok(Property::XName {
                name: prop.name,
                parameters: prop.parameters,
                value: prop.value,
                span: prop.span,
            }),

            // Unrecognized properties (not a known standard property)
            PropertyKind::Unrecognized(_name) => Ok(Property::Unrecognized {
                name: prop.name,
                parameters: prop.parameters,
                value: prop.value,
                span: prop.span,
            }),
        }
    }
}

impl<S: Clone + Display> Property<S> {
    /// Returns the `PropertyKind` for this property
    #[must_use]
    pub fn kind(&self) -> PropertyKind<S>
    where
        S: Clone,
    {
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

            // XName and Unrecognized properties
            Self::XName { name, .. } => PropertyKind::XName(name.clone()),
            Self::Unrecognized { name, .. } => PropertyKind::Unrecognized(name.clone()),
        }
    }
}
