// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides the tui command for the AIM CLI.
//! In general, they dont provide a comprehensive CLI command, but call TUI directly.

use std::error::Error;

use aimcal_core::{Aim, EventDraft, EventStatus, Id, Kind, Priority, TodoDraft, TodoStatus};
use clap::{ArgMatches, Command};

use crate::arg::{CommonArgs, EventArgs, EventOrTodoArgs, EventOrTodoStatus, TodoArgs};
use crate::cmd_event::{CmdEventEdit, CmdEventNew};
use crate::cmd_todo::{CmdTodoEdit, CmdTodoNew};
use crate::tui::{EventOrTodoDraft, draft_event_or_todo};
use crate::util::{OutputFormat, parse_datetime, parse_datetime_range};

#[derive(Debug, Clone)]
pub struct CmdNew {
    // fields
    pub kind: Option<Kind>,
    pub description: Option<String>,
    pub status: Option<EventOrTodoStatus>,
    pub summary: Option<String>,

    // fields (event specific)
    pub end: Option<String>,
    pub start: Option<String>,

    // fields (todo specific)
    pub due: Option<String>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,

    // options
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        let (args, event_args, todo_args) = args();
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new event or todo using TUI")
            // fields
            .arg(args.summary(true))
            .arg(args.description())
            .arg(args.status())
            // fields (event specific)
            .arg(event_args.start())
            .arg(event_args.end())
            // fields (todo specific)
            .arg(todo_args.due())
            .arg(todo_args.percent_complete())
            .arg(todo_args.priority())
            // options
            .arg(args.kind())
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            kind: EventOrTodoArgs::get_kind(matches),
            description: EventOrTodoArgs::get_description(matches),
            status: EventOrTodoArgs::get_status(matches),
            summary: EventOrTodoArgs::get_summary(matches),

            end: EventArgs::get_end(matches),
            start: EventArgs::get_start(matches),

            due: TodoArgs::get_due(matches),
            percent_complete: TodoArgs::get_percent_complete(matches),
            priority: TodoArgs::get_priority(matches),

            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        // TODO: check is it a event / todo
        tracing::debug!(?self, "adding new item using TUI...");

        // infer kind
        let inferred_kind = self.infer_kind();
        match inferred_kind {
            Some(Kind::Event) => {
                if self.due.is_some() || self.percent_complete.is_some() || self.priority.is_some()
                {
                    return Err("Cannot provide todo-specific fields for an event".into());
                } else if let Some(EventOrTodoStatus::Todo(_)) = self.status {
                    return Err("Cannot provide todo-specific status for an event".into());
                }

                // if all required fields are provided, create directly
                if !CmdEventNew::need_tui(&self.summary, &self.start) {
                    tracing::info!("creating new event");
                    let draft = self.draft_event(aim)?;
                    return CmdEventNew::new_event(aim, draft, self.output_format, self.verbose)
                        .await;
                }
            }
            Some(Kind::Todo) => {
                if self.start.is_some() || self.end.is_some() {
                    return Err("Cannot provide event-specific fields for a todo".into());
                } else if let Some(EventOrTodoStatus::Event(_)) = self.status {
                    return Err("Cannot provide event-specific status for a todo".into());
                }

                // if all required fields are provided, create directly
                if !CmdTodoNew::need_tui(&self.summary) {
                    tracing::info!("creating new todo");
                    let draft = self.draft_todo(aim)?;
                    return CmdTodoNew::new_todo(aim, draft, self.output_format, self.verbose)
                        .await;
                }
            }
            None => { /* do nothing */ }
        }

