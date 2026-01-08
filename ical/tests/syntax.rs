// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the iCalendar syntax parser
//!
//! These tests validate the syntax parser's behavior on realistic iCalendar content
//! and edge cases.

use aimcal_ical::lexer::{Token, lex_analysis};
use aimcal_ical::syntax::{SyntaxComponent, syntax_analysis};
use chumsky::error::Rich;

/// Test helper to parse iCalendar source and get components
fn parse_ical(src: &str) -> Result<Vec<SyntaxComponent<'_>>, Vec<Rich<'_, Token<'_>>>> {
    let token_stream = lex_analysis(src);
    syntax_analysis(src, token_stream)
}

/// Test helper to parse and get the first component
fn parse_first_component(src: &str) -> SyntaxComponent<'_> {
    let components = parse_ical(src).unwrap();
    components.into_iter().next().unwrap()
}

#[test]
fn syntax_empty_component() {
    let src = "\
BEGIN:VCALENDAR\r
END:VCALENDAR\r
";
    let components = parse_ical(src).unwrap();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].name, "VCALENDAR");
    assert!(components[0].properties.is_empty());
    assert!(components[0].children.is_empty());
}

#[test]
fn syntax_single_property() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties.len(), 1);
    assert_eq!(comp.properties[0].name.concatnate(), "VERSION");
    assert_eq!(comp.properties[0].value.concatnate(), "2.0");
    assert!(comp.properties[0].parameters.is_empty());
}

#[test]
fn syntax_multiple_properties() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
CALSCALE:GREGORIAN\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties.len(), 3);

    assert_eq!(comp.properties[0].name.concatnate(), "VERSION");
    assert_eq!(comp.properties[0].value.concatnate(), "2.0");

    assert_eq!(comp.properties[1].name.concatnate(), "PRODID");
    assert_eq!(
        comp.properties[1].value.concatnate(),
        "-//Example Corp.//CalDAV Client//EN"
    );

    assert_eq!(comp.properties[2].name.concatnate(), "CALSCALE");
    assert_eq!(comp.properties[2].value.concatnate(), "GREGORIAN");
}

#[test]
fn syntax_property_with_parameters() {
    let src = "\
BEGIN:VCALENDAR\r
DTSTART;TZID=America/New_York:20250101T090000\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties.len(), 1);

    let prop = &comp.properties[0];
    assert_eq!(prop.name.concatnate(), "DTSTART");
    assert_eq!(prop.parameters.len(), 1);

    let param = &prop.parameters[0];
    assert_eq!(param.name.concatnate(), "TZID");
    assert_eq!(param.values.len(), 1);
    assert_eq!(param.values[0].value.concatnate(), "America/New_York");
    assert!(!param.values[0].quoted);
}

#[test]
fn syntax_property_with_multiple_parameters() {
    let src = "\
BEGIN:VEVENT\r
ATTENDEE;RSVP=TRUE;CUTYPE=INDIVIDUAL;ROLE=REQ-PARTICIPANT:mailto:test@example.com\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties.len(), 1);

    let prop = &comp.properties[0];
    assert_eq!(prop.name.concatnate(), "ATTENDEE");
    assert_eq!(prop.parameters.len(), 3);

    assert_eq!(prop.parameters[0].name.concatnate(), "RSVP");
    assert_eq!(prop.parameters[0].values[0].value.concatnate(), "TRUE");

    assert_eq!(prop.parameters[1].name.concatnate(), "CUTYPE");
    assert_eq!(
        prop.parameters[1].values[0].value.concatnate(),
        "INDIVIDUAL"
    );

    assert_eq!(prop.parameters[2].name.concatnate(), "ROLE");
    assert_eq!(
        prop.parameters[2].values[0].value.concatnate(),
        "REQ-PARTICIPANT"
    );
}

#[test]
fn syntax_quoted_parameter_value() {
    let src = "\
BEGIN:VCALENDAR\r
X-CUSTOM;PARAM=\"value\":test\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties.len(), 1);

    let prop = &comp.properties[0];
    assert_eq!(prop.parameters.len(), 1);
    assert_eq!(prop.parameters[0].values[0].value.concatnate(), "value");
    assert!(prop.parameters[0].values[0].quoted);
}

