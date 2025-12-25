// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::keyword::{
    KW_ALTREP, KW_CN, KW_CUTYPE, KW_DELEGATED_FROM, KW_DELEGATED_TO, KW_DIR, KW_ENCODING,
    KW_FBTYPE, KW_FMTTYPE, KW_LANGUAGE, KW_MEMBER, KW_PARTSTAT, KW_RANGE, KW_RELATED, KW_RELTYPE,
    KW_ROLE, KW_RSVP, KW_SENT_BY, KW_TZID, KW_VALUE,
};
use crate::lexer::Span;
use crate::syntax::{SpannedSegments, SyntaxParameter};
use crate::typed::TypedAnalysisError;
use crate::typed::parameter_types::{
    AlarmTriggerRelationship, CalendarUserType, Encoding, FreeBusyType, ParticipationRole,
    ParticipationStatus, RecurrenceIdRange, RelationshipType, ValueType,
    parse_alarm_trigger_relationship, parse_cutype, parse_encoding, parse_fbtype,
    parse_multiple_quoted, parse_partstat, parse_range, parse_reltype, parse_role, parse_rsvp,
    parse_single, parse_single_quoted, parse_tzid, parse_value_type,
};
use std::str::FromStr;

/// A typed iCalendar parameter with validated values.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum TypedParameter<'src> {
    /// This parameter specifies a URI that points to an alternate
    /// representation for a textual property value. A property specifying
    /// this parameter MUST also include a value that reflects the default
    /// representation of the text value
    ///
    /// See also: RFC 5545 Section 3.2.1. Alternate Text Representation
    #[allow(dead_code)]
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
    /// the property. Applications MUST treat x-name and iana-token values they
    /// don't recognize the same way as they would the UNKNOWN value.
    ///
    /// See also: RFC 5545 Section 3.2.3. Calendar User Type
    CalendarUserType { value: CalendarUserType, span: Span },

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
    /// property value.
    ///
    /// See also: RFC 5545 Section 3.2.7. Inline Encoding
    #[allow(dead_code)]
    Encoding { value: Encoding, span: Span },

    /// This parameter can be specified on properties that are used to
    /// reference an object. The parameter specifies the media type [RFC4288]
    /// of the referenced object. For example, on the "ATTACH" property, an FTP
    /// type URI value does not, by itself, necessarily convey the type of
    /// content associated with the resource. The parameter value MUST be the
    /// text for either an IANA-registered media type or a non-standard media
    /// type.
    ///
    /// See also: RFC 5545 Section 3.2.8. Format Type
    #[allow(dead_code)]
    FormatType {
        value: SpannedSegments<'src>,
        span: Span,
    },

    /// This parameter specifies the free or busy time type. Applications MUST
    /// treat x-name and iana-token values they don't recognize the same way as
    /// they would the BUSY value.
    ///
    /// See also: RFC 5545 Section 3.2.9. Free/Busy Time Type
    #[allow(dead_code)]
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
        value: ParticipationStatus,
        span: Span,
    },

    /// This parameter can be specified on a property that specifies a
    /// recurrence identifier. The parameter specifies the effective range of
    /// recurrence instances that is specified by the property. The effective
    /// range is from the recurrence identifier specified by the property. If
    /// this parameter is not specified on an allowed property, then the
    /// default range is the single instance specified by the recurrence
    /// identifier value of the property.
    ///
    /// See also: RFC 5545 Section 3.2.13. Recurrence Identifier Range
    #[allow(dead_code)]
    RecurrenceIdRange {
        value: RecurrenceIdRange,
        span: Span,
    },

    /// This parameter can be specified on properties that specify an alarm
    /// trigger with a "DURATION" value type. The parameter specifies whether
    /// the alarm will trigger relative to the start or end of the calendar
    /// component.
    ///
    /// See also: RFC 5545 Section 3.2.14. Alarm Trigger Relationship
    #[allow(dead_code)]
    AlarmTriggerRelationship {
        value: AlarmTriggerRelationship,
        span: Span,
    },

    /// This parameter can be specified on a property that references another
    /// related calendar. The parameter specifies the hierarchical relationship
    /// type of the calendar component referenced by the property. Applications
    /// MUST treat x-name and iana-token values they don't recognize the same
    /// way as they would the PARENT value.
    ///
    /// See also: RFC 5545 Section 3.2.15. Relationship Type
    #[allow(dead_code)]
    RelationshipType { value: RelationshipType, span: Span },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the participation role for the calendar
    /// user specified by the property in the group schedule calendar component.
    /// Applications MUST treat x-name and iana-token values they don't
    /// recognize the same way as they would the REQ-PARTICIPANT value.
    ///
    /// See also: RFC 5545 Section 3.2.16. Participation Role
    ParticipationRole {
        value: ParticipationRole,
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
    #[allow(dead_code)]
    TimeZoneIdentifier {
        value: SpannedSegments<'src>,

        #[cfg(feature = "jiff")]
        tz: jiff::tz::TimeZone,

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
    /// Returns the type of the parameter
    #[must_use]
    pub fn kind(&self) -> TypedParameterKind {
        match self {
            TypedParameter::AlternateText { .. } => TypedParameterKind::AlternateText,
            TypedParameter::CommonName { .. } => TypedParameterKind::CommonName,
            TypedParameter::CalendarUserType { .. } => TypedParameterKind::CalendarUserType,
            TypedParameter::Delegators { .. } => TypedParameterKind::Delegators,
            TypedParameter::Delegatees { .. } => TypedParameterKind::Delegatees,
            TypedParameter::Directory { .. } => TypedParameterKind::Directory,
            TypedParameter::Encoding { .. } => TypedParameterKind::Encoding,
            TypedParameter::FormatType { .. } => TypedParameterKind::FormatType,
            TypedParameter::FreeBusyType { .. } => TypedParameterKind::FreeBusyType,
            TypedParameter::Language { .. } => TypedParameterKind::Language,
            TypedParameter::GroupOrListMembership { .. } => {
                TypedParameterKind::GroupOrListMembership
            }
            TypedParameter::ParticipationStatus { .. } => TypedParameterKind::ParticipationStatus,
            TypedParameter::RecurrenceIdRange { .. } => TypedParameterKind::RecurrenceIdRange,
            TypedParameter::AlarmTriggerRelationship { .. } => {
                TypedParameterKind::AlarmTriggerRelationship
            }
            TypedParameter::RelationshipType { .. } => TypedParameterKind::RelationshipType,
            TypedParameter::ParticipationRole { .. } => TypedParameterKind::ParticipationRole,
            TypedParameter::SendBy { .. } => TypedParameterKind::SendBy,
            TypedParameter::RsvpExpectation { .. } => TypedParameterKind::RsvpExpectation,
            TypedParameter::TimeZoneIdentifier { .. } => TypedParameterKind::TimeZoneIdentifier,
            TypedParameter::ValueType { .. } => TypedParameterKind::ValueType,
        }
    }

    /// Name of the parameter (keyword)
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.kind().name()
    }

    /// Span of the parameter
    #[must_use]
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

    fn try_from(mut param: SyntaxParameter<'src>) -> Result<Self, Self::Error> {
        // Parse the parameter kind
        let Ok(kind) = TypedParameterKind::from_str(param.name.resolve().as_ref()) else {
            return Err(vec![TypedAnalysisError::ParameterUnknown {
                span: param.name.span(),
                parameter: param.name,
            }]);
        };

        // Handle parsing based on the kind
        match kind {
            TypedParameterKind::AlternateText => {
                parse_single_quoted(&mut param, kind).map(|value| TypedParameter::AlternateText {
                    value,
                    span: param.span(),
                })
            }
            TypedParameterKind::CommonName => {
                parse_single(&mut param, kind).map(|v| TypedParameter::CommonName {
                    value: v.value,
                    span: param.span(),
                })
            }
            TypedParameterKind::CalendarUserType => parse_cutype(param),
            TypedParameterKind::Delegators => {
                let span = param.span();
                parse_multiple_quoted(param, kind)
                    .map(|values| TypedParameter::Delegators { values, span })
            }
            TypedParameterKind::Delegatees => {
                let span = param.span();
                parse_multiple_quoted(param, kind)
                    .map(|values| TypedParameter::Delegatees { values, span })
            }
            TypedParameterKind::Directory => {
                parse_single_quoted(&mut param, kind).map(|value| TypedParameter::Directory {
                    value,
                    span: param.span(),
                })
            }
            TypedParameterKind::Encoding => parse_encoding(param),
            TypedParameterKind::FormatType => {
                parse_single(&mut param, kind).map(|v| TypedParameter::FormatType {
                    value: v.value,
                    span: param.span(),
                })
            }
            TypedParameterKind::FreeBusyType => parse_fbtype(param),
            TypedParameterKind::Language => {
                parse_single(&mut param, kind).map(|v| TypedParameter::Language {
                    value: v.value,
                    span: param.span(),
                })
            }
            TypedParameterKind::GroupOrListMembership => {
                let span = param.span();
                parse_multiple_quoted(param, kind)
                    .map(|values| TypedParameter::GroupOrListMembership { values, span })
            }
            TypedParameterKind::ParticipationStatus => parse_partstat(param),
            TypedParameterKind::RecurrenceIdRange => parse_range(param),
            TypedParameterKind::AlarmTriggerRelationship => parse_alarm_trigger_relationship(param),
            TypedParameterKind::RelationshipType => parse_reltype(param),
            TypedParameterKind::ParticipationRole => parse_role(param),
            TypedParameterKind::SendBy => {
                parse_single_quoted(&mut param, kind).map(|value| TypedParameter::SendBy {
                    value,
                    span: param.span(),
                })
            }
            TypedParameterKind::RsvpExpectation => parse_rsvp(param),
            TypedParameterKind::TimeZoneIdentifier => parse_tzid(param),
            TypedParameterKind::ValueType => parse_value_type(param),
        }
    }
}