        self.run_tui(aim, inferred_kind).await
    }

    async fn run_tui(
        self,
        aim: &mut Aim,
        inferred_kind: Option<Kind>,
    ) -> Result<(), Box<dyn Error>> {
        tracing::info!("adding new item using TUI");
        let mut event_draft = aim.default_event_draft();
        let mut todo_draft = aim
            .default_todo_draft()
            .map_err(|e| format!("Failed to create default todo draft: {e}"))?;
        let now = aim.now();

        // fields
        //
        if let Some(desc) = self.description {
            event_draft.description = Some(desc.clone());
            todo_draft.description = Some(desc);
        }

        if let Some(summary) = self.summary {
            event_draft.summary.clone_from(&summary);
            todo_draft.summary.clone_from(&summary);
        }

        match self.status {
            Some(EventOrTodoStatus::Event(s)) => event_draft.status = s,
            Some(EventOrTodoStatus::Todo(s)) => todo_draft.status = s,
            None => { /* do nothing */ }
        }

        // fields (event specific)
        //
        match (self.start, self.end) {
            (Some(start), Some(end)) => {
                let (s, e) = parse_datetime_range(&now, &start, &end)?;
                event_draft.start = s;
                event_draft.end = e;
            }
            (Some(start), None) => event_draft.start = parse_datetime(&aim.now(), &start)?,
            (None, Some(end)) => event_draft.end = parse_datetime(&aim.now(), &end)?,
            (None, None) => { /* do nothing */ }
        }

        // fields (todo specific)
        //
        if let Some(due) = self.due {
            todo_draft.due = parse_datetime(&now, &due)?;
        }

        if let Some(pc) = self.percent_complete {
            todo_draft.percent_complete = Some(pc);
        }

        if let Some(priority) = self.priority {
            todo_draft.priority = Some(priority);
        }

        // launch TUI
        let draft = draft_event_or_todo(aim, inferred_kind, event_draft, todo_draft)?;
        match draft {
            Some(EventOrTodoDraft::Event(draft)) => {
                tracing::info!("creating new event");
                CmdEventNew::new_event(aim, draft, self.output_format, self.verbose).await
            }
            Some(EventOrTodoDraft::Todo(draft)) => {
                tracing::info!("creating new todo");
                CmdTodoNew::new_todo(aim, draft, self.output_format, self.verbose).await
            }
            None => {
                tracing::info!("user cancel the drafting");
                Ok(())
            }
        }
    }

    /// Infer the kind of the new item based on the provided fields.
    fn infer_kind(&self) -> Option<Kind> {
        if let Some(kind) = self.kind {
            Some(kind)
        } else if let Some(status) = self.status {
            // as status is mutually exclusive, we can infer the kind directly
            match status {
                EventOrTodoStatus::Event(_) => Some(Kind::Event),
                EventOrTodoStatus::Todo(_) => Some(Kind::Todo),
            }
        } else {
            let maybe_event = self.start.is_some() || self.end.is_some();
            let maybe_todo =
                self.due.is_some() || self.percent_complete.is_some() || self.priority.is_some();

            match (maybe_event, maybe_todo) {
                (true, false) => Some(Kind::Event),
                (false, true) => Some(Kind::Todo),
                (true, true) => {
                    tracing::info!(
                        "both event-specific and todo-specific fields are provided, cannot infer the kind"
                    );
                    None
                }
                (false, false) => None,
            }
        }
    }

    fn draft_event(&self, aim: &mut Aim) -> Result<EventDraft, Box<dyn Error>> {
        let mut draft = aim.default_event_draft();
        let now = aim.now();

        // fields
        //
        if let Some(desc) = &self.description {
            draft.description = Some(desc.clone());
        }

        if let Some(summary) = &self.summary {
            draft.summary.clone_from(summary);
        }

        if let Some(EventOrTodoStatus::Event(s)) = self.status {
            draft.status = s;
        }

        // fields (event specific)
        //
        match (&self.start, &self.end) {
            (Some(start), Some(end)) => {
                let (s, e) = parse_datetime_range(&now, start, end)?;
                draft.start = s;
                draft.end = e;
            }
            (Some(start), None) => draft.start = parse_datetime(&aim.now(), start)?,
            (None, Some(end)) => draft.end = parse_datetime(&aim.now(), end)?,
            (None, None) => { /* do nothing */ }
        }

        Ok(draft)
    }

    fn draft_todo(&self, aim: &mut Aim) -> Result<TodoDraft, Box<dyn Error>> {
        let mut draft = aim
            .default_todo_draft()
            .map_err(|e| format!("Failed to create default todo draft: {e}"))?;
        let now = aim.now();

        // fields
        //
        if let Some(desc) = &self.description {
            draft.description = Some(desc.clone());
        }

        if let Some(summary) = &self.summary {
            draft.summary.clone_from(summary);
        }

        if let Some(EventOrTodoStatus::Todo(s)) = self.status {
            draft.status = s;
        }

        // fields (todo specific)
        //
        if let Some(due) = &self.due {
            draft.due = parse_datetime(&now, due)?;
        }

        if let Some(pc) = self.percent_complete {
            draft.percent_complete = Some(pc);
        }

        if let Some(priority) = self.priority {
            draft.priority = Some(priority);
        }

        Ok(draft)
    }
}

#[derive(Debug, Clone)]
pub struct CmdEdit {
    pub id: Id,

    // fields
    pub description: Option<String>,
    pub status: Option<EventOrTodoStatus>,
    pub summary: Option<String>,

