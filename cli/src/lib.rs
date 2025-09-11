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

mod arg;
mod cli;
mod cmd_event;
mod cmd_generate_completion;
mod cmd_todo;
mod cmd_toplevel;
mod cmd_tui;
mod config;
mod event_formatter;
mod table;
mod todo_formatter;
mod tui;
mod util;

pub use crate::cli::{Cli, Commands, run};
pub use crate::config::Config;
