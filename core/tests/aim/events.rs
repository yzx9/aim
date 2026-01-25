// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event CRUD operation tests for the Aim application.
//!
//! Tests creating, reading, updating, and listing events.

use aimcal_core::{
    Aim, Config, Event, EventConditions, EventDraft, EventPatch, EventStatus, Id, LooseDateTime,
    Pager, Priority,
};

use crate::common::{setup_temp_dirs, test_event_draft};

#[tokio::test]
async fn aim_new_event_creates_file_and_database_entry() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create event
    let draft = test_event_draft("New Meeting");
    let event = aim.new_event(draft).await.unwrap();

    // Verify event was created
    assert_eq!(event.summary().as_ref(), "New Meeting");
    assert!(event.short_id().is_some(), "Event should have short ID");

    // Verify .ics file was created
    let uid = event.uid().as_ref().to_string();
    let expected_path = temp_dirs.calendar_path.join(format!("{uid}.ics"));
    assert!(expected_path.exists(), ".ics file should be created");

    // Verify event can be retrieved
    let retrieved = aim.get_event(&Id::Uid(uid.clone())).await.unwrap();
    assert_eq!(retrieved.uid().as_ref(), uid);
    assert_eq!(retrieved.summary().as_ref(), "New Meeting");
}

#[tokio::test]
async fn aim_get_event_resolves_short_id() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create event
    let draft = test_event_draft("Test Event");
    let event = aim.new_event(draft).await.unwrap();
    let short_id = event.short_id().unwrap();
    let uid = event.uid().as_ref().to_string();

    // Retrieve by short ID
    let retrieved = aim
        .get_event(&Id::ShortIdOrUid(short_id.get().to_string()))
        .await
        .unwrap();
    assert_eq!(retrieved.uid().as_ref(), uid);
    assert_eq!(retrieved.short_id(), Some(short_id));
}

#[tokio::test]
async fn aim_update_event_modifies_file_and_database() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create event
    let draft = test_event_draft("Original Title");
    let event = aim.new_event(draft).await.unwrap();
    let uid = event.uid().as_ref().to_string();

    // Update event
    let patch = EventPatch {
        summary: Some("Updated Title".to_string()),
        description: Some(Some("New description".to_string())),
        ..Default::default()
    };
    let updated = aim
        .update_event(&Id::Uid(uid.clone()), patch)
        .await
        .unwrap();

    assert_eq!(updated.summary().as_ref(), "Updated Title");
    assert_eq!(updated.description().as_deref(), Some("New description"));

    // Verify update persisted
    let retrieved = aim.get_event(&Id::Uid(uid)).await.unwrap();
    assert_eq!(retrieved.summary().as_ref(), "Updated Title");
}

