// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Display, str::FromStr};

use chumsky::{Parser, extra};

use crate::keyword::{
    KW_ALTREP, KW_BINARY, KW_BOOLEAN, KW_CAL_ADDRESS, KW_CN, KW_CUTYPE, KW_DATE, KW_DATETIME,
    KW_DELEGATED_FROM, KW_DELEGATED_TO, KW_DIR, KW_DURATION, KW_ENCODING, KW_ENCODING_8BIT,
    KW_ENCODING_BASE64, KW_FBTYPE, KW_FBTYPE_BUSY, KW_FBTYPE_BUSY_TENTATIVE,
    KW_FBTYPE_BUSY_UNAVAILABLE, KW_FBTYPE_FREE, KW_FLOAT, KW_FMTTYPE, KW_INTEGER, KW_LANGUAGE,
    KW_MEMBER, KW_PARTSTAT, KW_PERIOD, KW_RANGE, KW_RELATED, KW_RELTYPE, KW_ROLE, KW_RRULE,
    KW_RSVP, KW_SENT_BY, KW_TEXT, KW_TIME, KW_TZID, KW_URI, KW_UTC_OFFSET, KW_VALUE,
};
use crate::syntax::{SpannedSegments, SyntaxParameter};
use crate::typed::TypedAnalysisError;
use crate::typed::util::make_input;
use crate::typed::value::value_boolean;

