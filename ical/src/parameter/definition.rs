// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Display, str::FromStr};

use crate::keyword::{
    KW_BINARY, KW_BOOLEAN, KW_CAL_ADDRESS, KW_CUTYPE, KW_CUTYPE_GROUP, KW_CUTYPE_INDIVIDUAL,
    KW_CUTYPE_RESOURCE, KW_CUTYPE_ROOM, KW_CUTYPE_UNKNOWN, KW_DATE, KW_DATETIME, KW_DURATION,
    KW_ENCODING, KW_ENCODING_8BIT, KW_ENCODING_BASE64, KW_FBTYPE, KW_FBTYPE_BUSY,
    KW_FBTYPE_BUSY_TENTATIVE, KW_FBTYPE_BUSY_UNAVAILABLE, KW_FBTYPE_FREE, KW_FLOAT, KW_INTEGER,
    KW_PARTSTAT, KW_PARTSTAT_ACCEPTED, KW_PARTSTAT_COMPLETED, KW_PARTSTAT_DECLINED,
    KW_PARTSTAT_DELEGATED, KW_PARTSTAT_IN_PROCESS, KW_PARTSTAT_NEEDS_ACTION, KW_PARTSTAT_TENTATIVE,
    KW_PERIOD, KW_RANGE, KW_RANGE_THISANDFUTURE, KW_RELATED, KW_RELATED_END, KW_RELATED_START,
    KW_RELTYPE, KW_RELTYPE_CHILD, KW_RELTYPE_PARENT, KW_RELTYPE_SIBLING, KW_ROLE, KW_ROLE_CHAIR,
    KW_ROLE_NON_PARTICIPANT, KW_ROLE_OPT_PARTICIPANT, KW_ROLE_REQ_PARTICIPANT, KW_RRULE,
    KW_RSVP_FALSE, KW_RSVP_TRUE, KW_TEXT, KW_TIME, KW_URI, KW_UTC_OFFSET, KW_VALUE,
};
use crate::parameter::{TypedParameter, TypedParameterKind};
use crate::syntax::{SpannedSegments, SyntaxParameter, SyntaxParameterValue};
use crate::typed::TypedAnalysisError;

/// Parse RSVP expectation parameter.
///
/// # Errors
///
/// Returns an error if the parameter value is not `TRUE` or `FALSE`.
pub fn parse_rsvp(mut param: SyntaxParameter<'_>) -> ParseResult<'_> {
    let span = param.span();
    parse_single(&mut param, TypedParameterKind::RsvpExpectation).and_then(|v| {
        if v.value.eq_str_ignore_ascii_case(KW_RSVP_TRUE) {
            Ok(TypedParameter::RsvpExpectation { value: true, span })
        } else if v.value.eq_str_ignore_ascii_case(KW_RSVP_FALSE) {
            Ok(TypedParameter::RsvpExpectation { value: false, span })
        } else {
            Err(vec![TypedAnalysisError::ParameterValueInvalid {
                parameter: TypedParameterKind::RsvpExpectation.name(),
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
            Ok(tz) => Ok(TypedParameter::TimeZoneIdentifier {
                value: v.value,
                span,
                tz,
            }),
            Err(_) => Err(vec![TypedAnalysisError::ParameterValueInvalid {
                parameter: TypedParameterKind::TimeZoneIdentifier.name(),
                value: v.value,
                span,
            }]),
        }
    };

    #[cfg(not(feature = "jiff"))]
    let op = |v: SyntaxParameterValue<'src>| {
        Ok(TypedParameter::TimeZoneIdentifier {
            value: v.value,
            span,
        })
    };

    parse_single(&mut param, TypedParameterKind::TimeZoneIdentifier).and_then(op)
}

// TODO: add x-name and iana-token support
macro_rules! define_param_enum {
    (
        $(#[$meta:meta])*
        enum $Name:ident {
            $(
                $(#[$vmeta:meta])*
                $Variant:ident => $kw:ident
            ),+ $(,)?
        }

        parser {
            fn $parse_fn:ident;
            keyword = $param_kw:ident;
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[allow(missing_docs)]
        pub enum $Name {
            $(
                $(#[$vmeta])*
                $Variant,
            )+
        }

        impl TryFrom<&SpannedSegments<'_>> for $Name {
            type Error = ();

            fn try_from(segs: &SpannedSegments<'_>) -> Result<Self, Self::Error> {
                $(
                    if segs.eq_str_ignore_ascii_case($kw) {
                        return Ok(Self::$Variant);
                    }
                )+
                Err(())
            }
        }

        impl AsRef<str> for $Name {
            #[rustfmt::skip]
            fn as_ref(&self) -> &str {
                match self {
                    $(
                        Self::$Variant => $kw,
                    )+
                }
            }
        }

        impl Display for $Name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.as_ref().fmt(f)
            }
        }

        #[allow(missing_docs)]
        #[allow(clippy::missing_errors_doc)]
        pub fn $parse_fn(mut param: SyntaxParameter<'_>) -> ParseResult<'_> {
            let kind = TypedParameterKind::from_str($param_kw).unwrap();
            parse_single_not_quoted(&mut param, kind).and_then(|value| {
                $Name::try_from(&value)
                    .map(|value| TypedParameter::$Name {
                        value,
                        span: param.span(),
                    })
                    .map_err(|()| {
                        vec![TypedAnalysisError::ParameterValueInvalid {
                            span: value.span(),
                            parameter: kind.name(),
                            value,
                        }]
                    })
            })
        }
    };
}

define_param_enum! {
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
        keyword = KW_CUTYPE;
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
        keyword = KW_ENCODING;
    }
}

define_param_enum! {
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
        keyword = KW_FBTYPE;
    }
}

define_param_enum! {
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
        keyword = KW_PARTSTAT;
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
        keyword = KW_RANGE;
    }
}

define_param_enum! {
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
        keyword = KW_RELATED;
    }
}

define_param_enum! {
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
        keyword = KW_RELTYPE;
    }
}

define_param_enum! {
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
        keyword = KW_ROLE;
    }
}

