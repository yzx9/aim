// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{
    Aim, DateTimeAnchor, Id, Priority, SortOrder, Todo, TodoConditions, TodoDraft, TodoPatch,
    TodoSort, TodoStatus,
};
use clap::{Arg, ArgMatches, Command, arg};
use clap_num::number_range;
use colored::Colorize;

use crate::todo_formatter::{TodoColumn, TodoFormatter};
use crate::tui;
use crate::util::{ArgOutputFormat, arg_verbose, get_verbose, parse_datetime};

#[derive(Debug, Clone)]
pub struct CmdTodoNew {
    pub description: Option<String>,
    pub due: Option<String>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,
    pub status: Option<TodoStatus>,
    pub summary: Option<String>,

    pub tui: bool,
    pub output_format: ArgOutputFormat,
    pub verbose: bool,
}

impl CmdTodoNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new todo")
            .arg(arg_summary(true))
            .arg(arg_due())
            .arg(arg_description())
            .arg(arg_percent_complete())
            .arg(arg_priority())
            .arg(arg_status())
            .arg(ArgOutputFormat::arg())
            .arg(arg_verbose())
    }

    pub fn from(matches: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        let description = get_description(matches);
        let due = get_due(matches);
        let percent_complete = get_percent_complete(matches);
        let priority = get_priority(matches);
        let status = get_status(matches);

        let summary = match get_summary(matches) {
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
            output_format: ArgOutputFormat::from(matches),
            verbose: get_verbose(matches),
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
            output_format: ArgOutputFormat::Table,
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
            TodoDraft {
                description: self.description,
                due: self.due.map(|a| parse_datetime(&a)).transpose()?.flatten(),
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
        output_format: ArgOutputFormat,
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
    pub output_format: ArgOutputFormat,
    pub verbose: bool,
}

impl CmdTodoEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Edit a todo")
            .arg(arg_id())
            .arg(arg_summary(false))
            .arg(arg_due())
            .arg(arg_description())
            .arg(arg_percent_complete())
            .arg(arg_priority())
            .arg(arg_status())
            .arg(ArgOutputFormat::arg())
            .arg(arg_verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        let id = get_id(matches);
        let description = get_description(matches);
        let due = get_due(matches);
        let percent_complete = get_percent_complete(matches);
        let priority = get_priority(matches);
        let status = get_status(matches);
        let summary = get_summary(matches);

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
            output_format: ArgOutputFormat::from(matches),
            verbose: get_verbose(matches),
        }
    }

    pub fn new_tui(id: Id, output_format: ArgOutputFormat, verbose: bool) -> Self {
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
                due: self.due.as_ref().map(|a| parse_datetime(a)).transpose()?,
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
            pub output_format: ArgOutputFormat,
            pub verbose: bool,
        }

        impl $cmd {
            pub const NAME: &str = $name;

            pub fn command() -> Command {
                Command::new(Self::NAME)
                    .about(concat!("Mark a todo as ", $desc))
                    .arg(arg_ids())
                    .arg(ArgOutputFormat::arg())
                    .arg(arg_verbose())
            }

            pub fn from(matches: &ArgMatches) -> Self {
                Self {
                    ids: get_ids(matches),
                    output_format: ArgOutputFormat::from(matches),
                    verbose: get_verbose(matches),
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
    pub timedelta: String,
    pub output_format: ArgOutputFormat,
    pub verbose: bool,
}

impl CmdTodoDelay {
    pub const NAME: &str = "delay";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Delay a todo's due date by a specified time")
            .arg(arg_id())
            .arg(arg!(<TIMEDELTA> "Time to delay (datetime, time, or 'tomorrow')"))
            .arg(ArgOutputFormat::arg())
            .arg(arg_verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: get_id(matches),
            timedelta: matches
                .get_one::<String>("TIMEDELTA")
                .expect("timedelta is required")
                .clone(),
            output_format: ArgOutputFormat::from(matches),
            verbose: get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "delaying todo...");

        // Get the current todo
        aim.get_todo(&self.id).await?.ok_or("Todo not found")?;

        // Parse the timedelta
        let anchor: DateTimeAnchor = self.timedelta.parse()?;
        let new_due = anchor.parse_from_dt(&aim.now());

        // Update the todo
        let patch = TodoPatch {
            due: Some(Some(new_due)),
            ..Default::default()
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

#[derive(Debug, Clone, Copy)]
pub struct CmdTodoList {
    pub conds: TodoConditions,
    pub output_format: ArgOutputFormat,
    pub verbose: bool,
}

impl CmdTodoList {
    pub const NAME: &str = "list";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("List todos")
            .arg(ArgOutputFormat::arg())
            .arg(arg_verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            conds: TodoConditions {
                status: Some(TodoStatus::NeedsAction),
                due: Some(DateTimeAnchor::InDays(2)),
            },
            output_format: ArgOutputFormat::from(matches),
            verbose: get_verbose(matches),
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
        output_format: ArgOutputFormat,
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
        } else if todos.is_empty() && output_format == ArgOutputFormat::Table {
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
    output_format: ArgOutputFormat,
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

fn arg_id() -> Arg {
    arg!(id: <ID> "The short id or uid of the todo to edit")
}

fn get_id(matches: &ArgMatches) -> Id {
    let id = matches
        .get_one::<String>("id")
        .expect("id is required")
        .clone();

    Id::ShortIdOrUid(id)
}

fn arg_ids() -> Arg {
    arg!(id: <ID> "The short id or uid of the todo to edit").num_args(1..)
}

fn get_ids(matches: &ArgMatches) -> Vec<Id> {
    matches
        .get_many::<String>("id")
        .expect("id is required")
        .map(|a| Id::ShortIdOrUid(a.clone()))
        .collect()
}

fn arg_description() -> Arg {
    arg!(--description <DESCRIPTION> "Description of the todo")
}

fn get_description(matches: &ArgMatches) -> Option<String> {
    matches.get_one("description").cloned()
}

fn arg_due() -> Arg {
    arg!(--due <DUE> "Due date and time of the todo")
}

fn get_due(matches: &ArgMatches) -> Option<String> {
    matches.get_one("due").cloned()
}

fn arg_percent_complete() -> Arg {
    fn from_0_to_100(s: &str) -> Result<u8, String> {
        number_range(s, 0, 100)
    }

    arg!(--percent <PERCENT> "Percent complete of the todo (0-100)").value_parser(from_0_to_100)
}

fn get_percent_complete(matches: &ArgMatches) -> Option<u8> {
    matches.get_one("percent").copied()
}

fn arg_priority() -> Arg {
    clap::arg!(-p --priority <PRIORITY> "Priority of the todo")
        .value_parser(clap::value_parser!(Priority))
}

fn get_priority(matches: &ArgMatches) -> Option<Priority> {
    matches.get_one("priority").copied()
}

fn arg_status() -> Arg {
    clap::arg!(--status <STATUS> "Status of the todo").value_parser(clap::value_parser!(TodoStatus))
}

fn get_status(matches: &ArgMatches) -> Option<TodoStatus> {
    matches.get_one("status").copied()
}

fn arg_summary(positional: bool) -> Arg {
    match positional {
        true => arg!(summary: <SUMMARY> "Summary of the todo").required(false),
        false => arg!(summary: -s --summary <SUMMARY> "Summary of the todo"),
    }
}

fn get_summary(matches: &ArgMatches) -> Option<String> {
    matches.get_one("summary").cloned()
}

fn print_todos(aim: &Aim, todos: &[impl Todo], output_format: ArgOutputFormat, verbose: bool) {
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
                "timedelta",
                "--output-format",
                "json",
                "--verbose",
            ])
            .unwrap();

        let sub_matches = matches.subcommand_matches("delay").unwrap();
        let parsed = CmdTodoDelay::from(sub_matches);
        assert_eq!(parsed.id, Id::ShortIdOrUid("abc".to_string()));
        assert_eq!(parsed.timedelta, "timedelta");
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
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
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
        assert!(parsed.verbose);
    }
}
