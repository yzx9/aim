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

impl<S: StringStorage> From<ParameterKind<&S>> for ParameterKind<S> {
    fn from(value: ParameterKind<&S>) -> Self {
        match value {
            ParameterKind::AlternateText => ParameterKind::AlternateText,
            ParameterKind::CommonName => ParameterKind::CommonName,
            ParameterKind::CalendarUserType => ParameterKind::CalendarUserType,
            ParameterKind::Delegators => ParameterKind::Delegators,
            ParameterKind::Delegatees => ParameterKind::Delegatees,
            ParameterKind::Directory => ParameterKind::Directory,
            ParameterKind::Encoding => ParameterKind::Encoding,
            ParameterKind::FormatType => ParameterKind::FormatType,
            ParameterKind::FreeBusyType => ParameterKind::FreeBusyType,
            ParameterKind::Language => ParameterKind::Language,
            ParameterKind::GroupOrListMembership => ParameterKind::GroupOrListMembership,
            ParameterKind::ParticipationStatus => ParameterKind::ParticipationStatus,
            ParameterKind::RecurrenceIdRange => ParameterKind::RecurrenceIdRange,
            ParameterKind::AlarmTriggerRelationship => ParameterKind::AlarmTriggerRelationship,
            ParameterKind::RelationshipType => ParameterKind::RelationshipType,
            ParameterKind::ParticipationRole => ParameterKind::ParticipationRole,
            ParameterKind::SendBy => ParameterKind::SendBy,
            ParameterKind::RsvpExpectation => ParameterKind::RsvpExpectation,
            ParameterKind::TimeZoneIdentifier => ParameterKind::TimeZoneIdentifier,
            ParameterKind::ValueType => ParameterKind::ValueType,
            ParameterKind::XName(s) => ParameterKind::XName(s.to_owned()),
            ParameterKind::Unrecognized(s) => ParameterKind::Unrecognized(s.to_owned()),
        }
    }
}

impl ParameterKind<SpannedSegments<'_>> {
    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> ParameterKind<String> {
        match self {
            Self::AlternateText => ParameterKind::AlternateText,
            Self::CommonName => ParameterKind::CommonName,
            Self::CalendarUserType => ParameterKind::CalendarUserType,
            Self::Delegators => ParameterKind::Delegators,
            Self::Delegatees => ParameterKind::Delegatees,
            Self::Directory => ParameterKind::Directory,
            Self::Encoding => ParameterKind::Encoding,
            Self::FormatType => ParameterKind::FormatType,
            Self::FreeBusyType => ParameterKind::FreeBusyType,
            Self::Language => ParameterKind::Language,
            Self::GroupOrListMembership => ParameterKind::GroupOrListMembership,
            Self::ParticipationStatus => ParameterKind::ParticipationStatus,
            Self::RecurrenceIdRange => ParameterKind::RecurrenceIdRange,
            Self::AlarmTriggerRelationship => ParameterKind::AlarmTriggerRelationship,
            Self::RelationshipType => ParameterKind::RelationshipType,
            Self::ParticipationRole => ParameterKind::ParticipationRole,
            Self::SendBy => ParameterKind::SendBy,
            Self::RsvpExpectation => ParameterKind::RsvpExpectation,
            Self::TimeZoneIdentifier => ParameterKind::TimeZoneIdentifier,
            Self::ValueType => ParameterKind::ValueType,
            Self::XName(s) => ParameterKind::XName(s.to_owned()),
            Self::Unrecognized(s) => ParameterKind::Unrecognized(s.to_owned()),
        }
    }
}
