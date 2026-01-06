// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property kinds and value types for iCalendar properties.
//!
//! This module defines the `PropertyKind` enum that represents all standard
//! iCalendar properties defined in RFC 5545, along with their allowed value types.

use std::fmt;

use crate::keyword::{
    KW_ACTION, KW_ATTACH, KW_ATTENDEE, KW_CALSCALE, KW_CATEGORIES, KW_CLASS, KW_COMMENT,
    KW_COMPLETED, KW_CONTACT, KW_CREATED, KW_DESCRIPTION, KW_DTEND, KW_DTSTAMP, KW_DTSTART, KW_DUE,
    KW_DURATION, KW_EXDATE, KW_FREEBUSY, KW_GEO, KW_LAST_MODIFIED, KW_LOCATION, KW_METHOD,
    KW_ORGANIZER, KW_PERCENT_COMPLETE, KW_PRIORITY, KW_PRODID, KW_RDATE, KW_RECURRENCE_ID,
    KW_RELATED_TO, KW_REPEAT, KW_REQUEST_STATUS, KW_RESOURCES, KW_RRULE, KW_SEQUENCE, KW_STATUS,
    KW_SUMMARY, KW_TRANSP, KW_TRIGGER, KW_TZID, KW_TZNAME, KW_TZOFFSETFROM, KW_TZOFFSETTO,
    KW_TZURL, KW_UID, KW_URL, KW_VERSION,
};
use crate::parameter::ValueType;
use crate::syntax::SpannedSegments;

/// Macro to define `PropertyKind` with associated value types.
///
/// Usage: `property_kind!(Variant => KW => &[...], ...)`
macro_rules! property_kind {
    (
        $(
            $(#[$attr:meta])*
            $variant:ident => $kw:ident => $value_types:expr $(,)?
        )*
    ) => {
        /// Kind of iCalendar property.
        /// Represents all standard properties defined in RFC 5545.
        #[derive(Debug, Clone)]
        #[expect(missing_docs)]
        pub enum PropertyKind<'src> {
            $(
                $(#[$attr])*
                $variant,
            )*
            /// Custom experimental x-name property (must start with "X-" or "x-")
            XName(SpannedSegments<'src>),
            /// Unrecognized property (not a known standard property)
            Unrecognized(SpannedSegments<'src>),
        }

        impl<'src> PropertyKind<'src> {
            /// Returns the allowed value types for this property kind, if known.
            /// Returns `None` for unrecognized or x-name properties.
            #[must_use]
            pub(crate) fn value_types(&self) -> Option<&'static [ValueType<'static>]> {
                match self {
                    $(PropertyKind::$variant => Some($value_types),)*

                    // dont know the exact allowed types for unknown properties
                    PropertyKind::XName(_) | PropertyKind::Unrecognized(_) =>  None,
                }
            }
        }

        impl<'src> From<SpannedSegments<'src>> for PropertyKind<'src> {
            fn from(name: SpannedSegments<'src>) -> Self {
                let name_resolved = name.resolve();
                let name_str = name_resolved.as_ref();
                // Property names are case-insensitive per RFC 5545
                // Normalize to uppercase for matching
                let name_upper = name_str.to_uppercase();
                match name_upper.as_str() {
                    $(
                        $kw => PropertyKind::$variant,
                    )*
                    _ => {
                        if name_str.starts_with("X-") || name_str.starts_with("x-") {
                            PropertyKind::XName(name)
                        } else {
                            PropertyKind::Unrecognized(name)
                        }
                    }
                }
            }
        }

        impl fmt::Display for PropertyKind<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(PropertyKind::$variant => write!(f, "{}", $kw),)*
                    PropertyKind::XName(s) | PropertyKind::Unrecognized(s) => {
                        write!(f, "{}", s)
                    }
                }
            }
        }

        #[cfg(test)]
        const KINDS: &[PropertyKind<'static>] = &[
            $(
                PropertyKind::$variant,
            )*
        ];
    };
}

