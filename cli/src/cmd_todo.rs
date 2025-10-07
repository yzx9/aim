// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{
    Aim, DateTimeAnchor, Id, Kind, Priority, SortOrder, Todo, TodoConditions, TodoDraft, TodoPatch,
    TodoSort, TodoStatus,
};
use clap::{ArgMatches, Command};
use colored::Colorize;

use crate::arg::{CommonArgs, EventOrTodoArgs, TodoArgs};
use crate::prompt::{prompt_time, prompt_time_opt};
use crate::todo_formatter::{TodoColumn, TodoFormatter};
use crate::tui;
use crate::util::{OutputFormat, parse_datetime};

#[derive(Debug, Clone)]
pub struct CmdTodoNew {
    pub description: Option<String>,
    pub due: Option<String>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,
    pub status: Option<TodoStatus>,
    pub summary: Option<String>,

    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdTodoNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        let (args, todo_args) = args();
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new todo")
            // fields
            .arg(args.summary(true))
            .arg(todo_args.due())
            .arg(args.description())
            .arg(todo_args.percent_complete())
            .arg(todo_args.priority())
            .arg(todo_args.status())
            // options
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            description: EventOrTodoArgs::get_description(matches),
            due: TodoArgs::get_due(matches),
            percent_complete: TodoArgs::get_percent_complete(matches),
            priority: TodoArgs::get_priority(matches),
            status: TodoArgs::get_status(matches),
            summary: EventOrTodoArgs::get_summary(matches),

            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "adding new todo...");
        let tui = self.tui();
        let now = aim.now();

        // Prepare a draft with the provided arguments
        let mut draft = aim.default_todo_draft();

        if let Some(desc) = self.description {
            draft.description = Some(desc);
        }

        if let Some(due) = &self.due {
            draft.due = parse_datetime(&now, due)?;
        }

        if let Some(percent) = self.percent_complete {
            draft.percent_complete = Some(percent);
        }

        if let Some(priority) = self.priority {
            draft.priority = Some(priority);
        }

        if let Some(status) = self.status {
            draft.status = status;
        }

        if let Some(summary) = &self.summary {
            draft.summary = summary.clone();
        }

        // If TUI is needed, launch the TUI editor to let user edit the draft
        if tui {
            draft = match tui::draft_todo(aim, draft)? {
                Some(data) => data,
                None => {
                    tracing::info!("user cancel the todo editing");
                    return Ok(());
                }
            }
        }

        // Create the todo
        Self::new_todo(aim, draft, self.output_format, self.verbose).await
    }

    pub async fn new_todo(
        aim: &mut Aim,
        draft: TodoDraft,
        output_format: OutputFormat,
        verbose: bool,
    ) -> Result<(), Box<dyn Error>> {
        let todo = aim.new_todo(draft).await?;
        print_todos(aim, &[todo], output_format, verbose);
        Ok(())
    }

    pub(crate) fn tui(&self) -> bool {
        Self::need_tui(&self.summary)
    }

    /// Determine whether to use TUI mode, which is true if not all required fields are provided
    pub(crate) fn need_tui(summary: &Option<String>) -> bool {
        summary.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoEdit {
    pub id: Id,
    pub description: Option<String>,
    pub due: Option<String>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,
    pub status: Option<TodoStatus>,
    pub summary: Option<String>,

    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdTodoEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        let (args, todo_args) = args();
        Command::new(Self::NAME)
            .about("Edit a todo")
            .arg(args.id())
            .arg(args.summary(false))
            .arg(todo_args.due())
            .arg(args.description())
            .arg(todo_args.percent_complete())
            .arg(todo_args.priority())
            .arg(todo_args.status())
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: EventOrTodoArgs::get_id(matches),
            description: EventOrTodoArgs::get_description(matches),
            due: TodoArgs::get_due(matches),
            percent_complete: TodoArgs::get_percent_complete(matches),
            priority: TodoArgs::get_priority(matches),
            status: TodoArgs::get_status(matches),
            summary: EventOrTodoArgs::get_summary(matches),

            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub fn new_tui(id: Id, output_format: OutputFormat, verbose: bool) -> Self {
        Self {
            id,
            description: None,
            due: None,
            percent_complete: None,
            priority: None,
            status: None,
            summary: None,

            output_format,
            verbose,
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "editing todo...");
        let tui = self.tui();

        // Prepare a patch with the provided arguments
        let mut patch = TodoPatch {
            description: self.description.map(|d| (!d.is_empty()).then_some(d)),
            due: self
                .due
                .as_ref()
                .map(|a| parse_datetime(&aim.now(), a))
                .transpose()?,
            priority: self.priority,
            percent_complete: None,
            status: self.status,
            summary: self.summary,
        };

        // If TUI is needed, launch the TUI editor to let user edit the patch
        if tui {
            let todo = aim.get_todo(&self.id).await?;
            patch = match tui::patch_todo(aim, &todo, patch)? {
                Some(data) => data,
                None => {
                    tracing::info!("user cancel the todo editing");
                    return Ok(());
                }
            };
        }

        // If no fields to edit, do nothing
        let todo = aim.update_todo(&self.id, patch).await?;
        print_todos(aim, &[todo], self.output_format, self.verbose);
        Ok(())
    }

    /// Determine whether to use TUI mode, which is true if no fields to edit are provided
    pub(crate) fn tui(&self) -> bool {
        self.description.is_none()
            && self.due.is_none()
            && self.percent_complete.is_none()
            && self.priority.is_none()
            && self.status.is_none()
            && self.summary.is_none()
    }
}

macro_rules! cmd_status {
    ($cmd: ident, $status:ident, $name: expr, $desc: expr) => {
        #[derive(Debug, Clone)]
        pub struct $cmd {
            pub ids: Vec<Id>,
            pub output_format: OutputFormat,
            pub verbose: bool,
        }

        impl $cmd {
            pub const NAME: &str = $name;

            pub fn command() -> Command {
                let (args, _todo_args) = args();
                Command::new(Self::NAME)
                    .about(concat!("Mark a todo as ", $desc))
                    .arg(args.ids())
                    .arg(CommonArgs::output_format())
                    .arg(CommonArgs::verbose())
            }

            pub fn from(matches: &ArgMatches) -> Self {
                Self {
                    ids: EventOrTodoArgs::get_ids(matches),
                    output_format: CommonArgs::get_output_format(matches),
                    verbose: CommonArgs::get_verbose(matches),
                }
            }

            pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
                tracing::debug!(?self, concat!("marking todos as ", $desc));
                let mut todos = vec![];
                for id in self.ids {
                    let patch = TodoPatch {
                        status: Some(TodoStatus::$status),
                        ..Default::default()
                    };
                    let todo = aim.update_todo(&id, patch).await?;
                    todos.push(todo);
                }
                print_todos(aim, &todos, self.output_format, self.verbose);
                Ok(())
            }
        }
    };
}

