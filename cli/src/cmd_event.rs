// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{
    Aim, DateTimeAnchor, Event, EventConditions, EventDraft, EventPatch, EventStatus, Id, Pager,
};
use clap::{Arg, ArgMatches, Command, arg};
use colored::Colorize;

use crate::event_formatter::{EventColumn, EventFormatter};
use crate::tui;
use crate::util::{
    ArgOutputFormat, arg_verbose, get_verbose, parse_datetime, parse_datetime_range,
};

#[derive(Debug, Clone)]
pub struct CmdEventNew {
    pub description: Option<String>,
    pub end: Option<String>,
    pub start: Option<String>,
    pub status: Option<EventStatus>,
    pub summary: Option<String>,

    pub tui: bool,
    pub output_format: ArgOutputFormat,
    pub verbose: bool,
}

impl CmdEventNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new event")
            .arg(arg_summary(true))
            .arg(arg_start())
            .arg(arg_end())
            .arg(arg_description())
            .arg(arg_status())
            .arg(ArgOutputFormat::arg())
            .arg(arg_verbose())
    }

    pub fn from(matches: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        let description = get_description(matches);
        let start = get_start(matches);
        let end = get_end(matches);
        let status = get_status(matches);

        let summary = match get_summary(matches) {
            Some(summary) => Some(summary.clone()), // TODO: is start/end required?

            None if description.is_none()
                && end.is_none()
                && start.is_none()
                && status.is_none() =>
            {
                None
            }

            // If summary is not provided but other fields are set, we still require a summary.
            None => return Err("Summary is required for new event".into()),
        };

        let tui = summary.is_none();
        Ok(Self {
            description,
            start,
            end,
            status,
            summary,

            tui,
            output_format: ArgOutputFormat::from(matches),
            verbose: get_verbose(matches),
        })
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "adding new todo...");
        let draft = if self.tui {
            match tui::draft_event(aim)? {
                Some(data) => data,
                None => {
                    tracing::info!("user cancel the event creation");
                    return Ok(());
                }
            }
        } else {
            let (start, end) = match (self.start, self.end) {
                (Some(start), Some(end)) => parse_datetime_range(&start, &end)?,
                (Some(start), None) => (parse_datetime(&start)?, None),
                (None, Some(end)) => (None, parse_datetime(&end)?),
                (None, None) => (None, None),
            };
            EventDraft {
                description: self.description,
                end,
                start,
                status: self.status.unwrap_or_default(),
                summary: self.summary.unwrap_or_default(),
            }
        };
        Self::new_event(aim, draft, self.output_format, self.verbose).await
    }

    pub async fn new_event(
        aim: &mut Aim,
        draft: EventDraft,
        output_format: ArgOutputFormat,
        verbose: bool,
    ) -> Result<(), Box<dyn Error>> {
        let event = aim.new_event(draft).await?;
        print_events(aim, &[event], output_format, verbose);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdEventEdit {
    pub id: Id,
    pub description: Option<String>,
    pub end: Option<String>,
    pub start: Option<String>,
    pub status: Option<EventStatus>,
    pub summary: Option<String>,

    pub tui: bool,
    pub output_format: ArgOutputFormat,
    pub verbose: bool,
}

impl CmdEventEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Edit a todo item")
            .arg(arg_id())
            .arg(arg_summary(false))
            .arg(arg_start())
            .arg(arg_end())
            .arg(arg_description())
            .arg(arg_status())
            .arg(ArgOutputFormat::arg())
            .arg(arg_verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        let id = get_id(matches);
        let description = get_description(matches);
        let start = get_start(matches);
        let end = get_end(matches);
        let status = get_status(matches);
        let summary = get_summary(matches);

        let tui = description.is_none()
            && end.is_none()
            && start.is_none()
            && status.is_none()
            && summary.is_none();

        Self {
            id,
            description,
            start,
            end,
            status,
            summary,

            tui,
            output_format: ArgOutputFormat::from(matches),
            verbose: get_verbose(matches),
        }
    }

    pub fn new_tui(id: Id, output_format: ArgOutputFormat, verbose: bool) -> Self {
        Self {
            id,
            description: None,
            end: None,
            start: None,
            status: None,
            summary: None,

            tui: true,
            output_format,
            verbose,
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "editing event...");
        let patch = if self.tui {
            let event = aim.get_event(&self.id).await?.ok_or("Event not found")?;
            match tui::patch_event(aim, &event)? {
                Some(data) => data,
                None => {
                    tracing::info!(?self, "user cancel the todo editing");
                    return Ok(());
                }
            }
        } else {
            let (start, end) = match (self.start, self.end) {
                (Some(start), Some(end)) => {
                    let (a, b) = parse_datetime_range(&start, &end)?;
                    (Some(a), Some(b))
                }
                (Some(start), None) => (Some(parse_datetime(&start)?), None),
                (None, Some(end)) => (None, Some(parse_datetime(&end)?)),
                (None, None) => (None, None),
            };
            EventPatch {
                description: self.description.map(|d| (!d.is_empty()).then_some(d)),
                end,
                start,
                status: self.status,
                summary: self.summary,
            }
        };

        let event = aim.update_event(&self.id, patch).await?;
        print_events(aim, &[event], self.output_format, self.verbose);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CmdEventList {
    pub conds: EventConditions,
    pub output_format: ArgOutputFormat,
    pub verbose: bool,
}

impl CmdEventList {
    pub const NAME: &str = "list";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("List events")
            .arg(ArgOutputFormat::arg())
            .arg(arg_verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            conds: EventConditions {
                startable: Some(DateTimeAnchor::today()),
                ..Default::default()
            },
            output_format: ArgOutputFormat::from(matches),
            verbose: get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "listing events...");
        Self::list(aim, &self.conds, self.output_format, self.verbose).await
    }

    /// List events with the given conditions and output format.
    pub async fn list(
        aim: &Aim,
        conds: &EventConditions,
        output_format: ArgOutputFormat,
        verbose: bool,
    ) -> Result<(), Box<dyn Error>> {
        const MAX: i64 = 128;
        let pager: Pager = (MAX, 0).into();
        let events = aim.list_events(conds, &pager).await?;
        if events.len() >= (MAX as usize) {
            let total = aim.count_events(conds).await?;
            if total > MAX {
                let prompt = format!("Displaying the {MAX}/{total} todos");
                println!("{}", prompt.italic());
            }
        } else if events.is_empty() && output_format == ArgOutputFormat::Table {
            println!("{}", "No events found".italic());
            return Ok(());
        }

        print_events(aim, &events, output_format, verbose);
        Ok(())
    }
}

fn arg_id() -> Arg {
    arg!(id: <ID> "The short id or uid of the event to edit")
}

fn get_id(matches: &ArgMatches) -> Id {
    let id = matches
        .get_one::<String>("id")
        .expect("id is required")
        .clone();

    Id::ShortIdOrUid(id)
}

fn arg_description() -> Arg {
    arg!(--description <DESCRIPTION> "Description of the event")
}

fn get_description(matches: &ArgMatches) -> Option<String> {
    matches.get_one("description").cloned()
}

fn arg_start() -> Arg {
    arg!(--start <START> "Start date and time of the event")
}

fn get_start(matches: &ArgMatches) -> Option<String> {
    matches.get_one("start").cloned()
}

fn arg_end() -> Arg {
    arg!(--end <END> "End date and time of the event")
}

fn get_end(matches: &ArgMatches) -> Option<String> {
    matches.get_one("end").cloned()
}

fn arg_status() -> Arg {
    clap::arg!(--status <STATUS> "Status of the event")
        .value_parser(clap::value_parser!(EventStatus))
}

fn get_status(matches: &ArgMatches) -> Option<EventStatus> {
    matches.get_one("status").copied()
}

fn arg_summary(positional: bool) -> Arg {
    match positional {
        true => arg!(summary: <SUMMARY> "Summary of the todo").required(false),
        false => arg!(summary: -s --summary <SUMMARY> "Summary of the event"),
    }
}

fn get_summary(matches: &ArgMatches) -> Option<String> {
    matches.get_one("summary").cloned()
}

fn print_events(aim: &Aim, events: &[impl Event], output_format: ArgOutputFormat, verbose: bool) {
    let columns = if verbose {
        vec![
            EventColumn::id(),
            EventColumn::uid(),
            EventColumn::datetime_span(),
            EventColumn::summary(),
        ]
    } else {
        vec![
            EventColumn::id(),
            EventColumn::datetime_span(),
            EventColumn::summary(),
        ]
    };
    let formatter = EventFormatter::new(aim.now(), columns).with_output_format(output_format);
    println!("{}", formatter.format(events));
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    #[test]
    fn test_parse_event_new() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdEventNew::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
                "new",
                "Another summary",
                "--description",
                "A description",
                "--start",
                "2025-01-01 12:00:00",
                "--end",
                "2025-01-01 14:00:00",
                "--status",
                "tentative",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let sub_matches = matches.subcommand_matches("new").unwrap();
        let parsed = CmdEventNew::from(sub_matches).unwrap();

        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.end, Some("2025-01-01 14:00:00".to_string()));
        assert_eq!(parsed.start, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.status, Some(EventStatus::Tentative));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui);
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_new_tui() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdEventNew::command());

        let matches = cmd.try_get_matches_from(["test", "new"]).unwrap();
        let sub_matches = matches.subcommand_matches("new").unwrap();
        let parsed = CmdEventNew::from(sub_matches).unwrap();
        assert!(parsed.tui);
    }

    #[test]
    fn test_parse_new_tui_invalid() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdEventNew::command());

        let matches = cmd
            .try_get_matches_from(["test", "new", "--start", "2025-01-01 12:00"])
            .unwrap();
        let sub_matches = matches.subcommand_matches("new").unwrap();
        let parsed = CmdEventNew::from(sub_matches);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_edit() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdEventEdit::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
                "edit",
                "test_id",
                "--description",
                "A description",
                "--start",
                "2025-01-01 12:00:00",
                "--end",
                "2025-01-01 14:00:00",
                "--status",
                "tentative",
                "--summary",
                "Another summary",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let sub_matches = matches.subcommand_matches("edit").unwrap();
        let parsed = CmdEventEdit::from(sub_matches);

        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.end, Some("2025-01-01 14:00:00".to_string()));
        assert_eq!(parsed.start, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.status, Some(EventStatus::Tentative));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui);
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_edit_tui() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdEventEdit::command());

        let matches = cmd
            .try_get_matches_from(["test", "edit", "test_id"])
            .unwrap();
        let sub_matches = matches.subcommand_matches("edit").unwrap();
        let parsed = CmdEventEdit::from(sub_matches);
        assert!(parsed.tui);
        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
    }

    #[test]
    fn test_parse_list() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdEventList::command());

        let matches = cmd
            .try_get_matches_from(["test", "list", "--output-format", "json", "--verbose"])
            .unwrap();

        let sub_matches = matches.subcommand_matches("list").unwrap();
        let parsed = CmdEventList::from(sub_matches);
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
        assert!(parsed.verbose);
    }
}
