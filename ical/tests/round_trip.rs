// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Round-trip tests for the iCalendar parser and formatter.
//!
//! These tests verify that parsing, converting to owned, formatting,
//! and parsing again produces equivalent results.

use aimcal_ical::{formatter::format, parse};

#[test]
fn round_trip_simple_calendar() {
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

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars should be equal after round-trip"
    );
}

#[test]
fn round_trip_calendar_with_multiple_events() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
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
DESCRIPTION:This is a test description\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with multiple events should be equal after round-trip"
    );
}

#[test]
fn round_trip_calendar_with_todo() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VTODO\r\n\
UID:todo123@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T090000Z\r\n\
DUE:20250110T170000Z\r\n\
SUMMARY:Complete task\r\n\
STATUS:NEEDS-ACTION\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with todo should be equal after round-trip"
    );
}

#[test]
fn round_trip_calendar_with_journal() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VJOURNAL\r\n\
UID:journal123@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T090000Z\r\n\
SUMMARY:Daily Journal Entry\r\n\
END:VJOURNAL\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with journal should be equal after round-trip"
    );
}

#[test]
fn round_trip_calendar_with_alarm() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
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

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with alarm should be equal after round-trip"
    );
}

#[test]
fn round_trip_calendar_with_rrule() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
RRULE:FREQ=DAILY;COUNT=5\r\n\
SUMMARY:Daily Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with RRULE should be equal after round-trip"
    );

    // Verify RRULE is preserved in formatted output
    assert!(formatted.contains("RRULE:FREQ=DAILY;COUNT=5"));
}

#[test]
fn round_trip_calendar_with_parameters() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY;LANGUAGE=en:Test Event\r\n\
DESCRIPTION;ALTREP=\"http://example.com/desc\":Meeting description\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with parameters should be equal after round-trip"
    );

    // Verify parameters are preserved in formatted output
    assert!(formatted.contains("SUMMARY;LANGUAGE=en:Test Event"));
}

#[test]
fn round_trip_calendar_with_custom_property() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
X-CUSTOM-PROPERTY:custom value\r\n\
SUMMARY:Test Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with custom properties should be equal after round-trip"
    );

    // Verify custom property is preserved
    assert!(formatted.contains("X-CUSTOM-PROPERTY:custom value"));
}

#[test]
fn round_trip_calendar_with_mixed_components() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:event@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY:Meeting\r\n\
END:VEVENT\r\n\
BEGIN:VTODO\r\n\
UID:todo@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T090000Z\r\n\
DUE:20250110T170000Z\r\n\
SUMMARY:Task\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with mixed components should be equal after round-trip"
    );
}

#[test]
fn round_trip_calendar_with_duration() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DURATION:PT1H30M\r\n\
SUMMARY:One and a half hour meeting\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with duration should be equal after round-trip"
    );

    // Verify duration is preserved
    assert!(formatted.contains("DURATION:PT1H30M"));
}

#[test]
fn round_trip_calendar_with_date_only() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART;VALUE=DATE:20250110\r\n\
DTEND;VALUE=DATE:20250111\r\n\
SUMMARY:All day event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with date values should be equal after round-trip"
    );

    // Verify DATE values are preserved
    assert!(formatted.contains("DTSTART;VALUE=DATE:20250110"));
    assert!(formatted.contains("DTEND;VALUE=DATE:20250111"));
}

#[test]
fn round_trip_calendar_with_text_escaping() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY:Test with semicolon\\; and comma\\, and backslash\\\\\r\n\
DESCRIPTION:Line 1\\nLine 2\\nLine 3\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // Convert to owned
    let calendar_owned = calendar1.to_owned();

    // Format to string
    let formatted = format(&calendar_owned).unwrap();

    // Parse formatted version
    let calendars2 = parse(&formatted).unwrap();
    let calendar2 = &calendars2[0];

    // Compare calendars
    assert!(
        calendars_equal(calendar1, calendar2),
        "Calendars with text escaping should be equal after round-trip"
    );
}

#[test]
fn round_trip_double_format() {
    let original = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//Cal Client 1.0//EN\r\n\
BEGIN:VEVENT\r\n\
UID:test@example.com\r\n\
DTSTAMP:20250110T120000Z\r\n\
DTSTART:20250110T140000Z\r\n\
DTEND:20250110T150000Z\r\n\
SUMMARY:Test Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Parse original
    let calendars1 = parse(original).unwrap();
    let calendar1 = &calendars1[0];

    // First round-trip
    let calendar_owned1 = calendar1.to_owned();
    let formatted1 = format(&calendar_owned1).unwrap();
    let calendars2 = parse(&formatted1).unwrap();
    let calendar2 = &calendars2[0];

    // Second round-trip
    let calendar_owned2 = calendar2.to_owned();
    let formatted2 = format(&calendar_owned2).unwrap();
    let calendars3 = parse(&formatted2).unwrap();
    let calendar3 = &calendars3[0];

    // All should be equal
    assert!(
        calendars_equal(calendar1, calendar2),
        "First round-trip should preserve equality"
    );
    assert!(
        calendars_equal(calendar2, calendar3),
        "Second round-trip should preserve equality"
    );

    // Formatted strings should also be equal
    assert_eq!(
        formatted1, formatted2,
        "Double formatting should produce identical output"
    );
}

