// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
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

mod aim;
mod config;
mod datetime;
mod db;
mod event;
mod short_id;
mod store;
mod todo;
mod types;

pub use crate::aim::{Aim, CalendarDetails, CalendarStoreDetails};
pub use crate::config::{APP_NAME, CalendarEntry, Config, StoreDef};
pub use crate::db::calendars::CalendarRecord;
pub use crate::store::{CaldavStore, LocalStore, Store, StoreError, SyncResult}; // TODO: don't export this directly

// Re-export AuthMethod for use in config
pub use crate::datetime::{DateTimeAnchor, LooseDateTime, RangePosition};
pub use crate::event::{Event, EventConditions, EventDraft, EventPatch, EventStatus};
pub use crate::todo::{Todo, TodoConditions, TodoDraft, TodoPatch, TodoSort, TodoStatus};
pub use crate::types::{Id, Kind, Pager, Priority, SortOrder, StoreKind};
pub use aimcal_caldav::AuthMethod;
