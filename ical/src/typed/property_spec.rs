// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use std::num::NonZeroU8;

use crate::keyword::{
    KW_ACTION, KW_ATTACH, KW_ATTENDEE, KW_CALSCALE, KW_CATEGORIES, KW_CLASS, KW_COMMENT,
    KW_COMPLETED, KW_CONTACT, KW_CREATED, KW_DESCRIPTION, KW_DTEND, KW_DTSTAMP, KW_DTSTART, KW_DUE,
    KW_DURATION, KW_EXDATE, KW_FREEBUSY, KW_GEO, KW_LAST_MODIFIED, KW_LOCATION, KW_METHOD,
    KW_ORGANIZER, KW_PERCENT_COMPLETE, KW_PRIORITY, KW_PRODID, KW_RDATE, KW_RECURRENCE_ID,
    KW_RELATED_TO, KW_REPEAT, KW_REQUEST_STATUS, KW_RESOURCES, KW_RRULE, KW_SEQUENCE, KW_STATUS,
    KW_SUMMARY, KW_TRANSP, KW_TRIGGER, KW_TZID, KW_TZNAME, KW_TZOFFSETFROM, KW_TZOFFSETTO,
    KW_TZURL, KW_UID, KW_URL, KW_VERSION,
};
use crate::typed::parameter::TypedParameterKind;
use crate::typed::parameter_types::ValueType;

#[derive(Debug, Clone)]
pub struct PropertySpec<'a> {
    pub name: &'a str,
    /// Property cardinality: how many times this property can appear in a component
    pub property_cardinality: PropertyCardinality,
    /// Allowed parameter types for this property
    pub parameters: &'a [TypedParameterKind],
    /// Allowed value types for this property
    pub value_types: &'a [ValueType],
    /// The default value type for this property
    pub default_value_type: ValueType,
    /// Value cardinality: how many values in a single property line
    pub value_cardinality: ValueCardinality,
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

