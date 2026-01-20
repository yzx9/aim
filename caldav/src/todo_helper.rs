// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Helper functions for working with VTODO components.
//!
//! This module provides utilities for filtering and processing VTODO components.

use aimcal_ical::ICalendar;
use aimcal_ical::semantic::{CalendarComponent, TodoStatusValue, VTodo};

/// Extracts the first VTODO component from an iCalendar object.
#[must_use]
pub fn extract_first_todo(calendar: &ICalendar<String>) -> Option<&VTodo<String>> {
    calendar.components.iter().find_map(|comp| {
        if let CalendarComponent::Todo(todo) = comp {
            Some(todo)
        } else {
            None
        }
    })
}

/// Gets the status of a todo from an iCalendar object.
///
/// Returns `None` if the todo has no status property.
#[must_use]
pub fn get_todo_status(calendar: &ICalendar<String>) -> Option<TodoStatusValue> {
    extract_first_todo(calendar)
        .and_then(|todo| todo.status.as_ref())
        .map(|status| status.value)
}

/// Checks if a todo is pending (not completed).
///
/// A todo is pending if:
/// - It has no COMPLETED property, AND
/// - Its status is not COMPLETED or CANCELLED
#[must_use]
pub fn is_pending_todo(calendar: &ICalendar<String>) -> bool {
    if let Some(todo) = extract_first_todo(calendar) {
        // Check if completed
        if todo.completed.is_some() {
            return false;
        }

        // Check status
        if let Some(status) = todo.status.as_ref() {
            match status.value {
                TodoStatusValue::Completed | TodoStatusValue::Cancelled => return false,
                TodoStatusValue::NeedsAction | TodoStatusValue::InProcess => return true,
            }
        }

        // No completed property and no status means pending
        true
    } else {
        false
    }
}

/// Checks if a todo is completed.
///
/// A todo is completed if:
/// - It has a COMPLETED property, OR
/// - Its status is COMPLETED
#[must_use]
pub fn is_completed_todo(calendar: &ICalendar<String>) -> bool {
    if let Some(todo) = extract_first_todo(calendar) {
        // Check if completed
        if todo.completed.is_some() {
            return true;
        }

        // Check status
        if let Some(status) = todo.status.as_ref() {
            return matches!(status.value, TodoStatusValue::Completed);
        }

        false
    } else {
        false
    }
}

// TODO: Add unit tests for todo helper functions
// Tests should cover:
// - extract_first_todo() extraction and None cases
// - get_todo_status() with various status values
// - is_pending_todo() with all status/COMPLETED combinations
// - is_completed_todo() with all status/COMPLETED combinations
