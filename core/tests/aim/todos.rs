// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Todo CRUD operation tests for the Aim application.
//!
//! Tests creating, reading, updating, and listing todos.

use aimcal_core::{
    Aim, Config, DateTimeAnchor, Id, Pager, Priority, SortOrder, Todo, TodoConditions, TodoDraft,
    TodoPatch, TodoSort, TodoStatus,
};

use crate::common::{setup_temp_dirs, test_todo_draft};

#[tokio::test]
async fn aim_new_todo_creates_file_and_database_entry() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create todo
    let draft = test_todo_draft("New Task");
    let todo = aim.new_todo(draft).await.unwrap();

    // Verify todo was created
    assert_eq!(todo.summary().as_ref(), "New Task");
    assert!(todo.short_id().is_some(), "Todo should have short ID");

    // Verify .ics file was created
    let uid = todo.uid().as_ref().to_string();
    let expected_path = temp_dirs.calendar_path.join(format!("{uid}.ics"));
    assert!(expected_path.exists(), ".ics file should be created");

    // Verify todo can be retrieved
    let retrieved = aim.get_todo(&Id::Uid(uid.clone())).await.unwrap();
    assert_eq!(retrieved.uid().as_ref(), uid);
    assert_eq!(retrieved.summary().as_ref(), "New Task");
}

#[tokio::test]
async fn aim_get_todo_resolves_short_id() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create todo
    let draft = test_todo_draft("Test Task");
    let todo = aim.new_todo(draft).await.unwrap();
    let short_id = todo.short_id().unwrap();
    let uid = todo.uid().as_ref().to_string();

    // Retrieve by short ID
    let retrieved = aim
        .get_todo(&Id::ShortIdOrUid(short_id.get().to_string()))
        .await
        .unwrap();
    assert_eq!(retrieved.uid().as_ref(), uid);
    assert_eq!(retrieved.short_id(), Some(short_id));
}

#[tokio::test]
async fn aim_update_todo_modifies_file_and_database() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create todo
    let draft = test_todo_draft("Original Title");
    let todo = aim.new_todo(draft).await.unwrap();
    let uid = todo.uid().as_ref().to_string();

    // Update todo
    let patch = TodoPatch {
        summary: Some("Updated Title".to_string()),
        description: Some(Some("New description".to_string())),
        ..Default::default()
    };
    let updated = aim.update_todo(&Id::Uid(uid.clone()), patch).await.unwrap();

    assert_eq!(updated.summary().as_ref(), "Updated Title");
    assert_eq!(updated.description().as_deref(), Some("New description"));

    // Verify update persisted
    let retrieved = aim.get_todo(&Id::Uid(uid)).await.unwrap();
    assert_eq!(retrieved.summary().as_ref(), "Updated Title");
}

#[tokio::test]
async fn aim_list_todos_returns_all_todos() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create multiple todos
    for i in 1..=3 {
        let draft = test_todo_draft(&format!("Task {i}"));
        aim.new_todo(draft).await.unwrap();
    }

    // List all todos
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
    assert_eq!(todos.len(), 3);

    let summaries: Vec<_> = todos
        .iter()
        .map(|t| t.summary().as_ref().to_string())
        .collect();
    assert!(summaries.contains(&"Task 1".to_string()));
    assert!(summaries.contains(&"Task 2".to_string()));
    assert!(summaries.contains(&"Task 3".to_string()));
}

#[tokio::test]
async fn aim_list_todos_with_pagination() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create multiple todos
    for i in 1..=5 {
        let draft = test_todo_draft(&format!("Task {i}"));
        aim.new_todo(draft).await.unwrap();
    }

    // List with pagination
    let page1 = aim
        .list_todos(
            &TodoConditions {
                status: None,
                due: None,
            },
            &[],
            &Pager {
                limit: 2,
                offset: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(page1.len(), 2);

    let page2 = aim
        .list_todos(
            &TodoConditions {
                status: None,
                due: None,
            },
            &[],
            &Pager {
                limit: 2,
                offset: 2,
            },
        )
        .await
        .unwrap();
    assert_eq!(page2.len(), 2);

    let page3 = aim
        .list_todos(
            &TodoConditions {
                status: None,
                due: None,
            },
            &[],
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
async fn aim_count_todos_returns_correct_count() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Initially no todos
    let count = aim
        .count_todos(&TodoConditions {
            status: None,
            due: None,
        })
        .await
        .unwrap();
    assert_eq!(count, 0);

    // Create todos
    for i in 1..=3 {
        let draft = test_todo_draft(&format!("Task {i}"));
        aim.new_todo(draft).await.unwrap();
    }

    // Count should match
    let count = aim
        .count_todos(&TodoConditions {
            status: None,
            due: None,
        })
        .await
        .unwrap();
    assert_eq!(count, 3);
}

#[tokio::test]
async fn aim_default_todo_draft_uses_config_defaults() {
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

    assert_eq!(draft.priority, Some(Priority::P2));
    assert!(draft.due.is_some(), "Should have due date from config");
}

#[tokio::test]
async fn aim_update_todo_clears_optional_fields() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create todo with description
    let draft = TodoDraft {
        description: Some("Original description".to_string()),
        ..test_todo_draft("Test")
    };
    let todo = aim.new_todo(draft).await.unwrap();
    let uid = todo.uid().as_ref().to_string();

    // Clear description
    let patch = TodoPatch {
        description: Some(None), // Some(None) means clear the field
        ..Default::default()
    };
    let updated = aim.update_todo(&Id::Uid(uid.clone()), patch).await.unwrap();

    assert!(
        updated.description().is_none(),
        "Description should be cleared"
    );
}

#[tokio::test]
async fn aim_update_todo_status_to_completed_sets_completed_timestamp() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create todo
    let draft = test_todo_draft("Task to Complete");
    let todo = aim.new_todo(draft).await.unwrap();
    let uid = todo.uid().as_ref().to_string();

    assert!(
        todo.completed().is_none(),
        "Initially should have no completed timestamp"
    );

    // Update status to Completed
    let patch = TodoPatch {
        status: Some(TodoStatus::Completed),
        ..Default::default()
    };
    let updated = aim.update_todo(&Id::Uid(uid.clone()), patch).await.unwrap();

    assert!(
        updated.completed().is_some(),
        "Completed timestamp should be set"
    );
    assert_eq!(updated.status(), TodoStatus::Completed);
}

