// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Config, cli::ArgOutputFormat, cmd_event::CmdEventList, cmd_todo::CmdTodoList,
    short_id::ShortIdMap,
};
use aimcal_core::{Aim, EventConditions, TodoConditions, TodoStatus};
use chrono::{Duration, Local};
use clap::Command;
use colored::Colorize;
use std::{error::Error, path::PathBuf};
use tokio::try_join;

#[derive(Debug, Clone, Copy)]
pub struct CmdDashboard;

impl CmdDashboard {
    pub const NAME: &str = "dashboard";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Show the dashboard, which includes upcoming events and todos")
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse() -> Self {
        CmdDashboard
    }

    /// Show the dashboard with events and todos.
    pub async fn run(self, config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        log::debug!("Parsing configuration...");
        let config = Config::parse(config).await?;
        let (aim, map) = try_join!(Aim::new(&config.core), ShortIdMap::load_or_new(&config))?;

        log::debug!("Generating dashboard...");
        let now = Local::now().naive_local();

        println!("🗓️ {}", "Events".bold());
        let conds = EventConditions { now };
        CmdEventList::list(&aim, &map, &conds, ArgOutputFormat::Table).await?;
        println!();

        println!("✅ {}", "Todos".bold());
        let conds = TodoConditions {
            now,
            status: Some(TodoStatus::NeedsAction),
            due: Some(Duration::days(2)),
        };
        CmdTodoList::list(&aim, &map, &conds, ArgOutputFormat::Table).await?;

        map.dump(&config).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dashboard() {
        let cmd = Command::new("test").subcommand(CmdDashboard::command());
        let _ = cmd.try_get_matches_from(["test", "dashboard"]).unwrap();
        let _ = CmdDashboard::parse();
    }
}