/// Helper function to compare two ICalendars structurally for key properties.
///
/// This is a simplified comparison that checks the essential properties
/// are preserved through the round-trip process.
fn calendars_equal(
    cal1: &aimcal_ical::ICalendarRef<'_>,
    cal2: &aimcal_ical::ICalendarRef<'_>,
) -> bool {
    // Compare prod_id
    let prod_id1 = cal1.prod_id.value.to_string();
    let prod_id2 = cal2.prod_id.value.to_string();
    if prod_id1 != prod_id2 {
        return false;
    }

    // Compare version
    let version1 = format!("{:?}", cal1.version.value);
    let version2 = format!("{:?}", cal2.version.value);
    if version1 != version2 {
        return false;
    }

    // Compare calscale
    match (&cal1.calscale, &cal2.calscale) {
        (None, None) => {}
        (Some(c1), Some(c2)) if format!("{:?}", c1.value) == format!("{:?}", c2.value) => {}
        _ => return false,
    }

    // Compare method
    match (&cal1.method, &cal2.method) {
        (None, None) => {}
        (Some(m1), Some(m2)) if format!("{:?}", m1.value) == format!("{:?}", m2.value) => {}
        _ => return false,
    }

    // Compare number of components
    if cal1.components.len() != cal2.components.len() {
        return false;
    }

    // Compare components
    for (comp1, comp2) in cal1.components.iter().zip(cal2.components.iter()) {
        if !components_equal(comp1, comp2) {
            return false;
        }
    }

    // Compare x_properties count
    if cal1.x_properties.len() != cal2.x_properties.len() {
        return false;
    }

    true
}

/// Helper function to compare two CalendarComponents.
fn components_equal(
    comp1: &aimcal_ical::CalendarComponent<aimcal_ical::SpannedSegments<'_>>,
    comp2: &aimcal_ical::CalendarComponent<aimcal_ical::SpannedSegments<'_>>,
) -> bool {
    match (comp1, comp2) {
        (aimcal_ical::CalendarComponent::Event(e1), aimcal_ical::CalendarComponent::Event(e2)) => {
            events_equal(e1, e2)
        }
        (aimcal_ical::CalendarComponent::Todo(t1), aimcal_ical::CalendarComponent::Todo(t2)) => {
            todos_equal(t1, t2)
        }
        (
            aimcal_ical::CalendarComponent::VJournal(j1),
            aimcal_ical::CalendarComponent::VJournal(j2),
        ) => journals_equal(j1, j2),
        (
            aimcal_ical::CalendarComponent::VTimeZone(tz1),
            aimcal_ical::CalendarComponent::VTimeZone(tz2),
        ) => timezones_equal(tz1, tz2),
        (
            aimcal_ical::CalendarComponent::VFreeBusy(fb1),
            aimcal_ical::CalendarComponent::VFreeBusy(fb2),
        ) => freebusies_equal(fb1, fb2),
        (
            aimcal_ical::CalendarComponent::VAlarm(a1),
            aimcal_ical::CalendarComponent::VAlarm(a2),
        ) => alarms_equal(a1, a2),
        _ => false,
    }
}

/// Helper function to compare two VEvents.
fn events_equal(e1: &aimcal_ical::VEventRef<'_>, e2: &aimcal_ical::VEventRef<'_>) -> bool {
    // Compare UID
    if e1.uid.content.to_string() != e2.uid.content.to_string() {
        return false;
    }

    // Compare DTSTAMP
    if format!("{:?}", e1.dt_stamp.inner) != format!("{:?}", e2.dt_stamp.inner) {
        return false;
    }

    // Compare DTSTART
    if format!("{:?}", e1.dt_start.inner) != format!("{:?}", e2.dt_start.inner) {
        return false;
    }

    // Compare DTEND
    match (&e1.dt_end, &e2.dt_end) {
        (None, None) => {}
        (Some(de1), Some(de2)) if format!("{:?}", de1.inner) == format!("{:?}", de2.inner) => {}
        _ => return false,
    }

    // Compare SUMMARY
    match (&e1.summary, &e2.summary) {
        (None, None) => {}
        (Some(s1), Some(s2)) if s1.content.to_string() == s2.content.to_string() => {}
        _ => return false,
    }

    // Compare DESCRIPTION
    match (&e1.description, &e2.description) {
        (None, None) => {}
        (Some(d1), Some(d2)) if d1.content.to_string() == d2.content.to_string() => {}
        _ => return false,
    }

    // Compare LOCATION
    match (&e1.location, &e2.location) {
        (None, None) => {}
        (Some(l1), Some(l2)) if l1.content.to_string() == l2.content.to_string() => {}
        _ => return false,
    }

    // Compare STATUS
    match (&e1.status, &e2.status) {
        (None, None) => {}
        (Some(s1), Some(s2)) if format!("{:?}", s1.value) == format!("{:?}", s2.value) => {}
        _ => return false,
    }

    // Compare alarms count
    if e1.alarms.len() != e2.alarms.len() {
        return false;
    }

    true
}

