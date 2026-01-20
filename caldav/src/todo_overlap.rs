// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! RFC 4791 §9.9 VTODO time-range overlap logic.
//!
//! This module implements the complex overlap logic for VTODO components
//! as specified in RFC 4791 Section 9.9.

use std::ops::Neg;

use aimcal_ical::{
    CalendarComponent, DateTime, DateTimeUtc, ICalendar, Property, TodoStatusValue, VTodo,
    ValueDuration,
};
use jiff::tz::TimeZone;
use jiff::{Span, Zoned, civil};

/// Checks if a VTODO overlaps a given time range per RFC 4791 §9.9.
///
/// This implements the full overlap logic table from RFC 4791 Section 9.9
/// for VTODO components, considering:
/// - DTSTART + DURATION combinations
/// - DUE property
/// - COMPLETED property
/// - CREATED property
/// - STATUS value (cancelled todos are excluded)
///
/// # Arguments
///
/// * `calendar` - The iCalendar object containing the VTODO
/// * `range_start` - Start of time range (inclusive)
/// * `range_end` - End of time range (exclusive)
///
/// # Returns
///
/// `true` if the todo overlaps the time range according to RFC 4791 rules.
#[must_use]
pub fn todo_overlaps_time_range(
    calendar: &ICalendar<String>,
    range_start: &Zoned,
    range_end: &Zoned,
) -> bool {
    let Some(todo) = extract_todo(calendar) else {
        return false;
    };

    // Exclude cancelled todos
    if todo
        .status
        .as_ref()
        .is_some_and(|status| matches!(status.value, TodoStatusValue::Cancelled))
    {
        return false;
    }

    // Determine which overlap rule applies based on properties present
    let has_dtstart = todo.dt_start.is_some();
    let has_duration = todo.duration.is_some();
    let has_due = todo.due.is_some();
    let has_completed = todo.completed.is_some();
    let has_created = find_created_property(todo);

    match (
        has_dtstart,
        has_duration,
        has_due,
        has_completed,
        has_created,
    ) {
        // DTSTART + DURATION
        (true, true, false, _, _) => check_dtstart_duration_overlap(todo, range_start, range_end),

        // DTSTART + DUE (but no DURATION)
        (true, false, true, _, _) => check_dtstart_due_overlap(todo, range_start, range_end),

        // DTSTART only (no DURATION, no DUE)
        (true, false, false, _, _) => check_dtstart_only_overlap(todo, range_start, range_end),

        // DUE only (no DTSTART)
        (false, false, true, _, _) => check_due_only_overlap(todo, range_start, range_end),

        // Invalid combinations - per RFC 4791 these shouldn't occur, but handle gracefully
        // DTSTART + DURATION + DUE (DURATION and DUE are mutually exclusive per RFC 5545)
        (true, true, true, _, _) => {
            // Treat as DTSTART + DURATION (ignore DUE)
            check_dtstart_duration_overlap(todo, range_start, range_end)
        }

        // DURATION without DTSTART or DUE (invalid per RFC 5545)
        (false, true, _, _, _) => {
            // No valid anchor point, return false
            false
        }

        // COMPLETED + CREATED (no DTSTART, DURATION, or DUE)
        (false, false, false, true, true) => {
            check_completed_created_overlap(todo, range_start, range_end)
        }

        // COMPLETED only (no CREATED)
        (false, false, false, true, false) => {
            check_completed_only_overlap(todo, range_start, range_end)
        }

        // CREATED only (no COMPLETED)
        (false, false, false, false, true) => {
            check_created_only_overlap(todo, range_start, range_end)
        }

        // No DTSTART, DURATION, DUE, COMPLETED, or CREATED
        (false, false, false, false, false) => {
            // Per RFC 4791 §9.9: "TRUE" - always overlaps
            true
        }
    }
}

/// Extracts the first VTODO from an iCalendar.
fn extract_todo(calendar: &ICalendar<String>) -> Option<&VTodo<String>> {
    calendar.components.iter().find_map(|comp| {
        if let CalendarComponent::Todo(todo) = comp {
            Some(todo)
        } else {
            None
        }
    })
}

/// Finds the CREATED property in a VTODO (if present).
fn find_created_property(todo: &VTodo<String>) -> bool {
    todo.retained_properties
        .iter()
        .any(|p| matches!(p, Property::Created(_)))
}

// Below are the overlap check functions implementing each row of RFC 4791 §9.9 table

