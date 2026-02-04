// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Integration test for the common module.
//!
//! Verifies that common test utilities work correctly.

mod common;

use common::{setup_temp_dirs, test_config, test_event_draft};

#[tokio::test]
async fn common_module_imports_work() {
    let dirs = setup_temp_dirs().await.unwrap();
    assert!(dirs.calendar_path.exists());
    assert!(dirs.state_dir.exists());
}

#[tokio::test]
async fn common_module_fixtures_work() {
    let config = test_config("/test/cal", Some("/test/state"));
    assert_eq!(
        config
            .calendar_path
            .expect("calendar_path should be set")
            .to_str()
            .unwrap(),
        "/test/cal"
    );
}

#[tokio::test]
async fn common_module_event_draft_works() {
    let draft = test_event_draft("Test Event");
    assert_eq!(draft.summary, "Test Event");
}
