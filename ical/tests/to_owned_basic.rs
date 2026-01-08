// SPDX-License-Identifier: Apache-2.0

//! Basic functional tests for to_owned() methods

use aimcal_ical::{CalendarComponent, parse};

#[test]
fn test_event_to_owned_converts_successfully() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Test//Test//EN\r
BEGIN:VEVENT\r
UID:12345@example.com\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250101T100000Z\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";

    let calendars = parse(src).unwrap();
    let event_ref = match &calendars[0].components[0] {
        CalendarComponent::Event(e) => e,
        _ => panic!("Expected event"),
    };

    // Conversion should succeed without panicking
    let _event_owned = event_ref.to_owned();
}

#[test]
fn test_todo_to_owned_converts_successfully() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Test//Test//EN\r
BEGIN:VTODO\r
UID:todo-123@example.com\r
DTSTAMP:20250108T120000Z\r
DTSTART:20250108T090000Z\r
DUE:20250108T170000Z\r
SUMMARY:Complete task\r
STATUS:NEEDS-ACTION\r
END:VTODO\r
END:VCALENDAR\r
";

    let calendars = parse(src).unwrap();
    let todo_ref = match &calendars[0].components[0] {
        CalendarComponent::Todo(t) => t,
        _ => panic!("Expected todo"),
    };

    // Conversion should succeed
    let _todo_owned = todo_ref.to_owned();
}

#[test]
fn test_icalendar_to_owned_converts_successfully() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Test Corp//Test Calendar//EN\r
BEGIN:VEVENT\r
UID:event1@example.com\r
DTSTAMP:20250108T120000Z\r
DTSTART:20250108T100000Z\r
SUMMARY:Event 1\r
END:VEVENT\r
END:VCALENDAR\r
";

    let calendars = parse(src).unwrap();
    let cal_ref = &calendars[0];

    // Conversion should succeed
    let cal_owned = cal_ref.to_owned();

    // Verify structure is preserved
    assert_eq!(cal_ref.components.len(), cal_owned.components.len());
}

#[test]
fn test_multiple_components_to_owned() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Test//Test//EN\r
BEGIN:VEVENT\r
UID:event1@example.com\r
DTSTAMP:20250108T120000Z\r
DTSTART:20250108T100000Z\r
SUMMARY:Event 1\r
END:VEVENT\r
BEGIN:VTODO\r
UID:todo1@example.com\r
DTSTAMP:20250108T120000Z\r
DTSTART:20250108T090000Z\r
SUMMARY:Task 1\r
END:VTODO\r
END:VCALENDAR\r
";

    let calendars = parse(src).unwrap();
    let cal_ref = &calendars[0];
    let cal_owned = cal_ref.to_owned();

    // All components should be convertible
    assert_eq!(cal_ref.components.len(), 2);
    assert_eq!(cal_owned.components.len(), 2);
}

#[test]
fn test_event_with_optional_fields_to_owned() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Test//Test//EN\r
BEGIN:VEVENT\r
UID:67890@example.com\r
DTSTAMP:20250108T120000Z\r
DTSTART:20250108T100000Z\r
DTEND:20250108T110000Z\r
SUMMARY:Important Meeting\r
DESCRIPTION:Discuss project\r
LOCATION:Conference Room A\r
STATUS:CONFIRMED\r
END:VEVENT\r
END:VCALENDAR\r
";

    let calendars = parse(src).unwrap();
    let event_ref = match &calendars[0].components[0] {
        CalendarComponent::Event(e) => e,
        _ => panic!("Expected event"),
    };

    // Convert successfully
    let event_owned = event_ref.to_owned();

    // Verify optional fields are preserved
    assert!(event_ref.summary.is_some());
    assert!(event_owned.summary.is_some());

    assert!(event_ref.description.is_some());
    assert!(event_owned.description.is_some());

    assert!(event_ref.location.is_some());
    assert!(event_owned.location.is_some());

    assert!(event_ref.status.is_some());
    assert!(event_owned.status.is_some());
}
