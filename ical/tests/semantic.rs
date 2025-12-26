// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the iCalendar semantic analyzer
//!
//! These tests validate the semantic analyzer's behavior on realistic iCalendar content
//! and edge cases.

use aimcal_ical::lexer::{Token, lex_analysis};
use aimcal_ical::semantic::{
    CalendarComponent, CalendarScaleType, MethodType, VersionType, semantic_analysis,
};
use aimcal_ical::syntax::syntax_analysis;
use aimcal_ical::typed::typed_analysis;

/// Test helper to parse iCalendar source through semantic phase
fn parse_semantic(
    src: &str,
) -> Result<aimcal_ical::semantic::ICalendar, aimcal_ical::SemanticError> {
    use aimcal_ical::syntax::SyntaxComponent;
    use chumsky::error::Rich;

    let token_stream = lex_analysis(src);
    let syntax_components: Result<Vec<SyntaxComponent<'_>>, Vec<Rich<'_, Token<'_>>>> =
        syntax_analysis(src, token_stream);
    let typed_components = typed_analysis(syntax_components.unwrap()).unwrap();
    semantic_analysis(typed_components)
}

#[test]
fn semantic_rejects_empty_calendar() {
    let src = "\
BEGIN:VCALENDAR\r
END:VCALENDAR\r
";
    let result = parse_semantic(src);
    // Should fail because PRODID and VERSION are required
    assert!(result.is_err());
}

#[test]
fn semantic_parses_minimal_calendar() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert!(matches!(calendar.version, VersionType::V2_0));
    assert_eq!(calendar.prod_id.company, "-");
    assert_eq!(calendar.prod_id.product, "Example Corp.");
    assert_eq!(calendar.prod_id.language.unwrap(), "CalDAV Client");
    assert!(calendar.calscale.is_none());
    assert!(calendar.method.is_none());
    assert!(calendar.components.is_empty());
}

#[test]
fn semantic_recognizes_calscale_property() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
CALSCALE:GREGORIAN\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert!(calendar.calscale.is_some());
    assert!(matches!(
        calendar.calscale.as_ref().unwrap(),
        CalendarScaleType::Gregorian
    ));
}

#[test]
fn semantic_parses_simple_event() {
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
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert_eq!(event.summary.as_ref().unwrap().content, "Test Event");
        }
        _ => panic!("Expected Event component"),
    }
}

// SEMANTIC ANALYSIS NOT IMPLEMENTED: VTodo parsing is not yet implemented
#[test]
#[ignore = "semantic analysis not implemented: VTodo parsing"]
fn semantic_parses_simple_todo() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VTODO\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T090000Z\r
DUE:20250615T170000Z\r
SUMMARY:Complete task\r
STATUS:NEEDS-ACTION\r
PRIORITY:5\r
END:VTODO\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);

    match &calendar.components[0] {
        CalendarComponent::Todo(todo) => {
            assert_eq!(todo.summary.as_ref().unwrap().content, "Complete task");
        }
        _ => panic!("Expected Todo component"),
    }
}

// SEMANTIC ANALYSIS NOT IMPLEMENTED: VJournal parsing is not yet implemented
#[test]
#[ignore = "semantic analysis not implemented: VJournal parsing"]
fn semantic_parses_simple_journal() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VJOURNAL\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
SUMMARY:Daily Journal Entry\r
END:VJOURNAL\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);

    match &calendar.components[0] {
        CalendarComponent::VJournal(journal) => {
            assert_eq!(
                journal.summary.as_ref().unwrap().content,
                "Daily Journal Entry"
            );
        }
        _ => panic!("Expected VJournal component"),
    }
}

// SEMANTIC ANALYSIS NOT IMPLEMENTED: VFreeBusy parsing is not yet implemented
#[test]
#[ignore = "semantic analysis not implemented: VFreeBusy parsing"]
fn semantic_parses_simple_freebusy() {
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
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);

    match &calendar.components[0] {
        CalendarComponent::VFreeBusy(fb) => {
            // VFreeBusy has dt_start and dt_end fields
            assert!(fb.dt_start.date.day > 0);
            assert!(fb.dt_end.date.day > 0);
        }
        _ => panic!("Expected VFreeBusy component"),
    }
}

// SEMANTIC ANALYSIS NOT IMPLEMENTED: VALARM semantic parsing is not yet implemented
#[test]
#[ignore = "semantic analysis not implemented: VALARM parsing"]
fn semantic_parses_event_with_alarm() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
SUMMARY:Meeting\r
BEGIN:VALARM\r
TRIGGER:PT15M\r
ACTION:DISPLAY\r
DESCRIPTION:Reminder\r
END:VALARM\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert_eq!(event.alarms.len(), 1);
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_multiple_events() {
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
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 2);
}

#[test]
fn semantic_parses_event_with_dtstart_and_dtend() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
DTEND:20250615T110000Z\r
SUMMARY:One hour meeting\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert!(event.dt_start.date.day > 0);
            assert!(event.dt_end.is_some());
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_event_with_duration() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
DURATION:PT1H\r
SUMMARY:One hour meeting\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert!(event.dt_start.date.day > 0);
            assert!(event.duration.is_some());
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_event_location() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
LOCATION:Conference Room B\r
SUMMARY:Team Meeting\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert_eq!(
                event.location.as_ref().unwrap().content,
                "Conference Room B"
            );
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_event_geo() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
GEO:37.386013;-122.083932\r
SUMMARY:Team Meeting\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert!(event.geo.is_some());
            let geo = event.geo.as_ref().unwrap();
            assert!((geo.lat - 37.386013).abs() < f64::EPSILON);
            assert!((geo.lon - (-122.083932)).abs() < f64::EPSILON);
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_event_description() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
DESCRIPTION:This is a detailed description\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert_eq!(
                event.description.as_ref().unwrap().content,
                "This is a detailed description"
            );
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_event_status() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
STATUS:CONFIRMED\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert!(event.status.is_some());
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_event_classification() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
CLASS:PUBLIC\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert!(event.classification.is_some());
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_event_transparency() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
TRANSP:OPAQUE\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert!(event.transparency.is_some());
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_event_priority() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
PRIORITY:5\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert_eq!(event.priority, Some(5));
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_parses_event_sequence() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
SEQUENCE:2\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert_eq!(event.sequence, Some(2));
        }
        _ => panic!("Expected Event component"),
    }
}

