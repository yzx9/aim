// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Keywords defined in iCalendar RFC 5545.

#![allow(missing_docs)]

// 3.2.  Property Parameters
pub const KW_ALTREP: &str = "ALTREP";
pub const KW_CN: &str = "CN";
pub const KW_CUTYPE: &str = "CUTYPE";
pub const KW_DELEGATED_FROM: &str = "DELEGATED-FROM";
pub const KW_DELEGATED_TO: &str = "DELEGATED-TO";
pub const KW_DIR: &str = "DIR";
pub const KW_ENCODING: &str = "ENCODING";
pub const KW_FBTYPE: &str = "FBTYPE";
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

// 3.2.3.  Calendar User Type
pub const KW_CUTYPE_INDIVIDUAL: &str = "INDIVIDUAL";
pub const KW_CUTYPE_GROUP: &str = "GROUP";
pub const KW_CUTYPE_RESOURCE: &str = "RESOURCE";
pub const KW_CUTYPE_ROOM: &str = "ROOM";
pub const KW_CUTYPE_UNKNOWN: &str = "UNKNOWN";

// 3.2.7.  Inline Encoding
pub const KW_ENCODING_8BIT: &str = "8bit";
pub const KW_ENCODING_BASE64: &str = "base64";

// 3.2.9.  Free/Busy Time Type
pub const KW_FBTYPE_FREE: &str = "FREE";
pub const KW_FBTYPE_BUSY: &str = "BUSY";
pub const KW_FBTYPE_BUSY_UNAVAILABLE: &str = "BUSY-UNAVAILABLE";
pub const KW_FBTYPE_BUSY_TENTATIVE: &str = "BUSY-TENTATIVE";

// 3.2.12.  Participation Status
pub const KW_PARTSTAT_NEEDS_ACTION: &str = "NEEDS-ACTION";
pub const KW_PARTSTAT_ACCEPTED: &str = "ACCEPTED";
pub const KW_PARTSTAT_DECLINED: &str = "DECLINED";
pub const KW_PARTSTAT_TENTATIVE: &str = "TENTATIVE";
pub const KW_PARTSTAT_DELEGATED: &str = "DELEGATED";
pub const KW_PARTSTAT_COMPLETED: &str = "COMPLETED";
pub const KW_PARTSTAT_IN_PROCESS: &str = "IN-PROCESS";

// 3.2.13.  Recurrence Identifier Range
pub const KW_RANGE_THISANDFUTURE: &str = "THISANDFUTURE";

// 3.2.14.  Alarm Trigger Relationship
pub const KW_RELATED_START: &str = "START";
pub const KW_RELATED_END: &str = "END";

// 3.2.15.  Relationship Type
pub const KW_RELTYPE_PARENT: &str = "PARENT";
pub const KW_RELTYPE_CHILD: &str = "CHILD";
pub const KW_RELTYPE_SIBLING: &str = "SIBLING";

// 3.2.16.  Participation Role
pub const KW_ROLE_CHAIR: &str = "CHAIR";
pub const KW_ROLE_REQ_PARTICIPANT: &str = "REQ-PARTICIPANT";
pub const KW_ROLE_OPT_PARTICIPANT: &str = "OPT-PARTICIPANT";
pub const KW_ROLE_NON_PARTICIPANT: &str = "NON-PARTICIPANT";

// 3.2.17.  RSVP Expectation
pub const KW_RSVP_TRUE: &str = "TRUE";
pub const KW_RSVP_FALSE: &str = "FALSE";

// 3.3.  Property Value Data Types
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

// 3.4.  iCalendar Object
pub const KW_BEGIN: &str = "BEGIN";
pub const KW_END: &str = "END";

// 3.6.  Calendar Components
pub const KW_VCALENDAR: &str = "VCALENDAR";
pub const KW_VEVENT: &str = "VEVENT";
pub const KW_VTODO: &str = "VTODO";
pub const KW_VJOURNAL: &str = "VJOURNAL";
pub const KW_VFREEBUSY: &str = "VFREEBUSY";
pub const KW_VTIMEZONE: &str = "VTIMEZONE";
pub const KW_VALARM: &str = "VALARM";

// 3.6.5.  Time Zone Component Component Names
pub const KW_STANDARD: &str = "STANDARD";
pub const KW_DAYLIGHT: &str = "DAYLIGHT";

// 3.8.6.  Alarm Component Properties - ACTION values
pub const KW_ACTION_AUDIO: &str = "AUDIO";
pub const KW_ACTION_DISPLAY: &str = "DISPLAY";
pub const KW_ACTION_EMAIL: &str = "EMAIL";
pub const KW_ACTION_PROCEDURE: &str = "PROCEDURE";

