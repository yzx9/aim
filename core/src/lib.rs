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
    clippy::indexing_slicing,
    clippy::pedantic
)]
// Allow certain clippy lints that are too restrictive for this crate
#![allow(
    clippy::option_option,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::match_bool
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
