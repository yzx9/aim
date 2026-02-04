// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Test data factories for integration tests.
//!
//! This module provides helper functions to create test data including
//! configurations, drafts, and sample .ics file contents.

use std::path::{Path, PathBuf};

use aimcal_core::{
    Config, DateTimeAnchor, EventDraft, EventStatus, LooseDateTime, Priority, TodoDraft, TodoStatus,
};

/// Creates a test configuration with temporary directories.
///
/// # Arguments
///
/// * `calendar_path` - Path to the calendar directory
/// * `state_dir` - Optional path to the state directory
///
/// # Example
///
/// ```ignore
/// let config = test_config("/tmp/calendar", Some("/tmp/state"));
/// ```
#[must_use]
pub fn test_config(calendar_path: &str, state_dir: Option<&str>) -> Config {
    Config {
        calendar_path: Some(PathBuf::from(calendar_path)),
        state_dir: state_dir.map(PathBuf::from),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    }
}

/// Creates a test configuration with default due date.
///
/// # Arguments
///
/// * `calendar_path` - Path to the calendar directory
/// * `state_dir` - Optional path to the state directory
/// * `default_due` - Default due date for todos
#[must_use]
pub fn test_config_with_due(
    calendar_path: &str,
    state_dir: Option<&str>,
    default_due: DateTimeAnchor,
) -> Config {
    Config {
        calendar_path: Some(PathBuf::from(calendar_path)),
        state_dir: state_dir.map(PathBuf::from),
        default_due: Some(default_due),
        default_priority: Priority::None,
        default_priority_none_fist: false,
    }
}

/// Creates a test configuration with all default values.
#[must_use]
pub fn test_config_defaults() -> Config {
    Config {
        calendar_path: Some(PathBuf::from("/tmp/test-calendar")),
        state_dir: Some(PathBuf::from("/tmp/test-state")),
        default_due: Some(DateTimeAnchor::InDays(1)),
        default_priority: Priority::P5,
        default_priority_none_fist: true,
    }
}

/// Creates a test event draft with the given summary.
///
/// # Arguments
///
/// * `summary` - Event summary
///
/// # Example
///
/// ```ignore
/// let draft = test_event_draft("Team Meeting");
/// ```
#[must_use]
pub fn test_event_draft(summary: &str) -> EventDraft {
    EventDraft {
        description: None,
        start: None,
        end: None,
        status: EventStatus::Confirmed,
        summary: summary.to_string(),
    }
}

/// Creates a test event draft with all fields.
///
/// # Arguments
///
/// * `summary` - Event summary
/// * `description` - Event description
/// * `start` - Start datetime
/// * `end` - End datetime
#[must_use]
pub fn test_event_draft_full(
    summary: &str,
    description: &str,
    start: LooseDateTime,
    end: LooseDateTime,
) -> EventDraft {
    EventDraft {
        description: Some(description.to_string()),
        start: Some(start),
        end: Some(end),
        status: EventStatus::Confirmed,
        summary: summary.to_string(),
    }
}

/// Creates a test todo draft with the given summary.
///
/// # Arguments
///
/// * `summary` - Todo summary
#[must_use]
pub fn test_todo_draft(summary: &str) -> TodoDraft {
    TodoDraft {
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: summary.to_string(),
    }
}

/// Creates a test todo draft with all fields.
///
/// # Arguments
///
/// * `summary` - Todo summary
/// * `description` - Todo description
/// * `due` - Due datetime
/// * `priority` - Priority level
#[must_use]
pub fn test_todo_draft_full(
    summary: &str,
    description: &str,
    due: LooseDateTime,
    priority: Priority,
) -> TodoDraft {
    TodoDraft {
        description: Some(description.to_string()),
        due: Some(due),
        percent_complete: None,
        priority: Some(priority),
        status: TodoStatus::NeedsAction,
        summary: summary.to_string(),
    }
}

