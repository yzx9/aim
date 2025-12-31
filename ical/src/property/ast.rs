// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Abstract Syntax Tree for iCalendar properties.
//!
//! This module defines the unified `Property` enum that provides type-safe
//! access to all iCalendar properties with their corresponding semantic types.

use crate::property::alarm::{Action, Trigger};
use crate::property::cal::{CalendarScale, Method, ProductId, Version};
use crate::property::datetime::{DateTime, Period};
use crate::property::descriptive::{
    Attachment, Classification, Geo, Organizer, Text, parse_multi_text_property,
};
use crate::property::relationship::Attendee;
use crate::property::status::Status;
use crate::property::timezone::TimeZoneOffset;
use crate::property::transp::TimeTransparency;
use crate::property::util::take_single_value;
use crate::semantic::SemanticError;
use crate::typed::{PropertyKind, TypedProperty, Value, ValueType};
use crate::value::{RecurrenceRule, ValueDate, ValueDuration};

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
    Categories(Vec<Text<'src>>),

    /// 3.8.1.3 Classification
    Class(Classification),

    /// 3.8.1.4 Comment
    Comment(Text<'src>),

    /// 3.8.1.5 Description
    Description(Text<'src>),

    /// 3.8.1.6 Geographic Position
    Geo(Geo),

    /// 3.8.1.7 Location
    Location(Text<'src>),

    /// 3.8.1.8 Percent Complete
    PercentComplete(u8),

    /// 3.8.1.9 Priority
    Priority(u8),

    /// 3.8.1.10 Resources (multi-valued text)
    Resources(Vec<Text<'src>>),

    /// 3.8.1.11 Status
    Status(Status),

    /// 3.8.1.12 Summary
    Summary(Text<'src>),

    // Section 3.8.2 - Date and Time Properties
    /// 3.8.2.1 Date-Time Completed
    Completed(DateTime<'src>),

    /// 3.8.2.2 Date-Time End
    DtEnd(DateTime<'src>),

    /// 3.8.2.3 Date-Time Due
    Due(DateTime<'src>),

    /// 3.8.2.4 Date-Time Start
    DtStart(DateTime<'src>),

    /// 3.8.2.5 Duration
    Duration(ValueDuration),

    /// 3.8.2.6 Free/Busy Time
    FreeBusy(Vec<Period<'src>>),

    /// 3.8.2.7 Time Transparency
    Transp(TimeTransparency),

    // Section 3.8.3 - Time Zone Component Properties
    /// 3.8.3.1 Time Zone Identifier
    TzId(Text<'src>),

    /// 3.8.3.2 Time Zone Name
    TzName(Text<'src>),

    /// 3.8.3.3 Time Zone Offset From
    TzOffsetFrom(TimeZoneOffset),

    /// 3.8.3.4 Time Zone Offset To
    TzOffsetTo(TimeZoneOffset),

    /// 3.8.3.5 Time Zone URL
    TzUrl(Text<'src>),

    // Section 3.8.4 - Component Relationship Properties
    /// 3.8.4.1 Attendee
    Attendee(Attendee<'src>),

    /// 3.8.4.2 Contact
    Contact(Text<'src>),

    /// 3.8.4.3 Organizer
    Organizer(Organizer<'src>),

    /// 3.8.4.4 Recurrence ID
    RecurrenceId(DateTime<'src>),

    /// 3.8.4.5 Related To
    RelatedTo(Text<'src>),

    /// 3.8.4.6 URL
    Url(Text<'src>),

    /// 3.8.4.7 Unique Identifier
    Uid(Text<'src>),

    // Section 3.8.5 - Recurrence Properties
    /// 3.8.5.1 Exception Date-Times
    ExDate(Vec<ExDateValue<'src>>),

    /// 3.8.5.2 Recurrence Date-Times
    RDate(Vec<RDateValue<'src>>),

    /// 3.8.5.3 Recurrence Rule
    RRule(RecurrenceRule),

    // Section 3.8.6 - Alarm Component Properties
    /// 3.8.6.1 Action
    Action(Action),

    /// 3.8.6.2 Repeat Count
    Repeat(i32),

    /// 3.8.6.3 Trigger
    Trigger(Trigger<'src>),

    // Section 3.8.7 - Change Management Properties
    /// 3.8.7.1 Date-Time Created
    Created(DateTime<'src>),

    /// 3.8.7.2 Date-Time Stamp
    DtStamp(DateTime<'src>),

    /// 3.8.7.3 Last Modified
    LastModified(DateTime<'src>),

    /// 3.8.7.4 Sequence Number
    Sequence(i32),

    // Section 3.8.8 - Miscellaneous Properties
    /// 3.8.8.3 Request Status
    RequestStatus(Text<'src>),
}

/// Exception date-time value (can be DATE or DATE-TIME).
#[derive(Debug, Clone)]
pub enum ExDateValue<'src> {
    /// Date-only value
    Date(ValueDate),
    /// Date-time value
    DateTime(DateTime<'src>),
}

/// Recurrence date-time value (can be DATE, DATE-TIME, or PERIOD).
#[derive(Debug, Clone)]
pub enum RDateValue<'src> {
    /// Date-only value
    Date(ValueDate),
    /// Date-time value
    DateTime(DateTime<'src>),
    /// Period value
    Period(Period<'src>),
}

impl<'src> TryFrom<TypedProperty<'src>> for Property<'src> {
    type Error = Vec<SemanticError>;

    #[allow(clippy::too_many_lines)]
    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
        match prop.kind {
            // Section 3.7 - Calendar Properties
            PropertyKind::CalScale => CalendarScale::try_from(prop).map(Property::CalScale),
            PropertyKind::Method => Method::try_from(prop).map(Property::Method),
            PropertyKind::ProdId => ProductId::try_from(prop).map(Property::ProdId),
            PropertyKind::Version => Version::try_from(prop).map(Property::Version),

            // Section 3.8.1 - Descriptive Component Properties
            PropertyKind::Attach => Attachment::try_from(prop).map(Property::Attach),
            PropertyKind::Categories => Ok(Property::Categories(parse_multi_text_property(prop))),
            PropertyKind::Class => Classification::try_from(prop).map(Property::Class),
            PropertyKind::Comment => Text::try_from(prop).map(Property::Comment),
            PropertyKind::Description => Text::try_from(prop).map(Property::Description),
            PropertyKind::Geo => Geo::try_from(prop).map(Property::Geo),
            PropertyKind::Location => Text::try_from(prop).map(Property::Location),
            PropertyKind::PercentComplete => take_single_value(prop.kind, prop.values)
                .and_then(|v| match v {
                    Value::Integer(i) => {
                        Ok(u8::try_from(i).map_err(|_| SemanticError::InvalidValue {
                            property: prop.kind,
                            value: "Percent complete must be 0-100".to_string(),
                        })?)
                    }
                    _ => Err(SemanticError::UnexpectedType {
                        property: prop.kind,
                        expected: ValueType::Integer,
                    }),
                })
                .map(Property::PercentComplete)
                .map_err(|e| vec![e]),
            PropertyKind::Priority => take_single_value(prop.kind, prop.values)
                .and_then(|v| match v {
                    Value::Integer(i) => {
                        Ok(u8::try_from(i).map_err(|_| SemanticError::InvalidValue {
                            property: prop.kind,
                            value: "Priority must be 0-9".to_string(),
                        })?)
                    }
                    _ => Err(SemanticError::UnexpectedType {
                        property: prop.kind,
                        expected: ValueType::Integer,
                    }),
                })
                .map(Property::Priority)
                .map_err(|e| vec![e]),
            PropertyKind::Resources => Ok(Property::Resources(parse_multi_text_property(prop))),
            PropertyKind::Status => Status::try_from(prop).map(Property::Status),
            PropertyKind::Summary => Text::try_from(prop).map(Property::Summary),

            // Section 3.8.2 - Date and Time Properties
            PropertyKind::Completed => DateTime::try_from(prop).map(Property::Completed),
            PropertyKind::DtEnd => DateTime::try_from(prop).map(Property::DtEnd),
            PropertyKind::Due => DateTime::try_from(prop).map(Property::Due),
            PropertyKind::DtStart => DateTime::try_from(prop).map(Property::DtStart),
            PropertyKind::Duration => take_single_value(prop.kind, prop.values)
                .and_then(|v| match v {
                    Value::Duration(d) => Ok(d),
                    _ => Err(SemanticError::UnexpectedType {
                        property: prop.kind,
                        expected: ValueType::Duration,
                    }),
                })
                .map(Property::Duration)
                .map_err(|e| vec![e]),
            PropertyKind::FreeBusy => prop
                .values
                .into_iter()
                .map(|v| Period::try_from(&v).map_err(|e| vec![e]))
                .collect::<Result<Vec<_>, _>>()
                .map(Property::FreeBusy),
            PropertyKind::Transp => TimeTransparency::try_from(prop).map(Property::Transp),

            // Section 3.8.3 - Time Zone Component Properties
            PropertyKind::TzId => Text::try_from(prop).map(Property::TzId),
            PropertyKind::TzName => Text::try_from(prop).map(Property::TzName),
            PropertyKind::TzOffsetFrom => {
                TimeZoneOffset::try_from(prop).map(Property::TzOffsetFrom)
            }
            PropertyKind::TzOffsetTo => TimeZoneOffset::try_from(prop).map(Property::TzOffsetTo),
            PropertyKind::TzUrl => Text::try_from(prop).map(Property::TzUrl),

            // Section 3.8.4 - Component Relationship Properties
            PropertyKind::Attendee => Attendee::try_from(prop).map(Property::Attendee),
            PropertyKind::Contact => Text::try_from(prop).map(Property::Contact),
            PropertyKind::Organizer => Organizer::try_from(prop).map(Property::Organizer),
            PropertyKind::RecurrenceId => DateTime::try_from(prop).map(Property::RecurrenceId),
            PropertyKind::RelatedTo => Text::try_from(prop).map(Property::RelatedTo),
            PropertyKind::Url => Text::try_from(prop).map(Property::Url),
            PropertyKind::Uid => Text::try_from(prop).map(Property::Uid),

            // Section 3.8.5 - Recurrence Properties
            PropertyKind::ExDate => prop
                .values
                .into_iter()
                .map(|v| match v {
                    Value::Date(d) => Ok(ExDateValue::Date(d)),
                    Value::DateTime(dt) => Ok(ExDateValue::DateTime(DateTime::Floating {
                        date: dt.date,
                        time: dt.time.into(),
                    })),
                    _ => Err(vec![SemanticError::UnexpectedType {
                        property: prop.kind,
                        expected: ValueType::DateTime,
                    }]),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Property::ExDate),
            PropertyKind::RDate => prop
                .values
                .into_iter()
                .map(|v| match v {
                    Value::Date(d) => Ok(RDateValue::Date(d)),
                    Value::DateTime(dt) => Ok(RDateValue::DateTime(DateTime::Floating {
                        date: dt.date,
                        time: dt.time.into(),
                    })),
                    _ => match Period::try_from(&v) {
                        Ok(p) => Ok(RDateValue::Period(p)),
                        Err(_) => Err(vec![SemanticError::UnexpectedType {
                            property: prop.kind,
                            expected: ValueType::Period,
                        }]),
                    },
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Property::RDate),
            PropertyKind::RRule => {
                // TODO: Parse RRULE from text (Value::Text)
                // For now, return an error since RecurrenceRule parsing is not yet implemented
                Err(vec![SemanticError::InvalidValue {
                    property: prop.kind,
                    value: "RRULE parsing not yet implemented".to_string(),
                }])
            }

            // Section 3.8.6 - Alarm Component Properties
            PropertyKind::Action => Action::try_from(prop).map(Property::Action),
            PropertyKind::Repeat => take_single_value(prop.kind, prop.values)
                .and_then(|v| match v {
                    Value::Integer(i) => Ok(i),
                    _ => Err(SemanticError::UnexpectedType {
                        property: prop.kind,
                        expected: ValueType::Integer,
                    }),
                })
                .map(Property::Repeat)
                .map_err(|e| vec![e]),
            PropertyKind::Trigger => Trigger::try_from(prop).map(Property::Trigger),

            // Section 3.8.7 - Change Management Properties
            PropertyKind::Created => DateTime::try_from(prop).map(Property::Created),
            PropertyKind::DtStamp => DateTime::try_from(prop).map(Property::DtStamp),
            PropertyKind::LastModified => DateTime::try_from(prop).map(Property::LastModified),
            PropertyKind::Sequence => take_single_value(prop.kind, prop.values)
                .and_then(|v| match v {
                    Value::Integer(i) => Ok(i),
                    _ => Err(SemanticError::UnexpectedType {
                        property: prop.kind,
                        expected: ValueType::Integer,
                    }),
                })
                .map(Property::Sequence)
                .map_err(|e| vec![e]),

            // Section 3.8.8 - Miscellaneous Properties
            PropertyKind::RequestStatus => Text::try_from(prop).map(Property::RequestStatus),
        }
    }
}
