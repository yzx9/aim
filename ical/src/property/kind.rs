// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property kinds and value types for iCalendar properties.
//!
//! This module defines the `PropertyKind` enum that represents all standard
//! iCalendar properties defined in RFC 5545, along with their allowed value types.

use std::fmt;
use std::str::FromStr;

use crate::keyword::{
    KW_ACTION, KW_ATTACH, KW_ATTENDEE, KW_CALSCALE, KW_CATEGORIES, KW_CLASS, KW_COMMENT,
    KW_COMPLETED, KW_CONTACT, KW_CREATED, KW_DESCRIPTION, KW_DTEND, KW_DTSTAMP, KW_DTSTART, KW_DUE,
    KW_DURATION, KW_EXDATE, KW_FREEBUSY, KW_GEO, KW_LAST_MODIFIED, KW_LOCATION, KW_METHOD,
    KW_ORGANIZER, KW_PERCENT_COMPLETE, KW_PRIORITY, KW_PRODID, KW_RDATE, KW_RECURRENCE_ID,
    KW_RELATED_TO, KW_REPEAT, KW_REQUEST_STATUS, KW_RESOURCES, KW_RRULE, KW_SEQUENCE, KW_STATUS,
    KW_SUMMARY, KW_TRANSP, KW_TRIGGER, KW_TZID, KW_TZNAME, KW_TZOFFSETFROM, KW_TZOFFSETTO,
    KW_TZURL, KW_UID, KW_URL, KW_VERSION,
};
use crate::parameter::ValueKind;
use crate::syntax::SpannedSegments;

/// Macro to define `PropertyKind` with associated value types.
///
/// Usage: `property_kind!(Variant => KW => &[...], ...)`
macro_rules! property_kind {
    (
        $(
            $(#[$attr:meta])*
            $variant:ident => $kw:expr => $value_types:expr $(,)?
        )*
    ) => {
        /// Kind of iCalendar property.
        /// Represents all standard properties defined in RFC 5545.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
        #[expect(missing_docs)]
        pub enum PropertyKind {
            $(
                $(#[$attr])*
                $variant,
            )*
        }

        impl PropertyKind {
            /// Returns the keyword string for this property kind.
            #[must_use]
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(PropertyKind::$variant => $kw,)*
                }
            }

            /// Returns the allowed value types for this property kind.
            #[must_use]
            pub fn value_kinds(&self) -> &'static [ValueKind] {
                match self {
                    $(PropertyKind::$variant => $value_types,)*
                }
            }
        }

        impl FromStr for PropertyKind {
            type Err = ();
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                $(
                    if value.eq_ignore_ascii_case($kw) {
                        return Ok(PropertyKind::$variant);
                    }
                )*
                Err(())
            }
        }

        impl fmt::Display for PropertyKind {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.as_str().fmt(f)
            }
        }

        impl<'src> TryFrom<&SpannedSegments<'src>> for PropertyKind {
            type Error = ();
            fn try_from(value: &SpannedSegments<'src>) -> Result<Self, Self::Error> {
                $(
                    if value.eq_str_ignore_ascii_case($kw) {
                        return Ok(PropertyKind::$variant);
                    }
                )*
                Err(())
            }
        }
    };
}

