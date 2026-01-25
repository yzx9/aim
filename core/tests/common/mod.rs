// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Common test utilities for integration tests.
//!
//! This module provides shared test infrastructure including:
//! - Test data factories (fixtures)
//! - Custom assertion helpers
//! - Temporary directory management with auto-cleanup

mod assertions;
mod fixtures;
mod temp_dir;

#[allow(unused_imports)]
pub use assertions::{assert_event_matches_draft, assert_file_exists};
#[allow(unused_imports)]
pub use fixtures::{TestConfigBuilder, test_config, test_event_draft, test_todo_draft};
pub use temp_dir::setup_temp_dirs;
