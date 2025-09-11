// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::sync::OnceLock;

use aimcal_core::{EventStatus, Id, Kind, Priority, TodoStatus};
use clap::{Arg, ArgMatches, ValueEnum, arg, value_parser};
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

    pub fn kind(&self) -> Arg {
        let arg = arg!(-t --type <TYPE>)
            .value_parser(value_parser!(Kind))
            .help("Type of item to edit");

        match self.kind {
            Some(Kind::Event) => arg.default_value("event").hide(true),
            Some(Kind::Todo) => arg.default_value("todo").hide(true),
            None => arg,
        }
    }

    pub fn get_kind(matches: &ArgMatches) -> Option<Kind> {
        matches.get_one("type").cloned()
    }

    pub fn id(&self) -> Arg {
        arg!(id: <ID>).help(format!(
            "The short id or uid of the {} to edit",
            self.kind_name()
        ))
    }

    pub fn get_id(matches: &ArgMatches) -> Id {
        let id = matches
            .get_one::<String>("id")
            .expect("id is required")
            .clone();

        Id::ShortIdOrUid(id)
    }

    pub fn ids(&self) -> Arg {
        arg!(id: <ID>)
            .help(format!(
                "The short id or uid of the {} to edit",
                self.kind_name()
            ))
            .num_args(1..)
    }

    pub fn get_ids(matches: &ArgMatches) -> Vec<Id> {
        matches
            .get_many::<String>("id")
            .expect("id is required")
            .map(|a| Id::ShortIdOrUid(a.clone()))
            .collect()
    }

    pub fn description(&self) -> Arg {
        arg!(--description <DESCRIPTION>).help(format!("Description of the {}", self.kind_name()))
    }

    pub fn get_description(matches: &ArgMatches) -> Option<String> {
        matches.get_one("description").cloned()
    }

    /// Status of either event or todo. Prefer using `EventArgs::status` or `TodoArgs::status` if
    /// the kind is known.
    pub fn status(&self) -> Arg {
        arg!(--status <STATUS>)
            .help(format!("Status of the {}", self.kind_name()))
            .value_parser(value_parser!(EventOrTodoStatus))
    }

    pub fn get_status(matches: &ArgMatches) -> Option<EventOrTodoStatus> {
        matches.get_one("status").copied()
    }

    pub fn summary(&self, positional: bool) -> Arg {
        let help = format!("Summary of the {}", self.kind_name());
        if positional {
            arg!(summary: [SUMMARY]).help(help)
        } else {
            arg!(summary: -s --summary <SUMMARY>).help(help)
        }
    }

    pub fn get_summary(matches: &ArgMatches) -> Option<String> {
        matches.get_one("summary").cloned()
    }

    pub fn time_anchor(&self, kind: &str) -> Arg {
        arg!(<"TIME-ANCHOR">).help(format!("Time to {} (datetime, time, or 'tomorrow')", kind))
    }

    pub fn get_time_anchor(matches: &ArgMatches) -> String {
        matches
            .get_one::<String>("TIME-ANCHOR")
            .expect("time anchor is required")
            .clone()
    }

    fn kind_name(&self) -> String {
        match self.kind {
            Some(Kind::Event) => "event".to_string(),
            Some(Kind::Todo) => "todo".to_string(),
            None => "event or todo".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventArgs {
    monopolize: bool,
}

impl EventArgs {
    pub const fn new(monopolize: bool) -> Self {
        Self { monopolize }
    }

    pub fn start(&self) -> Arg {
        arg!(--start <START>).help(self.monopolize("Start date and time of the event"))
    }

    pub fn get_start(matches: &ArgMatches) -> Option<String> {
        matches.get_one("start").cloned()
    }

    pub fn end(&self) -> Arg {
        arg!(--end <END>).help(self.monopolize("End date and time of the event"))
    }

    pub fn get_end(matches: &ArgMatches) -> Option<String> {
        matches.get_one("end").cloned()
    }

    pub fn status(&self) -> Arg {
        arg!(--status <STATUS>)
            .help(self.monopolize("Status of the event"))
            .value_parser(value_parser!(EventStatus))
    }

    pub fn get_status(matches: &ArgMatches) -> Option<EventStatus> {
        matches.get_one("status").copied()
    }

    fn monopolize(&self, help: impl ToString) -> String {
        if self.monopolize {
            help.to_string()
        } else {
            help.to_string() + " (event specific)"
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TodoArgs {
    monopolize: bool,
}

impl TodoArgs {
    pub const fn new(monopolize: bool) -> Self {
        Self { monopolize }
    }

    pub fn due(&self) -> Arg {
        arg!(--due <DUE>).help(self.monopolize("Due date and time of the todo"))
    }

    pub fn get_due(matches: &ArgMatches) -> Option<String> {
        matches.get_one("due").cloned()
    }

    pub fn percent_complete(&self) -> Arg {
        pub fn from_0_to_100(s: &str) -> Result<u8, String> {
            number_range(s, 0, 100)
        }

        arg!(--percent <PERCENT>)
            .help(self.monopolize("Percent complete of the todo (0-100)"))
            .value_parser(from_0_to_100)
    }

    pub fn get_percent_complete(matches: &ArgMatches) -> Option<u8> {
        matches.get_one("percent").copied()
    }

    pub fn priority(&self) -> Arg {
        arg!(-p --priority <PRIORITY>)
            .help(self.monopolize("Priority of the todo"))
            .value_parser(value_parser!(Priority))
    }

    pub fn get_priority(matches: &ArgMatches) -> Option<Priority> {
        matches.get_one("priority").copied()
    }

    pub fn status(&self) -> Arg {
        arg!(--status <STATUS>)
            .help(self.monopolize("Status of the todo"))
            .value_parser(value_parser!(TodoStatus))
    }

    pub fn get_status(matches: &ArgMatches) -> Option<TodoStatus> {
        matches.get_one("status").copied()
    }

    fn monopolize(&self, help: impl ToString) -> String {
        if self.monopolize {
            help.to_string()
        } else {
            help.to_string() + " (todo specific)"
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventOrTodoStatus {
    Event(EventStatus),
    Todo(TodoStatus),
}

impl ValueEnum for EventOrTodoStatus {
    fn value_variants<'a>() -> &'a [Self] {
        static VARIANTS: OnceLock<Box<[EventOrTodoStatus]>> = OnceLock::new();

        VARIANTS.get_or_init(|| {
            let events = EventStatus::value_variants()
                .iter()
                .copied()
                .map(EventOrTodoStatus::Event);

            let todos = TodoStatus::value_variants()
                .iter()
                .copied()
                .map(EventOrTodoStatus::Todo);

            events.chain(todos).collect::<Vec<_>>().into_boxed_slice()
        })
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            EventOrTodoStatus::Event(status) => status.to_possible_value(),
            EventOrTodoStatus::Todo(status) => status.to_possible_value(),
        }
    }
}
