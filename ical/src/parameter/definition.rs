// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use crate::keyword::{
    KW_BINARY, KW_BOOLEAN, KW_CAL_ADDRESS, KW_CUTYPE_GROUP, KW_CUTYPE_INDIVIDUAL,
    KW_CUTYPE_RESOURCE, KW_CUTYPE_ROOM, KW_CUTYPE_UNKNOWN, KW_DATE, KW_DATETIME, KW_DURATION,
    KW_ENCODING_8BIT, KW_ENCODING_BASE64, KW_FBTYPE_BUSY, KW_FBTYPE_BUSY_TENTATIVE,
    KW_FBTYPE_BUSY_UNAVAILABLE, KW_FBTYPE_FREE, KW_FLOAT, KW_INTEGER, KW_PARTSTAT_ACCEPTED,
    KW_PARTSTAT_COMPLETED, KW_PARTSTAT_DECLINED, KW_PARTSTAT_DELEGATED, KW_PARTSTAT_IN_PROCESS,
    KW_PARTSTAT_NEEDS_ACTION, KW_PARTSTAT_TENTATIVE, KW_PERIOD, KW_RANGE_THISANDFUTURE,
    KW_RELATED_END, KW_RELATED_START, KW_RELTYPE_CHILD, KW_RELTYPE_PARENT, KW_RELTYPE_SIBLING,
    KW_ROLE_CHAIR, KW_ROLE_NON_PARTICIPANT, KW_ROLE_OPT_PARTICIPANT, KW_ROLE_REQ_PARTICIPANT,
    KW_RRULE, KW_RSVP_FALSE, KW_RSVP_TRUE, KW_TEXT, KW_TIME, KW_URI, KW_UTC_OFFSET,
};
use crate::parameter::util::{ParseResult, parse_single, parse_single_not_quoted};
use crate::parameter::{Parameter, ParameterKind};
use crate::syntax::{SpannedSegments, SyntaxParameterRef, SyntaxParameterValueRef};
use crate::typed::TypedError;

/// Parse RSVP expectation parameter.
///
/// # Errors
///
/// Returns an error if the parameter value is not `TRUE` or `FALSE`.
pub fn parse_rsvp(mut param: SyntaxParameterRef<'_>) -> ParseResult<'_> {
    let span = param.span();
    parse_single(&mut param, ParameterKind::RsvpExpectation).and_then(|v| {
        if v.value.eq_str_ignore_ascii_case(KW_RSVP_TRUE) {
            Ok(Parameter::RsvpExpectation { value: true, span })
        } else if v.value.eq_str_ignore_ascii_case(KW_RSVP_FALSE) {
            Ok(Parameter::RsvpExpectation { value: false, span })
        } else {
            Err(vec![TypedError::ParameterValueInvalid {
                parameter: ParameterKind::RsvpExpectation,
                value: v.value,
                span,
            }])
        }
    })
}

/// Parse timezone identifier parameter.
///
/// # Errors
///
/// Returns an error if:
/// - The parameter does not have exactly one value (when jiff feature is enabled)
/// - The timezone identifier is not valid (when jiff feature is enabled)
pub fn parse_tzid<'src>(mut param: SyntaxParameterRef<'src>) -> ParseResult<'src> {
    let span = param.span();

    #[cfg(feature = "jiff")]
    let op = |v: SyntaxParameterValueRef<'src>| {
        // Use jiff to validate time zone identifier
        let tzid_str = v.value.resolve();
        match jiff::tz::TimeZone::get(tzid_str.as_ref()) {
            Ok(tz) => Ok(Parameter::TimeZoneIdentifier {
                value: v.value,
                span,
                tz,
            }),
            Err(_) => Err(vec![TypedError::ParameterValueInvalid {
                parameter: ParameterKind::TimeZoneIdentifier,
                value: v.value,
                span,
            }]),
        }
    };

    #[cfg(not(feature = "jiff"))]
    let op = |v: SyntaxParameterValueRef<'src>| {
        Ok(Parameter::TimeZoneIdentifier {
            value: v.value,
            span,
        })
    };

    parse_single(&mut param, ParameterKind::TimeZoneIdentifier).and_then(op)
}

define_param_enum_with_unknown! {
    #[derive(Default)]
    pub enum CalendarUserType {
        /// An individual
        #[default]
        Individual => KW_CUTYPE_INDIVIDUAL,
        /// A group of individuals
        Group      => KW_CUTYPE_GROUP,
        /// A physical resource
        Resource   => KW_CUTYPE_RESOURCE,
        /// A room resource
        Room       => KW_CUTYPE_ROOM,
        /// Otherwise not known
        Unknown    => KW_CUTYPE_UNKNOWN,
    }

    ref    = pub type CalendarUserTypeRef;
    owned  = pub type CalendarUserTypeOwned;
    parser = pub fn parse_cutype;
}