/// Builder for creating test configurations with guaranteed temporary directories.
///
/// This builder ensures that `state_dir` is always provided, preventing tests
/// from accidentally creating files in the user's real state directory.
///
/// # Example
///
/// ```ignore
/// let temp_dirs = setup_temp_dirs().await.unwrap();
/// let config = TestConfigBuilder::new()
///     .with_calendar_path(&temp_dirs.calendar_path)
///     .with_state_dir(&temp_dirs.state_dir)
///     .build();
/// ```
#[must_use]
#[allow(dead_code)]
pub struct TestConfigBuilder {
    calendar_path: Option<PathBuf>,
    state_dir: Option<PathBuf>,
    default_due: Option<DateTimeAnchor>,
    default_priority: Priority,
    default_priority_none_fist: bool,
}

impl TestConfigBuilder {
    /// Creates a new builder with default values.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            calendar_path: None,
            state_dir: None,
            default_due: None,
            default_priority: Priority::None,
            default_priority_none_fist: false,
        }
    }

    /// Sets the calendar path.
    pub fn with_calendar_path(mut self, path: &Path) -> Self {
        self.calendar_path = Some(path.to_path_buf());
        self
    }

    /// Sets the state directory (required).
    ///
    /// # Panics
    ///
    /// The `build()` method will panic if this is not called.
    pub fn with_state_dir(mut self, dir: &Path) -> Self {
        self.state_dir = Some(dir.to_path_buf());
        self
    }

    /// Sets the default due date for todos.
    #[allow(dead_code)]
    pub fn with_default_due(mut self, due: DateTimeAnchor) -> Self {
        self.default_due = Some(due);
        self
    }

    /// Sets the default priority for todos.
    #[allow(dead_code)]
    pub fn with_default_priority(mut self, priority: Priority) -> Self {
        self.default_priority = priority;
        self
    }

    /// Sets the priority sorting behavior (none first).
    #[allow(dead_code)]
    pub fn with_priority_none_first(mut self, none_first: bool) -> Self {
        self.default_priority_none_fist = none_first;
        self
    }

    /// Builds the configuration.
    ///
    /// # Panics
    ///
    /// Panics if `calendar_path` or `state_dir` are not set.
    #[track_caller]
    pub fn build(self) -> Config {
        let calendar_path = self.calendar_path.expect(
            "calendar_path must be set before building test config. \
             Use with_calendar_path() to set it.",
        );
        let state_dir = self.state_dir.expect(
            "state_dir must be set before building test config. \
             Use with_state_dir() to set it. \
             This prevents tests from creating files in the real state directory.",
        );

        Config {
            calendar_path: Some(calendar_path),
            state_dir: Some(state_dir),
            default_due: self.default_due,
            default_priority: self.default_priority,
            default_priority_none_fist: self.default_priority_none_fist,
        }
    }
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a test configuration from temporary directories.
///
/// This is a convenience wrapper around [`TestConfigBuilder`] for the most
/// common case: using temporary directories for both calendar and state.
///
/// # Example
///
/// ```ignore
/// let temp_dirs = setup_temp_dirs().await.unwrap();
/// let config = test_config_from_dirs(&temp_dirs.calendar_path, &temp_dirs.state_dir);
/// ```
#[must_use]
#[allow(dead_code)]
pub fn test_config_from_dirs(calendar_path: &Path, state_dir: &Path) -> Config {
    TestConfigBuilder::new()
        .with_calendar_path(calendar_path)
        .with_state_dir(state_dir)
        .build()
}

/// Returns sample iCalendar content for a single event.
///
/// This can be used to create test .ics files.
///
/// # Example
///
/// ```ignore
/// let content = sample_event_ics("event-uid", "Team Meeting", "2025-01-15");
/// ```
#[must_use]
pub fn sample_event_ics(uid: &str, summary: &str, date: &str) -> String {
    format!(
        r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//AIM//Test//EN
BEGIN:VEVENT
UID:{uid}
DTSTAMP:{date}T120000Z
DTSTART:{date}T100000Z
DTEND:{date}T110000Z
SUMMARY:{summary}
END:VEVENT
END:VCALENDAR
"#
    )
}

