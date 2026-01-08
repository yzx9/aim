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
use crate::parameter::ValueTypeRef;
use crate::syntax::SpannedSegments;

/// Type alias for borrowed `PropertyKind`
pub type PropertyKindRef<'src> = PropertyKind<SpannedSegments<'src>>;

/// Type alias for owned `PropertyKind`
pub type PropertyKindOwned = PropertyKind<String>;

impl PropertyKindRef<'_> {
    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> PropertyKindOwned {
        match self {
            PropertyKind::Action => PropertyKindOwned::Action,
            PropertyKind::Attach => PropertyKindOwned::Attach,
            PropertyKind::Attendee => PropertyKindOwned::Attendee,
            PropertyKind::CalScale => PropertyKindOwned::CalScale,
            PropertyKind::Categories => PropertyKindOwned::Categories,
            PropertyKind::Class => PropertyKindOwned::Class,
            PropertyKind::Comment => PropertyKindOwned::Comment,
            PropertyKind::Completed => PropertyKindOwned::Completed,
            PropertyKind::Contact => PropertyKindOwned::Contact,
            PropertyKind::Created => PropertyKindOwned::Created,
            PropertyKind::Description => PropertyKindOwned::Description,
            PropertyKind::DtEnd => PropertyKindOwned::DtEnd,
            PropertyKind::DtStamp => PropertyKindOwned::DtStamp,
            PropertyKind::DtStart => PropertyKindOwned::DtStart,
            PropertyKind::Due => PropertyKindOwned::Due,
            PropertyKind::Duration => PropertyKindOwned::Duration,
            PropertyKind::ExDate => PropertyKindOwned::ExDate,
            PropertyKind::FreeBusy => PropertyKindOwned::FreeBusy,
            PropertyKind::Geo => PropertyKindOwned::Geo,
            PropertyKind::LastModified => PropertyKindOwned::LastModified,
            PropertyKind::Location => PropertyKindOwned::Location,
            PropertyKind::Method => PropertyKindOwned::Method,
            PropertyKind::Organizer => PropertyKindOwned::Organizer,
            PropertyKind::PercentComplete => PropertyKindOwned::PercentComplete,
            PropertyKind::Priority => PropertyKindOwned::Priority,
            PropertyKind::ProdId => PropertyKindOwned::ProdId,
            PropertyKind::RDate => PropertyKindOwned::RDate,
            PropertyKind::RecurrenceId => PropertyKindOwned::RecurrenceId,
            PropertyKind::RelatedTo => PropertyKindOwned::RelatedTo,
            PropertyKind::Repeat => PropertyKindOwned::Repeat,
            PropertyKind::RequestStatus => PropertyKindOwned::RequestStatus,
            PropertyKind::Resources => PropertyKindOwned::Resources,
            PropertyKind::RRule => PropertyKindOwned::RRule,
            PropertyKind::Sequence => PropertyKindOwned::Sequence,
            PropertyKind::Status => PropertyKindOwned::Status,
            PropertyKind::Summary => PropertyKindOwned::Summary,
            PropertyKind::Transp => PropertyKindOwned::Transp,
            PropertyKind::Trigger => PropertyKindOwned::Trigger,
            PropertyKind::TzId => PropertyKindOwned::TzId,
            PropertyKind::TzName => PropertyKindOwned::TzName,
            PropertyKind::TzOffsetFrom => PropertyKindOwned::TzOffsetFrom,
            PropertyKind::TzOffsetTo => PropertyKindOwned::TzOffsetTo,
            PropertyKind::TzUrl => PropertyKindOwned::TzUrl,
            PropertyKind::Uid => PropertyKindOwned::Uid,
            PropertyKind::Url => PropertyKindOwned::Url,
            PropertyKind::Version => PropertyKindOwned::Version,
            PropertyKind::XName(s) => PropertyKindOwned::XName(s.concatnate()),
            PropertyKind::Unrecognized(s) => PropertyKindOwned::Unrecognized(s.concatnate()),
        }
    }
}

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
        pub enum PropertyKind<S: Clone + fmt::Display> {
            $(
                $(#[$attr])*
                $variant,
            )*
            /// Custom experimental x-name property (must start with "X-" or "x-")
            XName(S),
            /// Unrecognized property (not a known standard property)
            Unrecognized(S),
        }

        impl<S: Clone + fmt::Display> PropertyKind<S> {
            /// Returns the allowed value types for this property kind, if known.
            /// Returns `None` for unrecognized or x-name properties.
            #[must_use]
            pub(crate) fn value_types(&self) -> Option<&'static [ValueTypeRef<'static>]> {
                match self {
                    $(PropertyKind::$variant => Some($value_types),)*

                    // dont know the exact allowed types for unknown properties
                    PropertyKind::XName(_) | PropertyKind::Unrecognized(_) =>  None,
                }
            }
        }

        impl<'src> From<SpannedSegments<'src>> for PropertyKindRef<'src> {
            fn from(name: SpannedSegments<'src>) -> Self {
                $(
                    if name.eq_str_ignore_ascii_case($kw) {
                        return PropertyKind::$variant;
                    }
                )*

                if name.starts_with_str_ignore_ascii_case("X-") {
                    PropertyKind::XName(name)
                } else {
                    PropertyKind::Unrecognized(name)
                }
            }
        }

        impl<S: Clone + fmt::Display> fmt::Display for PropertyKind<S> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(PropertyKind::$variant => write!(f, "{}", $kw),)*
                    PropertyKind::XName(s) | PropertyKind::Unrecognized(s) => write!(f, "{}", s),
                }
            }
        }

        #[cfg(test)]
        const KINDS: &[PropertyKindRef<'static>] = &[
            $(
                PropertyKind::$variant,
            )*
        ];
    };
}

