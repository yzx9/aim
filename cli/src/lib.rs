// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Command-line interface for the AIM calendar application.

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

mod cli;
mod command;
mod config;
mod event_formatter;
mod short_id;
mod table;
mod todo_formatter;

pub use crate::{
    cli::{Cli, Commands},
    command::{command_dashboard, command_done, command_events, command_todos, command_undo},
    config::Config,
};