pub static PROPERTY_SPECS: &[PropertySpec] = &[
    // 3.7.1.  Calendar Scale - Calendar scale used for the calendar information (GREGORIAN default)
    PropertySpec {
        name: KW_CALSCALE,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.7.2.  Method - iCalendar object method associated with the calendar object
    PropertySpec {
        name: KW_METHOD,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.7.3.  Product Identifier - Identifier for the product that created the iCalendar object
    PropertySpec {
        name: KW_PRODID,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.7.4.  Version - iCalendar specification version required to interpret the object
    PropertySpec {
        name: KW_VERSION,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.1.  Attachment - Default URI, can be BINARY for inline content
    PropertySpec {
        name: KW_ATTACH,
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::FormatType,
            TypedParameterKind::Encoding,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::Uri, ValueType::Binary],
        default_value_type: ValueType::Uri,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.2.  Categories - Comma-separated text values for classification
    PropertySpec {
        name: KW_CATEGORIES,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[TypedParameterKind::Language],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.1.3.  Classification - Access classification (PUBLIC/PRIVATE/CONFIDENTIAL)
    PropertySpec {
        name: KW_CLASS,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.4.  Comment - Non-processing information, free-form text
    PropertySpec {
        name: KW_COMMENT,
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.5.  Description - Event/task description, single occurrence
    PropertySpec {
        name: KW_DESCRIPTION,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.6.  Geographic Position - Latitude;longitude as FLOAT values
    PropertySpec {
        name: KW_GEO,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Float],
        default_value_type: ValueType::Float,
        value_cardinality: ValueCardinality::exactly(2),
    },
    // 3.8.1.7.  Location - Event/task location
    PropertySpec {
        name: KW_LOCATION,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.8.  Percent Complete - Task completion percentage (0-100)
    PropertySpec {
        name: KW_PERCENT_COMPLETE,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Integer],
        default_value_type: ValueType::Integer,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.9.  Priority - Event/task priority level (0-9, 0=undefined, 1=highest)
    PropertySpec {
        name: KW_PRIORITY,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Integer],
        default_value_type: ValueType::Integer,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.10.  Resources - Event/task resources, comma-separated text
    PropertySpec {
        name: KW_RESOURCES,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.1.11.  Status - Event/task status (TENTATIVE/CONFIRMED/CANCELLED, etc.)
    PropertySpec {
        name: KW_STATUS,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.1.12.  Summary - Event/task summary/title
    PropertySpec {
        name: KW_SUMMARY,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.1.  Date-Time Completed - When a to-do was completed (VTODO only)
    PropertySpec {
        name: KW_COMPLETED,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.2.  Date-Time End - When an event ends (DATE or DATE-TIME)
    PropertySpec {
        name: KW_DTEND,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.3.  Date-Time Due - When a to-do is due (DATE or DATE-TIME)
    PropertySpec {
        name: KW_DUE,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.4.  Date-Time Start - When an event starts (DATE or DATE-TIME)
    PropertySpec {
        name: KW_DTSTART,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.5.  Duration - Event/task duration in iCalendar duration format
    PropertySpec {
        name: KW_DURATION,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Duration],
        default_value_type: ValueType::Duration,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.2.6.  Free/Busy Time - One or more free/busy time intervals (VFREEBUSY only)
    PropertySpec {
        name: KW_FREEBUSY,
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::FreeBusyType,
            TypedParameterKind::TimeZoneIdentifier,
        ],
        value_types: &[ValueType::Period],
        default_value_type: ValueType::Period,
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.2.7.  Time Transparency - Event transparency to busy time searches
    PropertySpec {
        name: KW_TRANSP,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.1.  Time Zone Identifier - Unique identifier for VTIMEZONE component
    PropertySpec {
        name: KW_TZID,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.2.  Time Zone Name - Customary designation for time zone
    PropertySpec {
        name: KW_TZNAME,
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[TypedParameterKind::Language],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.3.  Time Zone Offset From - Offset prior to time zone observance
    PropertySpec {
        name: KW_TZOFFSETFROM,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::UtcOffset],
        default_value_type: ValueType::UtcOffset,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.4.  Time Zone Offset To - Offset in use during time zone observance
    PropertySpec {
        name: KW_TZOFFSETTO,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::UtcOffset],
        default_value_type: ValueType::UtcOffset,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.3.5.  Time Zone URL - URI pointing to VTIMEZONE component location
    PropertySpec {
        name: KW_TZURL,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Uri],
        default_value_type: ValueType::Uri,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.1.  Attendee - Defines an attendee within a calendar component
    PropertySpec {
        name: KW_ATTENDEE,
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
        default_value_type: ValueType::CalendarUserAddress,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.2.  Contact - Contact information or reference
    PropertySpec {
        name: KW_CONTACT,
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::AlternateText,
            TypedParameterKind::Language,
        ],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.3.  Organizer - Defines the organizer for a calendar component
    PropertySpec {
        name: KW_ORGANIZER,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::CommonName,
            TypedParameterKind::Directory,
            TypedParameterKind::Language,
            TypedParameterKind::SendBy,
        ],
        value_types: &[ValueType::CalendarUserAddress],
        default_value_type: ValueType::CalendarUserAddress,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.4.  Recurrence ID - Identifies specific instance of recurring component
    PropertySpec {
        name: KW_RECURRENCE_ID,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::RecurrenceIdRange,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.5.  Related To - Represents relationship between calendar components
    PropertySpec {
        name: KW_RELATED_TO,
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[TypedParameterKind::RelationshipType],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.6.  Uniform Resource Locator - URL associated with iCalendar object
    PropertySpec {
        name: KW_URL,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Uri],
        default_value_type: ValueType::Uri,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.4.7.  Unique Identifier - Persistent, globally unique identifier
    PropertySpec {
        name: KW_UID,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.5.1.  Exception Date-Times - List of DATE-TIME exceptions for recurring items
    PropertySpec {
        name: KW_EXDATE,
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.5.2.  Recurrence Date-Times - List of DATE-TIME values for recurring items
    PropertySpec {
        name: KW_RDATE,
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
        ],
        value_types: &[ValueType::Date, ValueType::DateTime, ValueType::Period],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::at_least(1),
    },
    // 3.8.5.3.  Recurrence Rule - Rule or repeating pattern for recurring items
    PropertySpec {
        name: KW_RRULE,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::RecurrenceRule],
        default_value_type: ValueType::RecurrenceRule,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.6.1.  Action - Action to invoke when alarm is triggered
    PropertySpec {
        name: KW_ACTION,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.6.2.  Repeat Count - Number of times alarm should repeat after initial trigger
    PropertySpec {
        name: KW_REPEAT,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Integer],
        default_value_type: ValueType::Integer,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.6.3.  Trigger - When an alarm will trigger
    PropertySpec {
        name: KW_TRIGGER,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[
            TypedParameterKind::TimeZoneIdentifier,
            TypedParameterKind::ValueType,
            TypedParameterKind::AlarmTriggerRelationship,
        ],
        value_types: &[ValueType::Duration, ValueType::DateTime],
        default_value_type: ValueType::Duration,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.7.1.  Date-Time Created - Date/time calendar information was created
    PropertySpec {
        name: KW_CREATED,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.7.2.  Date-Time Stamp - Date/time instance was created or info was last revised
    PropertySpec {
        name: KW_DTSTAMP,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.7.3.  Last Modified - Date/time calendar component was last revised
    PropertySpec {
        name: KW_LAST_MODIFIED,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.7.4.  Sequence Number - Revision sequence number within sequence of revisions
    PropertySpec {
        name: KW_SEQUENCE,
        property_cardinality: PropertyCardinality::AtMostOnce,
        parameters: &[],
        value_types: &[ValueType::Integer],
        default_value_type: ValueType::Integer,
        value_cardinality: ValueCardinality::exactly(1),
    },
    // 3.8.8.1.  IANA Properties
    // 3.8.8.2.  Non-Standard Properties
    // 3.8.8.3.  Request Status - Status code returned for a scheduling request
    PropertySpec {
        name: KW_REQUEST_STATUS,
        property_cardinality: PropertyCardinality::Multiple,
        parameters: &[TypedParameterKind::Language],
        // Format: statcode ";" statdesc [";" extdata] - semicolon-separated structured value
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        value_cardinality: ValueCardinality::exactly(1),
    },
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_unique_property_names() {
        let names = PROPERTY_SPECS
            .iter()
            .map(|spec| spec.name)
            .collect::<HashSet<_>>();

        assert_eq!(
            names.len(),
            PROPERTY_SPECS.len(),
            "Property names should be unique"
        );
    }

    #[test]
    fn test_property_specs() {
        for spec in PROPERTY_SPECS {
            let name = spec.name;
            assert!(!name.is_empty(), "Property name should not be empty");
            assert!(
                name.chars().all(|a| a.is_ascii_uppercase() || a == '-'),
                "Property {name}: name should be uppercase ASCII or hyphen"
            );
            assert!(
                spec.value_types.contains(&spec.default_value_type),
                "Property {name}: default_kind should be in allowed_kinds"
            );
        }
    }
}