#[test]
fn syntax_multi_value_parameter() {
    let src = "\
BEGIN:VCALENDAR\r
CATEGORIES;LANGUAGE=en:MEETING,TEAM,STRATEGY\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties.len(), 1);

    let prop = &comp.properties[0];
    assert_eq!(prop.name.concatnate(), "CATEGORIES");
    assert_eq!(prop.parameters.len(), 1);
    assert_eq!(prop.parameters[0].name.concatnate(), "LANGUAGE");
    assert_eq!(prop.parameters[0].values[0].value.concatnate(), "en");
}

#[test]
fn syntax_nested_components() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VEVENT\r
UID:123@example.com\r
DTSTART:20250615T133000Z\r
END:VEVENT\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.name, "VCALENDAR");
    assert_eq!(comp.properties.len(), 1);
    assert_eq!(comp.children.len(), 1);

    let event = &comp.children[0];
    assert_eq!(event.name, "VEVENT");
    assert_eq!(event.properties.len(), 2);
    assert_eq!(event.properties[0].name.concatnate(), "UID");
    assert_eq!(event.properties[1].name.concatnate(), "DTSTART");
}

#[test]
fn syntax_multiple_nested_components() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VEVENT\r
UID:1@example.com\r
END:VEVENT\r
BEGIN:VTODO\r
UID:2@example.com\r
END:VTODO\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.children.len(), 2);
    assert_eq!(comp.children[0].name, "VEVENT");
    assert_eq!(comp.children[1].name, "VTODO");
}

#[test]
fn syntax_deeply_nested_components() {
    let src = "\
BEGIN:VCALENDAR\r
BEGIN:VTIMEZONE\r
TZID:America/New_York\r
BEGIN:DAYLIGHT\r
DTSTART:20070311T020000\r
END:DAYLIGHT\r
END:VTIMEZONE\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.name, "VCALENDAR");
    assert_eq!(comp.children.len(), 1);

    let tz = &comp.children[0];
    assert_eq!(tz.name, "VTIMEZONE");
    assert_eq!(tz.children.len(), 1);

    let daylight = &tz.children[0];
    assert_eq!(daylight.name, "DAYLIGHT");
}

#[test]
fn syntax_property_with_escaped_chars() {
    let src = "\
BEGIN:VCALENDAR\r
DESCRIPTION:This is a test\\;And semicolon\\,And comma\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties.len(), 1);

    let prop = &comp.properties[0];
    assert_eq!(prop.name.concatnate(), "DESCRIPTION");
    // Escape sequences are preserved in the value - they'll be processed by the text parser
    assert!(prop.value.concatnate().contains('\\'));
}

#[test]
fn syntax_property_with_unicode() {
    let src = "\
BEGIN:VEVENT\r
SUMMARY:Team会议📅 Discuss Q1 goals\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "SUMMARY");
    assert_eq!(
        comp.properties[0].value.concatnate(),
        "Team会议📅 Discuss Q1 goals"
    );
}

#[test]
fn syntax_line_folding() {
    let src = "\
BEGIN:VCALENDAR\r
DESCRIPTION:This is a very long description that\r\n \
spans multiple lines and should be\r\n \
concatenated\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(
        comp.properties[0].value.concatnate(),
        "This is a very long description thatspans multiple lines and should beconcatenated"
    );
}

#[test]
fn syntax_rrule_property() {
    let src = "\
BEGIN:VEVENT\r
RRULE:FREQ=WEEKLY;BYDAY=MO,WE,FR;UNTIL=20251231T235959Z\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "RRULE");
    // RRULE value is a single value containing the rule syntax
    // (the typed analysis phase will parse it further)
    assert_eq!(
        comp.properties[0].value.concatnate(),
        "FREQ=WEEKLY;BYDAY=MO,WE,FR;UNTIL=20251231T235959Z"
    );
    assert!(comp.properties[0].parameters.is_empty());
}