// Row 1: DTSTART=Y, DURATION=Y, DUE=N
// Condition: (start <= DTSTART+DURATION) AND ((end > DTSTART) OR (end >= DTSTART+DURATION))
fn check_dtstart_duration_overlap(
    todo: &VTodo<String>,
    range_start: &Zoned,
    range_end: &Zoned,
) -> bool {
    let Some(dtstart) = extract_dtstart_zoned(todo) else {
        return false;
    };
    let Some(duration) = extract_duration_span(todo) else {
        return false;
    };

    let dtstart_end = dtstart.saturating_add(duration);

    let start_condition = range_start <= dtstart_end;
    let end_condition = range_end > dtstart || range_end >= dtstart_end;

    start_condition && end_condition
}

// Row 2: DTSTART=Y, DURATION=N, DUE=Y
// Condition: ((start < DUE) OR (start <= DTSTART)) AND ((end > DTSTART) OR (end >= DUE))
fn check_dtstart_due_overlap(todo: &VTodo<String>, range_start: &Zoned, range_end: &Zoned) -> bool {
    let Some(dtstart) = extract_dtstart_zoned(todo) else {
        return false;
    };
    let Some(due) = extract_due_zoned(todo) else {
        return false;
    };

    let start_condition = range_start < due || range_start <= dtstart;
    let end_condition = range_end > dtstart || range_end >= due;

    start_condition && end_condition
}

// Row 3: DTSTART=Y, DURATION=N, DUE=N
// Condition: (start <= DTSTART) AND (end > DTSTART)
fn check_dtstart_only_overlap(
    todo: &VTodo<String>,
    range_start: &Zoned,
    range_end: &Zoned,
) -> bool {
    let Some(dtstart) = extract_dtstart_zoned(todo) else {
        return false;
    };

    range_start <= dtstart && range_end > dtstart
}

// Row 4: DTSTART=N, DURATION=N, DUE=Y
// Condition: (start < DUE) AND (end >= DUE)
fn check_due_only_overlap(todo: &VTodo<String>, range_start: &Zoned, range_end: &Zoned) -> bool {
    let Some(due) = extract_due_zoned(todo) else {
        return false;
    };

    range_start < due && range_end >= due
}

// Row 5: DTSTART=N, DURATION=N, DUE=N, COMPLETED=Y, CREATED=Y
// Condition: ((start <= CREATED) OR (start <= COMPLETED)) AND
//             ((end >= CREATED) OR (end >= COMPLETED))
fn check_completed_created_overlap(
    todo: &VTodo<String>,
    range_start: &Zoned,
    range_end: &Zoned,
) -> bool {
    let Some(completed) = extract_completed_zoned(todo) else {
        return false;
    };
    let Some(created) = extract_created_zoned(todo) else {
        return false;
    };

    let start_condition = range_start <= created || range_start <= completed;
    let end_condition = range_end >= created || range_end >= completed;

    start_condition && end_condition
}

// Row 6: DTSTART=N, DURATION=N, DUE=N, COMPLETED=Y, CREATED=N
// Condition: (start <= COMPLETED) AND (end >= COMPLETED)
fn check_completed_only_overlap(
    todo: &VTodo<String>,
    range_start: &Zoned,
    range_end: &Zoned,
) -> bool {
    let Some(completed) = extract_completed_zoned(todo) else {
        return false;
    };

    range_start <= completed && range_end >= completed
}

// Row 7: DTSTART=N, DURATION=N, DUE=N, COMPLETED=N, CREATED=Y
// Condition: (end > CREATED)
fn check_created_only_overlap(
    todo: &VTodo<String>,
    _range_start: &Zoned,
    range_end: &Zoned,
) -> bool {
    let Some(created) = extract_created_zoned(todo) else {
        return false;
    };

    range_end > created
}

// Property extraction helpers

/// Extracts DTSTART as Zoned datetime (UTC or converted from timezone).
fn extract_dtstart_zoned(todo: &VTodo<String>) -> Option<Zoned> {
    todo.dt_start
        .as_ref()
        .and_then(|dt| extract_datetime_as_zoned(&dt.value))
}

/// Extracts DUE as Zoned datetime (UTC or converted from timezone).
fn extract_due_zoned(todo: &VTodo<String>) -> Option<Zoned> {
    todo.due
        .as_ref()
        .and_then(|due| extract_datetime_as_zoned(&due.value))
}

/// Extracts COMPLETED as Zoned datetime (UTC).
fn extract_completed_zoned(todo: &VTodo<String>) -> Option<Zoned> {
    todo.completed
        .as_ref()
        .and_then(|c| extract_datetime_utc_as_zoned(c))
}

/// Extracts CREATED as Zoned datetime (UTC).
fn extract_created_zoned(todo: &VTodo<String>) -> Option<Zoned> {
    todo.retained_properties.iter().find_map(|p| {
        if let Property::Created(created) = p {
            extract_datetime_utc_as_zoned(created)
        } else {
            None
        }
    })
}

