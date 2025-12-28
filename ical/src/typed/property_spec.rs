// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use std::fmt::{Display, Formatter};
use std::{num::NonZeroU8, str::FromStr};

use crate::keyword::{
    KW_ACTION, KW_ATTACH, KW_ATTENDEE, KW_CALSCALE, KW_CATEGORIES, KW_CLASS, KW_COMMENT,
    KW_COMPLETED, KW_CONTACT, KW_CREATED, KW_DESCRIPTION, KW_DTEND, KW_DTSTAMP, KW_DTSTART, KW_DUE,
    KW_DURATION, KW_EXDATE, KW_FREEBUSY, KW_GEO, KW_LAST_MODIFIED, KW_LOCATION, KW_METHOD,
    KW_ORGANIZER, KW_PERCENT_COMPLETE, KW_PRIORITY, KW_PRODID, KW_RDATE, KW_RECURRENCE_ID,
    KW_RELATED_TO, KW_REPEAT, KW_REQUEST_STATUS, KW_RESOURCES, KW_RRULE, KW_SEQUENCE, KW_STATUS,
    KW_SUMMARY, KW_TRANSP, KW_TRIGGER, KW_TZID, KW_TZNAME, KW_TZOFFSETFROM, KW_TZOFFSETTO,
    KW_TZURL, KW_UID, KW_URL, KW_VERSION,
};
use crate::syntax::SpannedSegments;
use crate::typed::parameter::TypedParameterKind;
use crate::typed::parameter_type::ValueType;