// PARSER LIMITATION: VJOURNAL semantic parser is not yet implemented
#[test]
#[ignore = "semantic analysis not implemented: VJOURNAL parsing"]
fn semantic_when_journal_has_date_only_start() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VJOURNAL\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615\r
SUMMARY:Daily Journal Entry\r
END:VJOURNAL\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);
}

// SEMANTIC ANALYSIS NOT IMPLEMENTED: VALARM semantic parsing is not yet implemented
#[test]
#[ignore = "semantic analysis not implemented: VALARM parsing"]
fn semantic_when_alarm_has_negative_trigger() {
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
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);
}

#[test]
fn semantic_parses_event_url() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
URL:http://example.com/event.html\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);
}

#[test]
fn semantic_parses_event_organizer() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
ORGANIZER;CN=John Doe:mailto:john.doe@example.com\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);
}

#[test]
fn semantic_parses_event_attendee() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
ATTENDEE;RSVP=TRUE:mailto:test@example.com\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);
}

#[test]
fn semantic_parses_event_last_modified() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
LAST-MODIFIED:20250102T120000Z\r
DTSTART:20250615T100000Z\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            assert!(event.last_modified.is_some());
        }
        _ => panic!("Expected Event component"),
    }
}

// SEMANTIC ANALYSIS NOT IMPLEMENTED: VTodo parsing is not yet implemented
#[test]
#[ignore = "semantic analysis not implemented: VTodo parsing"]
fn semantic_parses_todo_percent_complete() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VTODO\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T090000Z\r
DUE:20250615T170000Z\r
PERCENT-COMPLETE:75\r
SUMMARY:Task\r
END:VTODO\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Todo(todo) => {
            assert_eq!(todo.percent_complete, Some(75));
        }
        _ => panic!("Expected Todo component"),
    }
}

// SEMANTIC ANALYSIS NOT IMPLEMENTED: VTodo parsing is not yet implemented
#[test]
#[ignore = "semantic analysis not implemented: VTodo parsing in mixed components"]
fn semantic_parses_mixed_components() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:1\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
SUMMARY:Event\r
END:VEVENT\r
BEGIN:VTODO\r
UID:2\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250616T090000Z\r
DUE:20250616T170000Z\r
SUMMARY:Task\r
END:VTODO\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 2);
}

#[test]
fn semantic_when_event_has_tzid_parameter() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART;TZID=America/New_York:20250615T100000\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);
}

#[test]
fn semantic_handles_unicode_in_summary() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
BEGIN:VEVENT\r
UID:12345\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250615T100000Z\r
SUMMARY:Team会议📅\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();

    match &calendar.components[0] {
        CalendarComponent::Event(event) => {
            let summary = event.summary.as_ref().unwrap();
            assert!(summary.content.contains("会议") || summary.content.contains("📅"));
        }
        _ => panic!("Expected Event component"),
    }
}

#[test]
fn semantic_rejects_missing_prodid() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
END:VCALENDAR\r
";
    let result = parse_semantic(src);
    assert!(result.is_err());
}

#[test]
fn semantic_rejects_missing_version() {
    let src = "\
BEGIN:VCALENDAR\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
END:VCALENDAR\r
";
    let result = parse_semantic(src);
    assert!(result.is_err());
}

#[test]
fn semantic_parses_method_property() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Example Corp.//CalDAV Client//EN\r
METHOD:PUBLISH\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert!(calendar.method.is_some());
    assert!(matches!(
        calendar.method.as_ref().unwrap(),
        MethodType::Publish
    ));
}

#[test]
fn semantic_parses_complete_calendar() {
    let src = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//Mozilla.org/NONSGML Mozilla Calendar V1.1//EN\r
CALSCALE:GREGORIAN\r
BEGIN:VEVENT\r
UID:123456789-1234-1234-1234-123456789012\r
DTSTAMP:20250102T120000Z\r
DTSTART:20250615T140000Z\r
DTEND:20250615T150000Z\r
SUMMARY:Weekly Team Meeting\r
LOCATION:Conference Room B\r
DESCRIPTION:Weekly team sync\r
CLASS:PUBLIC\r
STATUS:CONFIRMED\r
TRANSP:OPAQUE\r
PRIORITY:5\r
SEQUENCE:0\r
END:VEVENT\r
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);
    assert!(calendar.calscale.is_some());
}

// SEMANTIC ANALYSIS NOT IMPLEMENTED: VTimeZone parsing is not yet implemented
#[test]
#[ignore = "semantic analysis not implemented: VTimeZone parsing"]
fn semantic_parses_nested_timezone() {
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
END:VCALENDAR\r
";
    let calendar = parse_semantic(src).unwrap();
    assert_eq!(calendar.components.len(), 1);

    match &calendar.components[0] {
        CalendarComponent::VTimeZone(tz) => {
            assert_eq!(tz.tz_id, "America/New_York");
        }
        _ => panic!("Expected VTimeZone component"),
    }
}
