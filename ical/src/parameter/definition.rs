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
use crate::syntax::{SpannedSegments, SyntaxParameter, SyntaxParameterValue};
use crate::typed::TypedError;

/// Parse RSVP expectation parameter.
///
/// # Errors
///
/// Returns an error if the parameter value is not `TRUE` or `FALSE`.
pub fn parse_rsvp(mut param: SyntaxParameter<'_>) -> ParseResult<'_> {
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
pub fn parse_tzid<'src>(mut param: SyntaxParameter<'src>) -> ParseResult<'src> {
    let span = param.span();

    #[cfg(feature = "jiff")]
    let op = |v: SyntaxParameterValue<'src>| {
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
    let op = |v: SyntaxParameterValue<'src>| {
        Ok(Parameter::TimeZoneIdentifier {
            value: v.value,
            span,
        })
    };

    parse_single(&mut param, ParameterKind::TimeZoneIdentifier).and_then(op)
}

/// Macro to define parameter enums without x-name/iana-token support.
///
/// This generates simple enums with Copy semantics for RFC 5545 parameter values
/// that don't support extensions.
macro_rules! define_param_enum {
    (
        $(#[$meta:meta])*
        enum $Name:ident {
            $(
                $(#[$vmeta:meta])*
                $Variant:ident => $kw:ident
            ),* $(,)?
        }

        parser {
            fn $parse_fn:ident;
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[allow(missing_docs)]
        pub enum $Name {
            $(
                $(#[$vmeta])*
                $Variant,
            )*
        }

        impl TryFrom<SpannedSegments<'_>> for $Name {
            type Error = ();

            fn try_from(segs: SpannedSegments<'_>) -> Result<Self, Self::Error> {
                $(
                    if segs.eq_str_ignore_ascii_case($kw) {
                        return Ok(Self::$Variant);
                    }
                )*
                Err(())
            }
        }

        impl fmt::Display for $Name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$Variant => $kw.fmt(f),
                    )*
                }
            }
        }

        pub fn $parse_fn(mut param: SyntaxParameter<'_>) -> ParseResult<'_> {
            parse_single_not_quoted(&mut param, ParameterKind::$Name).and_then(|value| {
                match $Name::try_from(value.clone()) { // PERF: avoid clone
                    Ok(value) => Ok(Parameter::$Name {
                        value,
                        span: param.span(),
                    }),
                    Err(()) => Err(vec![TypedError::ParameterValueInvalid {
                        span: value.span(),
                        parameter: ParameterKind::$Name,
                        value,
                    }])
                }
            })
        }
    };
}