// Define PropertyKind with all RFC 5545 properties and their value types
property_kind! {
    // 3.7.1.  Calendar Scale
    CalScale    => KW_CALSCALE  => &[ValueTypeRef::Text],
    // 3.7.2.  Method
    Method      => KW_METHOD    => &[ValueTypeRef::Text],
    // 3.7.3.  Product Identifier
    ProdId      => KW_PRODID    => &[ValueTypeRef::Text],
    // 3.7.4.  Version
    Version     => KW_VERSION   => &[ValueTypeRef::Text],
    // 3.8.1.1.  Attachment
    Attach      => KW_ATTACH    => &[ValueTypeRef::Uri, ValueTypeRef::Binary],
    // 3.8.1.2.  Categories
    Categories  => KW_CATEGORIES => &[ValueTypeRef::Text],
    // 3.8.1.3.  Classification
    Class       => KW_CLASS     => &[ValueTypeRef::Text],
    // 3.8.1.4.  Comment
    Comment     => KW_COMMENT   => &[ValueTypeRef::Text],
    // 3.8.1.5.  Description
    Description => KW_DESCRIPTION => &[ValueTypeRef::Text],
    // 3.8.1.6.  Geographic Position
    Geo         => KW_GEO       => &[ValueTypeRef::Text],
    // 3.8.1.7.  Location
    Location    => KW_LOCATION  => &[ValueTypeRef::Text],
    // 3.8.1.8.  Percent Complete
    PercentComplete => KW_PERCENT_COMPLETE => &[ValueTypeRef::Integer],
    // 3.8.1.9.  Priority
    Priority    => KW_PRIORITY  => &[ValueTypeRef::Integer],
    // 3.8.1.10.  Resources
    Resources   => KW_RESOURCES => &[ValueTypeRef::Text],
    // 3.8.1.11.  Status
    Status      => KW_STATUS    => &[ValueTypeRef::Text],
    // 3.8.1.12.  Summary
    Summary     => KW_SUMMARY   => &[ValueTypeRef::Text],
    // 3.8.2.1.  Date-Time Completed
    Completed   => KW_COMPLETED => &[ValueTypeRef::DateTime],
    // 3.8.2.2.  Date-Time End
    DtEnd       => KW_DTEND     => &[ValueTypeRef::DateTime, ValueTypeRef::Date],
    // 3.8.2.3.  Date-Time Due
    Due         => KW_DUE       => &[ValueTypeRef::DateTime, ValueTypeRef::Date],
    // 3.8.2.4.  Date-Time Start
    DtStart     => KW_DTSTART   => &[ValueTypeRef::DateTime, ValueTypeRef::Date],
    // 3.8.2.5.  Duration
    Duration    => KW_DURATION  => &[ValueTypeRef::Duration],
    // 3.8.2.6.  Free/Busy Time
    FreeBusy    => KW_FREEBUSY  => &[ValueTypeRef::Period],
    // 3.8.2.7.  Time Transparency
    Transp      => KW_TRANSP    => &[ValueTypeRef::Text],
    // 3.8.3.1.  Time Zone Identifier
    TzId        => KW_TZID      => &[ValueTypeRef::Text],
    // 3.8.3.2.  Time Zone Name
    TzName      => KW_TZNAME    => &[ValueTypeRef::Text],
    // 3.8.3.3.  Time Zone Offset From
    TzOffsetFrom => KW_TZOFFSETFROM => &[ValueTypeRef::UtcOffset],
    // 3.8.3.4.  Time Zone Offset To
    TzOffsetTo  => KW_TZOFFSETTO => &[ValueTypeRef::UtcOffset],
    // 3.8.3.5.  Time Zone URL
    TzUrl       => KW_TZURL     => &[ValueTypeRef::Uri],
    // 3.8.4.1.  Attendee
    Attendee    => KW_ATTENDEE  => &[ValueTypeRef::CalendarUserAddress],
    // 3.8.4.2.  Contact
    Contact     => KW_CONTACT   => &[ValueTypeRef::Text],
    // 3.8.4.3.  Organizer
    Organizer   => KW_ORGANIZER => &[ValueTypeRef::CalendarUserAddress],
    // 3.8.4.4.  Recurrence ID
    RecurrenceId => KW_RECURRENCE_ID => &[ValueTypeRef::DateTime, ValueTypeRef::Date],
    // 3.8.4.5.  Related To
    RelatedTo   => KW_RELATED_TO => &[ValueTypeRef::Text],
    // 3.8.4.6.  Uniform Resource Locator
    Url         => KW_URL       => &[ValueTypeRef::Uri],
    // 3.8.4.7.  Unique Identifier
    Uid         => KW_UID       => &[ValueTypeRef::Text],
    // 3.8.5.1.  Exception Date-Times
    ExDate      => KW_EXDATE    => &[ValueTypeRef::DateTime, ValueTypeRef::Date],
    // 3.8.5.2.  Recurrence Date-Times
    RDate       => KW_RDATE     => &[ValueTypeRef::DateTime, ValueTypeRef::Date, ValueTypeRef::Period],
    // 3.8.5.3.  Recurrence Rule
    RRule       => KW_RRULE     => &[ValueTypeRef::RecurrenceRule],
    // 3.8.6.1.  Action
    Action      => KW_ACTION    => &[ValueTypeRef::Text],
    // 3.8.6.2.  Repeat Count
    Repeat      => KW_REPEAT    => &[ValueTypeRef::Integer],
    // 3.8.6.3.  Trigger
    Trigger     => KW_TRIGGER   => &[ValueTypeRef::Duration, ValueTypeRef::DateTime],
    // 3.8.7.1.  Date-Time Created
    Created     => KW_CREATED   => &[ValueTypeRef::DateTime],
    // 3.8.7.2.  Date-Time Stamp
    DtStamp     => KW_DTSTAMP   => &[ValueTypeRef::DateTime],
    // 3.8.7.3.  Last Modified
    LastModified => KW_LAST_MODIFIED => &[ValueTypeRef::DateTime],
    // 3.8.7.4.  Sequence Number
    Sequence    => KW_SEQUENCE  => &[ValueTypeRef::Integer],
    // 3.8.8.3.  Request Status
    RequestStatus => KW_REQUEST_STATUS => &[ValueTypeRef::Text],
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
