// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Core library for the AIM calendar application.

#![warn(
    missing_docs,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    missing_debug_implementations,
    clippy::indexing_slicing,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::redundant_closure_for_method_calls
)]

mod aim;
mod config;
mod datetime;
mod event;
mod localdb;
mod short_id;
mod todo;
mod types;

pub use crate::aim::Aim;
pub use crate::config::{APP_NAME, Config};
pub use crate::datetime::{LooseDateTime, RangePosition};
pub use crate::event::{Event, EventConditions, EventStatus};
pub use crate::todo::{Todo, TodoConditions, TodoDraft, TodoPatch, TodoSort, TodoStatus};
pub use crate::types::{Id, Pager, Priority, SortOrder};
