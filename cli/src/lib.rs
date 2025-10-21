// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Command-line interface for the AIM calendar application.

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

mod arg;
mod cli;
mod cmd_event;
mod cmd_generate_completion;
mod cmd_todo;
mod cmd_toplevel;
mod cmd_tui;
mod config;
mod event_formatter;
mod prompt;
mod table;
mod todo_formatter;
mod tui;
mod util;

pub use crate::cli::{Cli, Commands, run};
pub use crate::config::Config;
