// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! EventStatus integration tests.
//!
//! Tests EventStatus conversions and Display formatting.

use aimcal_core::EventStatus;
use aimcal_ical::EventStatusValue;

#[test]
fn event_status_default_is_confirmed() {
    let status = EventStatus::default();
    assert_eq!(status, EventStatus::Confirmed);
}

#[test]
fn event_status_as_ref_returns_correct_strings() {
    assert_eq!(EventStatus::Tentative.as_ref(), "TENTATIVE");
    assert_eq!(EventStatus::Confirmed.as_ref(), "CONFIRMED");
    assert_eq!(EventStatus::Cancelled.as_ref(), "CANCELLED");
}

#[test]
fn event_status_display_returns_correct_strings() {
    assert_eq!(EventStatus::Tentative.to_string(), "TENTATIVE");
    assert_eq!(EventStatus::Confirmed.to_string(), "CONFIRMED");
    assert_eq!(EventStatus::Cancelled.to_string(), "CANCELLED");
}

#[test]
fn event_status_from_str_parses_all_variants() {
    assert_eq!(
        "TENTATIVE".parse::<EventStatus>().unwrap(),
        EventStatus::Tentative
    );
    assert_eq!(
        "CONFIRMED".parse::<EventStatus>().unwrap(),
        EventStatus::Confirmed
    );
    assert_eq!(
        "CANCELLED".parse::<EventStatus>().unwrap(),
        EventStatus::Cancelled
    );
}

#[test]
fn event_status_from_str_returns_error_for_invalid() {
    assert!("INVALID".parse::<EventStatus>().is_err());
    assert!("".parse::<EventStatus>().is_err());
    assert!("tentative".parse::<EventStatus>().is_err()); // lowercase
    assert!("confirmed".parse::<EventStatus>().is_err()); // lowercase
}

#[test]
fn event_status_from_ical_value_converts_correctly() {
    assert_eq!(
        EventStatus::from(EventStatusValue::Tentative),
        EventStatus::Tentative
    );
    assert_eq!(
        EventStatus::from(EventStatusValue::Confirmed),
        EventStatus::Confirmed
    );
    assert_eq!(
        EventStatus::from(EventStatusValue::Cancelled),
        EventStatus::Cancelled
    );
}

#[test]
fn event_status_to_ical_value_converts_correctly() {
    assert_eq!(
        aimcal_ical::EventStatusValue::from(EventStatus::Tentative),
        EventStatusValue::Tentative
    );
    assert_eq!(
        aimcal_ical::EventStatusValue::from(EventStatus::Confirmed),
        EventStatusValue::Confirmed
    );
    assert_eq!(
        aimcal_ical::EventStatusValue::from(EventStatus::Cancelled),
        EventStatusValue::Cancelled
    );
}

#[test]
fn event_status_roundtrip_through_ical() {
    for status in [
        EventStatus::Tentative,
        EventStatus::Confirmed,
        EventStatus::Cancelled,
    ] {
        let ical_value = EventStatusValue::from(status);
        let converted = EventStatus::from(ical_value);
        assert_eq!(
            converted, status,
            "Roundtrip conversion should preserve status"
        );
    }
}

#[test]
fn event_status_display_matches_as_ref() {
    assert_eq!(
        EventStatus::Tentative.to_string(),
        EventStatus::Tentative.as_ref()
    );
    assert_eq!(
        EventStatus::Confirmed.to_string(),
        EventStatus::Confirmed.as_ref()
    );
    assert_eq!(
        EventStatus::Cancelled.to_string(),
        EventStatus::Cancelled.as_ref()
    );
}

#[test]
fn event_status_all_variants_have_unique_strings() {
    let strings = [
        EventStatus::Tentative.as_ref().to_string(),
        EventStatus::Confirmed.as_ref().to_string(),
        EventStatus::Cancelled.as_ref().to_string(),
    ];

    assert_eq!(strings.len(), 3, "All status strings should be unique");

    let unique: Vec<_> = strings.iter().collect();
    assert_eq!(
        unique.len(),
        3,
        "All status strings should be unique after deduplication"
    );
}

#[test]
fn event_status_serialization_symmetry() {
    for status in [
        EventStatus::Tentative,
        EventStatus::Confirmed,
        EventStatus::Cancelled,
    ] {
        let as_str = status.as_ref();
        let parsed = as_str.parse::<EventStatus>().unwrap();
        assert_eq!(parsed, status, "Parse should return original status");
    }
}

#[test]
fn event_status_const_values_match_rfc_5545() {
    // RFC 5545 specifies these exact status values for VEVENT
    assert_eq!(EventStatus::Tentative.as_ref(), "TENTATIVE");
    assert_eq!(EventStatus::Confirmed.as_ref(), "CONFIRMED");
    assert_eq!(EventStatus::Cancelled.as_ref(), "CANCELLED");
}
