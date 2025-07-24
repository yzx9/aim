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
use std::error::Error;

pub use crate::{
    cli::{Cli, Commands},
    command::{
        command_add_todo, command_dashboard, command_done, command_events, command_todos,
        command_undo,
    },
    config::Config,
};

/// Run the AIM command-line interface.
pub async fn run() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Dashboard => command_dashboard(cli.config).await?,
        Commands::Events(args) => command_events(cli.config, &args).await?,
        Commands::Todos(args) => command_todos(cli.config, &args).await?,
        Commands::NewTodo(args) => command_add_todo(cli.config, &args).await?,
        Commands::Done(args) => command_done(cli.config, &args).await?,
        Commands::Undo(args) => command_undo(cli.config, &args).await?,
    }
    Ok(())
}