// Define PropertyKind with all RFC 5545 properties and their value types
property_kind! {
    // 3.7.1.  Calendar Scale
    CalScale    => KW_CALSCALE  => &[ValueKind::Text],
    // 3.7.2.  Method
    Method      => KW_METHOD    => &[ValueKind::Text],
    // 3.7.3.  Product Identifier
    ProdId      => KW_PRODID    => &[ValueKind::Text],
    // 3.7.4.  Version
    Version     => KW_VERSION   => &[ValueKind::Text],
    // 3.8.1.1.  Attachment
    Attach      => KW_ATTACH    => &[ValueKind::Uri, ValueKind::Binary],
    // 3.8.1.2.  Categories
    Categories  => KW_CATEGORIES => &[ValueKind::Text],
    // 3.8.1.3.  Classification
    Class       => KW_CLASS     => &[ValueKind::Text],
    // 3.8.1.4.  Comment
    Comment     => KW_COMMENT   => &[ValueKind::Text],
    // 3.8.1.5.  Description
    Description => KW_DESCRIPTION => &[ValueKind::Text],
    // 3.8.1.6.  Geographic Position
    Geo         => KW_GEO       => &[ValueKind::Text],
    // 3.8.1.7.  Location
    Location    => KW_LOCATION  => &[ValueKind::Text],
    // 3.8.1.8.  Percent Complete
    PercentComplete => KW_PERCENT_COMPLETE => &[ValueKind::Integer],
    // 3.8.1.9.  Priority
    Priority    => KW_PRIORITY  => &[ValueKind::Integer],
    // 3.8.1.10.  Resources
    Resources   => KW_RESOURCES => &[ValueKind::Text],
    // 3.8.1.11.  Status
    Status      => KW_STATUS    => &[ValueKind::Text],
    // 3.8.1.12.  Summary
    Summary     => KW_SUMMARY   => &[ValueKind::Text],
    // 3.8.2.1.  Date-Time Completed
    Completed   => KW_COMPLETED => &[ValueKind::DateTime],
    // 3.8.2.2.  Date-Time End
    DtEnd       => KW_DTEND     => &[ValueKind::DateTime, ValueKind::Date],
    // 3.8.2.3.  Date-Time Due
    Due         => KW_DUE       => &[ValueKind::DateTime, ValueKind::Date],
    // 3.8.2.4.  Date-Time Start
    DtStart     => KW_DTSTART   => &[ValueKind::DateTime, ValueKind::Date],
    // 3.8.2.5.  Duration
    Duration    => KW_DURATION  => &[ValueKind::Duration],
    // 3.8.2.6.  Free/Busy Time
    FreeBusy    => KW_FREEBUSY  => &[ValueKind::Period],
    // 3.8.2.7.  Time Transparency
    Transp      => KW_TRANSP    => &[ValueKind::Text],
    // 3.8.3.1.  Time Zone Identifier
    TzId        => KW_TZID      => &[ValueKind::Text],
    // 3.8.3.2.  Time Zone Name
    TzName      => KW_TZNAME    => &[ValueKind::Text],
    // 3.8.3.3.  Time Zone Offset From
    TzOffsetFrom => KW_TZOFFSETFROM => &[ValueKind::UtcOffset],
    // 3.8.3.4.  Time Zone Offset To
    TzOffsetTo  => KW_TZOFFSETTO => &[ValueKind::UtcOffset],
    // 3.8.3.5.  Time Zone URL
    TzUrl       => KW_TZURL     => &[ValueKind::Uri],
    // 3.8.4.1.  Attendee
    Attendee    => KW_ATTENDEE  => &[ValueKind::CalendarUserAddress],
    // 3.8.4.2.  Contact
    Contact     => KW_CONTACT   => &[ValueKind::Text],
    // 3.8.4.3.  Organizer
    Organizer   => KW_ORGANIZER => &[ValueKind::CalendarUserAddress],
    // 3.8.4.4.  Recurrence ID
    RecurrenceId => KW_RECURRENCE_ID => &[ValueKind::DateTime, ValueKind::Date],
    // 3.8.4.5.  Related To
    RelatedTo   => KW_RELATED_TO => &[ValueKind::Text],
    // 3.8.4.6.  Uniform Resource Locator
    Url         => KW_URL       => &[ValueKind::Uri],
    // 3.8.4.7.  Unique Identifier
    Uid         => KW_UID       => &[ValueKind::Text],
    // 3.8.5.1.  Exception Date-Times
    ExDate      => KW_EXDATE    => &[ValueKind::DateTime, ValueKind::Date],
    // 3.8.5.2.  Recurrence Date-Times
    RDate       => KW_RDATE     => &[ValueKind::DateTime, ValueKind::Date, ValueKind::Period],
    // 3.8.5.3.  Recurrence Rule
    RRule       => KW_RRULE     => &[ValueKind::RecurrenceRule],
    // 3.8.6.1.  Action
    Action      => KW_ACTION    => &[ValueKind::Text],
    // 3.8.6.2.  Repeat Count
    Repeat      => KW_REPEAT    => &[ValueKind::Integer],
    // 3.8.6.3.  Trigger
    Trigger     => KW_TRIGGER   => &[ValueKind::Duration, ValueKind::DateTime],
    // 3.8.7.1.  Date-Time Created
    Created     => KW_CREATED   => &[ValueKind::DateTime],
    // 3.8.7.2.  Date-Time Stamp
    DtStamp     => KW_DTSTAMP   => &[ValueKind::DateTime],
    // 3.8.7.3.  Last Modified
    LastModified => KW_LAST_MODIFIED => &[ValueKind::DateTime],
    // 3.8.7.4.  Sequence Number
    Sequence    => KW_SEQUENCE  => &[ValueKind::Integer],
    // 3.8.8.3.  Request Status
    RequestStatus => KW_REQUEST_STATUS => &[ValueKind::Text],
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn all_kinds_have_value_types() {
        // Verify that every PropertyKind variant has value types defined
        for kind in PropertyKind::iter() {
            let value_types = kind.value_kinds();
            assert!(
                !value_types.is_empty(),
                "Property {kind:?}: value_types must not be empty",
            );
        }
    }
}
