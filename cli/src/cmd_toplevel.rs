// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{
    Aim, DateTimeAnchor, EventConditions, Id, Kind, Pager, TodoConditions, TodoStatus,
};
use clap::{ArgMatches, Command};
use colored::Colorize;

use crate::arg::{CommonArgs, EventOrTodoArgs};
use crate::cmd_event::{CmdEventDelay, CmdEventReschedule};
use crate::cmd_todo::{CmdTodoDelay, CmdTodoList, CmdTodoReschedule};
use crate::event_formatter::{EventColumn, EventFormatter};
use crate::util::OutputFormat;

#[derive(Debug, Default, Clone, Copy)]
pub struct CmdDashboard;

impl CmdDashboard {
    pub const NAME: &str = "dashboard";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Show the dashboard, which includes upcoming events and todos")
    }

    pub fn from(_matches: &ArgMatches) -> Self {
        Self
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
                flag = false;

                println!(" {} {}", "►".green(), title.italic());
                if events.len() >= (MAX as usize) {
                    let total = aim.count_events(&conds).await?;
                    if total > MAX {
                        println!("Displaying the {total}/{MAX} events");
                    }
                }

                println!("{}", formatter.format(&events));
            }
        }

        if flag {
            println!("{}", "No upcoming events".italic());
        }
        Ok(())
    }

    async fn list_todos(aim: &Aim) -> Result<(), Box<dyn Error>> {
        println!("✅ {}", "To-Dos: within 2 days".bold());
        let conds = TodoConditions {
            status: Some(TodoStatus::NeedsAction),
            due: Some(DateTimeAnchor::InDays(2)),
        };
        CmdTodoList::list(aim, &conds, OutputFormat::Table, false).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdDelay {
    pub id: Id,
    pub time_anchor: String,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdDelay {
    pub const NAME: &str = "delay";

    pub fn command() -> Command {
        let args = EventOrTodoArgs::new(None);
        Command::new(Self::NAME)
            .about("Delay event or todo's time by a specified time based on original time")
            .arg(args.id())
            .arg(args.time_anchor("delay"))
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: EventOrTodoArgs::get_id(matches),
            time_anchor: EventOrTodoArgs::get_time_anchor(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        let kind = aim.get_kind(&self.id).await?;
        match kind {
            Kind::Event => {
                CmdEventDelay {
                    id: self.id,
                    time_anchor: self.time_anchor,
                    output_format: self.output_format,
                    verbose: self.verbose,
                }
                .run(aim)
                .await
            }
            Kind::Todo => {
                CmdTodoDelay {
                    id: self.id,
                    time_anchor: self.time_anchor,
                    output_format: self.output_format,
                    verbose: self.verbose,
                }
                .run(aim)
                .await
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CmdReschedule {
    pub id: Id,
    pub time_anchor: String,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdReschedule {
    pub const NAME: &str = "reschedule";

    pub fn command() -> Command {
        let args = EventOrTodoArgs::new(None);
        Command::new(Self::NAME)
            .about("Reschedule event or todo's time by a specified time based on current time")
            .arg(args.id())
            .arg(args.time_anchor("delay"))
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: EventOrTodoArgs::get_id(matches),
            time_anchor: EventOrTodoArgs::get_time_anchor(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        let kind = aim.get_kind(&self.id).await?;
        match kind {
            Kind::Event => {
                CmdEventReschedule {
                    id: self.id,
                    time_anchor: self.time_anchor,
                    output_format: self.output_format,
                    verbose: self.verbose,
                }
                .run(aim)
                .await
            }
            Kind::Todo => {
                CmdTodoReschedule {
                    id: self.id,
                    time_anchor: self.time_anchor,
                    output_format: self.output_format,
                    verbose: self.verbose,
                }
                .run(aim)
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dashboard() {
        let cmd = CmdDashboard::command();
        let matches = cmd.try_get_matches_from(["dashboard"]).unwrap();
        let _ = CmdDashboard::from(&matches);
    }

    #[test]
    fn test_parse_delay() {
        let cmd = CmdEventDelay::command();
        let matches = cmd
            .try_get_matches_from([
                "delay",
                "abc",
                "time anchor",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let parsed = CmdDelay::from(&matches);

        assert_eq!(parsed.id, Id::ShortIdOrUid("abc".to_string()));
        assert_eq!(parsed.time_anchor, "time anchor");
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_reschedule() {
        let cmd = CmdReschedule::command();
        let matches = cmd
            .try_get_matches_from([
                "reschedule",
                "abc",
                "time anchor",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let parsed = CmdReschedule::from(&matches);

        assert_eq!(parsed.id, Id::ShortIdOrUid("abc".to_string()));
        assert_eq!(parsed.time_anchor, "time anchor");
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }
}
