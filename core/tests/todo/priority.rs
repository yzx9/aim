// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Priority handling integration tests.
//!
//! Tests Priority enum behavior in the context of Todo operations.

use aimcal_core::{Priority, TodoDraft, TodoPatch, TodoStatus};

#[test]
fn priority_default_is_none() {
    let priority = Priority::default();
    assert_eq!(priority, Priority::None);
}

#[test]
fn todo_draft_priority_can_be_none() {
    let draft = TodoDraft {
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::NeedsAction,
        summary: String::new(),
    };

    assert!(draft.priority.is_none());
}

#[test]
fn todo_draft_priority_can_be_set() {
    for priority in [Priority::None, Priority::P1, Priority::P5, Priority::P9] {
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
fn todo_patch_priority_can_be_set_to_any_level() {
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
    }
}

#[test]
fn priority_converts_to_u8_correctly() {
    let conversions: Vec<(Priority, u8)> = vec![
        (Priority::None, 0),
        (Priority::P1, 1),
        (Priority::P2, 2),
        (Priority::P3, 3),
        (Priority::P4, 4),
        (Priority::P5, 5),
        (Priority::P6, 6),
        (Priority::P7, 7),
        (Priority::P8, 8),
        (Priority::P9, 9),
    ];

    for (priority, expected) in conversions {
        let u8_value: u8 = priority.into();
        assert_eq!(
            u8_value, expected,
            "Priority {:?} should convert to {}",
            priority, expected
        );
    }
}

#[test]
fn priority_converts_from_u8_correctly() {
    let conversions: Vec<(u8, Priority)> = vec![
        (0, Priority::None),
        (1, Priority::P1),
        (2, Priority::P2),
        (3, Priority::P3),
        (4, Priority::P4),
        (5, Priority::P5),
        (6, Priority::P6),
        (7, Priority::P7),
        (8, Priority::P8),
        (9, Priority::P9),
    ];

    for (value, expected) in conversions {
        let priority = Priority::from(value);
        assert_eq!(
            priority, expected,
            "Value {} should convert to {:?}",
            value, expected
        );
    }
}

#[test]
fn priority_roundtrip_conversion() {
    for priority in [
        Priority::None,
        Priority::P1,
        Priority::P2,
        Priority::P5,
        Priority::P9,
    ] {
        let u8_value: u8 = priority.into();
        let converted = Priority::from(u8_value);
        assert_eq!(
            converted, priority,
            "Roundtrip conversion should preserve priority"
        );
    }
}

#[test]
fn priority_named_levels_match_standard_values() {
    // Common named priority levels
    assert_eq!(u8::from(Priority::None), 0);
    assert_eq!(u8::from(Priority::P1), 1); // High
    assert_eq!(u8::from(Priority::P5), 5); // Medium
    assert_eq!(u8::from(Priority::P8), 8); // Low
}
