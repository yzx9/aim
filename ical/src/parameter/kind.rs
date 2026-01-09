// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use crate::keyword::{
    KW_ALTREP, KW_CN, KW_CUTYPE, KW_DELEGATED_FROM, KW_DELEGATED_TO, KW_DIR, KW_ENCODING,
    KW_FBTYPE, KW_FMTTYPE, KW_LANGUAGE, KW_MEMBER, KW_PARTSTAT, KW_RANGE, KW_RELATED, KW_RELTYPE,
    KW_ROLE, KW_RSVP, KW_SENT_BY, KW_TZID, KW_VALUE,
};
use crate::string_storage::{SpannedSegments, StringStorage};

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
        pub enum $ty<S: StringStorage> {
            $( $variant, )+
            /// Custom experimental x-name value (must start with "X-" or "x-")
            XName(S),
            /// Unrecognized value (not a known standard value)
            Unrecognized(S),
        }

        impl<'src> ::core::convert::From<SpannedSegments<'src>> for $ty<crate::string_storage::SpannedSegments<'src>> {
            fn from(name: crate::string_storage::SpannedSegments<'src>) -> Self {
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
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$variant => write!(f, "{}", $kw),
                    )+
                    Self::XName(s) | Self::Unrecognized(s) => write!(f, "{}", s),
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

/// Type alias for borrowed parameter kind
pub type ParameterKindRef<'src> = ParameterKind<SpannedSegments<'src>>;

/// Type alias for owned parameter kind
pub type ParameterKindOwned = ParameterKind<String>;

impl ParameterKindRef<'_> {
    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> ParameterKindOwned {
        match self {
            Self::AlternateText => ParameterKindOwned::AlternateText,
            Self::CommonName => ParameterKindOwned::CommonName,
            Self::CalendarUserType => ParameterKindOwned::CalendarUserType,
            Self::Delegators => ParameterKindOwned::Delegators,
            Self::Delegatees => ParameterKindOwned::Delegatees,
            Self::Directory => ParameterKindOwned::Directory,
            Self::Encoding => ParameterKindOwned::Encoding,
            Self::FormatType => ParameterKindOwned::FormatType,
            Self::FreeBusyType => ParameterKindOwned::FreeBusyType,
            Self::Language => ParameterKindOwned::Language,
            Self::GroupOrListMembership => ParameterKindOwned::GroupOrListMembership,
            Self::ParticipationStatus => ParameterKindOwned::ParticipationStatus,
            Self::RecurrenceIdRange => ParameterKindOwned::RecurrenceIdRange,
            Self::AlarmTriggerRelationship => ParameterKindOwned::AlarmTriggerRelationship,
            Self::RelationshipType => ParameterKindOwned::RelationshipType,
            Self::ParticipationRole => ParameterKindOwned::ParticipationRole,
            Self::SendBy => ParameterKindOwned::SendBy,
            Self::RsvpExpectation => ParameterKindOwned::RsvpExpectation,
            Self::TimeZoneIdentifier => ParameterKindOwned::TimeZoneIdentifier,
            Self::ValueType => ParameterKindOwned::ValueType,
            Self::XName(s) => ParameterKindOwned::XName(s.to_owned()),
            Self::Unrecognized(s) => ParameterKindOwned::Unrecognized(s.to_owned()),
        }
    }
}
