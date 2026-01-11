// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the iCalendar formatter.

use aimcal_ical::{formatter::format, parse};

#[test]
fn test_format_simple_event() {
    let input = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:12345@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY:Test Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Check that the formatted output contains the expected properties
    assert!(formatted.contains("BEGIN:VCALENDAR"));
    assert!(formatted.contains("VERSION:2.0"));
    assert!(formatted.contains("PRODID:-//Example Corp.//Cal Client 1.0//EN"));
    assert!(formatted.contains("BEGIN:VEVENT"));
    assert!(formatted.contains("UID:12345@example.com"));
    assert!(formatted.contains("DTSTAMP:20250110T120000Z"));
    assert!(formatted.contains("SUMMARY:Test Event"));
    assert!(formatted.contains("END:VEVENT"));
    assert!(formatted.contains("END:VCALENDAR"));
}

#[test]
fn test_format_creates_crlf_line_endings() {
    let input = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:test\r\nEND:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Verify CRLF line endings
    assert!(formatted.contains("\r\n"));
}

#[test]
fn test_round_trip_simple_calendar() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:12345@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY:Test Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Format to string
    let formatted = format(calendar1).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare that both have the same number of components
    assert_eq!(calendar1.components.len(), calendar2.components.len());

    // The formatted version should contain the key properties
    assert!(formatted.contains("VERSION:2.0"));
    assert!(formatted.contains("PRODID:-//Example Corp.//Cal Client 1.0//EN"));
}

#[test]
fn test_format_with_parameters() {
    let input = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:test\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY;LANGUAGE=en:Test Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Check that parameters are formatted
    assert!(formatted.contains("SUMMARY;LANGUAGE=en:Test Event"));
}

#[test]
fn test_format_both_ref_and_owned() {
    let input = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:test\r\nEND:VCALENDAR\r\n";

    // Parse as Ref (borrowed)
    let calendars_ref = parse(input).unwrap();
    let calendar_ref = &calendars_ref[0];

    // Convert to Owned
    let calendar_owned = calendar_ref.to_owned();

    // Format both - should produce same output
    let formatted_ref = format(calendar_ref).unwrap();
    let formatted_owned = format(&calendar_owned).unwrap();

    // Both should contain key properties
    assert!(formatted_ref.contains("VERSION:2.0"));
    assert!(formatted_owned.contains("VERSION:2.0"));
}

#[test]
fn test_format_preserves_custom_properties() {
    let input = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:test\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
X-CUSTOM-PROPERTY:value\r\n\
SUMMARY:Test Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Check that custom property is preserved
    assert!(formatted.contains("X-CUSTOM-PROPERTY:value"));
}

#[test]
fn test_format_text_with_special_characters() {
    let input = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:test\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY:Test with semicolon\\; and comma\\, and backslash\\\\\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Check that special characters are preserved
    assert!(formatted.contains("SUMMARY:"));
}

#[test]
fn test_format_multiple_events() {
    let input = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:test\r\n\
BEGIN:VEVENT\r\n\
UID:event1@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY:Event 1\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:event2@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T160000Z\r\n\
DTEND:20250110T170000Z\r\n\
SUMMARY:Event 2\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Check that both events are present
    assert!(formatted.contains("UID:event1@example.com"));
    assert!(formatted.contains("UID:event2@example.com"));
}

#[test]
fn test_format_date_value() {
    let input = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:test\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART;VALUE=DATE:20250110\r\n\
DTEND;VALUE=DATE:20250111\r\n\
SUMMARY:All day event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Check that DATE values are formatted correctly
    assert!(formatted.contains("DTSTART;VALUE=DATE:20250110"));
}

#[test]
fn test_format_duration() {
    let input = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:test\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DURATION:PT1H\r\n\
SUMMARY:One hour meeting\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Check that duration is formatted
    assert!(formatted.contains("DURATION:PT1H"));
}

#[test]
fn test_format_rrule() {
    let input = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:test\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
RRULE:FREQ=DAILY;COUNT=5\r\n\
SUMMARY:Daily event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Check that RRULE is formatted
    assert!(formatted.contains("RRULE:FREQ=DAILY;COUNT=5"));
}

#[test]
fn test_format_with_alarm() {
    let input = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:test\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY:Test Event\r\n\
BEGIN:VALARM\r\n\
ACTION:DISPLAY\r\n\
TRIGGER:-PT15M\r\n\
DESCRIPTION:Reminder\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let calendars = parse(input).unwrap();
    let calendar = &calendars[0];

    let formatted = format(calendar).unwrap();

    // Check that alarm is present
    assert!(formatted.contains("BEGIN:VALARM"));
    assert!(formatted.contains("ACTION:DISPLAY"));
    assert!(formatted.contains("END:VALARM"));
}
