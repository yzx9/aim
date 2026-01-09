// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parameter parsing module for iCalendar parameters.
//!
//! This module handles the parsing and validation of iCalendar parameters
//! as defined in RFC 5545 Section 3.2.

#[macro_use]
mod util;

mod definition;
mod kind;

pub use definition::{
    AlarmTriggerRelationship, CalendarUserType, CalendarUserTypeOwned, CalendarUserTypeRef,
    Encoding, FreeBusyType, FreeBusyTypeOwned, FreeBusyTypeRef, ParticipationRole,
    ParticipationRoleOwned, ParticipationRoleRef, ParticipationStatus, ParticipationStatusOwned,
    ParticipationStatusRef, RecurrenceIdRange, RelationshipType, RelationshipTypeOwned,
    RelationshipTypeRef, ValueType, ValueTypeOwned, ValueTypeRef,
};
pub use kind::{ParameterKind, ParameterKindOwned, ParameterKindRef};

use crate::parameter::definition::{
    parse_alarm_trigger_relationship, parse_cutype, parse_encoding, parse_fbtype, parse_partstat,
    parse_range, parse_reltype, parse_role, parse_rsvp, parse_tzid, parse_value_type,
};
use crate::parameter::util::{parse_multiple_quoted, parse_single, parse_single_quoted};
use crate::string_storage::{Span, SpannedSegments, StringStorage};
use crate::syntax::{SyntaxParameter, SyntaxParameterRef};
use crate::typed::TypedError;

