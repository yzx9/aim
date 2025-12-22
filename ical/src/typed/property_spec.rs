// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

use crate::keyword::{
    KW_ACTION, KW_ATTACH, KW_ATTENDEE, KW_CATEGORIES, KW_CLASS, KW_COMMENT, KW_COMPLETED,
    KW_CONTACT, KW_CREATED, KW_DESCRIPTION, KW_DTEND, KW_DTSTAMP, KW_DTSTART, KW_DUE, KW_DURATION,
    KW_EXDATE, KW_FREEBUSY, KW_GEO, KW_LAST_MODIFIED, KW_LOCATION, KW_ORGANIZER,
    KW_PERCENT_COMPLETE, KW_PRIORITY, KW_RDATE, KW_RECURRENCE_ID, KW_RELATED_TO, KW_REPEAT,
    KW_REQUEST_STATUS, KW_RESOURCES, KW_RRULE, KW_SEQUENCE, KW_STATUS, KW_SUMMARY, KW_TRANSP,
    KW_TRIGGER, KW_TZID, KW_TZNAME, KW_TZOFFSETFROM, KW_TZOFFSETTO, KW_TZURL, KW_UID, KW_URL,
};
use crate::typed::parameter_types::ValueType;

#[derive(Debug, Clone)]
pub struct PropertySpec<'a> {
    pub name: &'a str,
    pub value_types: &'a [ValueType],
    pub default_value_type: ValueType,
    pub multiple_values: bool,
}