/// Helper function to compare two VTodos.
fn todos_equal(t1: &aimcal_ical::VTodoRef<'_>, t2: &aimcal_ical::VTodoRef<'_>) -> bool {
    // Compare UID
    if t1.uid.content.to_string() != t2.uid.content.to_string() {
        return false;
    }

    // Compare DTSTAMP
    if format!("{:?}", t1.dt_stamp.inner) != format!("{:?}", t2.dt_stamp.inner) {
        return false;
    }

    // Compare DTSTART
    if format!("{:?}", t1.dt_start) != format!("{:?}", t2.dt_start) {
        return false;
    }

    // Compare DUE
    match (&t1.due, &t2.due) {
        (None, None) => {}
        (Some(d1), Some(d2)) if format!("{:?}", d1.inner) == format!("{:?}", d2.inner) => {}
        _ => return false,
    }

    // Compare SUMMARY
    match (&t1.summary, &t2.summary) {
        (None, None) => {}
        (Some(s1), Some(s2)) if s1.content.to_string() == s2.content.to_string() => {}
        _ => return false,
    }

    // Compare STATUS
    match (&t1.status, &t2.status) {
        (None, None) => {}
        (Some(s1), Some(s2)) if format!("{:?}", s1.value) == format!("{:?}", s2.value) => {}
        _ => return false,
    }

    true
}

/// Helper function to compare two VJournals.
fn journals_equal(j1: &aimcal_ical::VJournalRef<'_>, j2: &aimcal_ical::VJournalRef<'_>) -> bool {
    // Compare UID
    if j1.uid.content.to_string() != j2.uid.content.to_string() {
        return false;
    }

    // Compare DTSTAMP
    if format!("{:?}", j1.dt_stamp.inner) != format!("{:?}", j2.dt_stamp.inner) {
        return false;
    }

    // Compare DTSTART
    if format!("{:?}", j1.dt_start.inner) != format!("{:?}", j2.dt_start.inner) {
        return false;
    }

    true
}

/// Helper function to compare two VTimeZones.
fn timezones_equal(
    tz1: &aimcal_ical::VTimeZoneRef<'_>,
    tz2: &aimcal_ical::VTimeZoneRef<'_>,
) -> bool {
    // Compare TZID
    if tz1.tz_id.content.to_string() != tz2.tz_id.content.to_string() {
        return false;
    }

    // Compare standard count
    if tz1.standard.len() != tz2.standard.len() {
        return false;
    }

    // Compare daylight count
    if tz1.daylight.len() != tz2.daylight.len() {
        return false;
    }

    true
}

/// Helper function to compare two VFreeBusys.
fn freebusies_equal(
    fb1: &aimcal_ical::VFreeBusyRef<'_>,
    fb2: &aimcal_ical::VFreeBusyRef<'_>,
) -> bool {
    // Compare UID
    if fb1.uid.content.to_string() != fb2.uid.content.to_string() {
        return false;
    }

    // Compare DTSTAMP
    if format!("{:?}", fb1.dt_stamp.inner) != format!("{:?}", fb2.dt_stamp.inner) {
        return false;
    }

    // Compare DTSTART
    if format!("{:?}", fb1.dt_start) != format!("{:?}", fb2.dt_start) {
        return false;
    }

    // Compare DTEND
    if format!("{:?}", fb1.dt_end) != format!("{:?}", fb2.dt_end) {
        return false;
    }

    true
}

/// Helper function to compare two VAlarms.
fn alarms_equal(a1: &aimcal_ical::VAlarmRef<'_>, a2: &aimcal_ical::VAlarmRef<'_>) -> bool {
    // Compare ACTION
    if format!("{:?}", a1.action.value) != format!("{:?}", a2.action.value) {
        return false;
    }

    // Compare TRIGGER
    if format!("{:?}", a1.trigger.value) != format!("{:?}", a2.trigger.value) {
        return false;
    }

    // Compare DESCRIPTION
    match (&a1.description, &a2.description) {
        (None, None) => {}
        (Some(d1), Some(d2)) if d1.content.to_string() == d2.content.to_string() => {}
        _ => return false,
    }

    true
}