/// A typed iCalendar parameter with validated values.
#[derive(Debug, Clone)]
#[expect(missing_docs)]
pub enum Parameter<S: StringStorage> {
    /// This parameter specifies a URI that points to an alternate
    /// representation for a textual property value. A property specifying
    /// this parameter MUST also include a value that reflects the default
    /// representation of the text value
    ///
    /// See also: RFC 5545 Section 3.2.1. Alternate Text Representation
    AlternateText { value: S, span: Span },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the common name to be associated with
    /// the calendar user specified by the property. The parameter value is
    /// text. The parameter value can be used for display text to be associated
    /// with the calendar address specified by the property.
    ///
    /// See also: RFC 5545 Section 3.2.2. Common Name
    CommonName { value: S, span: Span },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter identifies the type of calendar user specified by
    /// the property. Applications MUST treat x-name and iana-token values they
    /// don't recognize the same way as they would the UNKNOWN value.
    ///
    /// See also: RFC 5545 Section 3.2.3. Calendar User Type
    CalendarUserType {
        value: CalendarUserType<S>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. This parameter specifies those calendar users that have delegated
    /// their participation in a group-scheduled event or to-do to the calendar
    /// user specified by the property.
    ///
    /// See also: RFC 5545 Section 3.2.4. Delegators
    Delegators { values: Vec<S>, span: Span },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. This parameter specifies those calendar users whom have been
    /// delegated participation in a group-scheduled event or to-do by the
    /// calendar user specified by the property.
    ///
    /// See also: RFC 5545 Section 3.2.5. Delegatees
    Delegatees { values: Vec<S>, span: Span },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies a reference to the directory entry
    /// associated with the calendar user specified by the property. The
    /// parameter value is a URI.
    ///
    /// See also: RFC 5545 Section 3.2.6. Directory Entry Reference
    Directory { value: S, span: Span },

    /// This property parameter identifies the inline encoding used in a
    /// property value.
    ///
    /// See also: RFC 5545 Section 3.2.7. Inline Encoding
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
    FormatType { value: S, span: Span },

    /// This parameter specifies the free or busy time type. Applications MUST
    /// treat x-name and iana-token values they don't recognize the same way as
    /// they would the BUSY value.
    ///
    /// See also: RFC 5545 Section 3.2.9. Free/Busy Time Type
    FreeBusyType { value: FreeBusyType<S>, span: Span },

    /// This parameter identifies the language of the text in the property
    /// value and of all property parameter values of the property. The value
    /// of the "LANGUAGE" property parameter is that defined in [RFC5646].
    ///
    /// For transport in a MIME entity, the Content-Language header field can
    /// be used to set the default language for the entire body part. Otherwise,
    /// no default language is assumed.
    ///
    /// See also: RFC 5545 Section 3.2.10. Language
    Language { value: S, span: Span },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter identifies the groups or list membership for the
    /// calendar user specified by the property. The parameter value is either
    /// a single calendar address in a quoted-string or a COMMA-separated list
    /// of calendar addresses, each in a quoted-string. The individual calendar
    /// address parameter values MUST each be specified in a quoted-string.
    ///
    /// See also: RFC 5545 Section 3.2.11. Group or List Membership
    GroupOrListMembership { values: Vec<S>, span: Span },

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
        value: ParticipationStatus<S>,
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
    RelationshipType {
        value: RelationshipType<S>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the participation role for the calendar
    /// user specified by the property in the group schedule calendar component.
    /// Applications MUST treat x-name and iana-token values they don't
    /// recognize the same way as they would the REQ-PARTICIPANT value.
    ///
    /// See also: RFC 5545 Section 3.2.16. Participation Role
    ParticipationRole {
        value: ParticipationRole<S>,
        span: Span,
    },

    /// This parameter can be specified on properties with a CAL-ADDRESS value
    /// type. The parameter specifies the calendar user that is acting on behalf
    /// of the calendar user specified by the property. The parameter value MUST
    /// be a mailto URI as defined in [RFC2368]. The individual calendar address
    /// parameter values MUST each be specified in a quoted-string.
    ///
    /// See also: RFC 5545 Section 3.2.18. Sent By
    SendBy { value: S, span: Span },

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
        /// The TZID parameter value
        value: S,
        /// The time zone definition associated with this TZID
        #[cfg(feature = "jiff")]
        tz: jiff::tz::TimeZone,
        /// The span of the parameter
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
    ValueType { value: ValueType<S>, span: Span },

    /// Custom experimental x-name parameter.
    ///
    /// Per RFC 5545 Section 3.2: Applications MUST ignore x-param values,
    /// but preserve the data for round-trip compatibility.
    ///
    /// See also: RFC 5545 Section 3.2 (Parameter definition)
    XName {
        /// Parameter name (including the "X-" prefix)
        name: S,
        /// Raw parameter (unparsed)
        raw: SyntaxParameter<S>,
    },

    /// Unrecognized iana-token parameter.
    ///
    /// Per RFC 5545 Section 3.2: Applications MUST ignore iana-param values
    /// they don't recognize, but preserve the data for round-trip compatibility.
    ///
    /// See also: RFC 5545 Section 3.2 (Parameter definition)
    Unrecognized {
        /// Parameter name
        name: S,
        /// Raw parameter (unparsed)
        raw: SyntaxParameter<S>,
    },
}

/// Type alias for borrowed parameter
pub type ParameterRef<'src> = Parameter<SpannedSegments<'src>>;

/// Type alias for owned parameter
pub type ParameterOwned = Parameter<String>;

impl<'src> ParameterRef<'src> {
    /// Returns the type of the parameter
    #[must_use]
    pub fn into_kind(self) -> ParameterKindRef<'src> {
        match self {
            Parameter::AlternateText { .. } => ParameterKind::AlternateText,
            Parameter::CommonName { .. } => ParameterKind::CommonName,
            Parameter::CalendarUserType { .. } => ParameterKind::CalendarUserType,
            Parameter::Delegators { .. } => ParameterKind::Delegators,
            Parameter::Delegatees { .. } => ParameterKind::Delegatees,
            Parameter::Directory { .. } => ParameterKind::Directory,
            Parameter::Encoding { .. } => ParameterKind::Encoding,
            Parameter::FormatType { .. } => ParameterKind::FormatType,
            Parameter::FreeBusyType { .. } => ParameterKind::FreeBusyType,
            Parameter::Language { .. } => ParameterKind::Language,
            Parameter::GroupOrListMembership { .. } => ParameterKind::GroupOrListMembership,
            Parameter::ParticipationStatus { .. } => ParameterKind::ParticipationStatus,
            Parameter::RecurrenceIdRange { .. } => ParameterKind::RecurrenceIdRange,
            Parameter::AlarmTriggerRelationship { .. } => ParameterKind::AlarmTriggerRelationship,
            Parameter::RelationshipType { .. } => ParameterKind::RelationshipType,
            Parameter::ParticipationRole { .. } => ParameterKind::ParticipationRole,
            Parameter::SendBy { .. } => ParameterKind::SendBy,
            Parameter::RsvpExpectation { .. } => ParameterKind::RsvpExpectation,
            Parameter::TimeZoneIdentifier { .. } => ParameterKind::TimeZoneIdentifier,
            Parameter::ValueType { .. } => ParameterKind::ValueType,
            Parameter::XName { name, .. } => ParameterKind::XName(name),
            Parameter::Unrecognized { name, .. } => ParameterKind::Unrecognized(name),
        }
    }

    /// Span of the parameter
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Parameter::AlternateText { span, .. }
            | Parameter::CommonName { span, .. }
            | Parameter::CalendarUserType { span, .. }
            | Parameter::Delegators { span, .. }
            | Parameter::Delegatees { span, .. }
            | Parameter::Directory { span, .. }
            | Parameter::Encoding { span, .. }
            | Parameter::FormatType { span, .. }
            | Parameter::FreeBusyType { span, .. }
            | Parameter::Language { span, .. }
            | Parameter::GroupOrListMembership { span, .. }
            | Parameter::ParticipationStatus { span, .. }
            | Parameter::RecurrenceIdRange { span, .. }
            | Parameter::AlarmTriggerRelationship { span, .. }
            | Parameter::RelationshipType { span, .. }
            | Parameter::ParticipationRole { span, .. }
            | Parameter::SendBy { span, .. }
            | Parameter::RsvpExpectation { span, .. }
            | Parameter::TimeZoneIdentifier { span, .. }
            | Parameter::ValueType { span, .. } => *span,

            Parameter::XName { raw, .. } | Parameter::Unrecognized { raw, .. } => raw.span(),
        }
    }

    /// Convert borrowed type to owned type
    #[must_use]
    #[expect(clippy::too_many_lines)]
    pub fn to_owned(&self) -> ParameterOwned {
        // TODO: how to remove span from owned type?
        let span = Span::new(0, 0); // Placeholder span for owned type
        match self {
            Parameter::AlternateText { value, .. } => ParameterOwned::AlternateText {
                value: value.to_owned(),
                span,
            },
            Parameter::CommonName { value, .. } => ParameterOwned::CommonName {
                value: value.to_owned(),
                span,
            },
            Parameter::CalendarUserType { value, .. } => ParameterOwned::CalendarUserType {
                value: value.to_owned(),
                span,
            },
            Parameter::Delegators { values, .. } => ParameterOwned::Delegators {
                values: values.iter().map(SpannedSegments::to_owned).collect(),
                span,
            },
            Parameter::Delegatees { values, .. } => ParameterOwned::Delegatees {
                values: values.iter().map(SpannedSegments::to_owned).collect(),
                span,
            },
            Parameter::Directory { value, .. } => ParameterOwned::Directory {
                value: value.to_owned(),
                span,
            },
            Parameter::Encoding { value, .. } => ParameterOwned::Encoding {
                value: *value,
                span,
            },
            Parameter::FormatType { value, .. } => ParameterOwned::FormatType {
                value: value.to_owned(),
                span,
            },
            Parameter::FreeBusyType { value, .. } => ParameterOwned::FreeBusyType {
                value: value.to_owned(),
                span,
            },
            Parameter::Language { value, .. } => ParameterOwned::Language {
                value: value.to_owned(),
                span,
            },
            Parameter::GroupOrListMembership { values, .. } => {
                ParameterOwned::GroupOrListMembership {
                    values: values.iter().map(SpannedSegments::to_owned).collect(),
                    span,
                }
            }
            Parameter::ParticipationStatus { value, .. } => ParameterOwned::ParticipationStatus {
                value: value.to_owned(),
                span,
            },
            Parameter::RecurrenceIdRange { value, .. } => ParameterOwned::RecurrenceIdRange {
                value: *value,
                span,
            },
            Parameter::AlarmTriggerRelationship { value, .. } => {
                ParameterOwned::AlarmTriggerRelationship {
                    value: *value,
                    span,
                }
            }
            Parameter::RelationshipType { value, .. } => ParameterOwned::RelationshipType {
                value: value.to_owned(),
                span,
            },
            Parameter::ParticipationRole { value, .. } => ParameterOwned::ParticipationRole {
                value: value.to_owned(),
                span,
            },
            Parameter::SendBy { value, .. } => ParameterOwned::SendBy {
                value: value.to_owned(),
                span,
            },
            Parameter::RsvpExpectation { value, .. } => ParameterOwned::RsvpExpectation {
                value: *value,
                span,
            },
            Parameter::TimeZoneIdentifier {
                value,
                #[cfg(feature = "jiff")]
                tz,
                ..
            } => ParameterOwned::TimeZoneIdentifier {
                value: value.to_owned(),
                #[cfg(feature = "jiff")]
                tz: tz.clone(),
                span,
            },
            Parameter::ValueType { value, .. } => ParameterOwned::ValueType {
                value: value.to_owned(),
                span,
            },
            Parameter::XName { name, raw } => ParameterOwned::XName {
                name: name.to_owned(),
                raw: raw.to_owned(),
            },
            Parameter::Unrecognized { name, raw } => ParameterOwned::Unrecognized {
                name: name.to_owned(),
                raw: raw.to_owned(),
            },
        }
    }
}

