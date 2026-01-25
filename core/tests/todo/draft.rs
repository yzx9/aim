// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! TodoDraft integration tests.
//!
//! Tests TodoDraft struct creation and field access (public API).
//! Note: Tests for resolve() and into_ics() are in src/todo.rs as unit tests
//! because they test pub(crate) methods.

use aimcal_core::{LooseDateTime, Priority, TodoDraft, TodoStatus};

#[test]
fn todo_draft_empty_fields_are_none_or_needs_action() {
    let draft = TodoDraft {
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };

    assert!(draft.description.is_none());
    assert!(draft.due.is_none());
    assert!(draft.percent_complete.is_none());
    assert!(draft.priority.is_none());
    assert_eq!(draft.status, TodoStatus::NeedsAction);
    assert!(draft.summary.is_empty());
}

#[test]
fn todo_draft_with_all_fields_populated() {
    let due = LooseDateTime::Local(jiff::Zoned::now());
    let draft = TodoDraft {
        description: Some("Test description".to_string()),
        due: Some(due.clone()),
        percent_complete: Some(50),
        priority: Some(Priority::P2),
        status: TodoStatus::InProcess,
        summary: "Test Todo".to_string(),
    };

    assert_eq!(draft.description.as_deref(), Some("Test description"));
    assert!(draft.due.is_some());
    assert_eq!(draft.percent_complete, Some(50));
    assert_eq!(draft.priority, Some(Priority::P2));
    assert_eq!(draft.status, TodoStatus::InProcess);
    assert_eq!(draft.summary, "Test Todo");
}

#[test]
fn todo_draft_can_be_created_with_builder_pattern() {
    let mut draft = TodoDraft {
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };

    draft.summary = "Builder Test".to_string();
    draft.description = Some("Builder description".to_string());
    draft.status = TodoStatus::InProcess;

    assert_eq!(draft.summary, "Builder Test");
    assert_eq!(draft.description.as_deref(), Some("Builder description"));
    assert_eq!(draft.status, TodoStatus::InProcess);
}

#[test]
fn todo_draft_status_can_be_all_variants() {
    let needs_action = TodoDraft {
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };

    let completed = TodoDraft {
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::Completed,
        summary: String::new(),
    };

    let in_process = TodoDraft {
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::InProcess,
        summary: String::new(),
    };

    let cancelled = TodoDraft {
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::Cancelled,
        summary: String::new(),
    };

    assert_eq!(needs_action.status, TodoStatus::NeedsAction);
    assert_eq!(completed.status, TodoStatus::Completed);
    assert_eq!(in_process.status, TodoStatus::InProcess);
    assert_eq!(cancelled.status, TodoStatus::Cancelled);
}

#[test]
fn todo_draft_priority_can_be_all_levels() {
    for priority in [
        Priority::None,
        Priority::P1,
        Priority::P2,
        Priority::P3,
        Priority::P4,
        Priority::P5,
        Priority::P6,
        Priority::P7,
        Priority::P8,
        Priority::P9,
    ] {
        let draft = TodoDraft {
            description: None,
            due: None,
            percent_complete: None,
            priority: Some(priority),
            status: TodoStatus::NeedsAction,
            summary: String::new(),
        };

        assert_eq!(draft.priority, Some(priority));
    }
}

#[test]
fn todo_draft_percent_complete_accepts_range() {
    let zero = TodoDraft {
        percent_complete: Some(0),
        description: None,
        due: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };
    let fifty = TodoDraft {
        percent_complete: Some(50),
        description: None,
        due: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };
    let hundred = TodoDraft {
        percent_complete: Some(100),
        description: None,
        due: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };

    assert_eq!(zero.percent_complete, Some(0));
    assert_eq!(fifty.percent_complete, Some(50));
    assert_eq!(hundred.percent_complete, Some(100));
}

#[test]
fn todo_draft_due_with_different_datetime_types() {
    let local = LooseDateTime::Local(jiff::Zoned::now());
    let draft1 = TodoDraft {
        due: Some(local.clone()),
        description: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };

    assert!(draft1.due.is_some());
    assert_eq!(draft1.due, Some(local));
}

#[test]
fn todo_draft_description_optional() {
    let with_desc = TodoDraft {
        description: Some("Has description".to_string()),
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };

    let without_desc = TodoDraft {
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };

    assert_eq!(with_desc.description.as_deref(), Some("Has description"));
    assert!(without_desc.description.is_none());
}
