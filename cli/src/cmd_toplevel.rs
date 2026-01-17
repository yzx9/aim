// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
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
use crate::prompt::prompt_time;
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

    #[expect(clippy::cast_possible_truncation)]
    async fn list_events(aim: &Aim) -> Result<(), Box<dyn Error>> {
        const MAX: i64 = 128;

        let pager: Pager = (MAX, 0).into();

        println!("🗓️ {}", "Events".bold());

        let mut flag = true;
        for (title, anchor) in [
            ("Tomorrow", DateTimeAnchor::tomorrow()),
            ("Today", DateTimeAnchor::today()),
        ] {
            let conds = EventConditions {
                startable: Some(anchor.clone()),
                cutoff: Some(anchor.clone()),
            };
            let events = aim.list_events(&conds, &pager).await?;
            if !events.is_empty() {
                if !flag {
                    println!();
                }
                flag = false;

                println!(" {} {}", "►".green(), title.italic());
                if events.len() >= MAX as usize {
                    let total = aim.count_events(&conds).await?;
                    if total > MAX {
                        println!("Displaying the {total}/{MAX} events");
                    }
                }

                let date = anchor
                    .resolve_at_start_of_day(&aim.now())
                    .map_err(|e| format!("Failed to resolve start of day: {e}"))?
                    .date();
                let columns = vec![
                    EventColumn::Id,
                    EventColumn::TimeSpan { date },
                    EventColumn::Summary,
                ];
                let formatter = EventFormatter::new(aim.now(), columns, OutputFormat::Table);
                println!("{}", formatter.format(&events));
            }
        }

        if flag {
            println!("{}", "No upcoming events".italic());
        }
        Ok(())
    }

    async fn list_todos(aim: &Aim) -> Result<(), Box<dyn Error>> {
        let now = aim.now();
        let days_from_monday = now.weekday().to_monday_zero_offset();
        let (days, label) = match days_from_monday {
            0..5 => (6 - days_from_monday, "this week"),
            _ => (3, "next 3 days"),
        };

        println!("✅ {} {}", "To-Dos: within".bold(), label.bold());
        let conds = TodoConditions {
            status: Some(TodoStatus::NeedsAction),
            due: Some(DateTimeAnchor::InDays(i64::from(days))),
        };
        CmdTodoList::list(aim, &conds, OutputFormat::Table, false).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdDelay {
    pub ids: Vec<Id>,
    pub time: Option<DateTimeAnchor>,
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
            time: EventOrTodoArgs::get_time(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        let (event_ids, todo_ids) = separate_ids(aim, self.ids.clone()).await?;
        if todo_ids.is_empty() {
            CmdEventDelay {
                ids: event_ids,
                time: self.time,
                output_format: OutputFormat::Table,
                verbose: self.verbose,
            }
            .run(aim)
            .await
        } else if event_ids.is_empty() {
            CmdTodoDelay {
                ids: todo_ids,
                time: self.time,
                output_format: OutputFormat::Table,
                verbose: self.verbose,
            }
            .run(aim)
            .await
        } else {
            self.run_mix(aim, event_ids, todo_ids).await
        }
    }

    async fn run_mix(
        self,
        aim: &mut Aim,
        event_ids: Vec<Id>,
        todo_ids: Vec<Id>,
    ) -> Result<(), Box<dyn Error>> {
        // Prompt for time if not provided
        let time = match self.time {
            Some(t) => t,
            None => prompt_time()?,
        };

        // TODO: handle formatting
        println!("🗓️ {}", "Events".bold());

        CmdEventDelay {
            ids: event_ids,
            time: Some(time.clone()),
            output_format: OutputFormat::Table,
            verbose: self.verbose,
        }
        .run(aim)
        .await?;

        println!();
        println!("✅ {}", "To-Dos".bold());

        CmdTodoDelay {
            ids: todo_ids,
            time: Some(time),
            output_format: OutputFormat::Table,
            verbose: self.verbose,
        }
        .run(aim)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdReschedule {
    pub ids: Vec<Id>,
    pub time: Option<DateTimeAnchor>,
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
        if todo_ids.is_empty() {
            CmdEventReschedule {
                ids: event_ids,
                time: self.time,
                output_format: OutputFormat::Table,
                verbose: self.verbose,
            }
            .run(aim)
            .await
        } else if event_ids.is_empty() {
            CmdTodoReschedule {
                ids: todo_ids,
                time: self.time,
                output_format: OutputFormat::Table,
                verbose: self.verbose,
            }
            .run(aim)
            .await
        } else {
            self.run_mix(aim, event_ids, todo_ids).await
        }
    }

    async fn run_mix(
        self,
        aim: &mut Aim,
        event_ids: Vec<Id>,
        todo_ids: Vec<Id>,
    ) -> Result<(), Box<dyn Error>> {
        // Prompt for time if not provided
        let time = match self.time {
            Some(t) => t,
            None => prompt_time()?,
        };

        println!("🗓️ {}", "Events".bold());

        CmdEventReschedule {
            ids: event_ids,
            time: Some(time.clone()),
            output_format: OutputFormat::Table,
            verbose: self.verbose,
        }
        .run(aim)
        .await?;

        println!();
        println!("✅ {}", "To-Dos".bold());

        CmdTodoReschedule {
            ids: todo_ids,
            time: Some(time),
            output_format: OutputFormat::Table,
            verbose: self.verbose,
        }
        .run(aim)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CmdFlush;

impl CmdFlush {
    pub const NAME: &str = "flush";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Flush the short IDs")
            .long_about(
                "\
Flush the short IDs by removing all entries from the short ID mapping table. \
This will clear all short ID mappings, requiring them to be regenerated as needed.",
            )
    }

    pub fn from(_matches: &ArgMatches) -> Self {
        Self
    }

    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "flushing short IDs...");
        aim.flush_short_ids().await?;
        println!("Short IDs flushed successfully.");
        Ok(())
    }
}

async fn separate_ids(aim: &Aim, ids: Vec<Id>) -> Result<(Vec<Id>, Vec<Id>), Box<dyn Error>> {
    let mut event_ids = vec![];
    let mut todo_ids = vec![];
    for id in ids {
        match aim.get_kind(&id).await? {
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
    fn parses_dashboard_command() {
        let args = ["dashboard"];
        let matches = CmdDashboard::command().try_get_matches_from(args).unwrap();
        let _ = CmdDashboard::from(&matches);
    }

    #[test]
    fn parses_delay_command() {
        let args = ["delay", "a", "b", "c", "--time", "1d", "--verbose"];
        let matches = CmdEventDelay::command().try_get_matches_from(args).unwrap();
        let parsed = CmdDelay::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.time, Some(DateTimeAnchor::InDays(1)));
        assert!(parsed.verbose);
    }

    #[test]
    fn parses_reschedule_command() {
        let args = ["reschedule", "a", "b", "c", "--time", "1h", "--verbose"];
        let matches = CmdReschedule::command().try_get_matches_from(args).unwrap();
        let parsed = CmdReschedule::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.time, Some(DateTimeAnchor::Relative(60 * 60)));
        assert!(parsed.verbose);
    }

    #[test]
    fn parses_flush_command() {
        let args = ["flush"];
        let matches = CmdFlush::command().try_get_matches_from(args).unwrap();
        let _ = CmdFlush::from(&matches);
    }
}
