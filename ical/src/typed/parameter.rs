// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Display, str::FromStr};

use crate::keyword::{
    KW_ALTREP, KW_BINARY, KW_BOOLEAN, KW_CAL_ADDRESS, KW_CN, KW_CUTYPE, KW_DATE, KW_DATETIME,
    KW_DELEGATED_FROM, KW_DELEGATED_TO, KW_DIR, KW_DURATION, KW_ENCODING, KW_ENCODING_8BIT,
    KW_ENCODING_BASE64, KW_FALSE, KW_FBTYPE, KW_FBTYPE_BUSY, KW_FBTYPE_BUSY_TENTATIVE,
    KW_FBTYPE_BUSY_UNAVAILABLE, KW_FBTYPE_FREE, KW_FLOAT, KW_FMTTYPE, KW_INTEGER, KW_LANGUAGE,
    KW_MEMBER, KW_PARTSTAT, KW_PERIOD, KW_RANGE, KW_RELATED, KW_RELTYPE, KW_ROLE, KW_RRULE,
    KW_RSVP, KW_SENT_BY, KW_TEXT, KW_TIME, KW_TRUE, KW_TZID, KW_URI, KW_UTC_OFFSET, KW_VALUE,
};
use crate::lexer::Span;
use crate::syntax::{SpannedSegments, SyntaxParameter, SyntaxParameterValue};
use crate::typed::TypedAnalysisError;