define_param_enum! {
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
        keyword = KW_VALUE;
    }
}

type ParseResult<'src> = Result<TypedParameter<'src>, Vec<TypedAnalysisError<'src>>>;

/// Parse a single value from a parameter.
///
/// # Errors
///
/// Returns an error if the parameter does not have exactly one value.
///
/// # Panics
///
/// Panics if the parameter has exactly one value but `Vec::pop()` returns `None`.
/// This should never happen in practice as the length check ensures there is
/// exactly one value.
pub fn parse_single<'src>(
    param: &mut SyntaxParameter<'src>,
    kind: TypedParameterKind,
) -> Result<SyntaxParameterValue<'src>, Vec<TypedAnalysisError<'src>>> {
    match param.values.len() {
        1 => Ok(param.values.pop().unwrap()),
        _ => Err(vec![
            TypedAnalysisError::ParameterMultipleValuesDisallowed {
                parameter: kind.name(),
                span: param.span(),
            },
        ]),
    }
}

/// Parse a single quoted value from a parameter.
///
/// # Errors
///
/// Returns an error if:
/// - The parameter does not have exactly one value
/// - The value is not quoted
pub fn parse_single_quoted<'src>(
    param: &mut SyntaxParameter<'src>,
    kind: TypedParameterKind,
) -> Result<SpannedSegments<'src>, Vec<TypedAnalysisError<'src>>> {
    parse_single(param, kind).and_then(|v| {
        if v.quoted {
            Ok(v.value)
        } else {
            Err(vec![TypedAnalysisError::ParameterValueMustBeQuoted {
                parameter: kind.name(),
                span: v.value.span(),
                value: v.value,
            }])
        }
    })
}

/// Parse a single unquoted value from a parameter.
///
/// # Errors
///
/// Returns an error if:
/// - The parameter does not have exactly one value
/// - The value is quoted
pub fn parse_single_not_quoted<'src>(
    param: &mut SyntaxParameter<'src>,
    kind: TypedParameterKind,
) -> Result<SpannedSegments<'src>, Vec<TypedAnalysisError<'src>>> {
    parse_single(param, kind).and_then(|v| {
        if v.quoted {
            Err(vec![TypedAnalysisError::ParameterValueMustNotBeQuoted {
                parameter: kind.name(),
                span: v.value.span(),
                value: v.value,
            }])
        } else {
            Ok(v.value)
        }
    })
}

/// Parse multiple quoted values from a parameter.
///
/// # Errors
///
/// Returns an error if any of the values are not quoted.
pub fn parse_multiple_quoted(
    param: SyntaxParameter<'_>,
    kind: TypedParameterKind,
) -> Result<Vec<SpannedSegments<'_>>, Vec<TypedAnalysisError<'_>>> {
    let mut values = Vec::with_capacity(param.values.len());
    let mut errors = Vec::new();
    for v in param.values {
        if v.quoted {
            values.push(v.value);
        } else {
            errors.push(TypedAnalysisError::ParameterValueMustBeQuoted {
                parameter: kind.name(),
                span: v.value.span(),
                value: v.value,
            });
        }
    }

    if errors.is_empty() {
        Ok(values)
    } else {
        Err(errors)
    }
}