#[tokio::test]
async fn aim_list_events_returns_all_events() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create multiple events
    for i in 1..=3 {
        let draft = test_event_draft(&format!("Event {i}"));
        aim.new_event(draft).await.unwrap();
    }

    // List all events
    let events = aim
        .list_events(
            &EventConditions {
                startable: None,
                cutoff: None,
            },
            &Pager {
                limit: 100,
                offset: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(events.len(), 3);

    let summaries: Vec<_> = events
        .iter()
        .map(|e| e.summary().as_ref().to_string())
        .collect();
    assert!(summaries.contains(&"Event 1".to_string()));
    assert!(summaries.contains(&"Event 2".to_string()));
    assert!(summaries.contains(&"Event 3".to_string()));
}

#[tokio::test]
async fn aim_list_events_with_pagination() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create multiple events
    for i in 1..=5 {
        let draft = test_event_draft(&format!("Event {i}"));
        aim.new_event(draft).await.unwrap();
    }

    // List with pagination
    let page1 = aim
        .list_events(
            &EventConditions {
                startable: None,
                cutoff: None,
            },
            &Pager {
                limit: 2,
                offset: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(page1.len(), 2);

    let page2 = aim
        .list_events(
            &EventConditions {
                startable: None,
                cutoff: None,
            },
            &Pager {
                limit: 2,
                offset: 2,
            },
        )
        .await
        .unwrap();
    assert_eq!(page2.len(), 2);

    let page3 = aim
        .list_events(
            &EventConditions {
                startable: None,
                cutoff: None,
            },
            &Pager {
                limit: 2,
                offset: 4,
            },
        )
        .await
        .unwrap();
    assert_eq!(page3.len(), 1);
}

#[tokio::test]
async fn aim_count_events_returns_correct_count() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Initially no events
    let count = aim
        .count_events(&EventConditions {
            startable: None,
            cutoff: None,
        })
        .await
        .unwrap();
    assert_eq!(count, 0);

    // Create events
    for i in 1..=3 {
        let draft = test_event_draft(&format!("Event {i}"));
        aim.new_event(draft).await.unwrap();
    }

    // Count should match
    let count = aim
        .count_events(&EventConditions {
            startable: None,
            cutoff: None,
        })
        .await
        .unwrap();
    assert_eq!(count, 3);
}

#[tokio::test]
async fn aim_new_event_assigns_sequential_short_ids() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create multiple events
    let mut short_ids = Vec::new();
    for i in 1..=5 {
        let draft = test_event_draft(&format!("Event {i}"));
        let event = aim.new_event(draft).await.unwrap();
        short_ids.push(event.short_id().unwrap().get());
    }

    // Short IDs should be sequential (1, 2, 3, 4, 5)
    let mut sorted = short_ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn aim_update_event_clears_optional_fields() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create event with description
    let draft = EventDraft {
        description: Some("Original description".to_string()),
        ..test_event_draft("Test")
    };
    let event = aim.new_event(draft).await.unwrap();
    let uid = event.uid().as_ref().to_string();

    // Clear description
    let patch = EventPatch {
        description: Some(None), // Some(None) means clear the field
        ..Default::default()
    };
    let updated = aim
        .update_event(&Id::Uid(uid.clone()), patch)
        .await
        .unwrap();

    assert!(
        updated.description().is_none(),
        "Description should be cleared"
    );
}

#[tokio::test]
async fn aim_get_event_returns_error_for_nonexistent() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Try to get nonexistent event
    let result = aim.get_event(&Id::Uid("nonexistent-uid".to_string())).await;
    assert!(result.is_err(), "Should return error for nonexistent event");
}

#[tokio::test]
async fn aim_update_event_returns_error_for_nonexistent() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Try to update nonexistent event
    let patch = EventPatch {
        summary: Some("Updated".to_string()),
        ..Default::default()
    };
    let result = aim
        .update_event(&Id::Uid("nonexistent-uid".to_string()), patch)
        .await;
    assert!(result.is_err(), "Should return error for nonexistent event");
}

#[tokio::test]
async fn aim_event_status_updates_correctly() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create event
    let draft = test_event_draft("Test Event");
    let event = aim.new_event(draft).await.unwrap();
    let uid = event.uid().as_ref().to_string();

    // Update status
    for status in [
        EventStatus::Tentative,
        EventStatus::Confirmed,
        EventStatus::Cancelled,
    ] {
        let patch = EventPatch {
            status: Some(status),
            ..Default::default()
        };
        let updated = aim
            .update_event(&Id::Uid(uid.clone()), patch)
            .await
            .unwrap();
        assert_eq!(updated.status(), Some(status));
    }
}

#[tokio::test]
async fn aim_event_with_custom_datetimes() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create event with specific datetimes
    let start = LooseDateTime::Local(jiff::Zoned::now() + jiff::Span::new().hours(1));
    let end = LooseDateTime::Local(jiff::Zoned::now() + jiff::Span::new().hours(2));

    let draft = EventDraft {
        start: Some(start.clone()),
        end: Some(end.clone()),
        ..test_event_draft("Scheduled Event")
    };
    let event = aim.new_event(draft).await.unwrap();

    assert!(event.start().is_some());
    assert!(event.end().is_some());
}

#[tokio::test]
async fn aim_flush_short_ids_removes_mappings() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create event
    let draft = test_event_draft("Test Event");
    let event = aim.new_event(draft).await.unwrap();
    let uid = event.uid().as_ref().to_string();
    let short_id = event.short_id().unwrap();

    // Flush short IDs
    aim.flush_short_ids().await.unwrap();

    // Short ID should no longer resolve
    let result = aim
        .get_event(&Id::ShortIdOrUid(short_id.get().to_string()))
        .await;
    assert!(result.is_err(), "Short ID should not resolve after flush");

    // But UID should still work
    let retrieved = aim.get_event(&Id::Uid(uid.clone())).await.unwrap();
    assert_eq!(retrieved.uid().as_ref(), uid);
}
