// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Keywords defined in iCalendar RFC 5545.

pub const KW_BEGIN: &str = "BEGIN";
pub const KW_END: &str = "END";

// pub const KW_VCALENDAR: &str = "VCALENDAR";
// pub const KW_VEVENT: &str = "VEVENT";
// pub const KW_VTODO: &str = "VTODO";
// pub const KW_VJOURNAL: &str = "VJOURNAL";
// pub const KW_VFREEBUSY: &str = "VFREEBUSY";
// pub const KW_VTIMEZONE: &str = "VTIMEZONE";
// pub const KW_VALARM: &str = "VALARM";

// Section 3.3 - Property Value Data Types
pub const KW_BINARY: &str = "BINARY";
pub const KW_BOOLEAN: &str = "BOOLEAN";
pub const KW_CAL_ADDRESS: &str = "CAL-ADDRESS";
pub const KW_DATE: &str = "DATE";
pub const KW_DATETIME: &str = "DATE-TIME";
pub const KW_FLOAT: &str = "FLOAT";
pub const KW_INTEGER: &str = "INTEGER";
pub const KW_PERIOD: &str = "PERIOD";
pub const KW_TEXT: &str = "TEXT";
pub const KW_TIME: &str = "TIME";
pub const KW_URI: &str = "URI";
pub const KW_UTC_OFFSET: &str = "UTC-OFFSET";

// Section 3.8.1 - Descriptive Component Properties
pub const KW_ATTACH: &str = "ATTACH";
pub const KW_CATEGORIES: &str = "CATEGORIES";
pub const KW_CLASS: &str = "CLASS";
pub const KW_COMMENT: &str = "COMMENT";
pub const KW_DESCRIPTION: &str = "DESCRIPTION";
pub const KW_GEO: &str = "GEO";
pub const KW_LOCATION: &str = "LOCATION";
pub const KW_PERCENT_COMPLETE: &str = "PERCENT-COMPLETE";
pub const KW_PRIORITY: &str = "PRIORITY";
pub const KW_RESOURCES: &str = "RESOURCES";
pub const KW_STATUS: &str = "STATUS";
pub const KW_SUMMARY: &str = "SUMMARY";

// Section 3.8.2 - Date and Time Component Properties
pub const KW_COMPLETED: &str = "COMPLETED";
pub const KW_DTSTART: &str = "DTSTART";
pub const KW_DTEND: &str = "DTEND";
pub const KW_DURATION: &str = "DURATION";
pub const KW_DUE: &str = "DUE";
pub const KW_FREEBUSY: &str = "FREEBUSY";
pub const KW_TRANSP: &str = "TRANSP";

// Section 3.8.3 - Time Zone Component Properties
pub const KW_TZID: &str = "TZID";
pub const KW_TZNAME: &str = "TZNAME";
pub const KW_TZOFFSETFROM: &str = "TZOFFSETFROM";
pub const KW_TZOFFSETTO: &str = "TZOFFSETTO";
pub const KW_TZURL: &str = "TZURL";

// Section 3.8.4 - Relationship Component Properties
pub const KW_ATTENDEE: &str = "ATTENDEE";
pub const KW_CONTACT: &str = "CONTACT";
pub const KW_ORGANIZER: &str = "ORGANIZER";
pub const KW_RECURRENCE_ID: &str = "RECURRENCE-ID";
pub const KW_RELATED_TO: &str = "RELATED-TO";
pub const KW_URL: &str = "URL";
pub const KW_UID: &str = "UID";

// Section 3.8.5 - Recurrence Component Properties
pub const KW_EXDATE: &str = "EXDATE";
pub const KW_RDATE: &str = "RDATE";
pub const KW_RRULE: &str = "RRULE";

// Section 3.8.6 - Alarm Component Properties
pub const KW_ACTION: &str = "ACTION";
pub const KW_REPEAT: &str = "REPEAT";
pub const KW_TRIGGER: &str = "TRIGGER";

// Section 3.8.7 - Change Management Component Properties
pub const KW_CREATED: &str = "CREATED";
pub const KW_DTSTAMP: &str = "DTSTAMP";
pub const KW_LAST_MODIFIED: &str = "LAST-MODIFIED";
pub const KW_SEQUENCE: &str = "SEQUENCE";

// Section 3.8.8 - Miscellaneous Component Properties
pub const KW_REQUEST_STATUS: &str = "REQUEST-STATUS";
