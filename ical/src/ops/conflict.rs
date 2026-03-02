// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event conflict detection utilities.

#![cfg(feature = "jiff")]

use crate::ops::rrule::EventOccurrence;

/// Extension trait for conflict detection.
pub trait ConflictExt {
    /// Detects time conflicts among events.
    ///
    /// Returns a list of conflicts, where each conflict contains at least two
    /// events that overlap in time.
    ///
    /// Events without an end time are treated as point-in-time events and
    /// do not participate in conflicts.
    fn detect_conflicts(&self) -> Vec<Conflict>;
}

impl<T: AsRef<[EventOccurrence<String>]>> ConflictExt for T {
    fn detect_conflicts(&self) -> Vec<Conflict> {
        let occurrences = self.as_ref();

        // Filter to only events with both start and end times
        let valid_events: Vec<(usize, &EventOccurrence<String>)> = occurrences
            .iter()
            .enumerate()
            .filter(|(_, occ)| occ.end.is_some())
            .collect();

        if valid_events.len() < 2 {
            return Vec::new();
        }

        let mut conflicts = Vec::new();

        // Compare all pairs using iterators
        for (i, (idx1, occ1)) in valid_events.iter().enumerate() {
            for (idx2, occ2) in valid_events.iter().skip(i + 1) {
                // SAFETY: We filtered for events with end times
                let end1 = occ1.end.unwrap();
                let end2 = occ2.end.unwrap();

                // Check overlap: occ1.start < occ2.end AND occ2.start < occ1.end
                if occ1.start < end2 && occ2.start < end1 {
                    let overlap_start = std::cmp::max(occ1.start, occ2.start);
                    let overlap_end = std::cmp::min(end1, end2);

                    conflicts.push(Conflict {
                        events: vec![
                            ConflictEvent {
                                index: *idx1,
                                overlap: ConflictRange {
                                    start: overlap_start,
                                    end: overlap_end,
                                },
                            },
                            ConflictEvent {
                                index: *idx2,
                                overlap: ConflictRange {
                                    start: overlap_start,
                                    end: overlap_end,
                                },
                            },
                        ],
                    });
                }
            }
        }

        conflicts
    }
}

/// A conflict between two or more events.
#[derive(Debug, Clone)]
pub struct Conflict {
    /// The events that conflict with each other
    pub events: Vec<ConflictEvent>,
}

/// An event involved in a conflict.
#[derive(Debug, Clone, Copy)]
pub struct ConflictEvent {
    /// Index of the event in the original list
    pub index: usize,
    /// The overlapping time range
    pub overlap: ConflictRange,
}

/// A time range where events overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConflictRange {
    /// Start of the conflict (inclusive)
    pub start: jiff::civil::DateTime,
    /// End of the conflict (inclusive)
    pub end: jiff::civil::DateTime,
}

#[cfg(test)]
mod tests {
    use crate::semantic::{ICalendar, VEvent, semantic_analysis};
    use crate::string_storage::Segments;
    use crate::syntax::syntax_analysis;
    use crate::typed::typed_analysis;

    use super::*;

    /// Helper to parse a minimal event and create an occurrence
    fn make_occurrence_from_event(
        event: VEvent<String>,
        start: &str,
        end: Option<&str>,
    ) -> EventOccurrence<String> {
        let start_dt: jiff::civil::DateTime = start.parse().unwrap();
        let end_dt: Option<jiff::civil::DateTime> = end.map(|s| s.parse().unwrap());
        EventOccurrence {
            event,
            start: start_dt,
            end: end_dt,
        }
    }

    /// Parse a minimal iCalendar string to get a `VEvent`
    fn parse_event(src: &str) -> VEvent<String> {
        let syntax_components = syntax_analysis(src).unwrap();
        let typed_components = typed_analysis(syntax_components).unwrap();
        let calendars: Vec<ICalendar<Segments<'_>>> = semantic_analysis(typed_components).unwrap();
        let calendar = calendars.first().unwrap();
        match calendar.components.first().unwrap() {
            crate::semantic::CalendarComponent::Event(event) => event.to_owned(),
            _ => panic!("Expected VEvent"),
        }
    }

    /// Create a minimal event for testing
    fn make_minimal_event(uid: &str) -> VEvent<String> {
        let src = format!(
            "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VEVENT\r\n\
UID:{uid}\r\n\
DTSTAMP:20250101T000000Z\r\n\
DTSTART:20250101T100000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
"
        );
        parse_event(&src)
    }

