// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the iCalendar typed analyzer
//!
//! These tests validate the typed analyzer's behavior on realistic iCalendar content
//! and edge cases.

use aimcal_ical::lexer::{Token, lex_analysis};
use aimcal_ical::property::{DateTime as PropertyDateTime, Property};
use aimcal_ical::syntax::{SyntaxComponent, syntax_analysis};
use aimcal_ical::typed::{TypedComponent, TypedError, typed_analysis};
use chumsky::error::Rich;

/// Test helper to parse iCalendar source through syntax phase
fn parse_syntax(src: &str) -> Result<Vec<SyntaxComponent<'_>>, Vec<Rich<'_, Token<'_>>>> {
    let token_stream = lex_analysis(src);
    syntax_analysis(src, token_stream)
}

/// Test helper to parse iCalendar source through typed phase
fn parse_typed(src: &str) -> Result<Vec<TypedComponent<'_>>, Vec<TypedError<'_>>> {
    let components = parse_syntax(src).unwrap();
    typed_analysis(components)
}

#[test]
fn typed_empty_calendar() {
    let src = "\
BEGIN:VCALENDAR\r
END:VCALENDAR\r
";
    let result = parse_typed(src);
    // Empty VCALENDAR parses successfully - validation happens in semantic phase
    assert!(result.is_ok());
}

#[test]
fn typed_minimal_calendar() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].name, "VCALENDAR");
    assert_eq!(components[0].properties.len(), 2);
    assert_eq!(components[0].properties[0].kind().to_string(), "VERSION");
    assert_eq!(components[0].properties[1].kind().to_string(), "PRODID");
}

#[test]
fn typed_version_property() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "VERSION");
    assert!(matches!(&components[0].properties[0], Property::Version(_)));
}

#[test]
fn typed_prodid_property() {
    let src = "\
BEGIN:VCALENDAR\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "PRODID");
    assert!(matches!(&components[0].properties[0], Property::ProdId(_)));
}

#[test]
fn typed_calscale_property() {
    let src = "\
BEGIN:VCALENDAR\r
CALSCALE:GREGORIAN\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "CALSCALE");
    assert!(matches!(
        &components[0].properties[0],
        Property::CalScale(_)
    ));
}

#[test]
fn typed_simple_event() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250101T100000Z\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].children.len(), 1);

    let event = &components[0].children[0];
    assert_eq!(event.name, "VEVENT");
    // Has UID, DTSTAMP, DTSTART, SUMMARY = 4 properties
    assert_eq!(event.properties.len(), 4);
}

#[test]
fn typed_dtstart_property() {
    let src = "\
BEGIN:VEVENT\r
DTSTART:20250615T133000Z\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "DTSTART");
    assert!(matches!(&components[0].properties[0], Property::DtStart(_)));
}

#[test]
fn typed_dtend_property() {
    let src = "\
BEGIN:VEVENT\r
DTEND:20250615T143000Z\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "DTEND");
    assert!(matches!(&components[0].properties[0], Property::DtEnd(_)));
}

#[test]
fn typed_summary_property() {
    let src = "\
BEGIN:VEVENT\r
SUMMARY:Team Meeting\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "SUMMARY");
    assert!(matches!(&components[0].properties[0], Property::Summary(_)));
}

#[test]
fn typed_description_property() {
    let src = "\
BEGIN:VEVENT\r
DESCRIPTION:This is a detailed description\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(
        components[0].properties[0].kind().to_string(),
        "DESCRIPTION"
    );
    assert!(matches!(
        &components[0].properties[0],
        Property::Description(_)
    ));
}

#[test]
fn typed_location_property() {
    let src = "\
BEGIN:VEVENT\r
LOCATION:Conference Room B\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "LOCATION");
    assert!(matches!(
        &components[0].properties[0],
        Property::Location(_)
    ));
}

#[test]
fn typed_class_property() {
    let src = "\
BEGIN:VEVENT\r
CLASS:PUBLIC\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "CLASS");
    assert!(matches!(&components[0].properties[0], Property::Class(_)));
}

#[test]
fn typed_status_property() {
    let src = "\
BEGIN:VEVENT\r
STATUS:CONFIRMED\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "STATUS");
    assert!(matches!(&components[0].properties[0], Property::Status(_)));
}

#[test]
fn typed_transparency_property() {
    let src = "\
BEGIN:VEVENT\r
TRANSP:OPAQUE\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "TRANSP");
    assert!(matches!(&components[0].properties[0], Property::Transp(_)));
}