#[tokio::test]
async fn aim_update_todo_status_from_completed_clears_completed_timestamp() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create and complete todo
    let draft = test_todo_draft("Task");
    let todo = aim.new_todo(draft).await.unwrap();
    let uid = todo.uid().as_ref().to_string();

    let patch1 = TodoPatch {
        status: Some(TodoStatus::Completed),
        ..Default::default()
    };
    aim.update_todo(&Id::Uid(uid.clone()), patch1)
        .await
        .unwrap();

    // Change status back to NeedsAction
    let patch2 = TodoPatch {
        status: Some(TodoStatus::NeedsAction),
        ..Default::default()
    };
    let updated = aim
        .update_todo(&Id::Uid(uid.clone()), patch2)
        .await
        .unwrap();

    assert!(
        updated.completed().is_none(),
        "Completed timestamp should be cleared"
    );
    assert_eq!(updated.status(), TodoStatus::NeedsAction);
}

#[tokio::test]
async fn aim_list_todos_with_status_filter() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create todos with different statuses
    for (i, status) in [
        TodoStatus::NeedsAction,
        TodoStatus::Completed,
        TodoStatus::InProcess,
    ]
    .iter()
    .enumerate()
    {
        let draft = TodoDraft {
            status: *status,
            ..test_todo_draft(&format!("Task {i}"))
        };
        aim.new_todo(draft).await.unwrap();
    }

    // Filter by NeedsAction status
    let conds = TodoConditions {
        status: Some(TodoStatus::NeedsAction),
        due: None,
    };
    let todos = aim
        .list_todos(
            &conds,
            &[],
            &Pager {
                limit: 100,
                offset: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].status(), TodoStatus::NeedsAction);
}

#[tokio::test]
async fn aim_list_todos_with_priority_sort() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Create todos with different priorities
    let priorities = [Priority::P5, Priority::P2, Priority::P9, Priority::None];
    for (i, priority) in priorities.iter().enumerate() {
        let draft = TodoDraft {
            priority: Some(*priority),
            ..test_todo_draft(&format!("Task {i}"))
        };
        aim.new_todo(draft).await.unwrap();
    }

    // Sort by priority ascending (1=high first, 9=low last)
    let sort = vec![TodoSort::Priority {
        order: SortOrder::Asc,
        none_first: Some(false),
    }];
    let todos = aim
        .list_todos(
            &TodoConditions {
                status: None,
                due: None,
            },
            &sort,
            &Pager {
                limit: 100,
                offset: 0,
            },
        )
        .await
        .unwrap();

    assert_eq!(todos.len(), 4);
    // None should be last when none_first=false
    assert_eq!(todos[3].priority(), Priority::None);
    // P2 (highest priority value of 2) should be first
    assert_eq!(todos[0].priority(), Priority::P2);
}

#[tokio::test]
async fn aim_get_todo_returns_error_for_nonexistent() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Try to get nonexistent todo
    let result = aim.get_todo(&Id::Uid("nonexistent-uid".to_string())).await;
    assert!(result.is_err(), "Should return error for nonexistent todo");
}

#[tokio::test]
async fn aim_update_todo_returns_error_for_nonexistent() {
    let temp_dirs = setup_temp_dirs().await.unwrap();

    let config = Config {
        calendar_path: temp_dirs.calendar_path.clone(),
        state_dir: Some(temp_dirs.state_dir.clone()),
        default_due: None,
        default_priority: Priority::None,
        default_priority_none_fist: false,
    };
    let aim = Aim::new(config).await.unwrap();

    // Try to update nonexistent todo
    let patch = TodoPatch {
        summary: Some("Updated".to_string()),
        ..Default::default()
    };
    let result = aim
        .update_todo(&Id::Uid("nonexistent-uid".to_string()), patch)
        .await;
    assert!(result.is_err(), "Should return error for nonexistent todo");
}