define_param_enum! {
    /// This parameter identifies the inline encoding used in a property value.
    #[derive(Default)]
    pub enum Encoding {
        /// The default encoding is "8BIT", corresponding to a property value
        /// consisting of text.
        #[default]
        Bit8   => KW_ENCODING_8BIT,
        /// The "BASE64" encoding type corresponds to a property value encoded
        /// using the "BASE64" encoding defined in [RFC2045].
        Base64 => KW_ENCODING_BASE64,
    }

    parser  = pub fn parse_encoding;
}

define_param_enum_with_unknown! {
    /// This parameter defines the free or busy time type for a time
    #[derive(Default)]
    pub enum FreeBusyType {
        /// The time interval is free for scheduling
        #[default]
        Free             => KW_FBTYPE_FREE,
        /// The time interval is busy because one or more events have been
        /// scheduled for that interval
        Busy             => KW_FBTYPE_BUSY,
        /// The time interval is busy and that the interval can not be scheduled.
        BusyUnavailable  => KW_FBTYPE_BUSY_UNAVAILABLE,
        /// The time interval is busy because one or more events have been
        /// tentatively scheduled for that interval.
        BusyTentative    => KW_FBTYPE_BUSY_TENTATIVE,
    }

    ref    = pub type FreeBusyTypeRef;
    owned  = pub type FreeBusyTypeOwned;
    parser = pub fn parse_fbtype;
}

define_param_enum_with_unknown! {
    pub enum ParticipationStatus {
        NeedsAction  => KW_PARTSTAT_NEEDS_ACTION,
        Accepted     => KW_PARTSTAT_ACCEPTED,
        Declined     => KW_PARTSTAT_DECLINED,
        Tentative    => KW_PARTSTAT_TENTATIVE,
        Delegated    => KW_PARTSTAT_DELEGATED,
        Completed    => KW_PARTSTAT_COMPLETED,
        InProcess    => KW_PARTSTAT_IN_PROCESS,
    }

    ref    = pub type ParticipationStatusRef;
    owned  = pub type ParticipationStatusOwned;
    parser = pub fn parse_partstat;
}

define_param_enum! {
    pub enum RecurrenceIdRange {
        /// A range defined by the recurrence identifier and all subsequent
        /// instances
        ThisAndFuture => KW_RANGE_THISANDFUTURE,

        // The value "THISANDPRIOR" is deprecated by this revision of iCalendar
        // and MUST NOT be generated by applications.
    }

    parser = pub fn parse_range;
}

define_param_enum! {
    /// This parameter defines the relationship of the alarm trigger to the
    #[derive(Default)]
    pub enum AlarmTriggerRelationship {
        /// The parameter value START will set the alarm to trigger off the
        /// start of the calendar component;
        #[default]
        Start => KW_RELATED_START,
        /// the parameter value END will set the alarm to trigger off the end
        /// of the calendar component.
        End   => KW_RELATED_END,
    }

    parser = pub fn parse_alarm_trigger_relationship;
}

define_param_enum_with_unknown! {
    #[derive(Default)]
    pub enum RelationshipType {
        /// The referenced calendar component is a superior of calendar component
        #[default]
        Parent  => KW_RELTYPE_PARENT,
        /// The referenced calendar component is a subordinate of the calendar
        /// component
        Child   => KW_RELTYPE_CHILD,
        /// The referenced calendar component is a peer of the calendar component
        Sibling => KW_RELTYPE_SIBLING,
    }

    ref    = pub type RelationshipTypeRef;
    owned  = pub type RelationshipTypeOwned;
    parser = pub fn parse_reltype;
}

define_param_enum_with_unknown! {
    #[derive(Default)]
    pub enum ParticipationRole {
        Chair             => KW_ROLE_CHAIR,
        #[default]
        ReqParticipant    => KW_ROLE_REQ_PARTICIPANT,
        OptParticipant    => KW_ROLE_OPT_PARTICIPANT,
        NonParticipant    => KW_ROLE_NON_PARTICIPANT,
    }

    ref    = pub type ParticipationRoleRef;
    owned  = pub type ParticipationRoleOwned;
    parser = pub fn parse_role;
}

define_param_enum_with_unknown! {
    pub enum ValueType {
        Binary              => KW_BINARY,
        Boolean             => KW_BOOLEAN,
        CalendarUserAddress => KW_CAL_ADDRESS,
        Date                => KW_DATE,
        DateTime            => KW_DATETIME,
        Duration            => KW_DURATION,
        Float               => KW_FLOAT,
        Integer             => KW_INTEGER,
        Period              => KW_PERIOD,
        RecurrenceRule      => KW_RRULE,
        Text                => KW_TEXT,
        Time                => KW_TIME,
        Uri                 => KW_URI,
        UtcOffset           => KW_UTC_OFFSET,
    }

    ref    = pub type ValueTypeRef;
    owned  = pub type ValueTypeOwned;
    parser = pub fn parse_value_type;
}
