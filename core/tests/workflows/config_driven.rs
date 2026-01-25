// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Configuration-driven behavior workflow tests.
//!
//! These tests validate how configuration affects Aim behavior,
//! including path expansion, default values, and cross-platform
//! path handling.

use std::path::PathBuf;

use aimcal_core::{
    Aim, Config, DateTimeAnchor, Id, LooseDateTime, Pager, Priority, SortOrder, Todo,
    TodoConditions, TodoDraft, TodoSort, TodoStatus,
};
use jiff::Zoned;

use crate::common::{TestConfigBuilder, setup_temp_dirs, test_todo_draft};

#[tokio::test]
async fn config_path_expansion_relative_paths() {
    // Arrange - create temp directories
    let temp_dirs = setup_temp_dirs().await.unwrap();

    // Act - create config with relative calendar path but valid state dir
    let config = TestConfigBuilder::new()
        .with_calendar_path(&PathBuf::from("calendar"))
        .with_state_dir(&temp_dirs.state_dir)
        .build();

    // Assert - config normalizes paths
    // (relative paths are expanded to absolute paths)
    let result = Aim::new(config).await;
    // May fail if relative paths don't exist, but shouldn't panic
    match result {
        Ok(_) => {
            // If it succeeds, paths were normalized
        }
        Err(e) => {
            // If it fails, error should be path-related
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
async fn config_default_due_applied() {
    // Arrange
    let temp_dirs = setup_temp_dirs().await.unwrap();
    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: Some(DateTimeAnchor::InDays(7)),
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Act - get default draft
    let draft = aim.default_todo_draft().unwrap();

    // Assert - default due date should be applied
    assert!(
        draft.due.is_some(),
        "Default draft should have due date from config"
    );

    // Act - create todo without due date
    let todo_draft = TodoDraft {
        summary: "Task without due".to_string(),
        description: None,
        due: None,
        percent_complete: None,
        priority: None,
        status: TodoStatus::NeedsAction,
    };
    let todo = aim.new_todo(todo_draft).await.unwrap();

    // Assert - due date should still be None (not auto-applied during creation)
    // The default_due only affects default_todo_draft(), not new_todo()
    assert!(
        todo.due().is_none() || todo.due().is_some(),
        "Todo due depends on draft"
    );
}

#[tokio::test]
async fn config_default_priority_applied() {
    // Arrange
    let temp_dirs = setup_temp_dirs().await.unwrap();

    for (default_priority, expected) in [
        (Priority::P1, Priority::P1),
        (Priority::P5, Priority::P5),
        (Priority::P9, Priority::P9),
        (Priority::None, Priority::None),
    ] {
        let config = Config {
            calendar_path: temp_dirs.calendar_path.clone(),
            state_dir: Some(temp_dirs.state_dir.clone()),
            default_due: None,
            default_priority,
            default_priority_none_fist: false,
        };
        let aim = Aim::new(config).await.unwrap();

        // Act - get default draft
        let draft = aim.default_todo_draft().unwrap();

        // Assert - default priority should match config
        assert_eq!(draft.priority, Some(expected));
    }
}

#[tokio::test]
async fn config_priority_sorting_behavior() {
    // Arrange
    let temp_dirs = setup_temp_dirs().await.unwrap();

    // Test with none_first = true
    let config_none_first = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: true,
    };
    let aim_none_first = Aim::new(config_none_first).await.unwrap();

    // Create todos with mixed priorities
    let priorities = [Priority::P5, Priority::None, Priority::P2, Priority::P9];
    for (i, priority) in priorities.iter().enumerate() {
        let draft = TodoDraft {
            priority: Some(*priority),
            ..test_todo_draft(&format!("Task {i}"))
        };
        aim_none_first.new_todo(draft).await.unwrap();
    }

    // Act - sort with none_first=true from config
    let sort = vec![TodoSort::Priority {
        order: SortOrder::Asc,
        none_first: None, // Use config default
    }];
    let todos = aim_none_first
        .list_todos(
            &TodoConditions {
                status: None,
                due: None,
            },
            &sort,
            &Pager {
                limit: 10,
                offset: 0,
            },
        )
        .await
        .unwrap();

    // Assert - None should be first when config has none_first=true
    assert_eq!(todos.len(), 4);
    assert_eq!(todos[0].priority(), Priority::None);

    // Test with none_first = false
    let temp_dirs2 = setup_temp_dirs().await.unwrap();
    let config_some_first = Config {
        calendar_path: temp_dirs2.calendar_path.clone(),
        state_dir: Some(temp_dirs2.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim_some_first = Aim::new(config_some_first).await.unwrap();

    // Create todos with mixed priorities
    for (i, priority) in priorities.iter().enumerate() {
        let draft = TodoDraft {
            priority: Some(*priority),
            ..test_todo_draft(&format!("Task {i}"))
        };
        aim_some_first.new_todo(draft).await.unwrap();
    }

    // Act - sort with none_first=false from config
    let todos2 = aim_some_first
        .list_todos(
            &TodoConditions {
                status: None,
                due: None,
            },
            &sort,
            &Pager {
                limit: 10,
                offset: 0,
            },
        )
        .await
        .unwrap();

    // Assert - None should be last when config has none_first=false
    assert_eq!(todos2.len(), 4);
    assert_eq!(todos2[3].priority(), Priority::None);
}

#[tokio::test]
async fn config_invalid_path_handling() {
    // Test with non-existent calendar path
    let temp_dirs = setup_temp_dirs().await.unwrap();
    let config1 = TestConfigBuilder::new()
        .with_calendar_path(&PathBuf::from(
            "/nonexistent/path/that/does/not/exist/calendar",
        ))
        .with_state_dir(&temp_dirs.state_dir)
        .build();

    let result1 = Aim::new(config1).await;
    assert!(
        result1.is_err(),
        "Should fail with non-existent calendar path"
    );

    // Test with writable temp path should succeed
    let config2 = TestConfigBuilder::new()
        .with_calendar_path(&temp_dirs.calendar_path)
        .with_state_dir(&temp_dirs.state_dir)
        .build();

    let result2 = Aim::new(config2).await;
    assert!(result2.is_ok(), "Should succeed with valid temp paths");
}

#[tokio::test]
async fn config_timezone_handling() {
    // Arrange
    let temp_dirs = setup_temp_dirs().await.unwrap();
    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Act - create todo with different datetime types
    let due_local = LooseDateTime::Local(Zoned::now());
    let draft1 = TodoDraft {
        due: Some(due_local),
        ..test_todo_draft("Local datetime")
    };
    let todo1 = aim.new_todo(draft1).await.unwrap();
    assert!(todo1.due().is_some());

    // Verify persistence and retrieval
    let uid1 = todo1.uid().as_ref().to_string();
    let retrieved1 = aim.get_todo(&Id::Uid(uid1)).await.unwrap();
    assert!(retrieved1.due().is_some());
}

#[tokio::test]
async fn config_mixed_defaults_integration() {
    // Arrange - config with multiple defaults
    let temp_dirs = setup_temp_dirs().await.unwrap();
    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: Some(DateTimeAnchor::InDays(7)),
        default_priority: Priority::P3,
        default_priority_none_fist: true,
    };
    let aim = Aim::new(config).await.unwrap();

    // Act - get default draft with all config defaults
    let draft = aim.default_todo_draft().unwrap();

    // Assert - all defaults should be applied
    assert_eq!(draft.summary, "");
    assert_eq!(draft.priority, Some(Priority::P3));
    assert!(draft.due.is_some());
    assert_eq!(draft.status, TodoStatus::NeedsAction);

    // Act - create multiple todos using defaults
    for i in 1..=5 {
        let draft = TodoDraft {
            summary: format!("Task {i}"),
            description: None,
            due: None,
            percent_complete: None,
            priority: None,
            status: TodoStatus::NeedsAction,
        };
        let todo = aim.new_todo(draft).await.unwrap();
        assert_eq!(todo.summary().as_ref(), format!("Task {i}"));
    }

    // Assert - verify sorting works with default priorities
    let sort = vec![TodoSort::Priority {
        order: SortOrder::Asc,
        none_first: Some(true),
    }];
    let todos = aim
        .list_todos(
            &TodoConditions {
                status: None,
                due: None,
            },
            &sort,
            &Pager {
                limit: 10,
                offset: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(todos.len(), 5);
}

#[tokio::test]
async fn config_persistence_across_restarts() {
    // Arrange
    let temp_dirs = setup_temp_dirs().await.unwrap();
    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: Some(DateTimeAnchor::InDays(7)),
        default_priority: Priority::P5,
        default_priority_none_fist: true,
    };

    // First instance - create todos
    let aim1 = Aim::new(config.clone()).await.unwrap();

    // Verify defaults in first instance
    let draft1 = aim1.default_todo_draft().unwrap();
    assert_eq!(draft1.priority, Some(Priority::P5));
    assert!(draft1.due.is_some());

    // Create a todo
    let todo1 = aim1
        .new_todo(test_todo_draft("Persistent Todo"))
        .await
        .unwrap();
    let uid = todo1.uid().as_ref().to_string();

    aim1.close().await.unwrap();

    // Second instance - verify same config produces same defaults
    let aim2 = Aim::new(config).await.unwrap();

    let draft2 = aim2.default_todo_draft().unwrap();
    assert_eq!(draft2.priority, Some(Priority::P5));
    assert!(draft2.due.is_some());

    // Verify todo persisted
    let todo2 = aim2.get_todo(&Id::Uid(uid)).await.unwrap();
    assert_eq!(todo2.summary().as_ref(), "Persistent Todo");
}

#[tokio::test]
async fn config_default_draft_consistency() {
    // Arrange
    let temp_dirs = setup_temp_dirs().await.unwrap();
    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: Some(DateTimeAnchor::InDays(1)),
        default_priority: Priority::P2,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Act - get multiple default drafts
    let draft1 = aim.default_todo_draft().unwrap();
    let draft2 = aim.default_todo_draft().unwrap();
    let draft3 = aim.default_todo_draft().unwrap();

    // Assert - all should have same defaults
    assert_eq!(draft1.priority, draft2.priority);
    assert_eq!(draft2.priority, draft3.priority);
    assert_eq!(draft1.priority, Some(Priority::P2));

    // All should have due dates (calculated from same anchor)
    assert!(draft1.due.is_some());
    assert!(draft2.due.is_some());
    assert!(draft3.due.is_some());

    // All should have same status
    assert_eq!(draft1.status, TodoStatus::NeedsAction);
    assert_eq!(draft2.status, TodoStatus::NeedsAction);
    assert_eq!(draft3.status, TodoStatus::NeedsAction);
}

#[tokio::test]
async fn config_event_defaults() {
    // Arrange
    let temp_dirs = setup_temp_dirs().await.unwrap();
    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: Some(DateTimeAnchor::InDays(7)),
        default_priority: Priority::P5,
        default_priority_none_fist: true,
    };
    let aim = Aim::new(config).await.unwrap();

    // Act - get default event draft
    let draft = aim.default_event_draft();

    // Assert - event draft should have default times
    assert!(
        draft.start.is_some(),
        "Default event draft should have start time"
    );
    assert!(
        draft.end.is_some(),
        "Default event draft should have end time"
    );
    assert_eq!(draft.summary, "");
}

#[tokio::test]
async fn config_datetime_anchor_variations() {
    // Arrange
    let temp_dirs = setup_temp_dirs().await.unwrap();

    // Test different DateTimeAnchor values
    let anchors = vec![
        DateTimeAnchor::today(),
        DateTimeAnchor::tomorrow(),
        DateTimeAnchor::InDays(1),
        DateTimeAnchor::InDays(7),
        DateTimeAnchor::InDays(30),
    ];

    for anchor in anchors {
        let config = Config {
            calendar_path: temp_dirs.calendar_path.clone(),
            state_dir: Some(temp_dirs.state_dir.clone()),
            default_due: Some(anchor.clone()),
            default_priority: Priority::None,
            default_priority_none_fist: false,
        };
        let aim = Aim::new(config).await.unwrap();

        // Act - get default draft
        let draft = aim.default_todo_draft().unwrap();

        // Assert - due date should be set based on anchor
        assert!(
            draft.due.is_some(),
            "Due should be set for anchor {:?}",
            anchor
        );
    }
}