#[derive(Debug, Clone)]
pub enum TypedParameter<'src> {
    /// This parameter specifies a URI that points to an alternate
    /// representation for a textual property value. A property specifying
    /// this parameter MUST also include a value that reflects the default
    /// representation of the text value
    ///
    /// See also: RFC 5545 Section 3.2.1. Alternate Text Representation
    AlternateText {
        value: SpannedSegments<'src>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the common name to be associated with
    /// the calendar user specified by the property. The parameter value is
    /// text. The parameter value can be used for display text to be associated
    /// with the calendar address specified by the property.
    ///
    /// See also: RFC 5545 Section 3.2.2. Common Name
    CommonName {
        value: SpannedSegments<'src>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter identifies the type of calendar user specified by
    /// the property. If not specified on a property that allows this parameter,
    /// the default is INDIVIDUAL. Applications MUST treat x-name and iana-
    /// token values they don't recognize the same way as they would the
    /// UNKNOWN value.
    ///
    /// See also: RFC 5545 Section 3.2.3. Calendar User Type
    CalendarUserType {
        value: SpannedSegments<'src>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. This parameter specifies those calendar users that have delegated
    /// their participation in a group-scheduled event or to-do to the calendar
    /// user specified by the property.
    ///
    /// See also: RFC 5545 Section 3.2.4. Delegators
    Delegators {
        values: Vec<SpannedSegments<'src>>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. This parameter specifies those calendar users whom have been
    /// delegated participation in a group-scheduled event or to-do by the
    /// calendar user specified by the property.
    ///
    /// See also: RFC 5545 Section 3.2.5. Delegatees
    Delegatees {
        values: Vec<SpannedSegments<'src>>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies a reference to the directory entry
    /// associated with the calendar user specified by the property. The
    /// parameter value is a URI.
    ///
    /// See also: RFC 5545 Section 3.2.6. Directory Entry Reference
    Directory {
        value: SpannedSegments<'src>,
        span: Span,
    },

    /// This property parameter identifies the inline encoding used in a
    /// property value.  The default encoding is "8BIT", corresponding to a
    /// property value consisting of text.  The "BASE64" encoding type
    /// corresponds to a property value encoded using the "BASE64" encoding
    /// defined in [RFC2045].
    ///
    /// See also: RFC 5545 Section 3.2.7. Inline Encoding
    Encoding { value: ParamEncoding, span: Span },

    /// This parameter can be specified on properties that are used to
    /// reference an object. The parameter specifies the media type [RFC4288]
    /// of the referenced object. For example, on the "ATTACH" property, an FTP
    /// type URI value does not, by itself, necessarily convey the type of
    /// content associated with the resource. The parameter value MUST be the
    /// text for either an IANA-registered media type or a non-standard media
    /// type.
    ///
    /// See also: RFC 5545 Section 3.2.8. Format Type
    FormatType {
        value: SpannedSegments<'src>,
        span: Span,
    },

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
    FreeBusyType { value: FreeBusyType, span: Span },

    /// This parameter identifies the language of the text in the property
    /// value and of all property parameter values of the property. The value
    /// of the "LANGUAGE" property parameter is that defined in [RFC5646].
    ///
    /// For transport in a MIME entity, the Content-Language header field can
    /// be used to set the default language for the entire body part. Otherwise,
    /// no default language is assumed.
    ///
    /// See also: RFC 5545 Section 3.2.10. Language
    Language {
        value: SpannedSegments<'src>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter identifies the groups or list membership for the
    /// calendar user specified by the property. The parameter value is either
    /// a single calendar address in a quoted-string or a COMMA-separated list
    /// of calendar addresses, each in a quoted-string. The individual calendar
    /// address parameter values MUST each be specified in a quoted-string.
    ///
    /// See also: RFC 5545 Section 3.2.11. Group or List Membership
    GroupOrListMembership {
        values: Vec<SpannedSegments<'src>>,
        span: Span,
    },

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
    ParticipationStatus {
        value: SpannedSegments<'src>,
        span: Span,
    },

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
    RecurrenceIdRange {
        value: SpannedSegments<'src>,
        span: Span,
    },

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
    AlarmTriggerRelationship {
        value: SpannedSegments<'src>,
        span: Span,
    },

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
    RelationshipType {
        value: SpannedSegments<'src>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the participation role for the calendar
    /// user specified by the property in the group schedule calendar component.
    /// If not specified on a property that allows this parameter, the default
    /// value is REQ-PARTICIPANT. Applications MUST treat x-name and iana-token
    /// values they don't recognize the same way as they would the REQ-PARTICIPANT value.
    ///
    /// See also: RFC 5545 Section 3.2.16. Participation Role
    ParticipationRole {
        value: SpannedSegments<'src>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the calendar user that is acting on behalf
    /// of the calendar user specified by the property. The parameter value MUST
    /// be a mailto URI as defined in [RFC2368]. The individual calendar address
    /// parameter values MUST each be specified in a quoted-string.
    ///
    /// See also: RFC 5545 Section 3.2.18. Sent By
    SendBy {
        value: SpannedSegments<'src>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter identifies the expectation of a reply from the
    /// calendar user specified by the property value. This parameter is used
    /// by the "Organizer" to request a participation status reply from an
    /// "Attendee" of a group-scheduled event or to-do. If not specified on a
    /// property that allows this parameter, the default value is FALSE.
    RsvpExpectation { value: bool, span: Span },

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
    TimeZoneIdentifier {
        value: SpannedSegments<'src>,
        span: Span,
    },

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
    ValueType { value: ValueType, span: Span },
}

impl TypedParameter<'_> {
    /// Name of the parameter
    pub fn name(&self) -> &'static str {
        match self {
            TypedParameter::AlternateText { .. } => KW_ALTREP,
            TypedParameter::CommonName { .. } => KW_CN,
            TypedParameter::CalendarUserType { .. } => KW_CUTYPE,
            TypedParameter::Delegators { .. } => KW_DELEGATED_FROM,
            TypedParameter::Delegatees { .. } => KW_DELEGATED_TO,
            TypedParameter::Directory { .. } => KW_DIR,
            TypedParameter::Encoding { .. } => KW_ENCODING,
            TypedParameter::FormatType { .. } => KW_FMTTYPE,
            TypedParameter::FreeBusyType { .. } => KW_FBTYPE,
            TypedParameter::Language { .. } => KW_LANGUAGE,
            TypedParameter::GroupOrListMembership { .. } => KW_MEMBER,
            TypedParameter::ParticipationStatus { .. } => KW_PARTSTAT,
            TypedParameter::RecurrenceIdRange { .. } => KW_RANGE,
            TypedParameter::AlarmTriggerRelationship { .. } => KW_RELATED,
            TypedParameter::RelationshipType { .. } => KW_RELTYPE,
            TypedParameter::ParticipationRole { .. } => KW_ROLE,
            TypedParameter::SendBy { .. } => KW_SENT_BY,
            TypedParameter::RsvpExpectation { .. } => KW_RSVP,
            TypedParameter::TimeZoneIdentifier { .. } => KW_TZID,
            TypedParameter::ValueType { .. } => KW_VALUE,
        }
    }

    /// Span of the parameter
    pub fn span(&self) -> Span {
        match self {
            TypedParameter::AlternateText { span, .. }
            | TypedParameter::CommonName { span, .. }
            | TypedParameter::CalendarUserType { span, .. }
            | TypedParameter::Delegators { span, .. }
            | TypedParameter::Delegatees { span, .. }
            | TypedParameter::Directory { span, .. }
            | TypedParameter::Encoding { span, .. }
            | TypedParameter::FormatType { span, .. }
            | TypedParameter::FreeBusyType { span, .. }
            | TypedParameter::Language { span, .. }
            | TypedParameter::GroupOrListMembership { span, .. }
            | TypedParameter::ParticipationStatus { span, .. }
            | TypedParameter::RecurrenceIdRange { span, .. }
            | TypedParameter::AlarmTriggerRelationship { span, .. }
            | TypedParameter::RelationshipType { span, .. }
            | TypedParameter::ParticipationRole { span, .. }
            | TypedParameter::SendBy { span, .. }
            | TypedParameter::RsvpExpectation { span, .. }
            | TypedParameter::TimeZoneIdentifier { span, .. }
            | TypedParameter::ValueType { span, .. } => span.clone(),
        }
    }
}

impl<'src> TryFrom<SyntaxParameter<'src>> for TypedParameter<'src> {
    type Error = Vec<TypedAnalysisError<'src>>;

    #[allow(clippy::too_many_lines)]
    fn try_from(param: SyntaxParameter<'src>) -> Result<Self, Self::Error> {
        let span = param.span();
        let single =
            |mut param: SyntaxParameter<'src>, parameter: &'src str| match param.values.len() {
                1 => Ok(param.values.pop().unwrap()),
                _ => Err(vec![
                    TypedAnalysisError::ParameterMultipleValuesDisallowed {
                        parameter,
                        span: param.span(),
                    },
                ]),
            };

        let multiple =
            |param: SyntaxParameter<'src>| Ok(param.values.into_iter().map(|v| v.value).collect());

        match param.name.resolve().as_ref() {
            // Single value parameters
            KW_ALTREP => single(param, KW_ALTREP).map(|v| TypedParameter::AlternateText {
                value: v.value,
                span,
            }),
            KW_CN => single(param, KW_CN).map(|v| TypedParameter::CommonName {
                value: v.value,
                span,
            }),
            KW_CUTYPE => single(param, KW_CUTYPE).map(|v| TypedParameter::CalendarUserType {
                value: v.value,
                span,
            }),
            KW_DIR => single(param, KW_DIR).map(|v| TypedParameter::Directory {
                value: v.value,
                span,
            }),
            KW_ENCODING => single(param, KW_ENCODING).and_then(|v| {
                v.value
                    .resolve()
                    .parse()
                    .map(|encoding| TypedParameter::Encoding {
                        value: encoding,
                        span,
                    })
                    .map_err(|()| {
                        vec![TypedAnalysisError::ParameterInvalidValue {
                            span: v.value.span(),
                            parameter: KW_ENCODING,
                            value: v.value,
                        }]
                    })
            }),
            KW_FMTTYPE => single(param, KW_FMTTYPE).map(|v| TypedParameter::FormatType {
                value: v.value,
                span,
            }),
            KW_FBTYPE => single(param, KW_FBTYPE).and_then(|v| {
                v.value
                    .resolve()
                    .parse()
                    .map(|fbtype| TypedParameter::FreeBusyType {
                        value: fbtype,
                        span,
                    })
                    .map_err(|()| {
                        vec![TypedAnalysisError::ParameterInvalidValue {
                            span: v.value.span(),
                            parameter: KW_FBTYPE,
                            value: v.value,
                        }]
                    })
            }),
            KW_LANGUAGE => single(param, KW_LANGUAGE).map(|v| TypedParameter::Language {
                value: v.value,
                span,
            }),
            KW_PARTSTAT => {
                single(param, KW_PARTSTAT).map(|v| TypedParameter::ParticipationStatus {
                    value: v.value,
                    span,
                })
            }
            KW_RANGE => single(param, KW_RANGE).map(|v| TypedParameter::RecurrenceIdRange {
                value: v.value,
                span,
            }),
            KW_RELATED => {
                single(param, KW_RELATED).map(|v| TypedParameter::AlarmTriggerRelationship {
                    value: v.value,
                    span,
                })
            }
            KW_RELTYPE => single(param, KW_RELTYPE).map(|v| TypedParameter::RelationshipType {
                value: v.value,
                span,
            }),
            KW_ROLE => single(param, KW_ROLE).map(|v| TypedParameter::ParticipationRole {
                value: v.value,
                span,
            }),
            KW_RSVP => single(param, KW_RSVP).and_then(|v| match parse_rsvp_value(&v) {
                Ok(value) => Ok(TypedParameter::RsvpExpectation { value, span }),
                Err(()) => Err(vec![TypedAnalysisError::ParameterInvalidValue {
                    parameter: KW_RSVP,
                    value: v.value,
                    span,
                }]),
            }),
            KW_SENT_BY => single(param, KW_SENT_BY).map(|v| TypedParameter::SendBy {
                value: v.value,
                span,
            }),
            KW_TZID => single(param, KW_TZID).map(|v| TypedParameter::TimeZoneIdentifier {
                value: v.value,
                span,
            }), // TODO: validate
            KW_VALUE => single(param, KW_VALUE).and_then(|v| {
                v.value
                    .resolve()
                    .parse()
                    .map(|value_type| TypedParameter::ValueType {
                        value: value_type,
                        span,
                    })
                    .map_err(|()| {
                        vec![TypedAnalysisError::ParameterInvalidValue {
                            span: v.value.span(),
                            parameter: KW_VALUE,
                            value: v.value,
                        }]
                    })
            }),

            // Multiple value parameters
            KW_DELEGATED_FROM => {
                multiple(param).map(|values| TypedParameter::Delegators { values, span })
            }
            KW_DELEGATED_TO => {
                multiple(param).map(|values| TypedParameter::Delegatees { values, span })
            }
            KW_MEMBER => {
                multiple(param).map(|values| TypedParameter::GroupOrListMembership { values, span })
            }

            // Unknown parameter - treat as unknown x-name or iana-token
            // According to RFC 5545, applications MUST treat x-name and iana-token values
            // they don't recognize the same way as they would the UNKNOWN value
            _ => Err(vec![TypedAnalysisError::ParameterUnknown {
                span: param.name.span(),
                parameter: param.name,
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

fn parse_rsvp_value(v: &SyntaxParameterValue) -> Result<bool, ()> {
    if v.value.eq_ignore_ascii_case(KW_TRUE) {
        Ok(true)
    } else if v.value.eq_ignore_ascii_case(KW_FALSE) {
        Ok(false)
    } else {
        Err(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
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

impl TryFrom<&SpannedSegments<'_>> for ValueType {
    type Error = ();

    fn try_from(segs: &SpannedSegments<'_>) -> Result<Self, Self::Error> {
        segs.resolve().parse() // PERF: avoid allocation
    }
}

impl FromStr for ValueType {
    type Err = ();

    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: check quote: Property parameter values that are not in quoted-strings are case-insensitive.
        match s {
            KW_BINARY      => Ok(ValueType::Binary),
            KW_BOOLEAN     => Ok(ValueType::Boolean),
            KW_CAL_ADDRESS => Ok(ValueType::CalendarUserAddress),
            KW_DATE        => Ok(ValueType::Date),
            KW_DATETIME    => Ok(ValueType::DateTime),
            KW_DURATION    => Ok(ValueType::Duration),
            KW_FLOAT       => Ok(ValueType::Float),
            KW_INTEGER     => Ok(ValueType::Integer),
            KW_PERIOD      => Ok(ValueType::Period),
            KW_RRULE       => Ok(ValueType::RecurrenceRule),
            KW_TEXT        => Ok(ValueType::Text),
            KW_URI         => Ok(ValueType::Uri),
            KW_TIME        => Ok(ValueType::Time),
            KW_UTC_OFFSET  => Ok(ValueType::UtcOffset),
            _ => Err(()),
        }
    }
}

impl AsRef<str> for ValueType {
    #[rustfmt::skip]
    fn as_ref(&self) -> &str {
        match self {
            ValueType::Binary              => KW_BINARY,
            ValueType::Boolean             => KW_BOOLEAN,
            ValueType::CalendarUserAddress => KW_CAL_ADDRESS,
            ValueType::Date                => KW_DATE,
            ValueType::DateTime            => KW_DATETIME,
            ValueType::Duration            => KW_DURATION,
            ValueType::Float               => KW_FLOAT,
            ValueType::Integer             => KW_INTEGER,
            ValueType::Period              => KW_PERIOD,
            ValueType::RecurrenceRule      => KW_RRULE,
            ValueType::Text                => KW_TEXT,
            ValueType::Time                => KW_TIME,
            ValueType::Uri                 => KW_URI,
            ValueType::UtcOffset           => KW_UTC_OFFSET,
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