#[derive(Debug, Clone)]
pub enum TypedParameter<'src> {
    /// This parameter specifies a URI that points to an alternate
    /// representation for a textual property value. A property specifying
    /// this parameter MUST also include a value that reflects the default
    /// representation of the text value
    ///
    /// See also: RFC 5545 Section 3.2.1. Alternate Text Representation
    AlternateText(SpannedSegments<'src>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the common name to be associated with
    /// the calendar user specified by the property. The parameter value is
    /// text. The parameter value can be used for display text to be associated
    /// with the calendar address specified by the property.
    ///
    /// See also: RFC 5545 Section 3.2.2. Common Name
    CommonName(SpannedSegments<'src>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter identifies the type of calendar user specified by
    /// the property. If not specified on a property that allows this parameter,
    /// the default is INDIVIDUAL. Applications MUST treat x-name and iana-
    /// token values they don't recognize the same way as they would the
    /// UNKNOWN value.
    ///
    /// See also: RFC 5545 Section 3.2.3. Calendar User Type
    CalendarUserType(SpannedSegments<'src>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. This parameter specifies those calendar users that have delegated
    /// their participation in a group-scheduled event or to-do to the calendar
    /// user specified by the property.
    ///
    /// See also: RFC 5545 Section 3.2.4. Delegators
    Delegators(Vec<SpannedSegments<'src>>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. This parameter specifies those calendar users whom have been
    /// delegated participation in a group-scheduled event or to-do by the
    /// calendar user specified by the property.
    ///
    /// See also: RFC 5545 Section 3.2.5. Delegatees
    Delegatees(Vec<SpannedSegments<'src>>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies a reference to the directory entry
    /// associated with the calendar user specified by the property. The
    /// parameter value is a URI.
    ///
    /// See also: RFC 5545 Section 3.2.6. Directory Entry Reference
    Directory(SpannedSegments<'src>),

    /// This property parameter identifies the inline encoding used in a
    /// property value.  The default encoding is "8BIT", corresponding to a
    /// property value consisting of text.  The "BASE64" encoding type
    /// corresponds to a property value encoded using the "BASE64" encoding
    /// defined in [RFC2045].
    ///
    /// See also: RFC 5545 Section 3.2.7. Inline Encoding
    Encoding(ParamEncoding),

    /// This parameter can be specified on properties that are used to
    /// reference an object. The parameter specifies the media type [RFC4288]
    /// of the referenced object. For example, on the "ATTACH" property, an FTP
    /// type URI value does not, by itself, necessarily convey the type of
    /// content associated with the resource. The parameter value MUST be the
    /// text for either an IANA-registered media type or a non-standard media
    /// type.
    ///
    /// See also: RFC 5545 Section 3.2.8. Format Type
    FormatType(SpannedSegments<'src>),

    /// This parameter specifies the free or busy time type. The value FREE
    /// indicates that the time interval is free for scheduling. The value BUSY
    /// indicates that the time interval is busy because one or more events
    /// have been scheduled for that interval. The value BUSY-UNAVAILABLE
    /// indicates that the time interval is busy and that the interval can not
    /// be scheduled. The value BUSY-TENTATIVE indicates that the time interval
    /// is busy because one or more events have been tentatively scheduled for
    /// that interval.  If not specified on a property that allows this
    /// parameter, the default is BUSY.  Applications MUST treat x-name and
    /// iana-token values they don't recognize the same way as they would the
    /// BUSY value.
    ///
    /// See also: RFC 5545 Section 3.2.9. Free/Busy Time Type
    FreeBusyType(FreeBusyType),

    /// This parameter identifies the language of the text in the property
    /// value and of all property parameter values of the property. The value
    /// of the "LANGUAGE" property parameter is that defined in [RFC5646].
    ///
    /// For transport in a MIME entity, the Content-Language header field can
    /// be used to set the default language for the entire body part. Otherwise,
    /// no default language is assumed.
    ///
    /// See also: RFC 5545 Section 3.2.10. Language
    Language(SpannedSegments<'src>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter identifies the groups or list membership for the
    /// calendar user specified by the property. The parameter value is either
    /// a single calendar address in a quoted-string or a COMMA-separated list
    /// of calendar addresses, each in a quoted-string. The individual calendar
    /// address parameter values MUST each be specified in a quoted-string.
    ///
    /// See also: RFC 5545 Section 3.2.11. Group or List Membership
    GroupOrListMembership(Vec<SpannedSegments<'src>>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter identifies the participation status for the
    /// calendar user specified by the property value. The parameter values
    /// differ depending on whether they are associated with a group-scheduled
    /// "VEVENT", "VTODO", or "VJOURNAL". The values MUST match one of the
    /// values allowed for the given calendar component.  If not specified on a
    /// property that allows this parameter, the default value is NEEDS-ACTION.
    /// Applications MUST treat x-name and iana-token values they don't
    /// recognize the same way as they would the NEEDS-ACTION value.
    ///
    /// See also: RFC 5545 Section 3.2.12. Participation Status
    ParticipationStatus(SpannedSegments<'src>),

    /// This parameter can be specified on a property that specifies a
    /// recurrence identifier. The parameter specifies the effective range of
    /// recurrence instances that is specified by the property. The effective
    /// range is from the recurrence identifier specified by the property. If
    /// this parameter is not specified on an allowed property, then the
    /// default range is the single instance specified by the recurrence
    /// identifier value of the property. The parameter value can only be
    /// "THISANDFUTURE" to indicate a range defined by the recurrence
    /// identifier and all subsequent instances. The value "THISANDPRIOR" is
    /// deprecated by this revision of iCalendar and MUST NOT be generated by
    /// applications.
    ///
    /// See also: RFC 5545 Section 3.2.13. Recurrence Identifier Range
    RecurrenceIdRange(SpannedSegments<'src>),

    /// This parameter can be specified on properties that specify an alarm
    /// trigger with a "DURATION" value type. The parameter specifies whether
    /// the alarm will trigger relative to the start or end of the calendar
    /// component. The parameter value START will set the alarm to trigger off
    /// the start of the calendar component; the parameter value END will set
    /// the alarm to trigger off the end of the calendar component. If the
    /// parameter is not specified on an allowable property, then the default
    /// is START.
    ///
    /// See also: RFC 5545 Section 3.2.14. Alarm Trigger Relationship
    AlarmTriggerRelationship(SpannedSegments<'src>),

    /// This parameter can be specified on a property that references another
    /// related calendar. The parameter specifies the hierarchical relationship
    /// type of the calendar component referenced by the property. The
    /// parameter value can be PARENT, to indicate that the referenced calendar
    /// component is a superior of calendar component; CHILD to indicate that
    /// the referenced calendar component is a subordinate of the calendar
    /// component; or SIBLING to indicate that the referenced calendar
    /// component is a peer of the calendar component. If this parameter is not
    /// specified on an allowable property, the default relationship type is
    /// PARENT. Applications MUST treat x-name and iana-token values they don't
    /// recognize the same way as they would the PARENT value.
    ///
    /// See also: RFC 5545 Section 3.2.15. Relationship Type
    RelationshipType(SpannedSegments<'src>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the participation role for the calendar
    /// user specified by the property in the group schedule calendar component.
    /// If not specified on a property that allows this parameter, the default
    /// value is REQ-PARTICIPANT. Applications MUST treat x-name and iana-token
    /// values they don't recognize the same way as they would the REQ-PARTICIPANT value.
    ///
    /// See also: RFC 5545 Section 3.2.16. Participation Role
    ParticipationRole(SpannedSegments<'src>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the calendar user that is acting on behalf
    /// of the calendar user specified by the property. The parameter value MUST
    /// be a mailto URI as defined in [RFC2368]. The individual calendar address
    /// parameter values MUST each be specified in a quoted-string.
    ///
    /// See also: RFC 5545 Section 3.2.18. Sent By
    SendBy(SpannedSegments<'src>),

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter identifies the expectation of a reply from the
    /// calendar user specified by the property value. This parameter is used
    /// by the "Organizer" to request a participation status reply from an
    /// "Attendee" of a group-scheduled event or to-do. If not specified on a
    /// property that allows this parameter, the default value is FALSE.
    RsvpExpectation(bool),

    /// This parameter MUST be specified on the "DTSTART", "DTEND", "DUE",
    /// "EXDATE", and "RDATE" properties when either a DATE-TIME or TIME value
    /// type is specified and when the value is neither a UTC or a "floating"
    /// time. Refer to the DATE-TIME or TIME value type definition for a
    /// description of UTC and "floating time" formats.  This property
    /// parameter specifies a text value that uniquely identifies the
    /// "VTIMEZONE" calendar component to be used when evaluating the time
    /// portion of the property.  The value of the "TZID" property parameter
    /// will be equal to the value of the "TZID" property for the matching time
    /// zone definition.  An individual "VTIMEZONE" calendar component MUST be
    /// specified for each unique "TZID" parameter value specified in the
    /// iCalendar object.
    ///
    /// See also: RFC 5545 Section 3.2.19. Time Zone Identifier
    TimeZoneIdentifier(SpannedSegments<'src>),

    /// This parameter specifies the value type and format of the property
    /// value. The property values MUST be of a single value type. For example,
    /// a "RDATE" property cannot have a combination of DATE-TIME and TIME
    /// value types.
    ///
    /// If the property's value is the default value type, then this parameter
    /// need not be specified.  However, if the property's default value type
    /// is overridden by some other allowable value type, then this parameter
    /// MUST be specified.
    ///
    /// See also: RFC 5545 Section 3.2.20. Value Data Types
    ValueType(ParamValueType),
}

impl<'src> TryFrom<SyntaxParameter<'src>> for TypedParameter<'src> {
    type Error = Vec<TypedAnalysisError<'src>>;

    fn try_from(param: SyntaxParameter<'src>) -> Result<Self, Self::Error> {
        let single = |mut param: SyntaxParameter<'src>| {
            if param.values.len() == 1 {
                Ok(param.values.pop().unwrap())
            } else {
                Err(vec![
                    TypedAnalysisError::ParameterMultipleValuesDisallowed {
                        property: String::new(), // TODO: fill property name
                        param: param.name.resolve().to_string(),
                        span: param.name.span(), // TODO: improve span to cover all values
                    },
                ])
            }
        };

        let multiple =
            |param: SyntaxParameter<'src>| Ok(param.values.into_iter().map(|v| v.value).collect());

        match param.name.resolve().as_ref() {
            // Single value parameters
            KW_ALTREP => single(param).map(|v| TypedParameter::AlternateText(v.value)),
            KW_CN => single(param).map(|v| TypedParameter::CommonName(v.value)),
            KW_CUTYPE => single(param).map(|v| TypedParameter::CalendarUserType(v.value)),
            KW_DIR => single(param).map(|v| TypedParameter::Directory(v.value)),
            KW_ENCODING => single(param).and_then(|v| {
                v.value
                    .resolve()
                    .parse()
                    .map(TypedParameter::Encoding)
                    .map_err(|()| {
                        vec![TypedAnalysisError::ParameterValueKindUnknown {
                            property: String::new(), // TODO: fill property name
                            kind: v.value.resolve().to_string(),
                            span: v.value.span(),
                        }]
                    })
            }),
            KW_FMTTYPE => single(param).map(|v| TypedParameter::FormatType(v.value)),
            KW_FBTYPE => single(param).and_then(|v| {
                v.value
                    .resolve()
                    .parse()
                    .map(TypedParameter::FreeBusyType)
                    .map_err(|()| {
                        vec![TypedAnalysisError::ParameterValueKindUnknown {
                            property: String::new(), // TODO: fill property name
                            kind: v.value.resolve().to_string(),
                            span: v.value.span(),
                        }]
                    })
            }),
            KW_LANGUAGE => single(param).map(|v| TypedParameter::Language(v.value)),
            KW_PARTSTAT => single(param).map(|v| TypedParameter::ParticipationStatus(v.value)),
            KW_RANGE => single(param).map(|v| TypedParameter::RecurrenceIdRange(v.value)),
            KW_RELATED => single(param).map(|v| TypedParameter::AlarmTriggerRelationship(v.value)),
            KW_RELTYPE => single(param).map(|v| TypedParameter::RelationshipType(v.value)),
            KW_ROLE => single(param).map(|v| TypedParameter::ParticipationRole(v.value)),
            KW_RSVP => single(param).and_then(|v| {
                value_boolean::<'_, _, extra::Err<_>>()
                    .parse(make_input(v.value))
                    .into_result()
                    .map(TypedParameter::RsvpExpectation)
                    .map_err(|errs| {
                        errs.into_iter()
                            .map(|err| TypedAnalysisError::ParameterValueSyntax {
                                property: String::new(),
                                parameter: KW_RSVP.to_string(),
                                err,
                            })
                            .collect()
                    })
            }),
            KW_SENT_BY => single(param).map(|v| TypedParameter::SendBy(v.value)),
            KW_TZID => single(param).map(|v| TypedParameter::TimeZoneIdentifier(v.value)),
            KW_VALUE => single(param).and_then(|v| {
                v.value
                    .resolve()
                    .parse()
                    .map(TypedParameter::ValueType)
                    .map_err(|()| {
                        vec![TypedAnalysisError::ParameterValueKindUnknown {
                            property: String::new(), // TODO: fill property name
                            kind: v.value.resolve().to_string(),
                            span: v.value.span(),
                        }]
                    })
            }),

            // Multiple value parameters
            KW_DELEGATED_FROM => multiple(param).map(TypedParameter::Delegators),
            KW_DELEGATED_TO => multiple(param).map(TypedParameter::Delegatees),
            KW_MEMBER => multiple(param).map(TypedParameter::GroupOrListMembership),

            // Unknown parameter - treat as unknown x-name or iana-token
            // According to RFC 5545, applications MUST treat x-name and iana-token values
            // they don't recognize the same way as they would the UNKNOWN value
            _ => Err(vec![TypedAnalysisError::ParameterValueKindUnknown {
                property: String::new(), // TODO: fill property name
                kind: param.name.resolve().to_string(),
                span: param.name.span(),
            }]),
        }
    }
}

/// This parameter identifies the inline encoding used in a property value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamEncoding {
    /// The default encoding is "8BIT", corresponding to a property value
    /// consisting of text.
    Bit8,

    /// The "BASE64" encoding type corresponds to a property value encoded
    /// using the "BASE64" encoding defined in [RFC2045].
    Base64,
}

impl TryFrom<&SpannedSegments<'_>> for ParamEncoding {
    type Error = ();

    fn try_from(segs: &SpannedSegments<'_>) -> Result<Self, Self::Error> {
        segs.resolve().parse() // PERF: avoid allocation
    }
}

impl FromStr for ParamEncoding {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: check quote: Property parameter values that are not in quoted-strings are case-insensitive.
        match s {
            KW_ENCODING_8BIT => Ok(ParamEncoding::Bit8),
            KW_ENCODING_BASE64 => Ok(ParamEncoding::Base64),
            _ => Err(()),
        }
    }
}

impl AsRef<str> for ParamEncoding {
    fn as_ref(&self) -> &str {
        match self {
            ParamEncoding::Bit8 => KW_ENCODING_8BIT,
            ParamEncoding::Base64 => KW_ENCODING_BASE64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeBusyType {
    Free,
    Busy,
    BusyUnavailable,
    BusyTentative,
}

impl TryFrom<&SpannedSegments<'_>> for FreeBusyType {
    type Error = ();

    fn try_from(segs: &SpannedSegments<'_>) -> Result<Self, Self::Error> {
        segs.resolve().parse() // PERF: avoid allocation
    }
}

impl FromStr for FreeBusyType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            KW_FBTYPE_FREE => Ok(FreeBusyType::Free),
            KW_FBTYPE_BUSY => Ok(FreeBusyType::Busy),
            KW_FBTYPE_BUSY_UNAVAILABLE => Ok(FreeBusyType::BusyUnavailable),
            KW_FBTYPE_BUSY_TENTATIVE => Ok(FreeBusyType::BusyTentative),
            _ => Err(()),
        }
    }
}

impl AsRef<str> for FreeBusyType {
    fn as_ref(&self) -> &str {
        match self {
            FreeBusyType::Free => KW_FBTYPE_FREE,
            FreeBusyType::Busy => KW_FBTYPE_BUSY,
            FreeBusyType::BusyUnavailable => KW_FBTYPE_BUSY_UNAVAILABLE,
            FreeBusyType::BusyTentative => KW_FBTYPE_BUSY_TENTATIVE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamValueType {
    Binary,
    Boolean,
    CalendarUserAddress,
    Date,
    DateTime,
    Duration,
    Float,
    Integer,
    Period,
    RecurrenceRule,
    Text,
    Time,
    Uri,
    UtcOffset,
    // TODO: add x-name and iana-token support
}

impl TryFrom<&SpannedSegments<'_>> for ParamValueType {
    type Error = ();

    fn try_from(segs: &SpannedSegments<'_>) -> Result<Self, Self::Error> {
        segs.resolve().parse() // PERF: avoid allocation
    }
}

impl FromStr for ParamValueType {
    type Err = ();

    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: check quote: Property parameter values that are not in quoted-strings are case-insensitive.
        match s {
            KW_BINARY      => Ok(ParamValueType::Binary),
            KW_BOOLEAN     => Ok(ParamValueType::Boolean),
            KW_CAL_ADDRESS => Ok(ParamValueType::CalendarUserAddress),
            KW_DATE        => Ok(ParamValueType::Date),
            KW_DATETIME    => Ok(ParamValueType::DateTime),
            KW_DURATION    => Ok(ParamValueType::Duration),
            KW_FLOAT       => Ok(ParamValueType::Float),
            KW_INTEGER     => Ok(ParamValueType::Integer),
            KW_PERIOD      => Ok(ParamValueType::Period),
            KW_RRULE       => Ok(ParamValueType::RecurrenceRule),
            KW_TEXT        => Ok(ParamValueType::Text),
            KW_URI         => Ok(ParamValueType::Uri),
            KW_TIME        => Ok(ParamValueType::Time),
            KW_UTC_OFFSET  => Ok(ParamValueType::UtcOffset),
            _ => Err(()),
        }
    }
}

impl AsRef<str> for ParamValueType {
    #[rustfmt::skip]
    fn as_ref(&self) -> &str {
        match self {
            ParamValueType::Binary              => KW_BINARY,
            ParamValueType::Boolean             => KW_BOOLEAN,
            ParamValueType::CalendarUserAddress => KW_CAL_ADDRESS,
            ParamValueType::Date                => KW_DATE,
            ParamValueType::DateTime            => KW_DATETIME,
            ParamValueType::Duration            => KW_DURATION,
            ParamValueType::Float               => KW_FLOAT,
            ParamValueType::Integer             => KW_INTEGER,
            ParamValueType::Period              => KW_PERIOD,
            ParamValueType::RecurrenceRule      => KW_RRULE,
            ParamValueType::Text                => KW_TEXT,
            ParamValueType::Time                => KW_TIME,
            ParamValueType::Uri                 => KW_URI,
            ParamValueType::UtcOffset           => KW_UTC_OFFSET,
        }
    }
}

impl Display for ParamValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