#[test]
fn syntax_alarm_component() {
    let src = "\
BEGIN:VCALENDAR\r
BEGIN:VEVENT\r
UID:123@example.com\r
BEGIN:VALARM\r
TRIGGER:-PT15M\r
ACTION:DISPLAY\r
DESCRIPTION:Meeting\r
END:VALARM\r
END:VEVENT\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    let event = &comp.children[0];
    assert_eq!(event.name, "VEVENT");
    assert_eq!(event.children.len(), 1);

    let alarm = &event.children[0];
    assert_eq!(alarm.name, "VALARM");
    assert_eq!(alarm.properties.len(), 3);
}

#[test]
fn syntax_exdate_property() {
    let src = "\
BEGIN:VEVENT\r
EXDATE:20250101T090000,20250108T090000,20250115T090000\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "EXDATE");
    // EXDATE value includes commas - they're part of the value, not separators
    assert!(comp.properties[0].value.concatnate().contains(','));
}

#[test]
fn syntax_geo_property() {
    let src = "\
BEGIN:VEVENT\r
GEO:37.386013;-122.083932\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "GEO");
    assert_eq!(
        comp.properties[0].value.concatnate(),
        "37.386013;-122.083932"
    );
}

#[test]
fn syntax_url_property() {
    let src = "\
BEGIN:VEVENT\r
URL:http://example.com/event.html\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "URL");
    assert_eq!(
        comp.properties[0].value.concatnate(),
        "http://example.com/event.html"
    );
}

#[test]
fn syntax_organizer_property() {
    let src = "\
BEGIN:VEVENT\r
ORGANIZER;CN=John Doe:mailto:john.doe@example.com\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "ORGANIZER");
    assert_eq!(comp.properties[0].parameters.len(), 1);
    assert_eq!(comp.properties[0].parameters[0].name.concatnate(), "CN");
    assert_eq!(
        comp.properties[0].parameters[0].values[0]
            .value
            .concatnate(),
        "John Doe"
    );
    assert_eq!(
        comp.properties[0].value.concatnate(),
        "mailto:john.doe@example.com"
    );
}

#[test]
fn syntax_complete_minimal_icalendar() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345@example.com\r
DTSTAMP:20250101T120000Z\r
DTSTART:20250615T133000Z\r
DTEND:20250615T143000Z\r
SUMMARY:Team Meeting\r
END:VEVENT\r
END:VCALENDAR\r
";
    let components = parse_ical(src).unwrap();
    assert_eq!(components.len(), 1);

    let cal = &components[0];
    assert_eq!(cal.name, "VCALENDAR");
    assert_eq!(cal.properties.len(), 2);
    assert_eq!(cal.children.len(), 1);

    let event = &cal.children[0];
    assert_eq!(event.name, "VEVENT");
    assert_eq!(event.properties.len(), 5);
}

#[test]
fn syntax_mismatched_begin_end() {
    let src = "\
BEGIN:VCALENDAR\r
END:VEVENT\r
";
    let result = parse_ical(src);
    assert!(result.is_err());
}

#[test]
fn syntax_missing_end() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
";
    let result = parse_ical(src);
    assert!(result.is_err());
}

#[test]
fn syntax_property_without_colon() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION 2.0\r
END:VCALENDAR\r
";
    let result = parse_ical(src);
    // The parser should fail because there's no colon
    assert!(result.is_err());
}

#[test]
fn syntax_multiple_components_at_root() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
END:VCALENDAR\r
BEGIN:VCALENDAR\r
VERSION:2.0\r
END:VCALENDAR\r
";
    let components = parse_ical(src).unwrap();
    assert_eq!(components.len(), 2);
    assert_eq!(components[0].name, "VCALENDAR");
    assert_eq!(components[1].name, "VCALENDAR");
}

#[test]
fn syntax_property_name_case_sensitivity() {
    let src = "\
BEGIN:VCALENDAR\r
version:2.0\r
Summary:Test Event\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    // Property names preserve case but are compared case-insensitively
    assert_eq!(comp.properties[0].name.concatnate(), "version");
    assert_eq!(comp.properties[1].name.concatnate(), "Summary");
}

