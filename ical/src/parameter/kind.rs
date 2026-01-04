// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use crate::keyword::{
    KW_ALTREP, KW_CN, KW_CUTYPE, KW_DELEGATED_FROM, KW_DELEGATED_TO, KW_DIR, KW_ENCODING,
    KW_FBTYPE, KW_FMTTYPE, KW_LANGUAGE, KW_MEMBER, KW_PARTSTAT, KW_RANGE, KW_RELATED, KW_RELTYPE,
    KW_ROLE, KW_RSVP, KW_SENT_BY, KW_TZID, KW_VALUE,
};
use crate::syntax::SpannedSegments;

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
        pub enum $ty<'src> {
            $( $variant, )+
            /// Custom experimental x-name value (must start with "X-" or "x-")
            XName(SpannedSegments<'src>),
            /// Unrecognized value (not a known standard value)
            Unrecognized(SpannedSegments<'src>),
        }

        impl<'src> From<SpannedSegments<'src>> for $ty<'src> {
            fn from(name: SpannedSegments<'src>) -> Self {
                // $(
                //     if name.eq_str_ignore_ascii_case($kw) {
                //         return Ok(Self::$variant);
                //     }
                // )*
                // Err(())

                let name_resolved = name.resolve(); // PERF: avoid allocation
                let name_str = name_resolved.as_ref();
                match name_str {
                    $(
                        $kw => Self::$variant,
                    )+
                    _ => {
                        if name_str.starts_with("X-") || name_str.starts_with("x-") {
                            Self::XName(name)
                        } else {
                            Self::Unrecognized(name)
                        }
                    }
                }
            }
        }

        impl fmt::Display for $ty<'_> {
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
