// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Custom assertion helpers for integration tests.
//!
//! This module provides specialized assertion functions for validating
//! events, todos, and file system state.

use std::path::Path;

use aimcal_core::{
    Event, EventDraft, EventStatus, LooseDateTime, Priority, Todo, TodoDraft, TodoStatus,
};

/// Asserts that an event matches the expected values from a draft.
///
/// # Arguments
///
/// * `event` - The event to validate
/// * `summary` - Expected summary
///
/// # Panics
///
/// Panics if the event's summary doesn't match the expected value.
///
/// # Example
///
/// ```ignore
/// assert_event_matches_draft(&event, "Team Meeting");
/// ```
pub fn assert_event_matches_draft<E: Event>(event: &E, summary: &str) {
    assert_eq!(event.summary().as_ref(), summary, "Event summary mismatch");
}

/// Asserts that an event matches all fields from a draft.
///
/// # Arguments
///
/// * `event` - The event to validate
/// * `draft` - The draft with expected values
///
/// # Panics
///
/// Panics if any field doesn't match the expected value.
#[allow(dead_code)]
pub fn assert_event_matches_draft_full<E: Event>(event: &E, draft: &EventDraft) {
    assert_eq!(event.summary().as_ref(), &draft.summary, "Summary mismatch");

    if let Some(ref description) = draft.description {
        assert_eq!(
            event.description().as_deref(),
            Some(description.as_str()),
            "Description mismatch"
        );
    } else {
        assert!(event.description().is_none(), "Expected no description");
    }

    if let Some(ref start) = draft.start {
        assert_eq!(event.start(), Some(start.clone()), "Start mismatch");
    } else {
        // Default draft should have set start
        assert!(event.start().is_some(), "Start should be set");
    }

    if let Some(ref end) = draft.end {
        assert_eq!(event.end(), Some(end.clone()), "End mismatch");
    } else {
        // Default draft should have set end
        assert!(event.end().is_some(), "End should be set");
    }
}

/// Asserts that a todo matches the expected values from a draft.
///
/// # Arguments
///
/// * `todo` - The todo to validate
/// * `summary` - Expected summary
///
/// # Panics
///
/// Panics if the todo's summary doesn't match the expected value.
pub fn assert_todo_matches_draft<T: Todo>(todo: &T, summary: &str) {
    assert_eq!(todo.summary().as_ref(), summary, "Todo summary mismatch");
}

/// Asserts that a todo matches all fields from a draft.
///
/// # Arguments
///
/// * `todo` - The todo to validate
/// * `draft` - The draft with expected values
///
/// # Panics
///
/// Panics if any field doesn't match the expected value.
#[allow(dead_code)]
pub fn assert_todo_matches_draft_full<T: Todo>(todo: &T, draft: &TodoDraft) {
    assert_eq!(todo.summary().as_ref(), &draft.summary, "Summary mismatch");

    if let Some(ref description) = draft.description {
        assert_eq!(
            todo.description().as_deref(),
            Some(description.as_str()),
            "Description mismatch"
        );
    } else {
        assert!(todo.description().is_none(), "Expected no description");
    }

    if let Some(ref due) = draft.due {
        assert_eq!(todo.due(), Some(due.clone()), "Due mismatch");
    }

    if let Some(priority) = draft.priority {
        assert_eq!(todo.priority(), priority, "Priority mismatch");
    }
}

/// Asserts that a file exists at the given path.
///
/// # Arguments
///
/// * `path` - Path to check
///
/// # Panics
///
/// Panics if the file doesn't exist.
pub fn assert_file_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(path.exists(), "File does not exist: {}", path.display());
}

/// Asserts that a file does NOT exist at the given path.
///
/// # Arguments
///
/// * `path` - Path to check
///
/// # Panics
///
/// Panics if the file exists.
pub fn assert_file_not_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(!path.exists(), "File should not exist: {}", path.display());
}

/// Asserts that two optional datetimes are equal within a tolerance.
///
/// This is useful for comparing datetimes that may have slight differences
/// due to rounding or timezone conversions.
///
/// # Arguments
///
/// * `actual` - The actual datetime value
/// * `expected` - The expected datetime value
/// * `_tolerance_seconds` - Allowed difference in seconds (currently unused)
///
/// # Panics
///
/// Panics if the values differ.
pub fn assert_datetime_approx(
    actual: Option<&LooseDateTime>,
    expected: Option<&LooseDateTime>,
    _tolerance_seconds: i64,
) {
    match (actual, expected) {
        (None, None) => {}
        (Some(_), None) => panic!("Expected None, got Some"),
        (None, Some(_)) => panic!("Expected Some, got None"),
        (Some(actual_dt), Some(expected_dt)) => {
            // For loose datetime comparison, use exact comparison
            assert_eq!(actual_dt, expected_dt, "LooseDateTime mismatch");
        }
    }
}

