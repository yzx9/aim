// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{Aim, DateTimeAnchor, EventConditions, Pager, TodoConditions, TodoStatus};
use clap::{ArgMatches, Command};
use colored::Colorize;

use crate::cmd_todo::CmdTodoList;
use crate::event_formatter::{EventColumn, EventFormatter};
use crate::util::ArgOutputFormat;

#[derive(Debug, Default, Clone, Copy)]
pub struct CmdDashboard;

impl CmdDashboard {
    pub const NAME: &str = "dashboard";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Show the dashboard, which includes upcoming events and todos")
    }

    pub fn from(_matches: &ArgMatches) -> Self {
        CmdDashboard
    }

    /// Show the dashboard with events and todos.
    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "generating dashboard...");

        Self::list_events(aim).await?;
        println!();

        Self::list_todos(aim).await?;
        Ok(())
    }

    async fn list_events(aim: &Aim) -> Result<(), Box<dyn Error>> {
        const MAX: i64 = 128;

        let pager: Pager = (MAX, 0).into();
        let columns = vec![
            EventColumn::id(),
            EventColumn::time_span(),
            EventColumn::summary(),
        ];
        let formatter = EventFormatter::new(aim.now(), columns);

        println!("🗓️ {}", "Events".bold());

        let mut flag = true;
        for (title, anchor) in [
            ("Tomorrow", DateTimeAnchor::tomorrow()),
            ("Today", DateTimeAnchor::today()),
        ] {
            let conds = EventConditions {
                startable: Some(anchor),
                cutoff: Some(anchor),
            };
            let events = aim.list_events(&conds, &pager).await?;
            if !events.is_empty() {
                if !flag {
                    println!();
                }

                println!(" {} {}", "►".green(), title.italic());
                if events.len() >= (MAX as usize) {
                    let total = aim.count_events(&conds).await?;
                    if total > MAX {
                        println!("Displaying the {total}/{MAX} events");
                    }
                }

                println!("{}", formatter.format(&events));
                flag = false;
            }
        }

        if flag {
            println!("No upcoming events");
        }
        Ok(())
    }

    async fn list_todos(aim: &Aim) -> Result<(), Box<dyn Error>> {
        println!("✅ {}", "Todos: in 2 days".bold());
        let conds = TodoConditions {
            status: Some(TodoStatus::NeedsAction),
            due: Some(DateTimeAnchor::InDays(2)),
        };
        CmdTodoList::list(aim, &conds, ArgOutputFormat::Table, false).await?;
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