/// Macro to define `PropertyKind` with all associated metadata.
///
/// Usage: `property_kind!(Variant => KW => PropertySpec { ... }, ...)`
macro_rules! property_kind {
    (
        $(
            $(#[$attr:meta])*
            $variant:ident => $kw:expr => PropertySpec {
                $($field:ident : $value:expr),* $(,)?
            }
            $(,)?
        )*
    ) => {
        /// Kind of iCalendar property.
        /// Represents all standard properties defined in RFC 5545.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
        #[allow(missing_docs)]
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

            /// Returns the property specification for this property kind.
            #[must_use]
            pub fn spec(&self) -> &'static PropertySpec<'static> {
                match self {
                    $(
                        PropertyKind::$variant => {
                            const SPEC: PropertySpec<'static> = PropertySpec {
                                kind: PropertyKind::$variant,
                                $($field: $value),*
                            };
                            &SPEC
                        }
                    )*
                }
            }
        }

        // PERF: Optimize with better lookup
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

        impl Display for PropertyKind {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

// Define PropertyKind with all RFC 5545 properties
property_kind! {
    // 3.7.1.  Calendar Scale - Calendar scale used for the calendar information (GREGORIAN default)
    CalScale => KW_CALSCALE => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.7.2.  Method - iCalendar object method associated with the calendar object
    Method => KW_METHOD => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.7.3.  Product Identifier - Identifier for the product that created the iCalendar object
    ProdId => KW_PRODID => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.7.4.  Version - iCalendar specification version required to interpret the object
    Version => KW_VERSION => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.1.  Attachment - Default URI, can be BINARY for inline content
    Attach => KW_ATTACH => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::FormatType,
            TypedParameterKind::Encoding,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::Uri, ValueType::Binary],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.2.  Categories - Comma-separated text values for classification
    Categories => KW_CATEGORIES => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[TypedParameterKind::Language],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.1.3.  Classification - Access classification (PUBLIC/PRIVATE/CONFIDENTIAL)
    Class => KW_CLASS => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.4.  Comment - Non-processing information, free-form text
    Comment => KW_COMMENT => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.5.  Description - Event/task description, single occurrence
    Description => KW_DESCRIPTION => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.6.  Geographic Position - Latitude;longitude (semicolon-separated float values)
    // Parsed as TEXT in typed phase, actual float conversion happens in semantic phase
    Geo => KW_GEO => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.7.  Location - Event/task location
    Location => KW_LOCATION => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.8.  Percent Complete - Task completion percentage (0-100)
    PercentComplete => KW_PERCENT_COMPLETE => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Integer],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.9.  Priority - Event/task priority level (0-9, 0=undefined, 1=highest)
    Priority => KW_PRIORITY => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Integer],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.10.  Resources - Event/task resources, comma-separated text
    Resources => KW_RESOURCES => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.1.11.  Status - Event/task status (TENTATIVE/CONFIRMED/CANCELLED, etc.)
    Status => KW_STATUS => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.12.  Summary - Event/task summary/title
    Summary => KW_SUMMARY => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.1.  Date-Time Completed - When a to-do was completed (VTODO only)
    Completed => KW_COMPLETED => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::DateTime],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.2.  Date-Time End - When an event ends (DATE or DATE-TIME)
    DtEnd => KW_DTEND => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::DateTime, ValueType::Date],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.3.  Date-Time Due - When a to-do is due (DATE or DATE-TIME)
    Due => KW_DUE => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::DateTime, ValueType::Date],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.4.  Date-Time Start - When an event starts (DATE or DATE-TIME)
    DtStart => KW_DTSTART => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::DateTime, ValueType::Date],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.5.  Duration - Event/task duration in iCalendar duration format
    Duration => KW_DURATION => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Duration],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.6.  Free/Busy Time - One or more free/busy time intervals (VFREEBUSY only)
    FreeBusy => KW_FREEBUSY => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::FreeBusyType,
            TypedParameterKind::TimeZoneIdentifier,
        ],
        value_types: &[ValueType::Period],
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.2.7.  Time Transparency - Event transparency to busy time searches
    Transp => KW_TRANSP => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.1.  Time Zone Identifier - Unique identifier for VTIMEZONE component
    TzId => KW_TZID => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.2.  Time Zone Name - Customary designation for time zone
    TzName => KW_TZNAME => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[TypedParameterKind::Language],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.3.  Time Zone Offset From - Offset prior to time zone observance
    TzOffsetFrom => KW_TZOFFSETFROM => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::UtcOffset],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.4.  Time Zone Offset To - Offset in use during time zone observance
    TzOffsetTo => KW_TZOFFSETTO => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::UtcOffset],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.5.  Time Zone URL - URI pointing to VTIMEZONE component location
    TzUrl => KW_TZURL => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Uri],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.1.  Attendee - Defines an attendee within a calendar component
    Attendee => KW_ATTENDEE => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::CommonName,
            TypedParameterKind::CalendarUserType,
            TypedParameterKind::Delegators,
            TypedParameterKind::Delegatees,
            TypedParameterKind::Directory,
            TypedParameterKind::Language,
            TypedParameterKind::GroupOrListMembership,
            TypedParameterKind::ParticipationStatus,
            TypedParameterKind::ParticipationRole,
            TypedParameterKind::RsvpExpectation,
            TypedParameterKind::SendBy,
        ],
        value_types: &[ValueType::CalendarUserAddress],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.2.  Contact - Contact information or reference
    Contact => KW_CONTACT => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.3.  Organizer - Defines the organizer for a calendar component
    Organizer => KW_ORGANIZER => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::CommonName,
            TypedParameterKind::Directory,
            TypedParameterKind::Language,
            TypedParameterKind::SendBy,
        ],
        value_types: &[ValueType::CalendarUserAddress],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.4.  Recurrence ID - Identifies specific instance of recurring component
    RecurrenceId => KW_RECURRENCE_ID => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::RecurrenceIdRange,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::DateTime, ValueType::Date],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.5.  Related To - Represents relationship between calendar components
    RelatedTo => KW_RELATED_TO => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[TypedParameterKind::RelationshipType],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.6.  Uniform Resource Locator - URL associated with iCalendar object
    Url => KW_URL => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Uri],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.7.  Unique Identifier - Persistent, globally unique identifier
    Uid => KW_UID => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.5.1.  Exception Date-Times - List of DATE-TIME exceptions for recurring items
    ExDate => KW_EXDATE => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::DateTime, ValueType::Date],
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.5.2.  Recurrence Date-Times - List of DATE-TIME values for recurring items
    RDate => KW_RDATE => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::DateTime, ValueType::Date, ValueType::Period],
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.5.3.  Recurrence Rule - Rule or repeating pattern for recurring items
    RRule => KW_RRULE => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::RecurrenceRule],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.6.1.  Action - Action to invoke when alarm is triggered
    Action => KW_ACTION => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.6.2.  Repeat Count - Number of times alarm should repeat after initial trigger
    Repeat => KW_REPEAT => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Integer],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.6.3.  Trigger - When an alarm will trigger
    Trigger => KW_TRIGGER => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
            TypedParameterKind::AlarmTriggerRelationship,
        ],
        value_types: &[ValueType::Duration, ValueType::DateTime],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.7.1.  Date-Time Created - Date/time calendar information was created
    Created => KW_CREATED => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::DateTime],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.7.2.  Date-Time Stamp - Date/time instance was created or info was last revised
    DtStamp => KW_DTSTAMP => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::DateTime],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.7.3.  Last Modified - Date/time calendar component was last revised
    LastModified => KW_LAST_MODIFIED => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::DateTime],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.7.4.  Sequence Number - Revision sequence number within sequence of revisions
    Sequence => KW_SEQUENCE => PropertySpec {
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Integer],
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.8.1.  IANA Properties
    // 3.8.8.2.  Non-Standard Properties
    // 3.8.8.3.  Request Status - Status code returned for a scheduling request
    RequestStatus => KW_REQUEST_STATUS => PropertySpec {
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[TypedParameterKind::Language],
        // Format: statcode ";" statdesc [";" extdata] - semicolon-separated structured value
        value_types: &[ValueType::Text],
        value_cardinality: ValueCardinality::exactly(1),
    },
}