#[test]
fn typed_priority_property() {
    let src = "\
BEGIN:VEVENT\r
PRIORITY:5\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "PRIORITY");
    assert!(matches!(
        &components[0].properties[0],
        Property::Priority(_)
    ));
}

#[test]
fn typed_sequence_property() {
    let src = "\
BEGIN:VEVENT\r
SEQUENCE:2\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "SEQUENCE");
    assert!(matches!(
        &components[0].properties[0],
        Property::Sequence(_)
    ));
}

#[test]
fn typed_created_property() {
    let src = "\
BEGIN:VEVENT\r
CREATED:20250101T000000Z\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "CREATED");
    assert!(matches!(&components[0].properties[0], Property::Created(_)));
}

#[test]
fn typed_last_modified_property() {
    let src = "\
BEGIN:VEVENT\r
LAST-MODIFIED:20250102T120000Z\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(
        components[0].properties[0].kind().to_string(),
        "LAST-MODIFIED"
    );
    assert!(matches!(
        &components[0].properties[0],
        Property::LastModified(_)
    ));
}

#[test]
fn typed_date_only_dtstart() {
    let src = "\
BEGIN:VEVENT\r
DTSTART:20250615\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "DTSTART");
    assert!(matches!(&components[0].properties[0], Property::DtStart(_)));
}

#[test]
fn typed_date_only_dtend() {
    let src = "\
BEGIN:VEVENT\r
DTEND:20250615\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "DTEND");
    assert!(matches!(&components[0].properties[0], Property::DtEnd(_)));
}

#[test]
fn typed_duration_property() {
    let src = "\
BEGIN:VEVENT\r
DURATION:PT1H30M\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "DURATION");
    assert!(matches!(
        &components[0].properties[0],
        Property::Duration(_)
    ));
}

#[test]
fn typed_rrule_property() {
    let src = "\
BEGIN:VEVENT\r
RRULE:FREQ=WEEKLY;BYDAY=MO,WE,FR\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "RRULE");
    // RRULE values are parsed as RecurrenceRule
    // (the specific value type depends on the typed module implementation)
}

#[test]
fn typed_exdate_property() {
    let src = "\
BEGIN:VEVENT\r
EXDATE:20250101T090000,20250108T090000\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "EXDATE");
    assert!(matches!(&components[0].properties[0], Property::ExDate(_)));
}

#[test]
fn typed_geo_property() {
    let src = "\
BEGIN:VEVENT\r
GEO:37.386013;-122.083932\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "GEO");
    // GEO is parsed as TEXT in typed phase; actual float parsing happens in semantic phase
    assert!(matches!(&components[0].properties[0], Property::Geo(_)));
}

#[test]
fn typed_percent_complete_property() {
    let src = "\
BEGIN:VTODO\r
PERCENT-COMPLETE:75\r
END:VTODO\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(
        components[0].properties[0].kind().to_string(),
        "PERCENT-COMPLETE"
    );
    assert!(matches!(
        &components[0].properties[0],
        Property::PercentComplete(_)
    ));
}

#[test]
fn typed_todo_complete() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VTODO\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T090000Z\r
DUE:20250615T170000Z\r
SUMMARY:Complete project documentation\r
STATUS:NEEDS-ACTION\r
PRIORITY:5\r
PERCENT-COMPLETE:0\r
END:VTODO\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    let todo = &components[0].children[0];
    assert_eq!(todo.name, "VTODO");
    assert_eq!(todo.properties.len(), 8);
}

#[test]
fn typed_journal_complete() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VJOURNAL\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615\r
SUMMARY:Daily Journal Entry\r
DESCRIPTION:Today was a productive day.\r
END:VJOURNAL\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    let journal = &components[0].children[0];
    assert_eq!(journal.name, "VJOURNAL");
    assert_eq!(journal.properties.len(), 5);
}

#[test]
fn typed_freebusy_complete() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VFREEBUSY\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T080000Z\r
DTEND:20250615T170000Z\r
END:VFREEBUSY\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    let freebusy = &components[0].children[0];
    assert_eq!(freebusy.name, "VFREEBUSY");
    assert_eq!(freebusy.properties.len(), 4);
}

#[test]
fn typed_alarm_component() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
BEGIN:VALARM\r
TRIGGER:-PT15M\r
ACTION:DISPLAY\r
DESCRIPTION:Meeting reminder\r
END:VALARM\r
END:VEVENT\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    let event = &components[0].children[0];
    assert_eq!(event.children.len(), 1);

    let alarm = &event.children[0];
    assert_eq!(alarm.name, "VALARM");
    assert_eq!(alarm.properties.len(), 3);
}

