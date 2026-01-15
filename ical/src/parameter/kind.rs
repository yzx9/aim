// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::keyword::{
    KW_ALTREP, KW_CN, KW_CUTYPE, KW_DELEGATED_FROM, KW_DELEGATED_TO, KW_DIR, KW_ENCODING,
    KW_FBTYPE, KW_FMTTYPE, KW_LANGUAGE, KW_MEMBER, KW_PARTSTAT, KW_RANGE, KW_RELATED, KW_RELTYPE,
    KW_ROLE, KW_RSVP, KW_SENT_BY, KW_TZID, KW_VALUE,
};
use crate::string_storage::{Segments, StringStorage};

macro_rules! impl_typed_parameter_kind_mapping {
    (
        $(#[$attr:meta])*
        enum $ty:ident {
            $(
                $variant:ident => $kw:ident
            ),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        $(#[$attr])*
        pub enum $ty<S: crate::string_storage::StringStorage> {
            $( $variant, )+
            /// Custom experimental x-name value (must start with "X-" or "x-")
            XName(S),
            /// Unrecognized value (not a known standard value)
            Unrecognized(S),
        }

        impl<'src> ::core::convert::From<crate::string_storage::Segments<'src>> for $ty<crate::string_storage::Segments<'src>> {
            fn from(name: crate::string_storage::Segments<'src>) -> Self {
                $(
                    if name.eq_str_ignore_ascii_case($kw) {
                        return Self::$variant;
                    }
                )*

                if name.starts_with_str_ignore_ascii_case("X-") {
                    Self::XName(name)
                } else {
                    Self::Unrecognized(name)
                }
            }
        }

        impl<S: StringStorage> ::core::fmt::Display for $ty<S> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    $(
                        Self::$variant => write!(f, "{}", $kw),
                    )+
                    Self::XName(s) | Self::Unrecognized(s) => write!(f, "{}", s),
                }
            }
        }

        impl<S: StringStorage> From<ParameterKind<&S>> for ParameterKind<S> {
            fn from(value: ParameterKind<&S>) -> Self {
                match value {
                    $(
                        ParameterKind::$variant => Self::$variant,
                    )+
                    ParameterKind::XName(s) => ParameterKind::XName(s.to_owned()),
                    ParameterKind::Unrecognized(s) => ParameterKind::Unrecognized(s.to_owned()),
                }
            }
        }

        impl ParameterKind<Segments<'_>> {
            /// Convert borrowed type to owned type
            #[must_use]
            pub fn to_owned(&self) -> ParameterKind<String> {
                match self {
                    $(
                        Self::$variant => ParameterKind::$variant,
                    )+
                    Self::XName(s) => ParameterKind::XName(s.to_owned()),
                    Self::Unrecognized(s) => ParameterKind::Unrecognized(s.to_owned()),
                }
            }
        }
    };
}

impl_typed_parameter_kind_mapping! {
    /// Kinds of iCalendar parameters
    #[expect(missing_docs)]
    enum ParameterKind {
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
