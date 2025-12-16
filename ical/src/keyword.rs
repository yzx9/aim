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

// Section 3.2 - Property Parameters
pub const KW_ALTREP: &str = "ALTREP";
pub const KW_CN: &str = "CN";
pub const KW_CUTYPE: &str = "CUTYPE";
pub const KW_DELEGATED_FROM: &str = "DELEGATED-FROM";
pub const KW_DELEGATED_TO: &str = "DELEGATED-TO";
pub const KW_DIR: &str = "DIR";
pub const KW_ENCODING: &str = "ENCODING";
pub const KW_ENCODING_8BIT: &str = "8bit";
pub const KW_ENCODING_BASE64: &str = "base64";
pub const KW_FBTYPE: &str = "FBTYPE";
pub const KW_FBTYPE_FREE: &str = "FREE";
pub const KW_FBTYPE_BUSY: &str = "BUSY";
pub const KW_FBTYPE_BUSY_UNAVAILABLE: &str = "BUSY-UNAVAILABLE";
pub const KW_FBTYPE_BUSY_TENTATIVE: &str = "BUSY-TENTATIVE";
pub const KW_LANGUAGE: &str = "LANGUAGE";
pub const KW_MEMBER: &str = "MEMBER";
pub const KW_PARTSTAT: &str = "PARTSTAT";
pub const KW_RANGE: &str = "RANGE";
pub const KW_FMTTYPE: &str = "FMTTYPE";
pub const KW_RELATED: &str = "RELATED";
pub const KW_RELTYPE: &str = "RELTYPE";
pub const KW_ROLE: &str = "ROLE";
pub const KW_RSVP: &str = "RSVP";
pub const KW_SENT_BY: &str = "SENT-BY";
pub const KW_TZID: &str = "TZID";
pub const KW_VALUE: &str = "VALUE";

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