#[test]
fn syntax_vtimezone_component() {
    let src = "\
BEGIN:VCALENDAR\r
BEGIN:VTIMEZONE\r
TZID:America/New_York\r
BEGIN:STANDARD\r
DTSTART:20071104T020000\r
TZOFFSETFROM:-0400\r
TZOFFSETTO:-0500\r
END:STANDARD\r
BEGIN:DAYLIGHT\r
DTSTART:20070311T020000\r
TZOFFSETFROM:-0500\r
TZOFFSETTO:-0400\r
END:DAYLIGHT\r
END:VTIMEZONE\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.children.len(), 1);

    let tz = &comp.children[0];
    assert_eq!(tz.name, "VTIMEZONE");
    assert_eq!(tz.properties.len(), 1);
    assert_eq!(tz.children.len(), 2);

    assert_eq!(tz.children[0].name, "STANDARD");
    assert_eq!(tz.children[1].name, "DAYLIGHT");
}

#[test]
fn syntax_vjournal_component() {
    let src = "\
BEGIN:VCALENDAR\r
BEGIN:VJOURNAL\r
UID:123@example.com\r
DTSTART:20250615\r
SUMMARY:Daily Journal Entry\r
DESCRIPTION:Today was a productive day.\r
END:VJOURNAL\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.children[0].name, "VJOURNAL");
    assert_eq!(comp.children[0].properties.len(), 4);
}

#[test]
fn syntax_vfreebusy_component() {
    let src = "\
BEGIN:VCALENDAR\r
BEGIN:VFREEBUSY\r
UID:123@example.com\r
DTSTART:20250615T080000Z\r
DTEND:20250615T170000Z\r
FREEBUSY:20250615T120000Z/20250615T130000Z\r
END:VFREEBUSY\r
END:VCALENDAR\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.children[0].name, "VFREEBUSY");
}

#[test]
fn syntax_percent_complete_property() {
    let src = "\
BEGIN:VTODO\r
PERCENT-COMPLETE:75\r
END:VTODO\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "PERCENT-COMPLETE");
    assert_eq!(comp.properties[0].value.concatnate(), "75");
}

#[test]
fn syntax_priority_property() {
    let src = "\
BEGIN:VTODO\r
PRIORITY:5\r
END:VTODO\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "PRIORITY");
    assert_eq!(comp.properties[0].value.concatnate(), "5");
}

#[test]
fn syntax_status_property() {
    let src = "\
BEGIN:VTODO\r
STATUS:NEEDS-ACTION\r
END:VTODO\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "STATUS");
    assert_eq!(comp.properties[0].value.concatnate(), "NEEDS-ACTION");
}

#[test]
fn syntax_classification_property() {
    let src = "\
BEGIN:VEVENT\r
CLASS:PUBLIC\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "CLASS");
    assert_eq!(comp.properties[0].value.concatnate(), "PUBLIC");
}

#[test]
fn syntax_transparency_property() {
    let src = "\
BEGIN:VEVENT\r
TRANSP:OPAQUE\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "TRANSP");
    assert_eq!(comp.properties[0].value.concatnate(), "OPAQUE");
}

#[test]
fn syntax_created_last_modified_properties() {
    let src = "\
BEGIN:VEVENT\r
CREATED:20250101T000000Z\r
LAST-MODIFIED:20250102T120000Z\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "CREATED");
    assert_eq!(comp.properties[1].name.concatnate(), "LAST-MODIFIED");
}

#[test]
fn syntax_sequence_property() {
    let src = "\
BEGIN:VEVENT\r
SEQUENCE:2\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "SEQUENCE");
    assert_eq!(comp.properties[0].value.concatnate(), "2");
}

#[test]
fn syntax_location_property_with_escaped_comma() {
    let src = "\
BEGIN:VEVENT\r
LOCATION:Conference Room B\\, Building 1\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "LOCATION");
    assert!(comp.properties[0].value.concatnate().contains("\\,"));
}