    #[test]
    fn no_conflicts_with_empty_list() {
        let occurrences: Vec<EventOccurrence<String>> = Vec::new();
        let conflicts = occurrences.detect_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn no_conflicts_with_single_event() {
        let event = make_minimal_event("1");
        let occurrences = vec![make_occurrence_from_event(
            event,
            "2025-01-01T10:00",
            Some("2025-01-01T11:00"),
        )];
        let conflicts = occurrences.detect_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn no_conflicts_with_non_overlapping_events() {
        let event1 = make_minimal_event("1");
        let event2 = make_minimal_event("2");

        let occurrences = vec![
            make_occurrence_from_event(event1, "2025-01-01T10:00", Some("2025-01-01T11:00")),
            make_occurrence_from_event(event2, "2025-01-01T11:00", Some("2025-01-01T12:00")),
        ];

        let conflicts = occurrences.detect_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn no_conflicts_with_separated_events() {
        let event1 = make_minimal_event("1");
        let event2 = make_minimal_event("2");

        let occurrences = vec![
            make_occurrence_from_event(event1, "2025-01-01T10:00", Some("2025-01-01T11:00")),
            make_occurrence_from_event(event2, "2025-01-01T13:00", Some("2025-01-01T14:00")),
        ];

        let conflicts = occurrences.detect_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn detects_simple_overlap() {
        let event1 = make_minimal_event("1");
        let event2 = make_minimal_event("2");

        let occurrences = vec![
            make_occurrence_from_event(event1, "2025-01-01T10:00", Some("2025-01-01T12:00")),
            make_occurrence_from_event(event2, "2025-01-01T11:00", Some("2025-01-01T13:00")),
        ];

        let conflicts = occurrences.detect_conflicts();
        assert_eq!(conflicts.len(), 1);

        let conflict = conflicts.first().unwrap();
        assert_eq!(conflict.events.len(), 2);

        // Check overlap range: max(10:00, 11:00) to min(12:00, 13:00)
        assert_eq!(
            conflict.events.first().unwrap().overlap.start,
            "2025-01-01T11:00".parse().unwrap()
        );
        assert_eq!(
            conflict.events.first().unwrap().overlap.end,
            "2025-01-01T12:00".parse().unwrap()
        );
    }

    #[test]
    fn detects_full_containment() {
        let event1 = make_minimal_event("1");
        let event2 = make_minimal_event("2");

        // Event 2 is completely inside Event 1
        let occurrences = vec![
            make_occurrence_from_event(event1, "2025-01-01T10:00", Some("2025-01-01T14:00")),
            make_occurrence_from_event(event2, "2025-01-01T11:00", Some("2025-01-01T12:00")),
        ];

        let conflicts = occurrences.detect_conflicts();
        assert_eq!(conflicts.len(), 1);

        let conflict = conflicts.first().unwrap();
        // Overlap should be the smaller event's range
        assert_eq!(
            conflict.events.first().unwrap().overlap.start,
            "2025-01-01T11:00".parse().unwrap()
        );
        assert_eq!(
            conflict.events.first().unwrap().overlap.end,
            "2025-01-01T12:00".parse().unwrap()
        );
    }

    #[test]
    fn no_conflict_for_events_without_end_time() {
        let event1 = make_minimal_event("1");
        let event2 = make_minimal_event("2");

        let occurrences = vec![
            make_occurrence_from_event(event1, "2025-01-01T10:00", None),
            make_occurrence_from_event(event2, "2025-01-01T10:00", Some("2025-01-01T12:00")),
        ];

        let conflicts = occurrences.detect_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn detects_multiple_conflicts() {
        let event1 = make_minimal_event("1");
        let event2 = make_minimal_event("2");
        let event3 = make_minimal_event("3");

        // Event 1 overlaps with Event 2
        // Event 2 overlaps with Event 3
        // Event 1 does NOT overlap with Event 3
        let occurrences = vec![
            make_occurrence_from_event(event1, "2025-01-01T10:00", Some("2025-01-01T11:00")),
            make_occurrence_from_event(event2, "2025-01-01T10:30", Some("2025-01-01T11:30")),
            make_occurrence_from_event(event3, "2025-01-01T11:00", Some("2025-01-01T12:00")),
        ];

        let conflicts = occurrences.detect_conflicts();
        assert_eq!(conflicts.len(), 2);

        // Check that the right pairs are conflicting
        let conflict_pairs: Vec<(usize, usize)> = conflicts
            .iter()
            .map(|c| {
                let mut indices: Vec<usize> = c.events.iter().map(|e| e.index).collect();
                indices.sort_unstable();
                (*indices.first().unwrap(), *indices.get(1).unwrap())
            })
            .collect();

        assert!(conflict_pairs.contains(&(0, 1)));
        assert!(conflict_pairs.contains(&(1, 2)));
    }

    #[test]
    fn detects_three_way_overlap() {
        let event1 = make_minimal_event("1");
        let event2 = make_minimal_event("2");
        let event3 = make_minimal_event("3");

        // All three events overlap at 11:00-11:30
        let occurrences = vec![
            make_occurrence_from_event(event1, "2025-01-01T10:00", Some("2025-01-01T12:00")),
            make_occurrence_from_event(event2, "2025-01-01T11:00", Some("2025-01-01T13:00")),
            make_occurrence_from_event(event3, "2025-01-01T11:30", Some("2025-01-01T14:00")),
        ];

        let conflicts = occurrences.detect_conflicts();
        // Should have 3 conflicts: (0,1), (0,2), (1,2)
        assert_eq!(conflicts.len(), 3);
    }

    #[test]
    fn conflict_range_is_correct_for_partial_overlap() {
        let event1 = make_minimal_event("1");
        let event2 = make_minimal_event("2");

        let occurrences = vec![
            make_occurrence_from_event(event1, "2025-01-01T10:00", Some("2025-01-01T11:30")),
            make_occurrence_from_event(event2, "2025-01-01T11:00", Some("2025-01-01T12:00")),
        ];

        let conflicts = occurrences.detect_conflicts();
        assert_eq!(conflicts.len(), 1);

        // Overlap is 11:00 to 11:30
        let overlap = conflicts.first().unwrap().events.first().unwrap().overlap;
        assert_eq!(overlap.start, "2025-01-01T11:00".parse().unwrap());
        assert_eq!(overlap.end, "2025-01-01T11:30".parse().unwrap());
    }
}
