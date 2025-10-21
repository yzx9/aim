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
use crate::prompt::prompt_time;
use crate::tui;
use crate::util::{OutputFormat, parse_datetime, parse_datetime_range};

#[derive(Debug, Clone)]
pub struct CmdEventNew {
    pub description: Option<String>,
    pub end: Option<String>,
    pub start: Option<String>,
    pub status: Option<EventStatus>,
    pub summary: Option<String>,

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

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            description: EventOrTodoArgs::get_description(matches),
            start: EventArgs::get_start(matches),
            end: EventArgs::get_end(matches),
            status: EventArgs::get_status(matches),
            summary: EventOrTodoArgs::get_summary(matches),

            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "adding new event...");

        let tui = self.tui();
        let now = aim.now();

        // Prepare a draft with the provided arguments
        let mut draft = aim.default_event_draft();

        let (start, end) = match (self.start, self.end) {
            (Some(start), Some(end)) => parse_datetime_range(&now, &start, &end)?,
            (Some(start), None) => (parse_datetime(&now, &start)?, None),
            (None, Some(end)) => (None, parse_datetime(&now, &end)?),
            (None, None) => (None, None),
        };
        draft.start = start;
        draft.end = end;

        if let Some(description) = self.description {
            draft.description = Some(description);
        }

        if let Some(status) = self.status {
            draft.status = status;
        }

        if let Some(summary) = self.summary {
            draft.summary = summary;
        }

        // If TUI is needed, launch the TUI to edit the draft
        if tui {
            draft = match tui::draft_event(aim)? {
                Some(data) => data,
                None => {
                    tracing::info!("user cancel the event creation");
                    return Ok(());
                }
            }
        }

        // Create the event
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

    pub(crate) fn tui(&self) -> bool {
        Self::need_tui(&self.summary, &self.start)
    }

    /// Determine whether TUI is needed based on the provided arguments.
    #[allow(clippy::ref_option)]
    pub(crate) fn need_tui(summary: &Option<String>, start: &Option<String>) -> bool {
        summary.is_none() || start.is_none()
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

    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEventEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        let (args, event_args) = args();
        Command::new(Self::NAME)
            .about("Edit a event item")
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
        Self {
            id: EventOrTodoArgs::get_id(matches),
            description: EventOrTodoArgs::get_description(matches),
            start: EventArgs::get_start(matches),
            end: EventArgs::get_end(matches),
            status: EventArgs::get_status(matches),
            summary: EventOrTodoArgs::get_summary(matches),

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

            output_format,
            verbose,
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "editing event...");
        let tui = self.tui();

        // Prepare the patch with the provided arguments
        let (start, end) = match (self.start, self.end) {
            (Some(start), Some(end)) => {
                let (a, b) = parse_datetime_range(&aim.now(), &start, &end)?;
                (Some(a), Some(b))
            }
            (Some(start), None) => (Some(parse_datetime(&aim.now(), &start)?), None),
            (None, Some(end)) => (None, Some(parse_datetime(&aim.now(), &end)?)),
            (None, None) => (None, None),
        };
        let mut patch = EventPatch {
            description: self.description.map(|d| (!d.is_empty()).then_some(d)),
            end,
            start,
            status: self.status,
            summary: self.summary,
        };

        // If TUI is needed, launch the TUI to edit the event
        if tui {
            let event = aim.get_event(&self.id).await?;
            patch = match tui::patch_event(aim, &event, patch)? {
                Some(data) => data,
                None => {
                    tracing::info!("user cancel the event editing");
                    return Ok(());
                }
            }
        }

        // Update the event
        let event = aim.update_event(&self.id, patch).await?;
        print_events(aim, &[event], self.output_format, self.verbose);
        Ok(())
    }

    /// Determine whether TUI is needed based on the provided arguments.
    pub(crate) fn tui(&self) -> bool {
        self.description.is_none()
            && self.end.is_none()
            && self.start.is_none()
            && self.status.is_none()
            && self.summary.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct CmdEventDelay {
    pub ids: Vec<Id>,
    pub time: Option<DateTimeAnchor>,
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
            time: EventOrTodoArgs::get_time(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "delaying event...");

        // Prompt for time if not provided
        let time = match self.time {
            Some(t) => t,
            None => prompt_time()?,
        };

        // Calculate new start and end based on original start and end if exists, otherwise based on now
        // TODO: move these logics to core crate, same for reschedule command
        let mut events = Vec::with_capacity(self.ids.len());
        for id in &self.ids {
            let event = aim.get_event(id).await?;
            let (start, end) = match (event.start(), event.end()) {
                (Some(start), end) => {
                    let s = time.resolve_at(&start);
                    let e = end.map(|a| time.resolve_at(&a));
                    (Some(s), e)
                }
                (None, Some(end)) => {
                    // TODO: should we set a start time with default duration? same for reschedule command
                    let e = time.resolve_at(&end);
                    (None, Some(e))
                }
                (None, None) => {
                    let s = time.resolve_since_datetime(&aim.now());
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
    pub time: Option<DateTimeAnchor>,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEventReschedule {
    pub const NAME: &str = "reschedule";

    pub fn command() -> Command {
        let (args, _event_args) = args();
        Command::new(Self::NAME)
            .about("Reschedule event's due to a specified time based on now")
            .arg(args.ids())
            .arg(args.time("reschedule"))
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            ids: EventOrTodoArgs::get_ids(matches),
            time: EventOrTodoArgs::get_time(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "rescheduling event...");

        // Prompt for time if not provided
        let time = match self.time {
            Some(t) => t,
            None => prompt_time()?,
        };

        // Calculate new start and end based on original start and end if exists, otherwise based on now
        let mut events = Vec::with_capacity(self.ids.len());
        for id in &self.ids {
            let event = aim.get_event(id).await?;
            let (start, end) = match (event.start(), event.end()) {
                (Some(start), Some(end)) => {
                    use LooseDateTime::{DateOnly, Floating, Local};
                    let s = time.resolve_since_datetime(&aim.now());
                    #[rustfmt::skip]
                    let e = match (start, end) {
                        (DateOnly(ds),  DateOnly(de))  => (s.date() + (de - ds)).into(),
                        (DateOnly(ds),  Floating(dte)) => (s.date() + (dte.date() - ds)).into(),
                        (DateOnly(ds),  Local(dte))    => (s.date() + (dte.date_naive() - ds)).into(),
                        (Floating(dts), DateOnly(dte)) => s + (dte - dts.date()),
                        (Floating(dts), Floating(dte)) => s + (dte - dts),
                        (Floating(dts), Local(dte))    => s + (dte.naive_local() - dts), // Treat floating as local
                        (Local(dts),    DateOnly(de))  => s + (de - dts.date_naive()),
                        (Local(dts),    Floating(dte)) => s + (dte - dts.naive_local()), // Treat floating as local
                        (Local(dts),    Local(dte))    => s + (dte - dts),
                    };
                    (Some(s), Some(e))
                }
                (_, None) => {
                    let s = time.resolve_since_datetime(&aim.now());
                    (Some(s), None)
                }
                (None, Some(_)) => {
                    let e = time.resolve_since_datetime(&aim.now());
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
    #[allow(clippy::cast_possible_truncation)]
    pub async fn list(
        aim: &Aim,
        conds: &EventConditions,
        output_format: OutputFormat,
        verbose: bool,
    ) -> Result<(), Box<dyn Error>> {
        const LIMIT: i64 = 128;

        let pager: Pager = (LIMIT, 0).into();
        let events = aim.list_events(conds, &pager).await?;
        if events.len() >= (LIMIT as usize) {
            let total = aim.count_events(conds).await?;
            if total > LIMIT {
                let prompt = format!("Displaying the {LIMIT}/{total} events");
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

// TODO: remove `verbose` in v0.12.0
fn print_events(aim: &Aim, events: &[impl Event], output_format: OutputFormat, verbose: bool) {
    use EventColumn::{DateTimeSpan, Id, ShortId, Summary, Uid, UidLegacy};
    let columns = match (output_format, verbose) {
        (_, true) => vec![Id, UidLegacy, DateTimeSpan, Summary],
        (OutputFormat::Table, false) => vec![Id, DateTimeSpan, Summary],
        (OutputFormat::Json, false) => vec![Uid, ShortId, DateTimeSpan, Summary],
    };
    let formatter = EventFormatter::new(aim.now(), columns, output_format);
    println!("{}", formatter.format(events));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event_new() {
        let args = [
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
        ];
        let matches = CmdEventNew::command().try_get_matches_from(args).unwrap();
        let parsed = CmdEventNew::from(&matches);

        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.end, Some("2025-01-01 14:00:00".to_string()));
        assert_eq!(parsed.start, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.status, Some(EventStatus::Tentative));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui());
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_new_tui() {
        let args = ["new"];
        let matches = CmdEventNew::command().try_get_matches_from(args).unwrap();
        let parsed = CmdEventNew::from(&matches);

        assert!(parsed.tui());
    }

    #[test]
    fn test_parse_edit() {
        let args = [
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
        ];
        let matches = CmdEventEdit::command().try_get_matches_from(args).unwrap();
        let parsed = CmdEventEdit::from(&matches);

        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.end, Some("2025-01-01 14:00:00".to_string()));
        assert_eq!(parsed.start, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.status, Some(EventStatus::Tentative));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui());
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_edit_tui() {
        let cmd = CmdEventEdit::command();
        let matches = cmd.try_get_matches_from(["edit", "test_id"]).unwrap();
        let parsed = CmdEventEdit::from(&matches);

        assert!(parsed.tui());
        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
    }

    #[test]
    fn test_parse_delay() {
        let args = [
            "delay",
            "a",
            "b",
            "c",
            "--time",
            "1d",
            "--output-format",
            "json",
            "--verbose",
        ];
        let matches = CmdEventDelay::command().try_get_matches_from(args).unwrap();
        let parsed = CmdEventDelay::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.time, Some(DateTimeAnchor::InDays(1)));
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_reschedule() {
        let args = [
            "reschedule",
            "a",
            "b",
            "c",
            "--time",
            "1d",
            "--output-format",
            "json",
            "--verbose",
        ];
        let matches = CmdEventReschedule::command()
            .try_get_matches_from(args)
            .unwrap();
        let parsed = CmdEventReschedule::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.time, Some(DateTimeAnchor::InDays(1)));
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_list() {
        let args = ["list", "--output-format", "json", "--verbose"];
        let matches = CmdEventList::command().try_get_matches_from(args).unwrap();
        let parsed = CmdEventList::from(&matches);

        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }
}