    // fields (event specific)
    pub end: Option<String>,
    pub start: Option<String>,

    // fields (todo specific)
    pub due: Option<String>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,

    // options
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        let (args, event_args, todo_args) = args();
        Command::new(Self::NAME)
            .about("Edit a event or todo using TUI")
            .arg(args.id())
            // fields
            .arg(args.summary(false))
            .arg(args.description())
            .arg(args.status())
            // fields (event specific)
            .arg(event_args.start())
            .arg(event_args.end())
            // fields (todo specific)
            .arg(todo_args.due())
            .arg(todo_args.percent_complete())
            .arg(todo_args.priority())
            // options
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: EventOrTodoArgs::get_id(matches),

            description: EventOrTodoArgs::get_description(matches),
            status: EventOrTodoArgs::get_status(matches),
            summary: EventOrTodoArgs::get_summary(matches),

            end: EventArgs::get_end(matches),
            start: EventArgs::get_start(matches),

            due: TodoArgs::get_due(matches),
            percent_complete: TodoArgs::get_percent_complete(matches),
            priority: TodoArgs::get_priority(matches),

            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "editing item using TUI...");

        let kind = aim.get_kind(&self.id).await?;

        // Check if any fields are provided to determine if we should use TUI mode
        let use_tui = match kind {
            Kind::Event => {
                tracing::debug!("item with id {} is an event", self.id.as_uid());
                if self.due.is_some() || self.percent_complete.is_some() || self.priority.is_some()
                {
                    return Err("Cannot provide todo-specific fields for an event".into());
                } else if let Some(EventOrTodoStatus::Todo(_)) = self.status {
                    return Err("Cannot provide todo-specific status for an event".into());
                }

                self.description.is_none()
                    && self.end.is_none()
                    && self.start.is_none()
                    && self.summary.is_none()
            }
            Kind::Todo => {
                tracing::debug!("item with id {} is a todo", self.id.as_uid());
                if self.start.is_some() || self.end.is_some() {
                    return Err("Cannot provide event-specific fields for a todo".into());
                } else if let Some(EventOrTodoStatus::Event(_)) = self.status {
                    return Err("Cannot provide event-specific status for a todo".into());
                }

                self.description.is_none()
                    && self.due.is_none()
                    && self.percent_complete.is_none()
                    && self.priority.is_none()
                    && self.summary.is_none()
            }
        };

        if use_tui {
            self.run_tui(aim, kind).await
        } else {
            self.run_direct(aim, kind).await
        }
    }

    /// Use TUI mode when no fields are provided
    async fn run_tui(self, aim: &mut Aim, kind: Kind) -> Result<(), Box<dyn Error>> {
        match kind {
            Kind::Event => {
                tracing::info!("editing event using TUI");
                CmdEventEdit::new_tui(self.id, self.output_format, self.verbose)
                    .run(aim)
                    .await
            }
            Kind::Todo => {
                tracing::info!("editing todo using TUI");
                CmdTodoEdit::new_tui(self.id, self.output_format, self.verbose)
                    .run(aim)
                    .await
            }
        }
    }

    /// Use direct edit mode when fields are provided
    async fn run_direct(self, aim: &mut Aim, kind: Kind) -> Result<(), Box<dyn Error>> {
        match kind {
            Kind::Event => {
                tracing::info!("editing event with provided fields");
                CmdEventEdit {
                    id: self.id,
                    description: self.description,
                    end: self.end,
                    start: self.start,
                    status: self.status.map(|s| match s {
                        EventOrTodoStatus::Event(status) => status,
                        EventOrTodoStatus::Todo(_) => EventStatus::default(),
                    }),
                    summary: self.summary,

                    output_format: self.output_format,
                    verbose: self.verbose,
                }
                .run(aim)
                .await
            }
            Kind::Todo => {
                tracing::info!("editing todo with provided fields");
                CmdTodoEdit {
                    id: self.id,
                    description: self.description,
                    due: self.due,
                    percent_complete: self.percent_complete,
                    priority: self.priority,
                    status: self.status.map(|s| match s {
                        EventOrTodoStatus::Todo(status) => status,
                        EventOrTodoStatus::Event(_) => TodoStatus::default(),
                    }),
                    summary: self.summary,

                    output_format: self.output_format,
                    verbose: self.verbose,
                }
                .run(aim)
                .await
            }
        }
    }
}

const fn args() -> (EventOrTodoArgs, EventArgs, TodoArgs) {
    (
        EventOrTodoArgs::new(None),
        EventArgs::new(false),
        TodoArgs::new(false),
    )
}