/// Simple enum for `TypedParameter` types that maps parameter type to keyword
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum TypedParameterKind {
    AlternateText,
    CommonName,
    CalendarUserType,
    Delegators,
    Delegatees,
    Directory,
    Encoding,
    FormatType,
    FreeBusyType,
    Language,
    GroupOrListMembership,
    ParticipationStatus,
    RecurrenceIdRange,
    AlarmTriggerRelationship,
    RelationshipType,
    ParticipationRole,
    SendBy,
    RsvpExpectation,
    TimeZoneIdentifier,
    ValueType,
}

macro_rules! impl_typed_parameter_kind_mapping {
    (
        impl $ty:ident {
            $(
                $variant:ident => $kw:ident
            ),+ $(,)?
        }
    ) => {
        impl $ty {
            /// Returns the name keyword for the parameter type
            pub const fn name(self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $kw,
                    )+
                }
            }
        }

        impl FromStr for $ty {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $kw => Ok(Self::$variant),
                    )+
                    _ => Err(()),
                }
            }
        }
    };
}

impl_typed_parameter_kind_mapping! {
    impl TypedParameterKind {
        AlternateText       => KW_ALTREP,
        CommonName          => KW_CN,
        CalendarUserType    => KW_CUTYPE,
        Delegators          => KW_DELEGATED_FROM,
        Delegatees          => KW_DELEGATED_TO,
        Directory           => KW_DIR,
        Encoding            => KW_ENCODING,
        FormatType          => KW_FMTTYPE,
        FreeBusyType        => KW_FBTYPE,
        Language            => KW_LANGUAGE,
        GroupOrListMembership => KW_MEMBER,
        ParticipationStatus => KW_PARTSTAT,
        RecurrenceIdRange   => KW_RANGE,
        AlarmTriggerRelationship => KW_RELATED,
        RelationshipType    => KW_RELTYPE,
        ParticipationRole   => KW_ROLE,
        SendBy              => KW_SENT_BY,
        RsvpExpectation     => KW_RSVP,
        TimeZoneIdentifier  => KW_TZID,
        ValueType           => KW_VALUE,
    }
}