pub static PROPERTY_SPECS: &[PropertySpec] = &[
    // 3.8.1.1.  Attachment - Default URI, can be BINARY for inline content
    PropertySpec {
        name: KW_ATTACH,
        value_types: &[ValueType::Uri, ValueType::Binary],
        default_value_type: ValueType::Uri,
        multiple_values: true,
    },
    // 3.8.1.2.  Categories - Comma-separated text values for classification
    PropertySpec {
        name: KW_CATEGORIES,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: true,
    },
    // 3.8.1.3.  Classification - Access classification (PUBLIC/PRIVATE/CONFIDENTIAL)
    PropertySpec {
        name: KW_CLASS,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: false,
    },
    // 3.8.1.4.  Comment - Non-processing information, free-form text
    PropertySpec {
        name: KW_COMMENT,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: true,
    },
    // 3.8.1.5.  Description - Event/task description, single occurrence
    PropertySpec {
        name: KW_DESCRIPTION,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: false,
    },
    // 3.8.1.6.  Geographic Position - Latitude;longitude as FLOAT values
    PropertySpec {
        name: KW_GEO,
        value_types: &[ValueType::Float],
        default_value_type: ValueType::Float,
        multiple_values: false,
    },
    // 3.8.1.7.  Location - Event/task location
    PropertySpec {
        name: KW_LOCATION,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: false,
    },
    // 3.8.1.8.  Percent Complete - Task completion percentage (0-100)
    PropertySpec {
        name: KW_PERCENT_COMPLETE,
        value_types: &[ValueType::Integer],
        default_value_type: ValueType::Integer,
        multiple_values: false,
    },
    // 3.8.1.9.  Priority - Event/task priority level (0-9, 0=undefined, 1=highest)
    PropertySpec {
        name: KW_PRIORITY,
        value_types: &[ValueType::Integer],
        default_value_type: ValueType::Integer,
        multiple_values: false,
    },
    // 3.8.1.10.  Resources - Event/task resources, comma-separated text
    PropertySpec {
        name: KW_RESOURCES,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: true,
    },
    // 3.8.1.11.  Status - Event/task status (TENTATIVE/CONFIRMED/CANCELLED, etc.)
    PropertySpec {
        name: KW_STATUS,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: false,
    },
    // 3.8.1.12.  Summary - Event/task summary/title
    PropertySpec {
        name: KW_SUMMARY,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: false,
    },
    // 3.8.2.1.  Date-Time Completed - When a to-do was completed (VTODO only)
    PropertySpec {
        name: KW_COMPLETED,
        value_types: &[ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        multiple_values: false,
    },
    // 3.8.2.2.  Date-Time End - When an event ends (DATE or DATE-TIME)
    PropertySpec {
        name: KW_DTEND,
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        multiple_values: false,
    },
    // 3.8.2.3.  Date-Time Due - When a to-do is due (DATE or DATE-TIME)
    PropertySpec {
        name: KW_DUE,
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        multiple_values: false,
    },
    // 3.8.2.4.  Date-Time Start - When an event starts (DATE or DATE-TIME)
    PropertySpec {
        name: KW_DTSTART,
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        multiple_values: false,
    },
    // 3.8.2.5.  Duration - Event/task duration in iCalendar duration format
    PropertySpec {
        name: KW_DURATION,
        value_types: &[ValueType::Duration],
        default_value_type: ValueType::Duration,
        multiple_values: false,
    },
    // 3.8.2.6.  Free/Busy Time - One or more free/busy time intervals (VFREEBUSY only)
    PropertySpec {
        name: KW_FREEBUSY,
        value_types: &[ValueType::Period],
        default_value_type: ValueType::Period,
        multiple_values: true,
    },
    // 3.8.2.7.  Time Transparency - Event transparency to busy time searches
    PropertySpec {
        name: KW_TRANSP,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: false,
    },
    // 3.8.3.1.  Time Zone Identifier - Unique identifier for VTIMEZONE component
    PropertySpec {
        name: KW_TZID,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: false,
    },
    // 3.8.3.2.  Time Zone Name - Customary designation for time zone
    PropertySpec {
        name: KW_TZNAME,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: true,
    },
    // 3.8.3.3.  Time Zone Offset From - Offset prior to time zone observance
    PropertySpec {
        name: KW_TZOFFSETFROM,
        value_types: &[ValueType::UtcOffset],
        default_value_type: ValueType::UtcOffset,
        multiple_values: false,
    },
    // 3.8.3.4.  Time Zone Offset To - Offset in use during time zone observance
    PropertySpec {
        name: KW_TZOFFSETTO,
        value_types: &[ValueType::UtcOffset],
        default_value_type: ValueType::UtcOffset,
        multiple_values: false,
    },
    // 3.8.3.5.  Time Zone URL - URI pointing to VTIMEZONE component location
    PropertySpec {
        name: KW_TZURL,
        value_types: &[ValueType::Uri],
        default_value_type: ValueType::Uri,
        multiple_values: false,
    },
    // 3.8.4.1.  Attendee - Defines an attendee within a calendar component
    PropertySpec {
        name: KW_ATTENDEE,
        value_types: &[ValueType::CalendarUserAddress],
        default_value_type: ValueType::CalendarUserAddress,
        multiple_values: true,
    },
    // 3.8.4.2.  Contact - Contact information or reference
    PropertySpec {
        name: KW_CONTACT,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: true,
    },
    // 3.8.4.3.  Organizer - Defines the organizer for a calendar component
    PropertySpec {
        name: KW_ORGANIZER,
        value_types: &[ValueType::CalendarUserAddress],
        default_value_type: ValueType::CalendarUserAddress,
        multiple_values: false,
    },
    // 3.8.4.4.  Recurrence ID - Identifies specific instance of recurring component
    PropertySpec {
        name: KW_RECURRENCE_ID,
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        multiple_values: false,
    },
    // 3.8.4.5.  Related To - Represents relationship between calendar components
    PropertySpec {
        name: KW_RELATED_TO,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: true,
    },
    // 3.8.4.6.  Uniform Resource Locator - URL associated with iCalendar object
    PropertySpec {
        name: KW_URL,
        value_types: &[ValueType::Uri],
        default_value_type: ValueType::Uri,
        multiple_values: true,
    },
    // 3.8.4.7.  Unique Identifier - Persistent, globally unique identifier
    PropertySpec {
        name: KW_UID,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: false,
    },
    // 3.8.5.1.  Exception Date-Times - List of DATE-TIME exceptions for recurring items
    PropertySpec {
        name: KW_EXDATE,
        value_types: &[ValueType::Date, ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        multiple_values: true,
    },
    // 3.8.5.2.  Recurrence Date-Times - List of DATE-TIME values for recurring items
    PropertySpec {
        name: KW_RDATE,
        value_types: &[ValueType::Date, ValueType::DateTime, ValueType::Period],
        default_value_type: ValueType::DateTime,
        multiple_values: true,
    },
    // 3.8.5.3.  Recurrence Rule - Rule or repeating pattern for recurring items
    PropertySpec {
        name: KW_RRULE,
        value_types: &[ValueType::RecurrenceRule],
        default_value_type: ValueType::RecurrenceRule,
        multiple_values: false,
    },
    // 3.8.6.1.  Action - Action to invoke when alarm is triggered
    PropertySpec {
        name: KW_ACTION,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: false,
    },
    // 3.8.6.2.  Repeat Count - Number of times alarm should repeat after initial trigger
    PropertySpec {
        name: KW_REPEAT,
        value_types: &[ValueType::Integer],
        default_value_type: ValueType::Integer,
        multiple_values: false,
    },
    // 3.8.6.3.  Trigger - When an alarm will trigger
    PropertySpec {
        name: KW_TRIGGER,
        value_types: &[ValueType::Duration, ValueType::DateTime],
        default_value_type: ValueType::Duration,
        multiple_values: false,
    },
    // 3.8.7.1.  Date-Time Created - Date/time calendar information was created
    PropertySpec {
        name: KW_CREATED,
        value_types: &[ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        multiple_values: false,
    },
    // 3.8.7.2.  Date-Time Stamp - Date/time instance was created or info was last revised
    PropertySpec {
        name: KW_DTSTAMP,
        value_types: &[ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        multiple_values: false,
    },
    // 3.8.7.3.  Last Modified - Date/time calendar component was last revised
    PropertySpec {
        name: KW_LAST_MODIFIED,
        value_types: &[ValueType::DateTime],
        default_value_type: ValueType::DateTime,
        multiple_values: false,
    },
    // 3.8.7.4.  Sequence Number - Revision sequence number within sequence of revisions
    PropertySpec {
        name: KW_SEQUENCE,
        value_types: &[ValueType::Integer],
        default_value_type: ValueType::Integer,
        multiple_values: false,
    },
    // 3.8.8.1.  IANA Properties
    // 3.8.8.2.  Non-Standard Properties
    // 3.8.8.3.  Request Status - Status code returned for a scheduling request
    PropertySpec {
        name: KW_REQUEST_STATUS,
        value_types: &[ValueType::Text],
        default_value_type: ValueType::Text,
        multiple_values: true,
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
