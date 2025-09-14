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
    pub ids: Vec<Id>,
    pub time_anchor: String,
    pub verbose: bool,
}

impl CmdDelay {
    pub const NAME: &str = "delay";

    pub fn command() -> Command {
        let args = EventOrTodoArgs::new(None);
        Command::new(Self::NAME)
            .about("Delay event or todo's time by a specified time based on original time")
            .arg(args.ids())
            .arg(args.time("delay"))
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            ids: EventOrTodoArgs::get_ids(matches),
            time_anchor: EventOrTodoArgs::get_time(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        let (event_ids, todo_ids) = separate_ids(aim, self.ids.clone()).await?;

        // TODO: handle formatting
        let printed_header = !event_ids.is_empty() && !todo_ids.is_empty();
        if !event_ids.is_empty() {
            if printed_header {
                println!("🗓️ {}", "Events".bold());
            }

            CmdEventDelay {
                ids: event_ids,
                time_anchor: self.time_anchor.clone(),
                output_format: OutputFormat::Table,
                verbose: self.verbose,
            }
            .run(aim)
            .await?;
        }

        if !todo_ids.is_empty() {
            if printed_header {
                println!();
                println!("✅ {}", "To-Dos".bold());
            }

            CmdTodoDelay {
                ids: todo_ids,
                time: self.time_anchor,
                output_format: OutputFormat::Table,
                verbose: self.verbose,
            }
            .run(aim)
            .await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdReschedule {
    pub ids: Vec<Id>,
    pub time: String,
    pub verbose: bool,
}

impl CmdReschedule {
    pub const NAME: &str = "reschedule";

    pub fn command() -> Command {
        let args = EventOrTodoArgs::new(None);
        Command::new(Self::NAME)
            .about("Reschedule event or todo's time by a specified time based on current time")
            .arg(args.ids())
            .arg(args.time("delay"))
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            ids: EventOrTodoArgs::get_ids(matches),
            time: EventOrTodoArgs::get_time(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        let (event_ids, todo_ids) = separate_ids(aim, self.ids.clone()).await?;

        let printed_header = !event_ids.is_empty() && !todo_ids.is_empty();
        if !event_ids.is_empty() {
            if printed_header {
                println!("🗓️ {}", "Events".bold());
            }

            CmdEventReschedule {
                ids: event_ids,
                time_anchor: self.time.clone(),
                output_format: OutputFormat::Table,
                verbose: self.verbose,
            }
            .run(aim)
            .await?;
        }

        if !todo_ids.is_empty() {
            if printed_header {
                println!();
                println!("✅ {}", "To-Dos".bold());
            }

            CmdTodoReschedule {
                ids: todo_ids,
                time: self.time,
                output_format: OutputFormat::Table,
                verbose: self.verbose,
            }
            .run(aim)
            .await?;
        }

        Ok(())
    }
}

async fn separate_ids(aim: &Aim, ids: Vec<Id>) -> Result<(Vec<Id>, Vec<Id>), Box<dyn Error>> {
    let mut event_ids = vec![];
    let mut todo_ids = vec![];
    for id in ids {
        let kind = aim.get_kind(&id).await?;
        match kind {
            Kind::Event => event_ids.push(id),
            Kind::Todo => todo_ids.push(id),
        }
    }
    Ok((event_ids, todo_ids))
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
            .try_get_matches_from(["delay", "a", "b", "c", "--time", "time", "--verbose"])
            .unwrap();
        let parsed = CmdDelay::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.time_anchor, "time");
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_reschedule() {
        let cmd = CmdReschedule::command();
        let matches = cmd
            .try_get_matches_from(["reschedule", "a", "b", "c", "--time", "time", "--verbose"])
            .unwrap();
        let parsed = CmdReschedule::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.time, "time");
        assert!(parsed.verbose);
    }
}