/// Asserts that a collection has the expected length.
///
/// # Arguments
///
/// * `collection` - The collection to check
/// * `expected_len` - Expected length
///
/// # Panics
///
/// Panics if the length doesn't match.
pub fn assert_len<T>(collection: &[T], expected_len: usize) {
    assert_eq!(
        collection.len(),
        expected_len,
        "Expected {} items, got {}",
        expected_len,
        collection.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    use jiff::civil::date;

    // Simple test event implementation for assertions testing
    struct TestEvent {
        uid: String,
        summary: String,
    }

    impl Event for TestEvent {
        fn uid(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.uid)
        }

        fn summary(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.summary)
        }

        fn description(&self) -> Option<Cow<'_, str>> {
            None
        }

        fn start(&self) -> Option<LooseDateTime> {
            None
        }

        fn end(&self) -> Option<LooseDateTime> {
            None
        }

        fn status(&self) -> Option<EventStatus> {
            None
        }
    }

    // Simple test todo implementation for assertions testing
    struct TestTodo {
        uid: String,
        summary: String,
    }

    impl Todo for TestTodo {
        fn uid(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.uid)
        }

        fn summary(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.summary)
        }

        fn description(&self) -> Option<Cow<'_, str>> {
            None
        }

        fn due(&self) -> Option<LooseDateTime> {
            None
        }

        fn completed(&self) -> Option<jiff::Zoned> {
            None
        }

        fn percent_complete(&self) -> Option<u8> {
            None
        }

        fn priority(&self) -> Priority {
            Priority::None
        }

        fn status(&self) -> TodoStatus {
            TodoStatus::NeedsAction
        }
    }

    #[test]
    fn test_assert_event_matches_draft() {
        let event = TestEvent {
            uid: "uid-1".to_string(),
            summary: "Meeting".to_string(),
        };
        assert_event_matches_draft(&event, "Meeting");
    }

    #[test]
    #[should_panic(expected = "Event summary mismatch")]
    fn test_assert_event_matches_draft_panics_on_mismatch() {
        let event = TestEvent {
            uid: "uid-1".to_string(),
            summary: "Meeting".to_string(),
        };
        assert_event_matches_draft(&event, "Wrong");
    }

    #[test]
    fn test_assert_todo_matches_draft() {
        let todo = TestTodo {
            uid: "uid-1".to_string(),
            summary: "Task".to_string(),
        };
        assert_todo_matches_draft(&todo, "Task");
    }

    #[test]
    #[should_panic(expected = "Todo summary mismatch")]
    fn test_assert_todo_matches_draft_panics_on_mismatch() {
        let todo = TestTodo {
            uid: "uid-1".to_string(),
            summary: "Task".to_string(),
        };
        assert_todo_matches_draft(&todo, "Wrong");
    }

    #[test]
    fn test_assert_file_exists_with_existing_file() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        assert_file_exists(temp.path());
    }

    #[test]
    #[should_panic(expected = "File does not exist")]
    fn test_assert_file_exists_panics_on_missing_file() {
        assert_file_exists("/nonexistent/path/that/does/not/exist.txt");
    }

    #[test]
    fn test_assert_file_not_exists_with_missing_file() {
        assert_file_not_exists("/nonexistent/path/that/does/not/exist.txt");
    }

    #[test]
    #[should_panic(expected = "File should not exist")]
    fn test_assert_file_not_exists_panics_on_existing_file() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        assert_file_not_exists(temp.path());
    }

    #[test]
    fn test_assert_datetime_approx_with_equal_values() {
        let dt = LooseDateTime::Local(jiff::Zoned::now());
        assert_datetime_approx(Some(&dt), Some(&dt), 5);
    }

    #[test]
    fn test_assert_datetime_approx_within_tolerance() {
        let dt = LooseDateTime::Local(jiff::Zoned::now());
        // Exact comparison
        assert_datetime_approx(Some(&dt), Some(&dt), 5);
    }

    #[test]
    fn test_assert_datetime_approx_exceeds_tolerance() {
        let dt1 = LooseDateTime::Local(jiff::Zoned::now());
        let dt2 = LooseDateTime::Local(jiff::Zoned::now() + jiff::Span::new().seconds(10));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_datetime_approx(Some(&dt1), Some(&dt2), 5);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_assert_datetime_approx_with_none() {
        assert_datetime_approx(None::<&LooseDateTime>, None, 5);
    }

    #[test]
    #[should_panic(expected = "Expected Some, got None")]
    fn test_assert_datetime_approx_panics_on_none_some() {
        assert_datetime_approx(None, Some(&LooseDateTime::DateOnly(date(2025, 1, 1))), 5);
    }

    #[test]
    fn test_assert_len_with_matching_length() {
        let items = vec![1, 2, 3, 4, 5];
        assert_len(&items, 5);
    }

    #[test]
    #[should_panic(expected = "Expected 10 items, got 5")]
    fn test_assert_len_panics_on_mismatch() {
        let items = vec![1, 2, 3, 4, 5];
        assert_len(&items, 10);
    }
}
