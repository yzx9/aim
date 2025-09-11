// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{
    Aim, DateTimeAnchor, Event, EventConditions, EventDraft, EventPatch, EventStatus, Id, Kind,
    LooseDateTime, Pager,
};
use clap::{ArgMatches, Command};
use colored::Colorize;

use crate::arg::{CommonArgs, EventArgs, EventOrTodoArgs};
use crate::event_formatter::{EventColumn, EventFormatter};
use crate::tui;
use crate::util::{OutputFormat, parse_datetime, parse_datetime_range};

#[derive(Debug, Clone)]
pub struct CmdEventNew {
    pub description: Option<String>,
    pub end: Option<String>,
    pub start: Option<String>,
    pub status: Option<EventStatus>,
    pub summary: Option<String>,

    pub tui: bool,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEventNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        let (args, event_args) = args();
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new event")
            .arg(args.summary(true))
            .arg(event_args.start())
            .arg(event_args.end())
            .arg(args.description())
            .arg(event_args.status())
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        let description = EventOrTodoArgs::get_description(matches);
        let start = EventArgs::get_start(matches);
        let end = EventArgs::get_end(matches);
        let status = EventArgs::get_status(matches);

        let summary = match EventOrTodoArgs::get_summary(matches) {
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
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
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
                (Some(start), Some(end)) => parse_datetime_range(&aim.now(), &start, &end)?,
                (Some(start), None) => (parse_datetime(&aim.now(), &start)?, None),
                (None, Some(end)) => (None, parse_datetime(&aim.now(), &end)?),
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
        output_format: OutputFormat,
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
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEventEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        let (args, event_args) = args();
        Command::new(Self::NAME)
            .about("Edit a todo item")
            .arg(args.id())
            .arg(args.summary(false))
            .arg(event_args.start())
            .arg(event_args.end())
            .arg(args.description())
            .arg(event_args.status())
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        let id = EventOrTodoArgs::get_id(matches);
        let description = EventOrTodoArgs::get_description(matches);
        let start = EventArgs::get_start(matches);
        let end = EventArgs::get_end(matches);
        let status = EventArgs::get_status(matches);
        let summary = EventOrTodoArgs::get_summary(matches);

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
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub fn new_tui(id: Id, output_format: OutputFormat, verbose: bool) -> Self {
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
                    let (a, b) = parse_datetime_range(&aim.now(), &start, &end)?;
                    (Some(a), Some(b))
                }
                (Some(start), None) => (Some(parse_datetime(&aim.now(), &start)?), None),
                (None, Some(end)) => (None, Some(parse_datetime(&aim.now(), &end)?)),
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

#[derive(Debug, Clone)]
pub struct CmdEventDelay {
    pub ids: Vec<Id>,
    pub time_anchor: String,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEventDelay {
    pub const NAME: &str = "delay";

    pub fn command() -> Command {
        let (args, _event_args) = args();
        Command::new(Self::NAME)
            .about("Delay an event's time by a specified time based on original start")
            .arg(args.ids())
            .arg(args.time("delay"))
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            ids: EventOrTodoArgs::get_ids(matches),
            time_anchor: EventOrTodoArgs::get_time(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "delaying todo...");

        // Calculate new start and end based on original start and end if exists, otherwise based on now
        // TODO: move these logics to core crate, same for reschedule command
        let anchor: DateTimeAnchor = self.time_anchor.parse()?;
        let mut events = Vec::with_capacity(self.ids.len());
        for id in &self.ids {
            let event = aim.get_event(id).await?.ok_or("Event not found")?;
            let (start, end) = match (event.start(), event.end()) {
                (Some(start), end) => {
                    let s = anchor.parse_from_loose(&start);
                    let e = end.map(|a| anchor.parse_from_loose(&a));
                    (Some(s), e)
                }
                (None, Some(end)) => {
                    // TODO: should we set a start time with default duration? same for reschedule command
                    let e = anchor.parse_from_loose(&end);
                    (None, Some(e))
                }
                (None, None) => {
                    let s = anchor.parse_from_dt(&aim.now());
                    // TODO: should we set a end time with default duration? same for reschedule command
                    (Some(s), None)
                }
            };

            // Update the event
            let patch = EventPatch {
                start: Some(start),
                end: Some(end),
                ..Default::default()
            };

            let event = aim.update_event(id, patch).await?;
            events.push(event);
        }
        print_events(aim, &events, self.output_format, self.verbose);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdEventReschedule {
    pub ids: Vec<Id>,
    pub time_anchor: String,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEventReschedule {
    pub const NAME: &str = "reschedule";

    pub fn command() -> Command {
        let (args, _event_args) = args();
        Command::new(Self::NAME)
            .about("Reschedule a todo's due to a specified time based on now")
            .arg(args.ids())
            .arg(args.time("reschedule"))
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            ids: EventOrTodoArgs::get_ids(matches),
            time_anchor: EventOrTodoArgs::get_time(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "rescheduling event...");

        // Calculate new start and end based on original start and end if exists, otherwise based on now
        let anchor: DateTimeAnchor = self.time_anchor.parse()?;
        let mut events = Vec::with_capacity(self.ids.len());
        for id in &self.ids {
            let event = aim.get_event(id).await?.ok_or("Event not found")?;
            let (start, end) = match (event.start(), event.end()) {
                (Some(start), Some(end)) => {
                    let s = anchor.parse_from_dt(&aim.now());
                    let e = match (start, end) {
                        (LooseDateTime::DateOnly(ds), LooseDateTime::DateOnly(de)) => {
                            (s.date() + (de - ds)).into()
                        }
                        (LooseDateTime::DateOnly(ds), LooseDateTime::Floating(dte)) => {
                            (s.date() + (dte.date() - ds)).into()
                        }
                        (LooseDateTime::DateOnly(ds), LooseDateTime::Local(dte)) => {
                            (s.date() + (dte.date_naive() - ds)).into()
                        }
                        (LooseDateTime::Floating(dts), LooseDateTime::DateOnly(dte)) => {
                            s + (dte - dts.date())
                        }
                        (LooseDateTime::Floating(dts), LooseDateTime::Floating(dte)) => {
                            s + (dte - dts)
                        }
                        (LooseDateTime::Floating(dts), LooseDateTime::Local(dte)) => {
                            s + (dte.naive_local() - dts) // Treat floating as local
                        }
                        (LooseDateTime::Local(dts), LooseDateTime::DateOnly(de)) => {
                            s + (de - dts.date_naive())
                        }
                        (LooseDateTime::Local(dts), LooseDateTime::Floating(dte)) => {
                            s + (dte - dts.naive_local()) // Treat floating as local
                        }
                        (LooseDateTime::Local(dts), LooseDateTime::Local(dte)) => s + (dte - dts),
                    };
                    (Some(s), Some(e))
                }
                (_, None) => {
                    let s = anchor.parse_from_dt(&aim.now());
                    (Some(s), None)
                }
                (None, Some(_)) => {
                    let e = anchor.parse_from_dt(&aim.now());
                    (None, Some(e))
                }
            };

            // Update the event
            let patch = EventPatch {
                start: Some(start),
                end: Some(end),
                ..Default::default()
            };
            let event = aim.update_event(id, patch).await?;
            events.push(event);
        }
        print_events(aim, &events, self.output_format, self.verbose);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CmdEventList {
    pub conds: EventConditions,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEventList {
    pub const NAME: &str = "list";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("List events")
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            conds: EventConditions {
                startable: Some(DateTimeAnchor::today()),
                ..Default::default()
            },
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
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
        output_format: OutputFormat,
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
        } else if events.is_empty() && output_format == OutputFormat::Table {
            println!("{}", "No events found".italic());
            return Ok(());
        }

        print_events(aim, &events, output_format, verbose);
        Ok(())
    }
}

const fn args() -> (EventOrTodoArgs, EventArgs) {
    (
        EventOrTodoArgs::new(Some(Kind::Event)),
        EventArgs::new(true),
    )
}

fn print_events(aim: &Aim, events: &[impl Event], output_format: OutputFormat, verbose: bool) {
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

    #[test]
    fn test_parse_event_new() {
        let cmd = CmdEventNew::command();
        let matches = cmd
            .try_get_matches_from([
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
        let parsed = CmdEventNew::from(&matches).unwrap();

        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.end, Some("2025-01-01 14:00:00".to_string()));
        assert_eq!(parsed.start, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.status, Some(EventStatus::Tentative));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_new_tui() {
        let cmd = CmdEventNew::command();
        let matches = cmd.try_get_matches_from(["new"]).unwrap();
        let parsed = CmdEventNew::from(&matches).unwrap();

        assert!(parsed.tui);
    }

    #[test]
    fn test_parse_new_tui_invalid() {
        let cmd = CmdEventNew::command();
        let matches = cmd
            .try_get_matches_from(["new", "--start", "2025-01-01 12:00"])
            .unwrap();
        let parsed = CmdEventNew::from(&matches);

        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_edit() {
        let cmd = CmdEventEdit::command();
        let matches = cmd
            .try_get_matches_from([
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
        let parsed = CmdEventEdit::from(&matches);

        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.end, Some("2025-01-01 14:00:00".to_string()));
        assert_eq!(parsed.start, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.status, Some(EventStatus::Tentative));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_edit_tui() {
        let cmd = CmdEventEdit::command();
        let matches = cmd.try_get_matches_from(["edit", "test_id"]).unwrap();
        let parsed = CmdEventEdit::from(&matches);

        assert!(parsed.tui);
        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
    }

    #[test]
    fn test_parse_delay() {
        let cmd = CmdEventDelay::command();
        let matches = cmd
            .try_get_matches_from([
                "delay",
                "a",
                "b",
                "c",
                "--time",
                "time",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let parsed = CmdEventDelay::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.time_anchor, "time");
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_reschedule() {
        let cmd = CmdEventReschedule::command();
        let matches = cmd
            .try_get_matches_from([
                "reschedule",
                "a",
                "b",
                "c",
                "--time",
                "time",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let parsed = CmdEventReschedule::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.time_anchor, "time");
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_list() {
        let cmd = CmdEventList::command();
        let matches = cmd
            .try_get_matches_from(["list", "--output-format", "json", "--verbose"])
            .unwrap();
        let parsed = CmdEventList::from(&matches);

        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }
}
