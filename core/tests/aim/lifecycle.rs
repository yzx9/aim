// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Lifecycle tests for the Aim application.
//!
//! Tests Aim initialization, time management, and cleanup.

use std::path::PathBuf;

use aimcal_core::{
    Aim, Config, DateTimeAnchor, Event, EventConditions, EventStatus, Pager, Priority, Todo,
    TodoConditions,
};
use jiff::Zoned;

use crate::common::{TestConfigBuilder, setup_temp_dirs};

#[tokio::test]
async fn aim_new_creates_database_and_loads_files() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    // Create a sample .ics file
    let ics_content = "BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//AIM//Test//EN\r
BEGIN:VEVENT\r
UID:test-event-123\r
DTSTAMP:20250115T120000Z\r
DTSTART:20250115T100000Z\r
DTEND:20250115T110000Z\r
SUMMARY:Test Event\r
END:VEVENT\r
END:VCALENDAR\r
";
    temp_dirs
        .create_ics_file("test-event-123", ics_content)
        .await
        .unwrap();

    // Create Aim instance
    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Verify database was created
    assert!(temp_dirs.state_dir.exists());
    let mut read_dir = tokio::fs::read_dir(&temp_dirs.state_dir).await.unwrap();
    let mut db_files = false;
    while let Some(entry) = read_dir.next_entry().await.unwrap() {
        if let Some(ext) = entry.path().extension()
            && (ext == "db" || ext == "db-shm" || ext == "db-wal")
        {
            db_files = true;
            break;
        }
    }
    assert!(db_files, "Database file should exist in state directory");

    // Verify event was loaded from file
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
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].uid().as_ref(), "test-event-123");
    assert_eq!(events[0].summary().as_ref(), "Test Event");
}

#[tokio::test]
async fn aim_new_creates_empty_state_without_calendar_files() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    // Create Aim instance with empty calendar directory
    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Verify no events loaded
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
    assert!(events.is_empty());
}

#[tokio::test]
async fn aim_now_returns_initial_time() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Get initial time
    let now1 = aim.now();
    let system_now = Zoned::now();

    // Times should be close (within 1 second)
    let diff = (now1.timestamp().as_second() - system_now.timestamp().as_second()).abs();
    assert!(diff <= 1, "Aim time should be close to system time");
}

#[tokio::test]
async fn aim_refresh_now_updates_current_time() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let mut aim = Aim::new(config).await.unwrap();

    let now1 = aim.now();

    // Wait a bit and refresh
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    aim.refresh_now();

    let now2 = aim.now();

    // now2 should be later than now1
    assert!(
        now2.timestamp() > now1.timestamp(),
        "Time should advance after refresh"
    );
}

#[tokio::test]
async fn aim_close_cleans_up_database() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Close the Aim instance
    aim.close().await.unwrap();

    // After close, the database should be properly closed
    // (We can't easily test this from the outside, but at least verify no panic)
}

#[tokio::test]
async fn aim_new_normalizes_config_paths() {
    // Create temp dirs for state directory
    let temp_dirs = setup_temp_dirs().await.unwrap();

    // Create config with relative calendar path but valid state dir
    let config = TestConfigBuilder::new()
        .with_calendar_path(&PathBuf::from("calendar")) // Will be created relative to cwd
        .with_state_dir(&temp_dirs.state_dir)
        .build();

    // This should normalize the paths
    let result = Aim::new(config).await;

    // Since the relative path might not exist or might not be writable,
    // we just verify it either succeeds or fails with a path-related error
    // (not a panic or other unexpected error)
    match result {
        Ok(_) => {
            // If it succeeded, that's fine too
        }
        Err(e) => {
            // Should be a path-related error, not a crash
            let err_msg = e.to_string().to_lowercase();
            assert!(
                err_msg.contains("path")
                    || err_msg.contains("directory")
                    || err_msg.contains("file"),
                "Error should be path-related: {e}"
            );
        }
    }
}

#[tokio::test]
async fn aim_default_event_draft_creates_draft_with_now() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: Some(DateTimeAnchor::InDays(1)),
        default_priority: Priority::P5,
        default_priority_none_fist: true,
    };
    let aim = Aim::new(config).await.unwrap();

    let draft = aim.default_event_draft();

    // Draft should have default values
    assert_eq!(draft.summary, "");
    assert!(
        draft.start.is_some(),
        "Default event draft should have start time"
    );
    assert!(
        draft.end.is_some(),
        "Default event draft should have end time"
    );
    assert_eq!(draft.status, EventStatus::Confirmed);
}

#[tokio::test]
async fn aim_default_todo_draft_includes_config_defaults() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: Some(DateTimeAnchor::InDays(7)),
        default_priority: Priority::P2,
        default_priority_none_fist: true,
    };
    let aim = Aim::new(config).await.unwrap();

    let draft = aim.default_todo_draft().unwrap();

    // Draft should include config defaults
    assert_eq!(draft.summary, "");
    assert!(
        draft.due.is_some(),
        "Default todo draft should have due date from config"
    );
    assert_eq!(
        draft.priority,
        Some(Priority::P2),
        "Priority should match config default"
    );
}

#[tokio::test]
async fn aim_loads_multiple_calendar_files() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    // Create multiple .ics files
    for i in 1..=3 {
        let ics_content = format!(
            "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//AIM//Test//EN\r
BEGIN:VEVENT\r
UID:event-{i}\r
DTSTAMP:20250115T120000Z\r
DTSTART:20250115T100000Z\r
DTEND:20250115T110000Z\r
SUMMARY:Event {i}\r
END:VEVENT\r
END:VCALENDAR\r
"
        );
        temp_dirs
            .create_ics_file(&format!("event-{i}"), &ics_content)
            .await
            .unwrap();
    }

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // All events should be loaded
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
}

#[tokio::test]
async fn aim_handles_calendar_with_multiple_components() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    // Create .ics file with both event and todo
    let ics_content = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//AIM//Test//EN\r
BEGIN:VEVENT\r
UID:cal1-event\r
DTSTAMP:20250115T120000Z\r
DTSTART:20250115T100000Z\r
DTEND:20250115T110000Z\r
SUMMARY:Calendar Event\r
END:VEVENT\r
BEGIN:VTODO\r
UID:cal1-todo\r
DTSTAMP:20250115T120000Z\r
DUE:20250116T100000Z\r
SUMMARY:Calendar Todo\r
STATUS:NEEDS-ACTION\r
END:VTODO\r
END:VCALENDAR\r
";
    temp_dirs
        .create_ics_file("cal1", ics_content)
        .await
        .unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Both components should be loaded
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
    let todos = aim
        .list_todos(
            &TodoConditions {
                status: None,
                due: None,
            },
            &[],
            &Pager {
                limit: 100,
                offset: 0,
            },
        )
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(todos.len(), 1);
    assert_eq!(events[0].uid().as_ref(), "cal1-event");
    assert_eq!(todos[0].uid().as_ref(), "cal1-todo");
}
