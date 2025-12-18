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
use crate::typed::parameter::ValueType;

#[derive(Debug, Clone)]
pub struct PropertySpec<'a> {
    pub name: &'a str,
    pub default_kind: ValueType,
    pub allowed_kinds: &'a [ValueType],
    pub multiple_valued: bool,
}

pub static PROPERTY_SPECS: &[PropertySpec] = &[
    // Sect 3.8.1.1. Attachment - Default URI, can be BINARY for inline content
    PropertySpec {
        name: KW_ATTACH,
        default_kind: ValueType::Uri,
        allowed_kinds: &[ValueType::Uri, ValueType::Binary],
        multiple_valued: true,
    },
    // Sect 3.8.1.2. Categories - Comma-separated text values for classification
    PropertySpec {
        name: KW_CATEGORIES,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: true,
    },
    // Sect 3.8.1.3. Classification - Access classification (PUBLIC/PRIVATE/CONFIDENTIAL)
    PropertySpec {
        name: KW_CLASS,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: false,
    },
    // Sect 3.8.1.4. Comment - Non-processing information, free-form text
    PropertySpec {
        name: KW_COMMENT,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: true,
    },
    // Sect 3.8.1.5. Description - Event/task description, single occurrence
    PropertySpec {
        name: KW_DESCRIPTION,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: false,
    },
    // Sect 3.8.1.6. Geographic Position - Latitude;longitude as FLOAT values
    PropertySpec {
        name: KW_GEO,
        default_kind: ValueType::Float,
        allowed_kinds: &[ValueType::Float],
        multiple_valued: false,
    },
    // Sect 3.8.1.7. Location - Event/task location
    PropertySpec {
        name: KW_LOCATION,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: false,
    },
    // Sect 3.8.1.8. Percent Complete - Task completion percentage (0-100)
    PropertySpec {
        name: KW_PERCENT_COMPLETE,
        default_kind: ValueType::Integer,
        allowed_kinds: &[ValueType::Integer],
        multiple_valued: false,
    },
    // Sect 3.8.1.9. Priority - Event/task priority level (0-9, 0=undefined, 1=highest)
    PropertySpec {
        name: KW_PRIORITY,
        default_kind: ValueType::Integer,
        allowed_kinds: &[ValueType::Integer],
        multiple_valued: false,
    },
    // Sect 3.8.1.10. Resources - Event/task resources, comma-separated text
    PropertySpec {
        name: KW_RESOURCES,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: true,
    },
    // Sect 3.8.1.11. Status - Event/task status (TENTATIVE/CONFIRMED/CANCELLED, etc.)
    PropertySpec {
        name: KW_STATUS,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: false,
    },
    // Sect 3.8.1.12. Summary - Event/task summary/title
    PropertySpec {
        name: KW_SUMMARY,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: false,
    },
    // Sect 3.8.2.1. Date-Time Completed - When a to-do was completed (VTODO only)
    PropertySpec {
        name: KW_COMPLETED,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::DateTime],
        multiple_valued: false,
    },
    // Sect 3.8.2.2. Date-Time End - When an event ends (DATE or DATE-TIME)
    PropertySpec {
        name: KW_DTEND,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::Date, ValueType::DateTime],
        multiple_valued: false,
    },
    // Sect 3.8.2.3. Date-Time Due - When a to-do is due (DATE or DATE-TIME)
    PropertySpec {
        name: KW_DUE,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::Date, ValueType::DateTime],
        multiple_valued: false,
    },
    // Sect 3.8.2.4. Date-Time Start - When an event starts (DATE or DATE-TIME)
    PropertySpec {
        name: KW_DTSTART,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::Date, ValueType::DateTime],
        multiple_valued: false,
    },
    // Sect 3.8.2.5. Duration - Event/task duration in iCalendar duration format
    PropertySpec {
        name: KW_DURATION,
        default_kind: ValueType::Duration,
        allowed_kinds: &[ValueType::Duration],
        multiple_valued: false,
    },
    // Sect 3.8.2.6. Free/Busy Time - One or more free/busy time intervals (VFREEBUSY only)
    PropertySpec {
        name: KW_FREEBUSY,
        default_kind: ValueType::Period,
        allowed_kinds: &[ValueType::Period],
        multiple_valued: true,
    },
    // Sect 3.8.2.7. Time Transparency - Event transparency to busy time searches
    PropertySpec {
        name: KW_TRANSP,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: false,
    },
    // Sect 3.8.3.1. Time Zone Identifier - Unique identifier for VTIMEZONE component
    PropertySpec {
        name: KW_TZID,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: false,
    },
    // Sect 3.8.3.2. Time Zone Name - Customary designation for time zone
    PropertySpec {
        name: KW_TZNAME,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: true,
    },
    // Sect 3.8.3.3. Time Zone Offset From - Offset prior to time zone observance
    PropertySpec {
        name: KW_TZOFFSETFROM,
        default_kind: ValueType::UtcOffset,
        allowed_kinds: &[ValueType::UtcOffset],
        multiple_valued: false,
    },
    // Sect 3.8.3.4. Time Zone Offset To - Offset in use during time zone observance
    PropertySpec {
        name: KW_TZOFFSETTO,
        default_kind: ValueType::UtcOffset,
        allowed_kinds: &[ValueType::UtcOffset],
        multiple_valued: false,
    },
    // Sect 3.8.3.5. Time Zone URL - URI pointing to VTIMEZONE component location
    PropertySpec {
        name: KW_TZURL,
        default_kind: ValueType::Uri,
        allowed_kinds: &[ValueType::Uri],
        multiple_valued: false,
    },
    // Sect 3.8.4.1. Attendee - Defines an attendee within a calendar component
    PropertySpec {
        name: KW_ATTENDEE,
        default_kind: ValueType::CalendarUserAddress,
        allowed_kinds: &[ValueType::CalendarUserAddress],
        multiple_valued: true,
    },
    // Sect 3.8.4.2. Contact - Contact information or reference
    PropertySpec {
        name: KW_CONTACT,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: true,
    },
    // Sect 3.8.4.3. Organizer - Defines the organizer for a calendar component
    PropertySpec {
        name: KW_ORGANIZER,
        default_kind: ValueType::CalendarUserAddress,
        allowed_kinds: &[ValueType::CalendarUserAddress],
        multiple_valued: false,
    },
    // Sect 3.8.4.4. Recurrence ID - Identifies specific instance of recurring component
    PropertySpec {
        name: KW_RECURRENCE_ID,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::Date, ValueType::DateTime],
        multiple_valued: false,
    },
    // Sect 3.8.4.5. Related To - Represents relationship between calendar components
    PropertySpec {
        name: KW_RELATED_TO,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: true,
    },
    // Sect 3.8.4.6. Uniform Resource Locator - URL associated with iCalendar object
    PropertySpec {
        name: KW_URL,
        default_kind: ValueType::Uri,
        allowed_kinds: &[ValueType::Uri],
        multiple_valued: true,
    },
    // Sect 3.8.4.7. Unique Identifier - Persistent, globally unique identifier
    PropertySpec {
        name: KW_UID,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: false,
    },
    // Sect 3.8.5.1. Exception Date-Times - List of DATE-TIME exceptions for recurring items
    PropertySpec {
        name: KW_EXDATE,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::Date, ValueType::DateTime],
        multiple_valued: true,
    },
    // Sect 3.8.5.2. Recurrence Date-Times - List of DATE-TIME values for recurring items
    PropertySpec {
        name: KW_RDATE,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::Date, ValueType::DateTime, ValueType::Period],
        multiple_valued: true,
    },
    // Sect 3.8.5.3. Recurrence Rule - Rule or repeating pattern for recurring items
    PropertySpec {
        name: KW_RRULE,
        default_kind: ValueType::RecurrenceRule,
        allowed_kinds: &[ValueType::RecurrenceRule],
        multiple_valued: false,
    },
    // Sect 3.8.6.1. Action - Action to invoke when alarm is triggered
    PropertySpec {
        name: KW_ACTION,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: false,
    },
    // Sect 3.8.6.2. Repeat Count - Number of times alarm should repeat after initial trigger
    PropertySpec {
        name: KW_REPEAT,
        default_kind: ValueType::Integer,
        allowed_kinds: &[ValueType::Integer],
        multiple_valued: false,
    },
    // Sect 3.8.6.3. Trigger - When an alarm will trigger
    PropertySpec {
        name: KW_TRIGGER,
        default_kind: ValueType::Duration,
        allowed_kinds: &[ValueType::Duration, ValueType::DateTime],
        multiple_valued: false,
    },
    // Sect 3.8.7.1. Date-Time Created - Date/time calendar information was created
    PropertySpec {
        name: KW_CREATED,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::DateTime],
        multiple_valued: false,
    },
    // Sect 3.8.7.2. Date-Time Stamp - Date/time instance was created or info was last revised
    PropertySpec {
        name: KW_DTSTAMP,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::DateTime],
        multiple_valued: false,
    },
    // Sect 3.8.7.3. Last Modified - Date/time calendar component was last revised
    PropertySpec {
        name: KW_LAST_MODIFIED,
        default_kind: ValueType::DateTime,
        allowed_kinds: &[ValueType::DateTime],
        multiple_valued: false,
    },
    // Sect 3.8.7.4. Sequence Number - Revision sequence number within sequence of revisions
    PropertySpec {
        name: KW_SEQUENCE,
        default_kind: ValueType::Integer,
        allowed_kinds: &[ValueType::Integer],
        multiple_valued: false,
    },
    // Sect 3.8.8.1. IANA Properties
    // Sect 3.8.8.2. Non-Standard Properties
    // Sect 3.8.8.3. Request Status - Status code returned for a scheduling request
    PropertySpec {
        name: KW_REQUEST_STATUS,
        default_kind: ValueType::Text,
        allowed_kinds: &[ValueType::Text],
        multiple_valued: true,
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
                spec.allowed_kinds.contains(&spec.default_kind),
                "Property {name}: default_kind should be in allowed_kinds"
            );
        }
    }
}