/// Returns sample iCalendar content for a single todo.
///
/// This can be used to create test .ics files.
///
/// # Example
///
/// ```ignore
/// let content = sample_todo_ics("todo-uid", "Buy groceries", "2025-01-15");
/// ```
#[must_use]
pub fn sample_todo_ics(uid: &str, summary: &str, due: &str) -> String {
    format!(
        r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//AIM//Test//EN
BEGIN:VTODO
UID:{uid}
DTSTAMP:{due}T120000Z
DUE:{due}T100000Z
SUMMARY:{summary}
STATUS:NEEDS-ACTION
END:VTODO
END:VCALENDAR
"#
    )
}

/// Returns sample iCalendar content with multiple components.
///
/// Creates a calendar with both an event and a todo.
#[must_use]
pub fn sample_calendar_content() -> String {
    r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//AIM//Test//EN
BEGIN:VEVENT
UID:event-123
DTSTAMP:20250115T120000Z
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Team Meeting
DESCRIPTION:Weekly team sync
END:VEVENT
BEGIN:VTODO
UID:todo-456
DTSTAMP:20250115T120000Z
DUE:20250116T100000Z
SUMMARY:Review PRs
PRIORITY:5
STATUS:NEEDS-ACTION
END:VTODO
END:VCALENDAR
"#
    .to_string()
}

/// Returns sample minimal iCalendar content.
///
/// Creates a minimal valid calendar with just an event.
#[must_use]
pub fn sample_minimal_calendar() -> String {
    r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//AIM//Test//EN
BEGIN:VEVENT
UID:minimal-event
DTSTAMP:20250115T120000Z
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Minimal Event
END:VEVENT
END:VCALENDAR
"#
    .to_string()
}

/// Returns invalid iCalendar content for negative testing.
#[must_use]
pub fn sample_invalid_ics() -> String {
    r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//AIM//Test//EN
BEGIN:VEVENT
UID:invalid-event
DTSTAMP:20250115T120000Z
SUMMARY:Invalid Event - Missing DTSTART
END:VEVENT
END:VCALENDAR
"#
    .to_string()
}

/// Returns empty iCalendar content for negative testing.
#[must_use]
pub fn sample_empty_ics() -> String {
    "".to_string()
}