/// Extracts DURATION as `jiff::Span`.
fn extract_duration_span(todo: &VTodo<String>) -> Option<Span> {
    todo.duration
        .as_ref()
        .map(|d| convert_value_duration_to_span(&d.value))
}

/// Extracts a `DateTime` (from `DtStart` or `Due`) as `Zoned`, handling UTC, floating, and zoned variants.
fn extract_datetime_as_zoned(dt: &DateTime) -> Option<Zoned> {
    match dt {
        // Use the civil_date_time() method if available
        DateTime::Utc { .. } | DateTime::Floating { .. } | DateTime::Zoned { .. } => dt
            .civil_date_time()
            .and_then(|civil| civil.to_zoned(TimeZone::UTC).ok()),

        // Date-only - treat as midnight UTC
        DateTime::Date(date) => {
            let civil = date.civil_date().at(0, 0, 0, 0);
            civil.to_zoned(TimeZone::UTC).ok()
        }
    }
}

/// Extracts a `DateTimeUtc` as `Zoned` in UTC timezone.
fn extract_datetime_utc_as_zoned(dt: &DateTimeUtc<String>) -> Option<Zoned> {
    let civil = civil::DateTime::from_parts(dt.date.civil_date(), dt.time.civil_time());
    civil.to_zoned(TimeZone::UTC).ok()
}