impl<'src> TryFrom<SyntaxParameterRef<'src>> for ParameterRef<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(mut param: SyntaxParameterRef<'src>) -> Result<Self, Self::Error> {
        // Parse the parameter kind
        let kind = ParameterKind::from(param.name.clone());

        // Handle parsing based on the kind
        match kind {
            ParameterKind::AlternateText => {
                parse_single_quoted(&mut param, kind).map(|value| Parameter::AlternateText {
                    value,
                    span: param.span(),
                })
            }
            ParameterKind::CommonName => {
                parse_single(&mut param, kind).map(|v| Parameter::CommonName {
                    value: v.value,
                    span: param.span(),
                })
            }
            ParameterKind::CalendarUserType => parse_cutype(param),
            ParameterKind::Delegators => {
                let span = param.span();
                parse_multiple_quoted(param, &kind)
                    .map(|values| Parameter::Delegators { values, span })
            }
            ParameterKind::Delegatees => {
                let span = param.span();
                parse_multiple_quoted(param, &kind)
                    .map(|values| Parameter::Delegatees { values, span })
            }
            ParameterKind::Directory => {
                parse_single_quoted(&mut param, kind).map(|value| Parameter::Directory {
                    value,
                    span: param.span(),
                })
            }
            ParameterKind::Encoding => parse_encoding(param),
            ParameterKind::FormatType => {
                parse_single(&mut param, kind).map(|v| Parameter::FormatType {
                    value: v.value,
                    span: param.span(),
                })
            }
            ParameterKind::FreeBusyType => parse_fbtype(param),
            ParameterKind::Language => {
                parse_single(&mut param, kind).map(|v| Parameter::Language {
                    value: v.value,
                    span: param.span(),
                })
            }
            ParameterKind::GroupOrListMembership => {
                let span = param.span();
                parse_multiple_quoted(param, &kind)
                    .map(|values| Parameter::GroupOrListMembership { values, span })
            }
            ParameterKind::ParticipationStatus => parse_partstat(param),
            ParameterKind::RecurrenceIdRange => parse_range(param),
            ParameterKind::AlarmTriggerRelationship => parse_alarm_trigger_relationship(param),
            ParameterKind::RelationshipType => parse_reltype(param),
            ParameterKind::ParticipationRole => parse_role(param),
            ParameterKind::SendBy => {
                parse_single_quoted(&mut param, kind).map(|value| Parameter::SendBy {
                    value,
                    span: param.span(),
                })
            }
            ParameterKind::RsvpExpectation => parse_rsvp(param),
            ParameterKind::TimeZoneIdentifier => parse_tzid(param),
            ParameterKind::ValueType => parse_value_type(param),
            // Preserve unknown parameter per RFC 5545 Section 3.2
            // TODO: emit warning for x-name / unrecognized iana-token parameter
            ParameterKind::XName(name) => Ok(Parameter::XName { name, raw: param }),
            ParameterKind::Unrecognized(name) => Ok(Parameter::Unrecognized { name, raw: param }),
        }
    }
}
