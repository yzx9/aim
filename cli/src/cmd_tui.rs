// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides the tui command for the AIM CLI.
//! In general, they dont provide a comprehensive CLI command, but call TUI directly.

use std::error::Error;

use aimcal_core::{Aim, Id};
use clap::{Arg, ArgMatches, Command, arg};

use crate::cmd_todo::{CmdTodoEdit, CmdTodoNew};
use crate::util::ArgOutputFormat;

#[derive(Debug, Clone, Copy)]
pub struct CmdNew {
    pub output_format: ArgOutputFormat,
}

impl CmdNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new event or todo using TUI")
            .arg(ArgOutputFormat::arg())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            output_format: ArgOutputFormat::from(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        // TODO: check is it a event / todo
        CmdTodoNew::new().run(aim).await
    }
}

#[derive(Debug, Clone)]
pub struct CmdEdit {
    pub uid_or_short_id: Id,
    pub output_format: ArgOutputFormat,
}

impl CmdEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Edit a event or todo using TUI")
            .arg(arg_id())
            .arg(ArgOutputFormat::arg())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            uid_or_short_id: get_id(matches),
            output_format: ArgOutputFormat::from(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        // TODO: check is it a event / todo
        CmdTodoEdit::new(self.uid_or_short_id, self.output_format)
            .run(aim)
            .await
    }
}

fn arg_id() -> Arg {
    arg!(id: <ID> "The short id or uid of the event or todo to edit")
}

fn get_id(matches: &ArgMatches) -> Id {
    let id = matches
        .get_one::<String>("id")
        .expect("id is required")
        .clone();

    Id::ShortIdOrUid(id)
}
