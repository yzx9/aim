// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Core library for the AIM calendar application.

#![warn(
    trivial_casts,
    trivial_numeric_casts,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::indexing_slicing,
    clippy::redundant_closure_for_method_calls,
    clippy::trivially_copy_pass_by_ref,
    clippy::large_types_passed_by_value
)]

mod aim;
mod config;
mod datetime;
mod event;
mod io;
mod localdb;
mod short_id;
mod todo;
mod types;

pub use crate::aim::Aim;
pub use crate::config::{APP_NAME, Config};
pub use crate::datetime::{DateTimeAnchor, LooseDateTime, RangePosition};
pub use crate::event::{Event, EventConditions, EventDraft, EventPatch, EventStatus};
pub use crate::todo::{Todo, TodoConditions, TodoDraft, TodoPatch, TodoSort, TodoStatus};
pub use crate::types::{Id, Kind, Pager, Priority, SortOrder};
