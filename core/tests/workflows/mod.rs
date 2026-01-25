// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! End-to-end workflow tests for the aimcal-core crate.
//!
//! These tests validate multi-step workflows that integrate multiple components,
//! including file-database coordination, configuration integration, and real-world
//! usage patterns.

mod config_driven;
mod event_lifecycle;
mod file_sync;
mod todo_lifecycle;
