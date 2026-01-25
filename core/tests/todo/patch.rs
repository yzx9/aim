// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! TodoPatch integration tests.
//!
//! Tests TodoPatch struct creation and is_empty() method (public API).
//! Note: Tests for resolve() and apply_to() are in src/todo.rs as unit tests
//! because they test pub(crate) methods.

use aimcal_core::{LooseDateTime, Priority, TodoPatch, TodoStatus};

#[test]
fn todo_patch_default_is_empty() {
    let patch = TodoPatch::default();

    assert!(patch.is_empty());
    assert!(patch.description.is_none());
    assert!(patch.due.is_none());
    assert!(patch.percent_complete.is_none());
    assert!(patch.priority.is_none());
    assert!(patch.status.is_none());
    assert!(patch.summary.is_none());
}

#[test]
fn todo_patch_with_description_set_is_not_empty() {
    let patch = TodoPatch {
        description: Some(Some("New description".to_string())),
        ..Default::default()
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_with_description_cleared_is_not_empty() {
    let patch = TodoPatch {
        description: Some(None),
        ..Default::default()
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_with_due_set_is_not_empty() {
    let due = LooseDateTime::Local(jiff::Zoned::now());
    let patch = TodoPatch {
        due: Some(Some(due)),
        ..Default::default()
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_with_due_cleared_is_not_empty() {
    let patch = TodoPatch {
        due: Some(None),
        ..Default::default()
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_with_percent_complete_set_is_not_empty() {
    let patch = TodoPatch {
        percent_complete: Some(Some(75)),
        ..Default::default()
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_with_percent_complete_cleared_is_not_empty() {
    let patch = TodoPatch {
        percent_complete: Some(None),
        ..Default::default()
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_with_priority_set_is_not_empty() {
    let patch = TodoPatch {
        priority: Some(Priority::P2),
        ..Default::default()
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_with_status_set_is_not_empty() {
    let patch = TodoPatch {
        status: Some(TodoStatus::Completed),
        ..Default::default()
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_with_summary_set_is_not_empty() {
    let patch = TodoPatch {
        summary: Some("Updated summary".to_string()),
        ..Default::default()
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_with_all_fields_set_is_not_empty() {
    let patch = TodoPatch {
        description: Some(Some("Description".to_string())),
        due: Some(Some(LooseDateTime::Local(jiff::Zoned::now()))),
        percent_complete: Some(Some(50)),
        priority: Some(Priority::P5),
        status: Some(TodoStatus::InProcess),
        summary: Some("Summary".to_string()),
    };

    assert!(!patch.is_empty());
}

#[test]
fn todo_patch_can_set_all_optional_fields_to_none() {
    let patch = TodoPatch {
        description: Some(None),
        due: Some(None),
        percent_complete: Some(None),
        priority: None,
        status: None,
        summary: None,
    };

    assert!(!patch.is_empty());
    assert_eq!(patch.description, Some(None));
    assert_eq!(patch.due, Some(None));
    assert_eq!(patch.percent_complete, Some(None));
}

#[test]
fn todo_patch_status_can_be_all_variants() {
    for status in [
        TodoStatus::NeedsAction,
        TodoStatus::Completed,
        TodoStatus::InProcess,
        TodoStatus::Cancelled,
    ] {
        let patch = TodoPatch {
            status: Some(status),
            ..Default::default()
        };

        assert_eq!(patch.status, Some(status));
        assert!(!patch.is_empty());
    }
}

#[test]
fn todo_patch_priority_can_be_all_levels() {
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
        let patch = TodoPatch {
            priority: Some(priority),
            ..Default::default()
        };

        assert_eq!(patch.priority, Some(priority));
        assert!(!patch.is_empty());
    }
}

#[test]
fn todo_patch_is_empty_detects_single_field_changes() {
    let fields = [
        TodoPatch {
            description: Some(Some("test".to_string())),
            ..Default::default()
        },
        TodoPatch {
            due: Some(Some(LooseDateTime::Local(jiff::Zoned::now()))),
            ..Default::default()
        },
        TodoPatch {
            percent_complete: Some(Some(50)),
            ..Default::default()
        },
        TodoPatch {
            priority: Some(Priority::P5),
            ..Default::default()
        },
        TodoPatch {
            status: Some(TodoStatus::Completed),
            ..Default::default()
        },
        TodoPatch {
            summary: Some("test".to_string()),
            ..Default::default()
        },
    ];

    for patch in fields {
        assert!(!patch.is_empty(), "Patch should not be empty");
    }
}

#[test]
fn todo_patch_clone_independence() {
    let patch1 = TodoPatch {
        summary: Some("Original".to_string()),
        ..Default::default()
    };

    let mut patch2 = patch1.clone();
    patch2.summary = Some("Modified".to_string());

    assert_eq!(patch1.summary.as_deref(), Some("Original"));
    assert_eq!(patch2.summary.as_deref(), Some("Modified"));
}