/// Converts `ValueDuration` to `jiff::Span`.
fn convert_value_duration_to_span(duration: &ValueDuration) -> Span {
    match duration {
        ValueDuration::DateTime {
            positive,
            day,
            hour,
            minute,
            second,
        } => {
            let mut span = Span::new()
                .days(i64::from(*day))
                .hours(i64::from(*hour))
                .minutes(i64::from(*minute))
                .seconds(i64::from(*second));

            if !positive {
                span = span.neg();
            }

            span
        }

        ValueDuration::Week { positive, week } => {
            let mut span = Span::new().weeks(i64::from(*week));

            if !positive {
                span = span.neg();
            }

            span
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aimcal_ical::parse;

    /// Helper to parse a VTODO from an iCalendar string
    fn parse_todo(ical_str: &str) -> ICalendar<String> {
        let calendar = parse(ical_str)
            .expect("Failed to parse iCalendar")
            .into_iter()
            .next()
            .expect("No calendar found");
        calendar.to_owned()
    }

    #[test]
    fn todo_overlap_dtstart_duration() {
        // DTSTART: 2024-01-15 10:00, DURATION: 2 hours
        // Range: 2024-01-15 09:00 - 12:00
        // Should overlap: (09:00 <= 12:00) AND ((12:00 > 10:00) OR (12:00 >= 12:00))
        let ical = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
UID:test1@example.com\r\n\
DTSTAMP:20240115T090000Z\r\n\
DTSTART:20240115T100000Z\r\n\
DURATION:PT2H\r\n\
SUMMARY:Task with duration\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let calendar = parse_todo(ical);
        let range_start = civil::date(2024, 1, 15)
            .at(9, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let range_end = civil::date(2024, 1, 15)
            .at(12, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert!(todo_overlaps_time_range(
            &calendar,
            &range_start,
            &range_end
        ));
    }

    #[test]
    fn todo_overlap_dtstart_due() {
        // DTSTART: 2024-01-15 10:00, DUE: 2024-01-15 18:00
        // Range: 2024-01-15 09:00 - 17:00
        // Should overlap: ((09:00 < 18:00) OR (09:00 <= 10:00)) AND ((17:00 > 10:00) OR (17:00 >= 18:00))
        let ical = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
UID:test2@example.com\r\n\
DTSTAMP:20240115T090000Z\r\n\
DTSTART:20240115T100000Z\r\n\
DUE:20240115T180000Z\r\n\
SUMMARY:Task with due date\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let calendar = parse_todo(ical);
        let range_start = civil::date(2024, 1, 15)
            .at(9, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let range_end = civil::date(2024, 1, 15)
            .at(17, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert!(todo_overlaps_time_range(
            &calendar,
            &range_start,
            &range_end
        ));
    }

    #[test]
    fn todo_overlap_dtstart_only() {
        // DTSTART: 2024-01-15 10:00
        // Range: 2024-01-15 09:00 - 11:00
        // Should overlap: (09:00 <= 10:00) AND (11:00 > 10:00)
        let ical = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
UID:test3@example.com\r\n\
DTSTAMP:20240115T090000Z\r\n\
DTSTART:20240115T100000Z\r\n\
SUMMARY:Task with start only\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let calendar = parse_todo(ical);
        let range_start = civil::date(2024, 1, 15)
            .at(9, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let range_end = civil::date(2024, 1, 15)
            .at(11, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert!(todo_overlaps_time_range(
            &calendar,
            &range_start,
            &range_end
        ));
    }

    #[test]
    fn todo_overlap_due_only() {
        // DUE: 2024-01-15 17:00
        // Range: 2024-01-15 16:00 - 18:00
        // Should overlap: (16:00 < 17:00) AND (18:00 >= 17:00)
        let ical = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
UID:test4@example.com\r\n\
DTSTAMP:20240115T090000Z\r\n\
DUE:20240115T170000Z\r\n\
SUMMARY:Task with due only\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let calendar = parse_todo(ical);
        let range_start = civil::date(2024, 1, 15)
            .at(16, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let range_end = civil::date(2024, 1, 15)
            .at(18, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert!(todo_overlaps_time_range(
            &calendar,
            &range_start,
            &range_end
        ));
    }

    #[test]
    fn todo_overlap_cancelled_excluded() {
        // Cancelled todo should not overlap
        let ical = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
UID:test5@example.com\r\n\
DTSTAMP:20240115T090000Z\r\n\
DTSTART:20240115T100000Z\r\n\
STATUS:CANCELLED\r\n\
SUMMARY:Cancelled task\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let calendar = parse_todo(ical);
        let range_start = civil::date(2024, 1, 15)
            .at(9, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let range_end = civil::date(2024, 1, 15)
            .at(11, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert!(!todo_overlaps_time_range(
            &calendar,
            &range_start,
            &range_end
        ));
    }

    #[test]
    fn todo_overlap_no_properties_always_true() {
        // Todo with no DTSTART, DUE, DURATION, COMPLETED, or CREATED should always overlap
        let ical = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
UID:test6@example.com\r\n\
DTSTAMP:20240101T000000Z\r\n\
SUMMARY:Task with no date properties\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let calendar = parse_todo(ical);
        let range_start = civil::date(2024, 1, 1)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let range_end = civil::date(2024, 12, 31)
            .at(23, 59, 59, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert!(todo_overlaps_time_range(
            &calendar,
            &range_start,
            &range_end
        ));
    }

    #[test]
    fn todo_overlap_completed_created() {
        // Row 5: COMPLETED + CREATED (no DTSTART, DUE, DURATION)
        // COMPLETED: 2024-01-15 12:00, CREATED: 2024-01-15 10:00
        // Range: 2024-01-15 09:00 - 13:00
        // Should overlap: ((09:00 <= 10:00) OR (09:00 <= 12:00)) AND ((13:00 >= 10:00) OR (13:00 >= 12:00))
        let ical = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
UID:test7@example.com\r\n\
DTSTAMP:20240115T090000Z\r\n\
CREATED:20240115T100000Z\r\n\
COMPLETED:20240115T120000Z\r\n\
SUMMARY:Completed task\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let calendar = parse_todo(ical);
        let range_start = civil::date(2024, 1, 15)
            .at(9, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let range_end = civil::date(2024, 1, 15)
            .at(13, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert!(todo_overlaps_time_range(
            &calendar,
            &range_start,
            &range_end
        ));
    }

    #[test]
    fn todo_overlap_completed_only() {
        // Row 6: COMPLETED only (no CREATED)
        // COMPLETED: 2024-01-15 12:00
        // Range: 2024-01-15 11:00 - 13:00
        // Should overlap: (11:00 <= 12:00) AND (13:00 >= 12:00)
        let ical = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
UID:test8@example.com\r\n\
DTSTAMP:20240115T090000Z\r\n\
COMPLETED:20240115T120000Z\r\n\
SUMMARY:Completed task\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let calendar = parse_todo(ical);
        let range_start = civil::date(2024, 1, 15)
            .at(11, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let range_end = civil::date(2024, 1, 15)
            .at(13, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert!(todo_overlaps_time_range(
            &calendar,
            &range_start,
            &range_end
        ));
    }

    #[test]
    fn todo_overlap_created_only() {
        // Row 7: CREATED only (no COMPLETED)
        // CREATED: 2024-01-15 10:00
        // Range: 2024-01-15 09:00 - 11:00
        // Should overlap: (11:00 > 10:00)
        let ical = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
UID:test9@example.com\r\n\
DTSTAMP:20240115T090000Z\r\n\
CREATED:20240115T100000Z\r\n\
SUMMARY:Created task\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let calendar = parse_todo(ical);
        let range_start = civil::date(2024, 1, 15)
            .at(9, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let range_end = civil::date(2024, 1, 15)
            .at(11, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert!(todo_overlaps_time_range(
            &calendar,
            &range_start,
            &range_end
        ));
    }
}
