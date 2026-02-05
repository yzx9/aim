// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Test utilities for the db module.
//!
//! This module provides helper functions and test doubles for unit testing
//! the db module components (events, todos, `` `short_ids` ``).

use std::borrow::Cow;

use crate::db::Db;
use crate::{Event, EventStatus, LooseDateTime, Priority, Todo, TodoStatus};

/// Creates an in-memory test database.
///
/// This function creates a new in-memory `` `SQLite` `` database, runs all migrations,
/// and returns a `Db` instance ready for testing.
///
/// # Example
///
/// ```ignore
/// let db = setup_test_db().await;
/// // Use db for testing...
/// ```
pub async fn setup_test_db() -> Db {
    Db::open(None)
        .await
        .expect("Failed to create test database")
}

/// A test implementation of the `Event` trait.
///
/// This struct is used for testing database operations without depending on
/// the full `VEvent` implementation from `` `aimcal_ical` ``.
#[derive(Debug, Clone)]
pub struct TestEvent {
    /// The unique identifier for the event.
    pub uid: String,
    /// The summary of the event.
    pub summary: String,
    /// The description of the event, if available.
    pub description: Option<String>,
    /// The start date and time of the event.
    pub start: Option<LooseDateTime>,
    /// The end date and time of the event.
    pub end: Option<LooseDateTime>,
    /// The status of the event.
    pub status: Option<EventStatus>,
}

impl TestEvent {
    /// Creates a new test event with the given UID and summary.
    ///
    /// # Example
    ///
    /// ```
    /// let event = test_event("event-1", "Test Event");
    /// ```
    pub fn new(uid: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            uid: uid.into(),
            summary: summary.into(),
            description: None,
            start: None,
            end: None,
            status: None,
        }
    }

    /// Sets the description for the test event.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the start datetime for the test event.
    pub fn with_start(mut self, start: LooseDateTime) -> Self {
        self.start = Some(start);
        self
    }

    /// Sets the end datetime for the test event.
    pub fn with_end(mut self, end: LooseDateTime) -> Self {
        self.end = Some(end);
        self
    }

    /// Sets the status for the test event.
    pub fn with_status(mut self, status: EventStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl Event for TestEvent {
    fn uid(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.uid)
    }

    fn summary(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.summary)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.description.as_deref().map(Cow::Borrowed)
    }

    fn start(&self) -> Option<LooseDateTime> {
        self.start.clone()
    }

    fn end(&self) -> Option<LooseDateTime> {
        self.end.clone()
    }

    fn status(&self) -> Option<EventStatus> {
        self.status
    }
}

/// Creates a test event with the given UID and summary.
///
/// # Example
///
/// ```
/// let event = test_event("event-1", "Test Event");
/// ```
pub fn test_event(uid: impl Into<String>, summary: impl Into<String>) -> TestEvent {
    TestEvent::new(uid, summary)
}

/// A test implementation of the `Todo` trait.
///
/// This struct is used for testing database operations without depending on
/// the full `VTodo` implementation from `` `aimcal_ical` ``.
#[derive(Debug, Clone)]
pub struct TestTodo {
    /// The unique identifier for the todo.
    pub uid: String,
    /// The summary of the todo.
    pub summary: String,
    /// The description of the todo, if available.
    pub description: Option<String>,
    /// The due date and time of the todo.
    pub due: Option<LooseDateTime>,
    /// The completion datetime of the todo.
    pub completed: Option<jiff::Zoned>,
    /// The percent complete, from 0 to 100.
    pub percent_complete: Option<u8>,
    /// The priority from 1 to 9.
    pub priority: Priority,
    /// The status of the todo.
    pub status: TodoStatus,
}

impl TestTodo {
    /// Creates a new test todo with the given UID and summary.
    ///
    /// # Example
    ///
    /// ```
    /// let todo = test_todo("todo-1", "Test Todo");
    /// ```
    pub fn new(uid: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            uid: uid.into(),
            summary: summary.into(),
            description: None,
            due: None,
            completed: None,
            percent_complete: None,
            priority: Priority::default(),
            status: TodoStatus::default(),
        }
    }

    /// Sets the description for the test todo.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the due datetime for the test todo.
    pub fn with_due(mut self, due: LooseDateTime) -> Self {
        self.due = Some(due);
        self
    }

    /// Sets the completed datetime for the test todo.
    #[allow(dead_code)]
    pub fn with_completed(mut self, completed: jiff::Zoned) -> Self {
        self.completed = Some(completed);
        self
    }

    /// Sets the percent complete for the test todo.
    pub fn with_percent_complete(mut self, percent: u8) -> Self {
        self.percent_complete = Some(percent);
        self
    }

    /// Sets the priority for the test todo.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the status for the test todo.
    pub fn with_status(mut self, status: TodoStatus) -> Self {
        self.status = status;
        self
    }
}

impl Todo for TestTodo {
    fn uid(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.uid)
    }

    fn summary(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.summary)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.description.as_deref().map(Cow::Borrowed)
    }

    fn due(&self) -> Option<LooseDateTime> {
        self.due.clone()
    }

    fn completed(&self) -> Option<jiff::Zoned> {
        self.completed.clone()
    }

    fn percent_complete(&self) -> Option<u8> {
        self.percent_complete
    }

    fn priority(&self) -> Priority {
        self.priority
    }

    fn status(&self) -> TodoStatus {
        self.status
    }
}

/// Creates a test todo with the given UID and summary.
///
/// # Example
///
/// ```
/// let todo = test_todo("todo-1", "Test Todo");
/// ```
pub fn test_todo(uid: impl Into<String>, summary: impl Into<String>) -> TestTodo {
    TestTodo::new(uid, summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tests_utils_setup_creates_in_memory_database() {
        let db = setup_test_db().await;
        // Verify we can perform basic operations
        assert!(db.events.get("nonexistent").await.unwrap().is_none());
        assert!(db.todos.get("nonexistent").await.unwrap().is_none());
    }

    #[test]
    fn test_event_implements_event_trait() {
        let event = test_event("test-uid", "Test Summary")
            .with_description("Test description")
            .with_status(EventStatus::Confirmed);

        assert_eq!(event.uid(), "test-uid");
        assert_eq!(event.summary(), "Test Summary");
        assert_eq!(event.description(), Some(Cow::Borrowed("Test description")));
        assert_eq!(event.status(), Some(EventStatus::Confirmed));
    }

    #[test]
    fn test_todo_implements_todo_trait() {
        let todo = test_todo("test-uid", "Test Summary")
            .with_description("Test description")
            .with_priority(Priority::P2)
            .with_status(TodoStatus::InProcess);

        assert_eq!(todo.uid(), "test-uid");
        assert_eq!(todo.summary(), "Test Summary");
        assert_eq!(todo.description(), Some(Cow::Borrowed("Test description")));
        assert_eq!(todo.priority(), Priority::P2);
        assert_eq!(todo.status(), TodoStatus::InProcess);
    }
}
