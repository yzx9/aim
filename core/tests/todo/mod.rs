// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Todo trait integration tests.
//!
//! Tests for TodoStatus (public API tests only).
//! Note: TodoDraft and TodoPatch tests are in src/todo.rs as unit tests
//! because they test pub(crate) methods that are not part of the public API.

mod draft;
mod patch;
mod priority;
mod status;
