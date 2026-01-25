// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! TodoStatus integration tests.
//!
//! Tests TodoStatus conversions and Display formatting.

use aimcal_core::TodoStatus;
use aimcal_ical::TodoStatusValue;

#[test]
fn todo_status_default_is_needs_action() {
    let status = TodoStatus::default();
    assert_eq!(status, TodoStatus::NeedsAction);
}

#[test]
fn todo_status_as_ref_returns_correct_strings() {
    assert_eq!(TodoStatus::NeedsAction.as_ref(), "NEEDS-ACTION");
    assert_eq!(TodoStatus::Completed.as_ref(), "COMPLETED");
    assert_eq!(TodoStatus::InProcess.as_ref(), "IN-PROGRESS");
    assert_eq!(TodoStatus::Cancelled.as_ref(), "CANCELLED");
}

#[test]
fn todo_status_display_returns_correct_strings() {
    assert_eq!(TodoStatus::NeedsAction.to_string(), "NEEDS-ACTION");
    assert_eq!(TodoStatus::Completed.to_string(), "COMPLETED");
    assert_eq!(TodoStatus::InProcess.to_string(), "IN-PROGRESS");
    assert_eq!(TodoStatus::Cancelled.to_string(), "CANCELLED");
}

#[test]
fn todo_status_from_str_parses_all_variants() {
    assert_eq!(
        "NEEDS-ACTION".parse::<TodoStatus>().unwrap(),
        TodoStatus::NeedsAction
    );
    assert_eq!(
        "COMPLETED".parse::<TodoStatus>().unwrap(),
        TodoStatus::Completed
    );
    assert_eq!(
        "IN-PROGRESS".parse::<TodoStatus>().unwrap(),
        TodoStatus::InProcess
    );
    assert_eq!(
        "CANCELLED".parse::<TodoStatus>().unwrap(),
        TodoStatus::Cancelled
    );
}

#[test]
fn todo_status_from_str_returns_error_for_invalid() {
    assert!("INVALID".parse::<TodoStatus>().is_err());
    assert!("".parse::<TodoStatus>().is_err());
    assert!("needs-action".parse::<TodoStatus>().is_err()); // lowercase
    assert!("completed".parse::<TodoStatus>().is_err()); // lowercase
}

#[test]
fn todo_status_from_ical_value_converts_correctly() {
    assert_eq!(
        TodoStatus::from(TodoStatusValue::NeedsAction),
        TodoStatus::NeedsAction
    );
    assert_eq!(
        TodoStatus::from(TodoStatusValue::Completed),
        TodoStatus::Completed
    );
    assert_eq!(
        TodoStatus::from(TodoStatusValue::InProcess),
        TodoStatus::InProcess
    );
    assert_eq!(
        TodoStatus::from(TodoStatusValue::Cancelled),
        TodoStatus::Cancelled
    );
}

#[test]
fn todo_status_to_ical_value_converts_correctly() {
    assert_eq!(
        aimcal_ical::TodoStatusValue::from(TodoStatus::NeedsAction),
        TodoStatusValue::NeedsAction
    );
    assert_eq!(
        aimcal_ical::TodoStatusValue::from(TodoStatus::Completed),
        TodoStatusValue::Completed
    );
    assert_eq!(
        aimcal_ical::TodoStatusValue::from(TodoStatus::InProcess),
        TodoStatusValue::InProcess
    );
    assert_eq!(
        aimcal_ical::TodoStatusValue::from(TodoStatus::Cancelled),
        TodoStatusValue::Cancelled
    );
}

#[test]
fn todo_status_roundtrip_through_ical() {
    for status in [
        TodoStatus::NeedsAction,
        TodoStatus::Completed,
        TodoStatus::InProcess,
        TodoStatus::Cancelled,
    ] {
        let ical_value = TodoStatusValue::from(status);
        let converted = TodoStatus::from(ical_value);
        assert_eq!(
            converted, status,
            "Roundtrip conversion should preserve status"
        );
    }
}

#[test]
fn todo_status_display_matches_as_ref() {
    assert_eq!(
        TodoStatus::NeedsAction.to_string(),
        TodoStatus::NeedsAction.as_ref()
    );
    assert_eq!(
        TodoStatus::Completed.to_string(),
        TodoStatus::Completed.as_ref()
    );
    assert_eq!(
        TodoStatus::InProcess.to_string(),
        TodoStatus::InProcess.as_ref()
    );
    assert_eq!(
        TodoStatus::Cancelled.to_string(),
        TodoStatus::Cancelled.as_ref()
    );
}

#[test]
fn todo_status_all_variants_have_unique_strings() {
    let strings = [
        TodoStatus::NeedsAction.as_ref().to_string(),
        TodoStatus::Completed.as_ref().to_string(),
        TodoStatus::InProcess.as_ref().to_string(),
        TodoStatus::Cancelled.as_ref().to_string(),
    ];

    assert_eq!(strings.len(), 4, "All status strings should be unique");

    let unique: Vec<_> = strings.iter().collect();
    assert_eq!(
        unique.len(),
        4,
        "All status strings should be unique after deduplication"
    );
}

#[test]
fn todo_status_serialization_symmetry() {
    for status in [
        TodoStatus::NeedsAction,
        TodoStatus::Completed,
        TodoStatus::InProcess,
        TodoStatus::Cancelled,
    ] {
        let as_str = status.as_ref();
        let parsed = as_str.parse::<TodoStatus>().unwrap();
        assert_eq!(parsed, status, "Parse should return original status");
    }
}

#[test]
fn todo_status_const_values_match_rfc_5545() {
    // RFC 5545 specifies these exact status values for VTODO
    assert_eq!(TodoStatus::NeedsAction.as_ref(), "NEEDS-ACTION");
    assert_eq!(TodoStatus::Completed.as_ref(), "COMPLETED");
    assert_eq!(TodoStatus::InProcess.as_ref(), "IN-PROGRESS");
    assert_eq!(TodoStatus::Cancelled.as_ref(), "CANCELLED");
}
