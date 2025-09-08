// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides the tui command for the AIM CLI.
//! In general, they dont provide a comprehensive CLI command, but call TUI directly.

use std::error::Error;

use aimcal_core::{Aim, Id, Kind};
use clap::{ArgMatches, Command};

use crate::arg::{CommonArgs, EventOrTodoArgs};
use crate::cmd_event::{CmdEventEdit, CmdEventNew};
use crate::cmd_todo::{CmdTodoEdit, CmdTodoNew};
use crate::tui::{EventOrTodoDraft, draft_event_or_todo};
use crate::util::OutputFormat;

#[derive(Debug, Clone, Copy)]
pub struct CmdNew {
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new event or todo using TUI")
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        // TODO: check is it a event / todo
        tracing::debug!(?self, "adding new item using TUI...");
        let draft = draft_event_or_todo(aim)?;
        match draft {
            Some(EventOrTodoDraft::Event(draft)) => {
                tracing::info!("creating new event");
                CmdEventNew::new_event(aim, draft, self.output_format, self.verbose).await
            }
            Some(EventOrTodoDraft::Todo(draft)) => {
                tracing::info!("creating new todo");
                CmdTodoNew::new_todo(aim, draft, self.output_format, self.verbose).await
            }
            None => {
                tracing::info!("user cancel the drafting");
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CmdEdit {
    pub id: Id,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        let args = args();
        Command::new(Self::NAME)
            .about("Edit a event or todo using TUI")
            .arg(args.id())
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: EventOrTodoArgs::get_id(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "editing item using TUI...");
        let kind = aim.get_kind(&self.id).await?;
        match kind {
            Kind::Event => {
                tracing::info!("editing event");
                CmdEventEdit::new_tui(self.id, self.output_format, self.verbose)
                    .run(aim)
                    .await
            }
            Kind::Todo => {
                tracing::info!("editing todo");
                CmdTodoEdit::new_tui(self.id, self.output_format, self.verbose)
                    .run(aim)
                    .await
            }
        }
    }
}

const fn args() -> EventOrTodoArgs {
    EventOrTodoArgs::new(None)
}
