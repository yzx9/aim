// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aimcal_core::{EventStatus, Id, Kind, Priority, TodoStatus};
use clap::{Arg, ArgMatches, arg, value_parser};
use clap_num::number_range;

use crate::util::OutputFormat;

#[derive(Debug, Clone, Copy)]
pub struct CommonArgs;

impl CommonArgs {
    pub fn verbose() -> Arg {
        arg!(-v --verbose "Show more detailed information")
    }

    pub fn get_verbose(matches: &ArgMatches) -> bool {
        matches.get_flag("verbose")
    }

    pub fn output_format() -> Arg {
        arg!(--"output-format" <FORMAT> "Output format")
            .value_parser(value_parser!(OutputFormat))
            .default_value("table")
    }

    pub fn get_output_format(matches: &ArgMatches) -> OutputFormat {
        matches
            .get_one("output-format")
            .copied()
            .unwrap_or(OutputFormat::Table)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventOrTodoArgs {
    kind: Option<Kind>,
}

impl EventOrTodoArgs {
    pub const fn new(kind: Option<Kind>) -> Self {
        Self { kind }
    }

    pub fn id(self) -> Arg {
        let help = format!("The short id or uid of the {} to edit", self.kind());
        arg!(id: <ID>).help(help)
    }

    pub fn get_id(matches: &ArgMatches) -> Id {
        let id = matches
            .get_one::<String>("id")
            .expect("id is required")
            .clone();

        Id::ShortIdOrUid(id)
    }

    pub fn ids(self) -> Arg {
        let help = format!("The short id or uid of the {} to edit", self.kind());
        arg!(id: <ID>).help(help).num_args(1..)
    }

    pub fn get_ids(matches: &ArgMatches) -> Vec<Id> {
        matches
            .get_many::<String>("id")
            .expect("id is required")
            .map(|a| Id::ShortIdOrUid(a.clone()))
            .collect()
    }

    pub fn description(self) -> Arg {
        let help = format!("Description of the {}", self.kind());
        arg!(--description <DESCRIPTION>).help(help)
    }

    pub fn get_description(matches: &ArgMatches) -> Option<String> {
        matches.get_one("description").cloned()
    }

    pub fn summary(self, positional: bool) -> Arg {
        let help = format!("Summary of the {}", self.kind());
        if positional {
            arg!(summary: <SUMMARY>).help(help).required(false)
        } else {
            arg!(summary: -s --summary <SUMMARY>).help(help)
        }
    }

    pub fn get_summary(matches: &ArgMatches) -> Option<String> {
        matches.get_one("summary").cloned()
    }

    fn kind(&self) -> String {
        match self.kind {
            Some(Kind::Event) => "event".to_string(),
            Some(Kind::Todo) => "todo".to_string(),
            None => "event or todo".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventArgs;

impl EventArgs {
    pub fn start() -> Arg {
        arg!(--start <START> "Start date and time of the event")
    }

    pub fn get_start(matches: &ArgMatches) -> Option<String> {
        matches.get_one("start").cloned()
    }

    pub fn end() -> Arg {
        arg!(--end <END> "End date and time of the event")
    }

    pub fn get_end(matches: &ArgMatches) -> Option<String> {
        matches.get_one("end").cloned()
    }

    pub fn status() -> Arg {
        arg!(--status <STATUS> "Status of the event").value_parser(value_parser!(EventStatus))
    }

    pub fn get_status(matches: &ArgMatches) -> Option<EventStatus> {
        matches.get_one("status").copied()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoArgs;

impl TodoArgs {
    pub fn due() -> Arg {
        arg!(--due <DUE> "Due date and time of the todo")
    }

    pub fn get_due(matches: &ArgMatches) -> Option<String> {
        matches.get_one("due").cloned()
    }

    pub fn percent_complete() -> Arg {
        pub fn from_0_to_100(s: &str) -> Result<u8, String> {
            number_range(s, 0, 100)
        }

        arg!(--percent <PERCENT> "Percent complete of the todo (0-100)").value_parser(from_0_to_100)
    }

    pub fn get_percent_complete(matches: &ArgMatches) -> Option<u8> {
        matches.get_one("percent").copied()
    }

    pub fn priority() -> Arg {
        arg!(-p --priority <PRIORITY> "Priority of the todo").value_parser(value_parser!(Priority))
    }

    pub fn get_priority(matches: &ArgMatches) -> Option<Priority> {
        matches.get_one("priority").copied()
    }

    pub fn status() -> Arg {
        arg!(--status <STATUS> "Status of the todo").value_parser(value_parser!(TodoStatus))
    }

    pub fn get_status(matches: &ArgMatches) -> Option<TodoStatus> {
        matches.get_one("status").copied()
    }

    pub fn time_anchor(kind: &str) -> Arg {
        arg!(<"TIME-ANCHOR">).help(format!("Time to {kind} (datetime, time, or 'tomorrow')"))
    }

    pub fn get_time_anchor(matches: &ArgMatches) -> String {
        matches
            .get_one::<String>("TIME-ANCHOR")
            .expect("time anchor is required")
            .clone()
    }
}