cmd_status!(CmdTodoUndo, NeedsAction, "undo", "needs-action");
cmd_status!(CmdTodoDone, Completed, "done", "completed");
cmd_status!(CmdTodoCancel, Cancelled, "cancel", "canceled");

#[derive(Debug, Clone)]
pub struct CmdTodoDelay {
    pub ids: Vec<Id>,
    pub time: Option<DateTimeAnchor>,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdTodoDelay {
    pub const NAME: &str = "delay";

    pub fn command() -> Command {
        let (args, _todo_args) = args();
        Command::new(Self::NAME)
            .about("Delay todo's due by a specified time based on original due")
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
        tracing::debug!(?self, "delaying todo...");

        // Prompt for time if not provided
        let time = match self.time {
            Some(t) => t,
            None => prompt_time()?,
        };

        let mut todos = vec![];
        for id in &self.ids {
            // Calculate new due based on original due if exists, otherwise based on now
            let todo = aim.get_todo(id).await?;
            let new_due = Some(match todo.due() {
                Some(due) => time.resolve_at(&due),
                None => time.resolve_since_datetime(&aim.now()),
            });

            // Update the todo
            let patch = TodoPatch {
                due: Some(new_due),
                ..Default::default()
            };
            let todo = aim.update_todo(id, patch).await?;
            todos.push(todo);
        }
        print_todos(aim, &todos, self.output_format, self.verbose);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoReschedule {
    pub ids: Vec<Id>,
    pub time: Option<DateTimeAnchor>,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdTodoReschedule {
    pub const NAME: &str = "reschedule";

    pub fn command() -> Command {
        let (args, _todo_args) = args();
        Command::new(Self::NAME)
            .about("Reschedule todo's due to a specified time based on now")
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
        tracing::debug!(?self, "rescheduling todo...");

        // Prompt for time if not provided
        let time = match self.time {
            Some(t) => Some(t),
            None => prompt_time_opt()?,
        };

        let mut todos = vec![];
        for id in &self.ids {
            // Calculate new due based on now
            let new_due = time.map(|a| a.resolve_since_datetime(&aim.now()));

            // Update the todo
            let patch = TodoPatch {
                due: Some(new_due),
                ..Default::default()
            };
            let todo = aim.update_todo(id, patch).await?;
            todos.push(todo);
        }
        print_todos(aim, &todos, self.output_format, self.verbose);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CmdTodoList {
    pub conds: TodoConditions,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdTodoList {
    pub const NAME: &str = "list";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("List todos")
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            conds: TodoConditions {
                status: Some(TodoStatus::NeedsAction),
                due: None,
            },
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "listing todos...");
        Self::list(aim, &self.conds, self.output_format, self.verbose).await?;
        Ok(())
    }

    pub async fn list(
        aim: &Aim,
        conds: &TodoConditions,
        output_format: OutputFormat,
        verbose: bool,
    ) -> Result<(), Box<dyn Error>> {
        const MAX: i64 = 128;
        let pager = (MAX, 0).into();
        let sort = vec![
            TodoSort::Priority {
                order: SortOrder::Desc,
                none_first: None,
            },
            TodoSort::Due(SortOrder::Desc),
        ];
        let todos = aim.list_todos(conds, &sort, &pager).await?;
        if todos.len() >= (MAX as usize) {
            let total = aim.count_todos(conds).await?;
            if total > MAX {
                let prompt = format!("Displaying the {MAX}/{total} todos");
                println!("{}", prompt.italic());
            }
        } else if todos.is_empty() && output_format == OutputFormat::Table {
            println!("{}", "No todos found".italic());
        }

        print_todos(aim, &todos, output_format, verbose);
        Ok(())
    }
}

const fn args() -> (EventOrTodoArgs, TodoArgs) {
    (EventOrTodoArgs::new(Some(Kind::Todo)), TodoArgs::new(true))
}

// TODO: remove `verbose` in v0.12.0
fn print_todos(aim: &Aim, todos: &[impl Todo], output_format: OutputFormat, verbose: bool) {
    let columns = match (output_format, verbose) {
        (_, true) => vec![
            TodoColumn::Status,
            TodoColumn::Id,
            TodoColumn::UidLegacy,
            TodoColumn::Priority,
            TodoColumn::Due,
            TodoColumn::Summary,
        ],
        (OutputFormat::Table, false) => vec![
            TodoColumn::Status,
            TodoColumn::Id,
            TodoColumn::Priority,
            TodoColumn::Due,
            TodoColumn::Summary,
        ],
        (OutputFormat::Json, false) => vec![
            TodoColumn::Uid,
            TodoColumn::ShortId,
            TodoColumn::Status,
            TodoColumn::Priority,
            TodoColumn::Due,
            TodoColumn::Summary,
        ],
    };
    let formatter = TodoFormatter::new(aim.now(), columns, output_format);
    println!("{}", formatter.format(todos));
}

#[cfg(test)]
mod tests {
    use aimcal_core::Priority;

    use super::*;

    #[test]
    fn test_parse_new() {
        let cmd = CmdTodoNew::command();
        let matches = cmd
            .try_get_matches_from([
                "new",
                "Another summary",
                "--description",
                "A description",
                "--due",
                "2025-01-01 12:00:00",
                "--percent",
                "66",
                "--priority",
                "1",
                "--status",
                "completed",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let parsed = CmdTodoNew::from(&matches);

        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.due, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.percent_complete, Some(66));
        assert_eq!(parsed.priority, Some(Priority::P1));
        assert_eq!(parsed.status, Some(TodoStatus::Completed));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui());
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_new_tui() {
        let cmd = CmdTodoNew::command();
        let matches = cmd
            .try_get_matches_from(["new", "--output-format", "json", "--verbose"])
            .unwrap();
        let parsed = CmdTodoNew::from(&matches);

        assert!(parsed.tui());
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_edit() {
        let cmd = CmdTodoEdit::command();
        let matches = cmd
            .try_get_matches_from([
                "edit",
                "test_id",
                "--description",
                "A description",
                "--due",
                "2025-01-01 12:00:00",
                "--priority",
                "1",
                "--percent",
                "66",
                "--status",
                "needs-action",
                "--summary",
                "Another summary",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let parsed = CmdTodoEdit::from(&matches);

        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.due, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.priority, Some(Priority::P1));
        assert_eq!(parsed.percent_complete, Some(66));
        assert_eq!(parsed.status, Some(TodoStatus::NeedsAction));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui());
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_edit_tui() {
        let cmd = CmdTodoEdit::command();
        let matches = cmd
            .try_get_matches_from(["edit", "test_id", "--output-format", "json", "--verbose"])
            .unwrap();
        let parsed = CmdTodoEdit::from(&matches);

        assert!(parsed.tui());
        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_done() {
        let cmd = CmdTodoDone::command();
        let matches = cmd
            .try_get_matches_from(["done", "abc", "--output-format", "json", "--verbose"])
            .unwrap();
        let parsed = CmdTodoDone::from(&matches);

        assert_eq!(parsed.ids, vec![Id::ShortIdOrUid("abc".to_string())]);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_done_multi() {
        let cmd = CmdTodoDone::command();
        let matches = cmd
            .try_get_matches_from([
                "done",
                "a",
                "b",
                "c",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let parsed = CmdTodoDone::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_undo() {
        let cmd = CmdTodoUndo::command();
        let matches = cmd
            .try_get_matches_from(["undo", "a", "b", "c", "--output-format", "json"])
            .unwrap();
        let parsed = CmdTodoUndo::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.output_format, OutputFormat::Json);
    }

    #[test]
    fn test_parse_cancel() {
        let cmd = CmdTodoCancel::command();
        let matches = cmd
            .try_get_matches_from(["cancel", "a", "b", "c", "--output-format", "json"])
            .unwrap();
        let parsed = CmdTodoCancel::from(&matches);

        let expected_ids = vec![
            Id::ShortIdOrUid("a".to_string()),
            Id::ShortIdOrUid("b".to_string()),
            Id::ShortIdOrUid("c".to_string()),
        ];
        assert_eq!(parsed.ids, expected_ids);
        assert_eq!(parsed.output_format, OutputFormat::Json);
    }

    #[test]
    fn test_parse_delay() {
        let cmd = CmdTodoDelay::command();
        let matches = cmd
            .try_get_matches_from([
                "delay",
                "a",
                "b",
                "c",
                "--time",
                "1d",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let parsed = CmdTodoDelay::from(&matches);

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
        let cmd = CmdTodoReschedule::command();
        let matches = cmd
            .try_get_matches_from([
                "reschedule",
                "a",
                "b",
                "c",
                "--time",
                "1d",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let parsed = CmdTodoReschedule::from(&matches);

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
        let cmd = CmdTodoList::command();
        let matches = cmd
            .try_get_matches_from(["list", "--output-format", "json", "--verbose"])
            .unwrap();
        let parsed = CmdTodoList::from(&matches);

        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }
}