// 3.7.  Calendar Properties
pub const KW_CALSCALE: &str = "CALSCALE";
pub const KW_METHOD: &str = "METHOD";
pub const KW_PRODID: &str = "PRODID";
pub const KW_VERSION: &str = "VERSION";

// 3.7.1. Calendar Scale values
pub const KW_CALSCALE_GREGORIAN: &str = "GREGORIAN";

// 3.7.2. Method values
pub const KW_METHOD_PUBLISH: &str = "PUBLISH";
pub const KW_METHOD_REQUEST: &str = "REQUEST";
pub const KW_METHOD_REPLY: &str = "REPLY";
pub const KW_METHOD_ADD: &str = "ADD";
pub const KW_METHOD_CANCEL: &str = "CANCEL";
pub const KW_METHOD_REFRESH: &str = "REFRESH";
pub const KW_METHOD_COUNTER: &str = "COUNTER";
pub const KW_METHOD_DECLINECOUNTER: &str = "DECLINECOUNTER";

// 3.7.4. Version values
pub const KW_VERSION_2_0: &str = "2.0";

// 3.8.1.  Descriptive Component Properties
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

// 3.8.1.11.  Status
// Status values for a "VEVENT"
pub const KW_EVENT_STATUS_TENTATIVE: &str = "TENTATIVE";
pub const KW_EVENT_STATUS_CONFIRMED: &str = "CONFIRMED";
pub const KW_EVENT_STATUS_CANCELLED: &str = "CANCELLED";
// Status values for a "VTODO"
pub const KW_TODO_STATUS_NEEDS_ACTION: &str = "NEEDS-ACTION";
pub const KW_TODO_STATUS_COMPLETED: &str = "COMPLETED";
pub const KW_TODO_STATUS_IN_PROCESS: &str = "IN-PROCESS";
pub const KW_TODO_STATUS_CANCELLED: &str = "CANCELLED";
// Status values for a "VJOURNAL"
pub const KW_JOURNAL_STATUS_DRAFT: &str = "DRAFT";
pub const KW_JOURNAL_STATUS_FINAL: &str = "FINAL";
pub const KW_JOURNAL_STATUS_CANCELLED: &str = "CANCELLED";

// 3.8.1.3.  Classification values
pub const KW_CLASS_PUBLIC: &str = "PUBLIC";
pub const KW_CLASS_PRIVATE: &str = "PRIVATE";
pub const KW_CLASS_CONFIDENTIAL: &str = "CONFIDENTIAL";

// 3.8.2.  Date and Time Component Properties
pub const KW_COMPLETED: &str = "COMPLETED";
pub const KW_DTSTART: &str = "DTSTART";
pub const KW_DTEND: &str = "DTEND";
pub const KW_DURATION: &str = "DURATION";
pub const KW_DUE: &str = "DUE";
pub const KW_FREEBUSY: &str = "FREEBUSY";
pub const KW_TRANSP: &str = "TRANSP";

// 3.8.2.7.  Time Transparency values
pub const KW_TRANSP_OPAQUE: &str = "OPAQUE";
pub const KW_TRANSP_TRANSPARENT: &str = "TRANSPARENT";

// 3.8.3.  Time Zone Component Properties
pub const KW_TZNAME: &str = "TZNAME";
pub const KW_TZOFFSETFROM: &str = "TZOFFSETFROM";
pub const KW_TZOFFSETTO: &str = "TZOFFSETTO";
pub const KW_TZURL: &str = "TZURL";

// 3.8.4.  Relationship Component Properties
pub const KW_ATTENDEE: &str = "ATTENDEE";
pub const KW_CONTACT: &str = "CONTACT";
pub const KW_ORGANIZER: &str = "ORGANIZER";
pub const KW_RECURRENCE_ID: &str = "RECURRENCE-ID";
pub const KW_RELATED_TO: &str = "RELATED-TO";
pub const KW_URL: &str = "URL";
pub const KW_UID: &str = "UID";

// 3.8.5.  Recurrence Component Properties
pub const KW_EXDATE: &str = "EXDATE";
pub const KW_RDATE: &str = "RDATE";
pub const KW_RRULE: &str = "RRULE";

// 3.8.6.  Alarm Component Properties
pub const KW_ACTION: &str = "ACTION";
pub const KW_REPEAT: &str = "REPEAT";
pub const KW_TRIGGER: &str = "TRIGGER";

// 3.8.7.  Change Management Component Properties
pub const KW_CREATED: &str = "CREATED";
pub const KW_DTSTAMP: &str = "DTSTAMP";
pub const KW_LAST_MODIFIED: &str = "LAST-MODIFIED";
pub const KW_SEQUENCE: &str = "SEQUENCE";

// 3.8.8.  Miscellaneous Component Properties
pub const KW_REQUEST_STATUS: &str = "REQUEST-STATUS";
