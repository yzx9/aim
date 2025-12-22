// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crate::keyword::{
    KW_BINARY, KW_BOOLEAN, KW_CAL_ADDRESS, KW_CUTYPE, KW_CUTYPE_GROUP, KW_CUTYPE_INDIVIDUAL,
    KW_CUTYPE_RESOURCE, KW_CUTYPE_ROOM, KW_CUTYPE_UNKNOWN, KW_DATE, KW_DATETIME, KW_DURATION,
    KW_ENCODING, KW_ENCODING_8BIT, KW_ENCODING_BASE64, KW_FALSE, KW_FBTYPE, KW_FBTYPE_BUSY,
    KW_FBTYPE_BUSY_TENTATIVE, KW_FBTYPE_BUSY_UNAVAILABLE, KW_FBTYPE_FREE, KW_FLOAT, KW_INTEGER,
    KW_PARTSTAT, KW_PARTSTAT_ACCEPTED, KW_PARTSTAT_COMPLETED, KW_PARTSTAT_DECLINED,
    KW_PARTSTAT_DELEGATED, KW_PARTSTAT_IN_PROCESS, KW_PARTSTAT_NEEDS_ACTION, KW_PARTSTAT_TENTATIVE,
    KW_PERIOD, KW_RANGE, KW_RANGE_THISANDFUTURE, KW_RELATED, KW_RELATED_END, KW_RELATED_START,
    KW_RELTYPE, KW_RELTYPE_CHILD, KW_RELTYPE_PARENT, KW_RELTYPE_SIBLING, KW_ROLE, KW_ROLE_CHAIR,
    KW_ROLE_NON_PARTICIPANT, KW_ROLE_OPT_PARTICIPANT, KW_ROLE_REQ_PARTICIPANT, KW_RRULE, KW_TEXT,
    KW_TIME, KW_TRUE, KW_URI, KW_UTC_OFFSET, KW_VALUE,
};
use crate::syntax::{SpannedSegments, SyntaxParameter, SyntaxParameterValue};
use crate::typed::TypedAnalysisError;
use crate::typed::parameter::{TypedParameter, TypedParameterKind};

pub fn parse_rsvp(mut param: SyntaxParameter<'_>) -> ParseResult<'_> {
    let span = param.span();
    parse_single(&mut param, TypedParameterKind::RsvpExpectation).and_then(|v| {
        if v.value.eq_ignore_ascii_case(KW_TRUE) {
            Ok(TypedParameter::RsvpExpectation { value: true, span })
        } else if v.value.eq_ignore_ascii_case(KW_FALSE) {
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
                    if segs.eq_ignore_ascii_case($kw) {
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

        impl std::fmt::Display for $Name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_ref())
            }
        }

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
    enum CalendarUserType {
        /// An individual
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
    enum Encoding {
        /// The default encoding is "8BIT", corresponding to a property value
        /// consisting of text.
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
    enum FreeBusyType {
        Free             => KW_FBTYPE_FREE,
        Busy             => KW_FBTYPE_BUSY,
        BusyUnavailable  => KW_FBTYPE_BUSY_UNAVAILABLE,
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
        ThisAndFuture => KW_RANGE_THISANDFUTURE,
        // THISANDPRIOR is deprecated and MUST NOT be generated by applications
    }

    parser {
        fn parse_range;
        keyword = KW_RANGE;
    }
}

define_param_enum! {
    enum AlarmTriggerRelationship {
        Start => KW_RELATED_START,
        End   => KW_RELATED_END,
    }

    parser {
        fn parse_alarm_trigger_relationship;
        keyword = KW_RELATED;
    }
}

define_param_enum! {
    enum RelationshipType {
        Parent  => KW_RELTYPE_PARENT,
        Child   => KW_RELTYPE_CHILD,
        Sibling => KW_RELTYPE_SIBLING,
    }

    parser {
        fn parse_reltype;
        keyword = KW_RELTYPE;
    }
}

define_param_enum! {
    enum ParticipationRole {
        Chair             => KW_ROLE_CHAIR,
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
