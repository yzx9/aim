// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event trait integration tests.
//!
//! Tests for EventStatus (public API tests only).
//! Note: EventDraft and EventPatch tests are in src/event.rs as unit tests
//! because they test pub(crate) methods that are not part of the public API.

mod status;
