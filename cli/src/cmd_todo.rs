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

    pub tui: bool,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdTodoNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        let args = args();
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new todo")
            // fields
            .arg(args.summary(true))
            .arg(TodoArgs::due())
            .arg(args.description())
            .arg(TodoArgs::percent_complete())
            .arg(TodoArgs::priority())
            .arg(TodoArgs::status())
            // options
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        let description = EventOrTodoArgs::get_description(matches);
        let due = TodoArgs::get_due(matches);
        let percent_complete = TodoArgs::get_percent_complete(matches);
        let priority = TodoArgs::get_priority(matches);
        let status = TodoArgs::get_status(matches);

        let summary = match EventOrTodoArgs::get_summary(matches) {
            Some(summary) => Some(summary.clone()),

            None if description.is_none()
                && due.is_none()
                && percent_complete.is_none()
                && priority.is_none()
                && status.is_none() =>
            {
                None
            }

            // If summary is not provided but other fields are set, we still require a summary.
            None => return Err("Summary is required for new todo".into()),
        };

        let tui = summary.is_none();
        Ok(Self {
            description,
            due,
            percent_complete,
            priority,
            status,
            summary,

            tui,
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        })
    }

    pub fn new_tui() -> Self {
        Self {
            description: None,
            due: None,
            percent_complete: None,
            priority: None,
            status: None,
            summary: None,

            tui: true,
            output_format: OutputFormat::Table,
            verbose: false,
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "adding new todo...");
        let draft = if self.tui {
            match tui::draft_todo(aim)? {
                Some(data) => data,
                None => {
                    tracing::info!("user cancel the todo editing");
                    return Ok(());
                }
            }
        } else {
            let now = aim.now();
            TodoDraft {
                description: self.description,
                due: self
                    .due
                    .map(|a| parse_datetime(&now, &a))
                    .transpose()?
                    .flatten(),
                percent_complete: self.percent_complete,
                priority: self.priority,
                status: self.status.unwrap_or_default(),
                summary: self.summary.unwrap_or_default(),
            }
        };
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

    pub tui: bool,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdTodoEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        let args = args();
        Command::new(Self::NAME)
            .about("Edit a todo")
            .arg(args.id())
            .arg(args.summary(false))
            .arg(TodoArgs::due())
            .arg(args.description())
            .arg(TodoArgs::percent_complete())
            .arg(TodoArgs::priority())
            .arg(TodoArgs::status())
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        let id = EventOrTodoArgs::get_id(matches);
        let description = EventOrTodoArgs::get_description(matches);
        let due = TodoArgs::get_due(matches);
        let percent_complete = TodoArgs::get_percent_complete(matches);
        let priority = TodoArgs::get_priority(matches);
        let status = TodoArgs::get_status(matches);
        let summary = EventOrTodoArgs::get_summary(matches);

        let tui = description.is_none()
            && due.is_none()
            && percent_complete.is_none()
            && priority.is_none()
            && status.is_none()
            && summary.is_none();

        Self {
            id,
            description,
            due,
            percent_complete,
            priority,
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
            due: None,
            percent_complete: None,
            priority: None,
            status: None,
            summary: None,

            tui: true,
            output_format,
            verbose,
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "editing todo...");
        let patch = if self.tui {
            let todo = aim.get_todo(&self.id).await?.ok_or("Todo not found")?;
            match tui::patch_todo(aim, &todo)? {
                Some(data) => data,
                None => {
                    tracing::info!("user cancel the todo editing");
                    return Ok(());
                }
            }
        } else {
            TodoPatch {
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
            }
        };

        TodoEdit {
            id: self.id,
            patch,
            output_format: self.output_format,
            verbose: self.verbose,
        }
        .run(aim)
        .await
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
                let args = args();
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
                for id in self.ids {
                    TodoEdit {
                        id,
                        patch: TodoPatch {
                            status: Some(TodoStatus::$status),
                            ..Default::default()
                        },
                        output_format: self.output_format,
                        verbose: self.verbose,
                    }
                    .run(aim)
                    .await?;
                }
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
    pub id: Id,
    pub time_anchor: String,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdTodoDelay {
    pub const NAME: &str = "delay";

    pub fn command() -> Command {
        let args = args();
        Command::new(Self::NAME)
            .about("Delay a todo's due by a specified time based on original due")
            .arg(args.id())
            .arg(TodoArgs::time_anchor("delay"))
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: EventOrTodoArgs::get_id(matches),
            time_anchor: TodoArgs::get_time_anchor(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "delaying todo...");

        // Get the current todo
        let todo = aim.get_todo(&self.id).await?.ok_or("Todo not found")?;

        // Parse the time anchor
        let new_due = if !self.time_anchor.is_empty() {
            let anchor: DateTimeAnchor = self.time_anchor.parse()?;
            Some(match todo.due() {
                Some(due) => anchor.parse_from_loose(&due),
                None => anchor.parse_from_dt(&aim.now()),
            })
        } else {
            None
        };

        // Update the todo
        TodoEdit {
            id: self.id,
            patch: TodoPatch {
                due: Some(new_due),
                ..Default::default()
            },
            output_format: self.output_format,
            verbose: self.verbose,
        }
        .run(aim)
        .await
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoReschedule {
    pub id: Id,
    pub time_anchor: String,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdTodoReschedule {
    pub const NAME: &str = "reschedule";

    pub fn command() -> Command {
        let args = args();
        Command::new(Self::NAME)
            .about("Reschedule a todo's due to a specified time based on now")
            .arg(args.id())
            .arg(TodoArgs::time_anchor("reschedule"))
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: EventOrTodoArgs::get_id(matches),
            time_anchor: TodoArgs::get_time_anchor(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "delaying todo...");

        // Parse the time anchor
        let new_due = if !self.time_anchor.is_empty() {
            let anchor: DateTimeAnchor = self.time_anchor.parse()?;
            Some(anchor.parse_from_dt(&aim.now()))
        } else {
            None
        };

        // Update the todo
        TodoEdit {
            id: self.id,
            patch: TodoPatch {
                due: Some(new_due),
                ..Default::default()
            },
            output_format: self.output_format,
            verbose: self.verbose,
        }
        .run(aim)
        .await
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
                due: Some(DateTimeAnchor::InDays(2)),
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

#[derive(Debug, Clone)]
struct TodoEdit {
    id: Id,
    patch: TodoPatch,
    output_format: OutputFormat,
    verbose: bool,
}

impl TodoEdit {
    async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "edit todo ...");
        let todo = aim.update_todo(&self.id, self.patch).await?;
        print_todos(aim, &[todo], self.output_format, self.verbose);
        Ok(())
    }
}

const fn args() -> EventOrTodoArgs {
    EventOrTodoArgs::new(Some(Kind::Todo))
}

fn print_todos(aim: &Aim, todos: &[impl Todo], output_format: OutputFormat, verbose: bool) {
    let columns = if verbose {
        vec![
            TodoColumn::status(),
            TodoColumn::id(),
            TodoColumn::uid(),
            TodoColumn::priority(),
            TodoColumn::due(),
            TodoColumn::summary(),
        ]
    } else {
        vec![
            TodoColumn::status(),
            TodoColumn::id(),
            TodoColumn::priority(),
            TodoColumn::due(),
            TodoColumn::summary(),
        ]
    };
    let formatter = TodoFormatter::new(aim.now(), columns).with_output_format(output_format);
    println!("{}", formatter.format(todos));
}

#[cfg(test)]
mod tests {
    use super::*;
    use aimcal_core::Priority;
    use clap::Command;

    #[test]
    fn test_parse_new() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoNew::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
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
        let sub_matches = matches.subcommand_matches("new").unwrap();
        let parsed = CmdTodoNew::from(sub_matches).unwrap();

        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.due, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.percent_complete, Some(66));
        assert_eq!(parsed.priority, Some(Priority::P1));
        assert_eq!(parsed.status, Some(TodoStatus::Completed));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_new_tui() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoNew::command());

        let matches = cmd
            .try_get_matches_from(["test", "new", "--output-format", "json", "--verbose"])
            .unwrap();
        let sub_matches = matches.subcommand_matches("new").unwrap();
        let parsed = CmdTodoNew::from(sub_matches).unwrap();
        assert!(parsed.tui);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_new_tui_invalid() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoNew::command());

        let matches = cmd
            .try_get_matches_from(["test", "new", "--priority", "1"])
            .unwrap();
        let sub_matches = matches.subcommand_matches("new").unwrap();
        let parsed = CmdTodoNew::from(sub_matches);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_edit() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoEdit::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
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
        let sub_matches = matches.subcommand_matches("edit").unwrap();
        let parsed = CmdTodoEdit::from(sub_matches);

        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.due, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.priority, Some(Priority::P1));
        assert_eq!(parsed.percent_complete, Some(66));
        assert_eq!(parsed.status, Some(TodoStatus::NeedsAction));
        assert_eq!(parsed.summary, Some("Another summary".to_string()));

        assert!(!parsed.tui);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_edit_tui() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoEdit::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
                "edit",
                "test_id",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let sub_matches = matches.subcommand_matches("edit").unwrap();
        let parsed = CmdTodoEdit::from(sub_matches);

        assert!(parsed.tui);
        assert_eq!(parsed.id, Id::ShortIdOrUid("test_id".to_string()));
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_done() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoDone::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
                "done",
                "abc",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let sub_matches = matches.subcommand_matches("done").unwrap();
        let parsed = CmdTodoDone::from(sub_matches);
        assert_eq!(parsed.ids, vec![Id::ShortIdOrUid("abc".to_string())]);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_done_multi() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoDone::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
                "done",
                "a",
                "b",
                "c",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();
        let sub_matches = matches.subcommand_matches("done").unwrap();
        let parsed = CmdTodoDone::from(sub_matches);
        assert_eq!(
            parsed.ids,
            vec![
                Id::ShortIdOrUid("a".to_string()),
                Id::ShortIdOrUid("b".to_string()),
                Id::ShortIdOrUid("c".to_string())
            ]
        );
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_undo() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoUndo::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
                "undo",
                "abc",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();

        let sub_matches = matches.subcommand_matches("undo").unwrap();
        let parsed = CmdTodoUndo::from(sub_matches);
        assert_eq!(parsed.ids, vec![Id::ShortIdOrUid("abc".to_string())]);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_undo_multi() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoUndo::command());

        let matches = cmd
            .try_get_matches_from(["test", "undo", "a", "b", "c", "--output-format", "json"])
            .unwrap();

        let sub_matches = matches.subcommand_matches("undo").unwrap();
        let parsed = CmdTodoUndo::from(sub_matches);
        assert_eq!(
            parsed.ids,
            vec![
                Id::ShortIdOrUid("a".to_string()),
                Id::ShortIdOrUid("b".to_string()),
                Id::ShortIdOrUid("c".to_string())
            ]
        );
        assert_eq!(parsed.output_format, OutputFormat::Json);
    }

    #[test]
    fn test_parse_cancel() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoCancel::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
                "cancel",
                "abc",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();

        let sub_matches = matches.subcommand_matches("cancel").unwrap();
        let parsed = CmdTodoCancel::from(sub_matches);
        assert_eq!(parsed.ids, vec![Id::ShortIdOrUid("abc".to_string())]);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_cancel_multi() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoCancel::command());

        let matches = cmd
            .try_get_matches_from(["test", "cancel", "a", "b", "c", "--output-format", "json"])
            .unwrap();

        let sub_matches = matches.subcommand_matches("cancel").unwrap();
        let parsed = CmdTodoCancel::from(sub_matches);
        assert_eq!(
            parsed.ids,
            vec![
                Id::ShortIdOrUid("a".to_string()),
                Id::ShortIdOrUid("b".to_string()),
                Id::ShortIdOrUid("c".to_string())
            ]
        );
        assert_eq!(parsed.output_format, OutputFormat::Json);
    }

    #[test]
    fn test_parse_delay() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoDelay::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
                "delay",
                "abc",
                "time anchor",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();

        let sub_matches = matches.subcommand_matches("delay").unwrap();
        let parsed = CmdTodoDelay::from(sub_matches);
        assert_eq!(parsed.id, Id::ShortIdOrUid("abc".to_string()));
        assert_eq!(parsed.time_anchor, "time anchor");
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_reschedule() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoReschedule::command());

        let matches = cmd
            .try_get_matches_from([
                "test",
                "reschedule",
                "abc",
                "time anchor",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();

        let sub_matches = matches.subcommand_matches("reschedule").unwrap();
        let parsed = CmdTodoReschedule::from(sub_matches);
        assert_eq!(parsed.id, Id::ShortIdOrUid("abc".to_string()));
        assert_eq!(parsed.time_anchor, "time anchor");
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }

    #[test]
    fn test_parse_list() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoList::command());

        let matches = cmd
            .try_get_matches_from(["test", "list", "--output-format", "json", "--verbose"])
            .unwrap();

        let sub_matches = matches.subcommand_matches("list").unwrap();
        let parsed = CmdTodoList::from(sub_matches);
        assert_eq!(parsed.output_format, OutputFormat::Json);
        assert!(parsed.verbose);
    }
}
