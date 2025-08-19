// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{Aim, DateTimeAnchor, EventConditions, TodoConditions, TodoStatus};
use chrono::Duration;
use clap::{ArgMatches, Command};
use colored::Colorize;

use crate::{cmd_event::CmdEventList, cmd_todo::CmdTodoList, util::ArgOutputFormat};

#[derive(Debug, Default, Clone, Copy)]
pub struct CmdDashboard;

impl CmdDashboard {
    pub const NAME: &str = "dashboard";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Show the dashboard, which includes upcoming events and todos")
            .arg(ArgOutputFormat::arg())
    }

    pub fn from(_matches: &ArgMatches) -> Self {
        CmdDashboard
    }

    /// Show the dashboard with events and todos.
    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "generating dashboard...");
        println!("🗓️ {}", "Events".bold());
        let conds = EventConditions {
            startable: Some(DateTimeAnchor::today()),
        };
        CmdEventList::list(aim, &conds, ArgOutputFormat::Table).await?;
        println!();

        println!("✅ {}", "Todos".bold());
        let conds = TodoConditions {
            status: Some(TodoStatus::NeedsAction),
            due: Some(Duration::days(2)),
        };
        CmdTodoList::list(aim, &conds, ArgOutputFormat::Table).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dashboard() {
        let cmd = Command::new("test").subcommand(CmdDashboard::command());
        let matches = cmd.try_get_matches_from(["test", "dashboard"]).unwrap();
        let _ = CmdDashboard::from(&matches);
    }
}