/// Specification for an iCalendar property.
#[derive(Debug, Clone)]
pub struct PropertySpec<'a> {
    /// The kind of property (enum)
    pub kind: PropertyKind,
    /// Property cardinality: how many times this property can appear in a component
    pub property_cardinality: PropertyCardinality,
    /// Allowed parameter types for this property
    pub parameters: &'a [TypedParameterKind],
    /// Allowed value types for this property
    pub value_types: &'a [ValueType],
    /// Value cardinality: how many values in a single property line
    pub value_cardinality: ValueCardinality,
}

impl PropertySpec<'_> {
    /// Returns the property name as a static string.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.kind.as_str()
    }
}

/// Specifies the cardinality of values within a single property instance.
/// This refers to comma-separated values on a single property line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueCardinality {
    /// Exactly N values are required in this property instance
    Exactly(NonZeroU8),
    /// At least N values are allowed in this property instance (comma-separated)
    AtLeast(NonZeroU8),
}

impl ValueCardinality {
    /// Create a variant that requires exactly N values.
    #[must_use]
    const fn exactly(n: u8) -> Self {
        match NonZeroU8::new(n) {
            Some(n) => ValueCardinality::Exactly(n),
            None => panic!("exactly requires a non-zero value"),
        }
    }

    /// Create a variant that allows at least N values.
    #[must_use]
    const fn at_least(n: u8) -> Self {
        match NonZeroU8::new(n) {
            Some(n) => ValueCardinality::AtLeast(n),
            None => panic!("at_least requires a non-zero value"),
        }
    }
}

/// Specifies how many times a property can appear in a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyCardinality {
    /// This property can appear at most once in the component
    AtMostOnce,
    /// This property can appear multiple times in the component
    Multiple,
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn all_kinds_have_spec() {
        // Verify that every PropertyKind variant has a corresponding spec
        for kind in PropertyKind::iter() {
            let spec = kind.spec();
            assert_eq!(spec.kind, kind, "PropertySpec mismatch for {kind:?}");

            // Verify spec integrity
            let name = spec.name();
            assert!(!name.is_empty(), "Property name should not be empty");
            assert!(
                name.chars().all(|a| a.is_ascii_uppercase() || a == '-'),
                "Property {name}: name should be uppercase ASCII or hyphen"
            );
            // Verify value_types is not empty
            assert!(
                !spec.value_types.is_empty(),
                "Property {name}: value_types must not be empty"
            );
        }
    }
}