// Define PropertyKind with all RFC 5545 properties and their value types
property_kind! {
    // 3.7.1.  Calendar Scale
    CalScale    => KW_CALSCALE  => &[ValueType::Text],
    // 3.7.2.  Method
    Method      => KW_METHOD    => &[ValueType::Text],
    // 3.7.3.  Product Identifier
    ProdId      => KW_PRODID    => &[ValueType::Text],
    // 3.7.4.  Version
    Version     => KW_VERSION   => &[ValueType::Text],
    // 3.8.1.1.  Attachment
    Attach      => KW_ATTACH    => &[ValueType::Uri, ValueType::Binary],
    // 3.8.1.2.  Categories
    Categories  => KW_CATEGORIES => &[ValueType::Text],
    // 3.8.1.3.  Classification
    Class       => KW_CLASS     => &[ValueType::Text],
    // 3.8.1.4.  Comment
    Comment     => KW_COMMENT   => &[ValueType::Text],
    // 3.8.1.5.  Description
    Description => KW_DESCRIPTION => &[ValueType::Text],
    // 3.8.1.6.  Geographic Position
    Geo         => KW_GEO       => &[ValueType::Text],
    // 3.8.1.7.  Location
    Location    => KW_LOCATION  => &[ValueType::Text],
    // 3.8.1.8.  Percent Complete
    PercentComplete => KW_PERCENT_COMPLETE => &[ValueType::Integer],
    // 3.8.1.9.  Priority
    Priority    => KW_PRIORITY  => &[ValueType::Integer],
    // 3.8.1.10.  Resources
    Resources   => KW_RESOURCES => &[ValueType::Text],
    // 3.8.1.11.  Status
    Status      => KW_STATUS    => &[ValueType::Text],
    // 3.8.1.12.  Summary
    Summary     => KW_SUMMARY   => &[ValueType::Text],
    // 3.8.2.1.  Date-Time Completed
    Completed   => KW_COMPLETED => &[ValueType::DateTime],
    // 3.8.2.2.  Date-Time End
    DtEnd       => KW_DTEND     => &[ValueType::DateTime, ValueType::Date],
    // 3.8.2.3.  Date-Time Due
    Due         => KW_DUE       => &[ValueType::DateTime, ValueType::Date],
    // 3.8.2.4.  Date-Time Start
    DtStart     => KW_DTSTART   => &[ValueType::DateTime, ValueType::Date],
    // 3.8.2.5.  Duration
    Duration    => KW_DURATION  => &[ValueType::Duration],
    // 3.8.2.6.  Free/Busy Time
    FreeBusy    => KW_FREEBUSY  => &[ValueType::Period],
    // 3.8.2.7.  Time Transparency
    Transp      => KW_TRANSP    => &[ValueType::Text],
    // 3.8.3.1.  Time Zone Identifier
    TzId        => KW_TZID      => &[ValueType::Text],
    // 3.8.3.2.  Time Zone Name
    TzName      => KW_TZNAME    => &[ValueType::Text],
    // 3.8.3.3.  Time Zone Offset From
    TzOffsetFrom => KW_TZOFFSETFROM => &[ValueType::UtcOffset],
    // 3.8.3.4.  Time Zone Offset To
    TzOffsetTo  => KW_TZOFFSETTO => &[ValueType::UtcOffset],
    // 3.8.3.5.  Time Zone URL
    TzUrl       => KW_TZURL     => &[ValueType::Uri],
    // 3.8.4.1.  Attendee
    Attendee    => KW_ATTENDEE  => &[ValueType::CalendarUserAddress],
    // 3.8.4.2.  Contact
    Contact     => KW_CONTACT   => &[ValueType::Text],
    // 3.8.4.3.  Organizer
    Organizer   => KW_ORGANIZER => &[ValueType::CalendarUserAddress],
    // 3.8.4.4.  Recurrence ID
    RecurrenceId => KW_RECURRENCE_ID => &[ValueType::DateTime, ValueType::Date],
    // 3.8.4.5.  Related To
    RelatedTo   => KW_RELATED_TO => &[ValueType::Text],
    // 3.8.4.6.  Uniform Resource Locator
    Url         => KW_URL       => &[ValueType::Uri],
    // 3.8.4.7.  Unique Identifier
    Uid         => KW_UID       => &[ValueType::Text],
    // 3.8.5.1.  Exception Date-Times
    ExDate      => KW_EXDATE    => &[ValueType::DateTime, ValueType::Date],
    // 3.8.5.2.  Recurrence Date-Times
    RDate       => KW_RDATE     => &[ValueType::DateTime, ValueType::Date, ValueType::Period],
    // 3.8.5.3.  Recurrence Rule
    RRule       => KW_RRULE     => &[ValueType::RecurrenceRule],
    // 3.8.6.1.  Action
    Action      => KW_ACTION    => &[ValueType::Text],
    // 3.8.6.2.  Repeat Count
    Repeat      => KW_REPEAT    => &[ValueType::Integer],
    // 3.8.6.3.  Trigger
    Trigger     => KW_TRIGGER   => &[ValueType::Duration, ValueType::DateTime],
    // 3.8.7.1.  Date-Time Created
    Created     => KW_CREATED   => &[ValueType::DateTime],
    // 3.8.7.2.  Date-Time Stamp
    DtStamp     => KW_DTSTAMP   => &[ValueType::DateTime],
    // 3.8.7.3.  Last Modified
    LastModified => KW_LAST_MODIFIED => &[ValueType::DateTime],
    // 3.8.7.4.  Sequence Number
    Sequence    => KW_SEQUENCE  => &[ValueType::Integer],
    // 3.8.8.3.  Request Status
    RequestStatus => KW_REQUEST_STATUS => &[ValueType::Text],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_kinds_have_value_types() {
        for kind in KINDS {
            if matches!(kind, PropertyKind::XName(_) | PropertyKind::Unrecognized(_)) {
                continue;
            }

            let value_types = kind
                .value_types()
                .expect("Known property kind must have value types");
            assert!(
                !value_types.is_empty(),
                "Property {kind:?}: value_types must not be empty",
            );
        }
    }
}