#[test]
fn syntax_attendee_property_with_full_params() {
    let src = "\
BEGIN:VEVENT\r
ATTENDEE;RSVP=TRUE;CUTYPE=INDIVIDUAL;PARTSTAT=NEEDS-ACTION;\r\n \
ROLE=REQ-PARTICIPANT:mailto:test@example.com\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "ATTENDEE");

    let params = &comp.properties[0].parameters;
    assert!(params.iter().any(|p| p.name.concatnate() == "RSVP"));
    assert!(params.iter().any(|p| p.name.concatnate() == "CUTYPE"));
    assert!(params.iter().any(|p| p.name.concatnate() == "PARTSTAT"));
    assert!(params.iter().any(|p| p.name.concatnate() == "ROLE"));
}

#[test]
fn syntax_uid_property() {
    let src = "\
BEGIN:VEVENT\r
UID:1234567890@example.com\r
END:VEVENT\r
";
    let comp = parse_first_component(src);
    assert_eq!(comp.properties[0].name.concatnate(), "UID");
    assert_eq!(
        comp.properties[0].value.concatnate(),
        "1234567890@example.com"
    );
}

#[test]
fn syntax_complex_real_world_icalendar() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Mozilla.org/NONSGML Mozilla Calendar V1.1//EN\r
BEGIN:VTIMEZONE\r
TZID:America/New_York\r
BEGIN:DAYLIGHT\r
TZOFFSETFROM:-0500\r
TZOFFSETTO:-0400\r
DTSTART:19700308T020000\r
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU\r
TZNAME:EDT\r
END:DAYLIGHT\r
BEGIN:STANDARD\r
TZOFFSETFROM:-0400\r
TZOFFSETTO:-0500\r
DTSTART:19701101T020000\r
RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU\r
TZNAME:EST\r
END:STANDARD\r
END:VTIMEZONE\r
BEGIN:VEVENT\r
CREATED:20250101T120000Z\r
LAST-MODIFIED:20250102T120000Z\r
DTSTAMP:20250102T120000Z\r
UID:123456789-1234-1234-1234-123456789012@example.com\r
SUMMARY:Weekly Team Meeting\r
DTSTART;TZID=America/New_York:20250615T140000\r
DTEND;TZID=America/New_York:20250615T150000\r
RRULE:FREQ=WEEKLY;BYDAY=MO,WE,FR;UNTIL=20251231T235959Z\r
LOCATION:Conference Room B\\, Building 1\r
DESCRIPTION:Weekly team sync to discuss project progress\\nand blockers.\r
CLASS:PUBLIC\r
STATUS:CONFIRMED\r
TRANSP:OPAQUE\r
ORGANIZER;CN=John Doe:mailto:john.doe@example.com\r
ATTENDEE;RSVP=TRUE;PARTSTAT=NEEDS-ACTION;ROLE=REQ-PARTICIPANT;\r\n \
 CN=Jane Smith:mailto:jane.smith@example.com\r
BEGIN:VALARM\r
TRIGGER:-PT15M\r
ACTION:DISPLAY\r
DESCRIPTION:Meeting starting in 15 minutes\r
END:VALARM\r
END:VEVENT\r
END:VCALENDAR\r
";
    let components = parse_ical(src).unwrap();
    assert_eq!(components.len(), 1);

    let cal = &components[0];
    assert_eq!(cal.name, "VCALENDAR");
    assert!(cal.children.iter().any(|c| c.name == "VTIMEZONE"));
    assert!(cal.children.iter().any(|c| c.name == "VEVENT"));

    let event = cal.children.iter().find(|c| c.name == "VEVENT").unwrap();
    assert!(
        event
            .properties
            .iter()
            .any(|p| p.name.concatnate() == "SUMMARY")
    );
    assert!(
        event
            .properties
            .iter()
            .any(|p| p.name.concatnate() == "RRULE")
    );
    assert!(event.children.iter().any(|c| c.name == "VALARM"));
}
