// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides the tui command for the AIM CLI.
//! In general, they dont provide a comprehensive CLI command, but call TUI directly.

use crate::{
    Config,
    cmd_todo::{CmdTodoEdit, CmdTodoNew},
    parser::{ArgOutputFormat, ArgUidOrShortId},
    short_id::ShortIdMap,
};
use aimcal_core::Aim;
use clap::{ArgMatches, Command};
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct CmdNew {
    pub output_format: ArgOutputFormat,
}

impl CmdNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("New a event or todo using TUI")
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            output_format: ArgOutputFormat::parse(matches),
        }
    }

    pub async fn run(
        self,
        config: &Config,
        aim: &Aim,
        map: &ShortIdMap,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: check is it a event / todo
        CmdTodoNew::new().run(config, aim, map).await
    }
}

#[derive(Debug, Clone)]
pub struct CmdEdit {
    pub uid_or_short_id: ArgUidOrShortId,
    pub output_format: ArgOutputFormat,
}

impl CmdEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Edit a event or todo using TUI")
            .arg(ArgUidOrShortId::arg())
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            uid_or_short_id: ArgUidOrShortId::parse(matches),
            output_format: ArgOutputFormat::parse(matches),
        }
    }

    pub async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        // TODO: check is it a event / todo
        CmdTodoEdit::new(self.uid_or_short_id, self.output_format)
            .run(aim, map)
            .await
    }
}