/// Returns iCalendar content without any components.
#[must_use]
pub fn sample_calendar_no_components() -> String {
    r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//AIM//Test//EN
END:VCALENDAR
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creates_config() {
        let config = test_config("/cal", Some("/state"));

        assert_eq!(config.calendar_path, Some(PathBuf::from("/cal")));
        assert_eq!(config.state_dir, Some(PathBuf::from("/state")));
        assert!(config.default_due.is_none());
        assert_eq!(config.default_priority, Priority::None);
        assert!(!config.default_priority_none_fist);
    }

    #[test]
    fn test_config_with_due_includes_default_due() {
        let config = test_config_with_due("/cal", Some("/state"), DateTimeAnchor::InDays(7));

        assert_eq!(config.default_due, Some(DateTimeAnchor::InDays(7)));
    }

    #[test]
    fn test_config_defaults_has_all_values() {
        let config = test_config_defaults();

        assert_eq!(
            config.calendar_path,
            Some(PathBuf::from("/tmp/test-calendar"))
        );
        assert_eq!(config.state_dir, Some(PathBuf::from("/tmp/test-state")));
        assert_eq!(config.default_due, Some(DateTimeAnchor::InDays(1)));
        assert_eq!(config.default_priority, Priority::P5);
        assert!(config.default_priority_none_fist);
    }

    #[test]
    fn test_event_draft_creates_basic_draft() {
        let draft = test_event_draft("Meeting");

        assert_eq!(draft.summary, "Meeting");
        assert!(draft.description.is_none());
        assert!(draft.start.is_none());
        assert!(draft.end.is_none());
    }

    #[test]
    fn test_event_draft_full_creates_complete_draft() {
        let start = LooseDateTime::Local(jiff::Zoned::now());
        let end = LooseDateTime::Local(jiff::Zoned::now() + jiff::Span::new().hours(1));
        let draft = test_event_draft_full("Meeting", "Team sync", start.clone(), end.clone());

        assert_eq!(draft.summary, "Meeting");
        assert_eq!(draft.description.as_ref().unwrap(), "Team sync");
        assert_eq!(draft.start.as_ref().unwrap(), &start);
        assert_eq!(draft.end.as_ref().unwrap(), &end);
    }

    #[test]
    fn test_todo_draft_creates_basic_draft() {
        let draft = test_todo_draft("Task");

        assert_eq!(draft.summary, "Task");
        assert!(draft.description.is_none());
        assert!(draft.due.is_none());
        assert!(draft.priority.is_none());
    }

    #[test]
    fn test_todo_draft_full_creates_complete_draft() {
        let due = LooseDateTime::Local(jiff::Zoned::now());
        let draft = test_todo_draft_full("Task", "Complete assignment", due.clone(), Priority::P2);

        assert_eq!(draft.summary, "Task");
        assert_eq!(draft.description.as_ref().unwrap(), "Complete assignment");
        assert_eq!(draft.due.as_ref().unwrap(), &due);
        assert_eq!(draft.priority.unwrap(), Priority::P2);
    }

    #[test]
    fn sample_event_ics_produces_valid_format() {
        let content = sample_event_ics("uid-123", "My Event", "20250115");

        assert!(content.contains("UID:uid-123"));
        assert!(content.contains("SUMMARY:My Event"));
        assert!(content.contains("DTSTART:20250115"));
        assert!(content.contains("DTEND:20250115"));
        assert!(content.contains("BEGIN:VEVENT"));
        assert!(content.contains("END:VEVENT"));
    }

    #[test]
    fn sample_todo_ics_produces_valid_format() {
        let content = sample_todo_ics("uid-456", "My Todo", "20250116");

        assert!(content.contains("UID:uid-456"));
        assert!(content.contains("SUMMARY:My Todo"));
        assert!(content.contains("DUE:20250116"));
        assert!(content.contains("BEGIN:VTODO"));
        assert!(content.contains("END:VTODO"));
    }

    #[test]
    fn sample_calendar_content_contains_both_components() {
        let content = sample_calendar_content();

        assert!(content.contains("BEGIN:VEVENT"));
        assert!(content.contains("UID:event-123"));
        assert!(content.contains("SUMMARY:Team Meeting"));
        assert!(content.contains("BEGIN:VTODO"));
        assert!(content.contains("UID:todo-456"));
        assert!(content.contains("SUMMARY:Review PRs"));
    }

    #[test]
    fn sample_minimal_calendar_has_single_event() {
        let content = sample_minimal_calendar();

        assert!(content.contains("BEGIN:VEVENT"));
        assert!(content.contains("UID:minimal-event"));
        assert!(content.contains("SUMMARY:Minimal Event"));
        assert!(!content.contains("BEGIN:VTODO"));
    }

    #[test]
    fn sample_invalid_ics_missing_dtstart() {
        let content = sample_invalid_ics();

        assert!(content.contains("BEGIN:VEVENT"));
        assert!(content.contains("UID:invalid-event"));
        // Check that DTSTART property line doesn't exist (DTSTAMP is OK)
        assert!(!content.contains("\nDTSTART:"));
    }

    #[test]
    fn sample_empty_ics_returns_empty_string() {
        let content = sample_empty_ics();
        assert!(content.is_empty());
    }

    #[test]
    fn sample_calendar_no_components_has_calendar_only() {
        let content = sample_calendar_no_components();

        assert!(content.contains("BEGIN:VCALENDAR"));
        assert!(content.contains("END:VCALENDAR"));
        assert!(!content.contains("BEGIN:VEVENT"));
        assert!(!content.contains("BEGIN:VTODO"));
    }
}