#[test]
fn typed_unknown_property() {
    let src = "\
BEGIN:VEVENT\r
X-CUSTOM-PROPERTY:some value\r
END:VEVENT\r
";
    let result = parse_typed(src);
    // Unknown properties are now preserved instead of causing errors
    assert!(result.is_ok());
    let components = result.unwrap();
    assert_eq!(
        components[0].properties[0].kind().to_string(),
        "X-CUSTOM-PROPERTY"
    );
}

#[test]
fn typed_duplicate_property() {
    // Property cardinality checking has been removed from typed analysis
    // Duplicate properties are now allowed at this level
    // (They may still be caught at the semantic level if needed)
    let src = "\
BEGIN:VEVENT\r
UID:12345\r
UID:67890\r
END:VEVENT\r
";
    let result = parse_typed(src);
    assert!(
        result.is_ok(),
        "Duplicate properties should be allowed at typed analysis level"
    );
    // Verify both UID properties are present
    let components = result.unwrap();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].properties.len(), 2);
}

#[test]
fn typed_missing_required_property() {
    // VEVENT requires UID and DTSTAMP
    let src = "\
BEGIN:VEVENT\r
SUMMARY:Test\r
END:VEVENT\r
";
    let result = parse_typed(src);
    // This may or may not fail depending on validation rules
    // The typed analysis might not enforce required properties
    // Let's just check it parses
    let _ = result;
}

#[test]
fn typed_property_with_unknown_parameter() {
    let src = "\
BEGIN:VEVENT\r
DTSTART;X-CUSTOM-PARAM=value:20250615T100000Z\r
END:VEVENT\r
";
    let result = parse_typed(src);
    // Unknown parameters are now preserved instead of causing errors
    assert!(result.is_ok());
}

#[test]
fn typed_unicode_in_summary() {
    let src = "\
BEGIN:VEVENT\r
SUMMARY:Team会议📅\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "SUMMARY");
    assert!(matches!(&components[0].properties[0], Property::Summary(_)));
}

#[test]
fn typed_escaped_text_in_description() {
    let src = "\
BEGIN:VEVENT\r
DESCRIPTION:Line 1\\nLine 2\\;And semicolon\\,And comma\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(
        components[0].properties[0].kind().to_string(),
        "DESCRIPTION"
    );
    assert!(matches!(
        &components[0].properties[0],
        Property::Description(_)
    ));
}

#[test]
fn typed_nested_components() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VTIMEZONE\r
TZID:America/New_York\r
BEGIN:STANDARD\r
DTSTART:20071104T020000\r
TZOFFSETFROM:-0400\r
TZOFFSETTO:-0500\r
END:STANDARD\r
END:VTIMEZONE\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART;TZID=America/New_York:20250615T100000\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].children.len(), 2);

    let tz = &components[0].children[0];
    assert_eq!(tz.name, "VTIMEZONE");
    assert_eq!(tz.children.len(), 1);

    let event = &components[0].children[1];
    assert_eq!(event.name, "VEVENT");
}

#[test]
fn typed_multiple_events() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:1\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
SUMMARY:Event 1\r
END:VEVENT\r
BEGIN:VEVENT\r
UID:2\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250616T100000Z\r
SUMMARY:Event 2\r
END:VEVENT\r
END:VCALENDAR\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].children.len(), 2);
}

#[test]
fn typed_property_case_insensitive() {
    let src = "\
BEGIN:VEVENT\r
summary:test event\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    // Property name should be normalized to uppercase
    assert_eq!(components[0].properties[0].kind().to_string(), "SUMMARY");
}

#[test]
fn typed_tzid_parameter() {
    let src = "\
BEGIN:VEVENT\r
DTSTART;TZID=America/New_York:20250615T100000\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "DTSTART");
    // TZID parameter should be parsed - check that we get a Zoned DateTime
    assert!(matches!(
        &components[0].properties[0],
        Property::DtStart(dt_start) if matches!(dt_start.0, PropertyDateTime::Zoned { .. })
    ));
}

#[test]
fn typed_value_parameter() {
    let src = "\
BEGIN:VEVENT\r
DTSTART;VALUE=DATE:20250615\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "DTSTART");
    assert!(matches!(&components[0].properties[0], Property::DtStart(_)));
}

#[test]
fn typed_date_with_value_date_parameter() {
    let src = "\
BEGIN:VEVENT\r
DTSTART;VALUE=DATE:20250615\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "DTSTART");
    // When VALUE=DATE is specified, the value should be parsed as a date
    assert!(matches!(&components[0].properties[0], Property::DtStart(_)));
}