/// Macro to define parameter enums with x-name and unrecognized value support.
///
/// This generates enums with lifetime parameters for zero-copy storage of
/// extension values per RFC 5545.
macro_rules! define_param_enum_with_unknown {
    (
        $(#[$meta:meta])*
        enum $Name:ident {
            $(
                $(#[$vmeta:meta])*
                $Variant:ident => $kw:ident
            ),* $(,)?
        }

        parser {
            fn $parse_fn:ident;
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        #[allow(missing_docs)]
        pub enum $Name<'src> {
            $(
                $(#[$vmeta])*
                $Variant,
            )*
            /// Custom experimental x-name value (must start with "X-" or "x-")
            XName(SpannedSegments<'src>),
            /// Unrecognized value (not a known standard value)
            Unrecognized(SpannedSegments<'src>),
        }

        impl<'src> From<SpannedSegments<'src>> for $Name<'src> {
            fn from(segs: SpannedSegments<'src>) -> Self {
                $(
                    if segs.eq_str_ignore_ascii_case($kw) {
                        return Self::$Variant;
                    }
                )*

                // Check for x-name prefix
                let resolved = segs.resolve();
                let s = resolved.as_ref();
                if s.starts_with("X-") || s.starts_with("x-") { // PERF: add starts_with
                    Self::XName(segs.clone())
                } else {
                    // Otherwise, treat as unrecognized value
                    Self::Unrecognized(segs.clone())
                }
            }
        }

        impl<'src> fmt::Display for $Name<'src> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$Variant => $kw.fmt(f),
                    )*
                    Self::XName(segs) | Self::Unrecognized(segs) => {
                        write!(f, "{}", segs.resolve().as_ref())
                    }
                }
            }
        }

        pub fn $parse_fn(mut param: SyntaxParameter<'_>) -> ParseResult<'_> {
            parse_single_not_quoted(&mut param, ParameterKind::$Name).map(|value| {
                let enum_value = $Name::try_from(value).unwrap(); // Never fails due to XName/Unrecognized variants
                Parameter::$Name {
                    value: enum_value,
                    span: param.span(),
                }
            })
        }
    };
}

define_param_enum_with_unknown! {
    #[derive(Default)]
    enum CalendarUserType {
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

    parser {
        fn parse_cutype;
    }
}

define_param_enum! {
    /// This parameter identifies the inline encoding used in a property value.
    #[derive(Default)]
    enum Encoding {
        /// The default encoding is "8BIT", corresponding to a property value
        /// consisting of text.
        #[default]
        Bit8   => KW_ENCODING_8BIT,
        /// The "BASE64" encoding type corresponds to a property value encoded
        /// using the "BASE64" encoding defined in [RFC2045].
        Base64 => KW_ENCODING_BASE64,
    }

    parser {
        fn parse_encoding;
    }
}

define_param_enum_with_unknown! {
    /// This parameter defines the free or busy time type for a time
    #[derive(Default)]
    enum FreeBusyType {
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

    parser {
        fn parse_fbtype;
    }
}

define_param_enum_with_unknown! {
    enum ParticipationStatus {
        NeedsAction  => KW_PARTSTAT_NEEDS_ACTION,
        Accepted     => KW_PARTSTAT_ACCEPTED,
        Declined     => KW_PARTSTAT_DECLINED,
        Tentative    => KW_PARTSTAT_TENTATIVE,
        Delegated    => KW_PARTSTAT_DELEGATED,
        Completed    => KW_PARTSTAT_COMPLETED,
        InProcess    => KW_PARTSTAT_IN_PROCESS,
    }

    parser {
        fn parse_partstat;
    }
}

define_param_enum! {
    enum RecurrenceIdRange {
        /// A range defined by the recurrence identifier and all subsequent
        /// instances
        ThisAndFuture => KW_RANGE_THISANDFUTURE,

        // The value "THISANDPRIOR" is deprecated by this revision of iCalendar
        // and MUST NOT be generated by applications.
    }

    parser {
        fn parse_range;
    }
}

define_param_enum! {
    /// This parameter defines the relationship of the alarm trigger to the
    #[derive(Default)]
    enum AlarmTriggerRelationship {
        /// The parameter value START will set the alarm to trigger off the
        /// start of the calendar component;
        #[default]
        Start => KW_RELATED_START,
        /// the parameter value END will set the alarm to trigger off the end
        /// of the calendar component.
        End   => KW_RELATED_END,
    }

    parser {
        fn parse_alarm_trigger_relationship;
    }
}

define_param_enum_with_unknown! {
    #[derive(Default)]
    enum RelationshipType {
        /// The referenced calendar component is a superior of calendar component
        #[default]
        Parent  => KW_RELTYPE_PARENT,
        /// The referenced calendar component is a subordinate of the calendar
        /// component
        Child   => KW_RELTYPE_CHILD,
        /// The referenced calendar component is a peer of the calendar component
        Sibling => KW_RELTYPE_SIBLING,
    }

    parser {
        fn parse_reltype;
    }
}

define_param_enum_with_unknown! {
    #[derive(Default)]
    enum ParticipationRole {
        Chair             => KW_ROLE_CHAIR,
        #[default]
        ReqParticipant    => KW_ROLE_REQ_PARTICIPANT,
        OptParticipant    => KW_ROLE_OPT_PARTICIPANT,
        NonParticipant    => KW_ROLE_NON_PARTICIPANT,
    }

    parser {
        fn parse_role;
    }
}

define_param_enum_with_unknown! {
    enum ValueType {
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

    parser {
        fn parse_value_type;
    }
}

#[allow(
    clippy::elidable_lifetime_names,
    reason = "explicit lifetime is clearer for this type"
)]
impl<'src> ValueType<'src> {
    /// Returns the standard value kind if this is a known type, or None for x-name/unrecognized.
    #[must_use]
    pub fn as_standard(&self) -> Option<StandardValueType> {
        match self {
            ValueType::Binary => Some(StandardValueType::Binary),
            ValueType::Boolean => Some(StandardValueType::Boolean),
            ValueType::CalendarUserAddress => Some(StandardValueType::CalendarUserAddress),
            ValueType::Date => Some(StandardValueType::Date),
            ValueType::DateTime => Some(StandardValueType::DateTime),
            ValueType::Duration => Some(StandardValueType::Duration),
            ValueType::Float => Some(StandardValueType::Float),
            ValueType::Integer => Some(StandardValueType::Integer),
            ValueType::Period => Some(StandardValueType::Period),
            ValueType::RecurrenceRule => Some(StandardValueType::RecurrenceRule),
            ValueType::Text => Some(StandardValueType::Text),
            ValueType::Time => Some(StandardValueType::Time),
            ValueType::Uri => Some(StandardValueType::Uri),
            ValueType::UtcOffset => Some(StandardValueType::UtcOffset),
            ValueType::XName(_) | ValueType::Unrecognized(_) => None,
        }
    }
}

/// Standard value types (no x-name/unrecognized variants).
///
/// This enum contains only the RFC 5545 standard value types and is Copy,
/// making it suitable for use in `PropertyKind` definitions where lifetime
/// parameters would be cumbersome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardValueType {
    /// Binary value type
    Binary,
    /// Boolean value type
    Boolean,
    /// Calendar user address value type
    CalendarUserAddress,
    /// Date value type
    Date,
    /// Date-time value type
    DateTime,
    /// Duration value type
    Duration,
    /// Float value type
    Float,
    /// Integer value type
    Integer,
    /// Period value type
    Period,
    /// Recurrence rule value type
    RecurrenceRule,
    /// Text value type
    Text,
    /// Time value type
    Time,
    /// URI value type
    Uri,
    /// UTC offset value type
    UtcOffset,
}

impl From<StandardValueType> for ValueType<'_> {
    fn from(value: StandardValueType) -> Self {
        match value {
            StandardValueType::Binary => ValueType::Binary,
            StandardValueType::Boolean => ValueType::Boolean,
            StandardValueType::CalendarUserAddress => ValueType::CalendarUserAddress,
            StandardValueType::Date => ValueType::Date,
            StandardValueType::DateTime => ValueType::DateTime,
            StandardValueType::Duration => ValueType::Duration,
            StandardValueType::Float => ValueType::Float,
            StandardValueType::Integer => ValueType::Integer,
            StandardValueType::Period => ValueType::Period,
            StandardValueType::RecurrenceRule => ValueType::RecurrenceRule,
            StandardValueType::Text => ValueType::Text,
            StandardValueType::Time => ValueType::Time,
            StandardValueType::Uri => ValueType::Uri,
            StandardValueType::UtcOffset => ValueType::UtcOffset,
        }
    }
}

impl fmt::Display for StandardValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ValueType::from(*self).fmt(f)
    }
}