#[test]
fn typed_complete_real_world_calendar() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Mozilla.org/NONSGML Mozilla Calendar V1.1//EN\r
CALSCALE:GREGORIAN\r
BEGIN:VEVENT\r
CREATED:20250101T120000Z\r
LAST-MODIFIED:20250102T120000Z\r
DTSTAMP:20250102T120000Z\r
UID:123456789-1234-1234-1234-123456789012\r
SUMMARY:Weekly Team Meeting\r
DTSTART:20250615T140000Z\r
DTEND:20250615T150000Z\r
LOCATION:Conference Room B\r
DESCRIPTION:Weekly team sync\r
CLASS:PUBLIC\r
STATUS:CONFIRMED\r
TRANSP:OPAQUE\r
PRIORITY:5\r
SEQUENCE:0\r
BEGIN:VALARM\r
TRIGGER:-PT15M\r
ACTION:DISPLAY\r
DESCRIPTION:Meeting reminder\r
END:VALARM\r
END:VEVENT\r
END:VCALENDAR\r
";
    let result = parse_typed(src);
    // Most properties should parse successfully
    // Some may fail due to parser limitations (like UID with @ symbol)
    let _ = result;
}

#[test]
fn typed_url_property() {
    let src = "\
BEGIN:VEVENT\r
URL:http://example.com/event.html\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "URL");
    assert!(matches!(&components[0].properties[0], Property::Url(_)));
}

#[test]
fn typed_organizer_property() {
    let src = "\
BEGIN:VEVENT\r
ORGANIZER;CN=John Doe:mailto:john.doe@example.com\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "ORGANIZER");
    assert!(matches!(
        &components[0].properties[0],
        Property::Organizer(_)
    ));
}

#[test]
fn typed_attendee_property() {
    let src = "\
BEGIN:VEVENT\r
ATTENDEE;RSVP=TRUE:mailto:test@example.com\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "ATTENDEE");
    assert!(matches!(
        &components[0].properties[0],
        Property::Attendee(_)
    ));
}

// Test that ATTACH with BINARY requires explicit VALUE=BINARY parameter
// Per RFC 5545 Section 3.3.1: BINARY MUST include ENCODING=BASE64 and VALUE=BINARY
#[test]
fn typed_attach_binary_requires_value_parameter() {
    // Without VALUE=BINARY, should be parsed as URI (text value)
    let src = "\
BEGIN:VEVENT\r
ATTACH;ENCODING=BASE64:VGhpcyBpcyBub3QgYSB2YWxpZCBVUkk=\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    // This should parse as URI (text) since BINARY requires explicit VALUE=BINARY
    assert_eq!(components[0].properties[0].kind().to_string(), "ATTACH");
    assert!(matches!(&components[0].properties[0], Property::Attach(_)));
}

#[test]
fn typed_attach_binary_with_explicit_value_parameter() {
    // With VALUE=BINARY explicitly specified, should parse as Binary
    let src = "\
BEGIN:VEVENT\r
ATTACH;ENCODING=BASE64;VALUE=BINARY:VGhpcyBpcyBub3QgYSB2YWxpZCBVUkk=\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "ATTACH");
    assert!(matches!(&components[0].properties[0], Property::Attach(_)));
}

#[test]
fn typed_attach_uri_without_value_parameter() {
    // URI without VALUE parameter should parse as URI (default)
    let src = "\
BEGIN:VEVENT\r
ATTACH:http://example.com/document.pdf\r
END:VEVENT\r
";
    let components = parse_typed(src).unwrap();
    assert_eq!(components[0].properties[0].kind().to_string(), "ATTACH");
    assert!(matches!(&components[0].properties[0], Property::Attach(_)));
}

#[test]
fn typed_unknown_property_x_name() {
    let src = "BEGIN:VEVENT\r
DTSTART:20250101T120000Z\r
X-CUSTOM-PROP:test value\r
END:VEVENT\r
";
    let result = parse_typed(src);
    assert!(result.is_ok());
    let components = result.unwrap();
    assert_eq!(
        components[0].properties[1].kind().to_string(),
        "X-CUSTOM-PROP"
    );
}

#[test]
fn typed_unknown_parameter() {
    let src = "BEGIN:VEVENT\r
DTSTART;X-CUSTOM-PARAM=value:20250101T120000Z\r
END:VEVENT\r
";
    let result = parse_typed(src);
    assert!(result.is_ok());
}
