// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{
    Aim, Id, Priority, SortOrder, TodoConditions, TodoDraft, TodoPatch, TodoSort, TodoStatus,
};
use chrono::Duration;
use clap::{Arg, ArgMatches, Command, arg};
use clap_num::number_range;
use colored::Colorize;

use crate::todo_formatter::TodoFormatter;
use crate::tui;
use crate::util::{ArgOutputFormat, parse_datetime};

#[derive(Debug, Clone)]
pub struct CmdTodoNew {
    pub description: Option<String>,
    pub due: Option<String>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,
    pub status: Option<TodoStatus>,
    pub summary: Option<String>,

    pub output_format: ArgOutputFormat,
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

        Ok(Self {
            description,
            due,
            percent_complete,
            priority,
            status,
            summary,

            output_format: ArgOutputFormat::from(matches),
        })
    }

    pub fn new() -> Self {
        Self {
            description: None,
            due: None,
            percent_complete: None,
            priority: None,
            status: None,
            summary: None,

            output_format: ArgOutputFormat::Table,
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        log::debug!("Adding new todo...");

        let draft = if let Some(summary) = self.summary {
            TodoDraft {
                description: self.description,
                due: self.due.map(|a| parse_datetime(&a)).transpose()?.flatten(),
                percent_complete: self.percent_complete,
                priority: self.priority,
                status: self.status,
                summary,
            }
        } else {
            match tui::draft_todo(aim)? {
                Some(data) => data,
                None => {
                    log::info!("User canceled the todo edit");
                    return Ok(());
                }
            }
        };
        let todo = aim.new_todo(draft).await?;

        let formatter = TodoFormatter::new(aim.now()).with_output_format(self.output_format);
        println!("{}", formatter.format(&[todo]));

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

    pub output_format: ArgOutputFormat,
}

impl CmdTodoEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Edit a todo item")
            .arg(arg_id())
            .arg(arg_summary(false))
            .arg(arg_due())
            .arg(arg_description())
            .arg(arg_percent_complete())
            .arg(arg_priority())
            .arg(arg_status())
            .arg(ArgOutputFormat::arg())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: get_id(matches),
            description: get_description(matches),
            due: get_due(matches),
            percent_complete: get_percent_complete(matches),
            priority: get_priority(matches),
            status: get_status(matches),
            summary: get_summary(matches),

            output_format: ArgOutputFormat::from(matches),
        }
    }

    pub fn new(id: Id, output_format: ArgOutputFormat) -> Self {
        Self {
            id,
            description: None,
            due: None,
            percent_complete: None,
            priority: None,
            status: None,
            summary: None,

            output_format,
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        let patch = if self.is_empty() {
            let todo = aim.get_todo(&self.id).await?.ok_or("Todo not found")?;
            match tui::patch_todo(aim, &todo)? {
                Some(data) => data,
                None => {
                    log::info!("User canceled the todo edit");
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
        }
        .run(aim)
        .await
    }

    fn is_empty(&self) -> bool {
        self.description.is_none()
            && self.due.is_none()
            && self.percent_complete.is_none()
            && self.priority.is_none()
            && self.status.is_none()
            && self.summary.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoDone {
    pub ids: Vec<Id>,
    pub output_format: ArgOutputFormat,
}

impl CmdTodoDone {
    pub const NAME: &str = "done";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Mark a todo item as done")
            .arg(arg_ids())
            .arg(ArgOutputFormat::arg())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            ids: get_ids(matches),
            output_format: ArgOutputFormat::from(matches),
        }
    }

    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        for id in self.ids {
            match &id {
                Id::ShortIdOrUid(id) => log::debug!("Marking todo {id} as done"),
                Id::Uid(uid) => log::debug!("Marking todo {uid} as done"),
            }

            TodoEdit {
                id,
                output_format: self.output_format,
                patch: TodoPatch {
                    status: Some(TodoStatus::Completed),
                    ..Default::default()
                },
            }
            .run(aim)
            .await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoUndo {
    pub ids: Vec<Id>,
    pub output_format: ArgOutputFormat,
}

impl CmdTodoUndo {
    pub const NAME: &str = "undo";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Mark a todo item as undone")
            .arg(arg_ids())
            .arg(ArgOutputFormat::arg())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            ids: get_ids(matches),
            output_format: ArgOutputFormat::from(matches),
        }
    }

    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        for id in self.ids {
            match &id {
                Id::ShortIdOrUid(id) => log::debug!("Marking todo {id} as undone"),
                Id::Uid(uid) => log::debug!("Marking todo {uid} as undone"),
            }

            TodoEdit {
                id,
                output_format: self.output_format,
                patch: TodoPatch {
                    status: Some(TodoStatus::NeedsAction),
                    ..Default::default()
                },
            }
            .run(aim)
            .await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CmdTodoList {
    pub conds: TodoConditions,
    pub output_format: ArgOutputFormat,
}

impl CmdTodoList {
    pub const NAME: &str = "list";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("List todos")
            .arg(ArgOutputFormat::arg())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            conds: TodoConditions {
                status: Some(TodoStatus::NeedsAction),
                due: Some(Duration::days(2)),
            },
            output_format: ArgOutputFormat::from(matches),
        }
    }

    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        log::debug!("Listing todos...");
        Self::list(aim, &self.conds, self.output_format).await?;
        Ok(())
    }

    pub async fn list(
        aim: &Aim,
        conds: &TodoConditions,
        output_format: ArgOutputFormat,
    ) -> Result<(), Box<dyn Error>> {
        const MAX: i64 = 16;
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
                println!("Displaying the {total}/{MAX} todos");
            }
        } else if todos.is_empty() && output_format == ArgOutputFormat::Table {
            println!("{}", "No todos found".italic());
        }

        let formatter = TodoFormatter::new(aim.now()).with_output_format(output_format);
        println!("{}", formatter.format(&todos));
        Ok(())
    }
}

struct TodoEdit {
    id: Id,
    patch: TodoPatch,
    output_format: ArgOutputFormat,
}

impl TodoEdit {
    async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        log::debug!("Edit todo ...");
        let todo = aim.update_todo(&self.id, self.patch).await?;
        let formatter = TodoFormatter::new(aim.now()).with_output_format(self.output_format);
        println!("{}", formatter.format(&[todo]));
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
    matches.get_one::<String>("summary").cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aimcal_core::Priority;
    use clap::Command;

    #[test]
    fn test_parse_todo_new() {
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
    }

    #[test]
    fn test_parse_todo_new_tui() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoNew::command());

        let matches = cmd.try_get_matches_from(["test", "new"]).unwrap();
        let sub_matches = matches.subcommand_matches("new").unwrap();
        let parsed = CmdTodoNew::from(sub_matches).unwrap();
        assert_eq!(parsed.description, None);
        assert_eq!(parsed.due, None);
        assert_eq!(parsed.percent_complete, None);
        assert_eq!(parsed.priority, None);
        assert_eq!(parsed.status, None);
        assert_eq!(parsed.summary, None);
    }

    #[test]
    fn test_parse_todo_new_tui_invalid() {
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
    fn test_parse_todo_edit() {
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
    }

    #[test]
    fn test_parse_todo_done() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoDone::command());

        let matches = cmd
            .try_get_matches_from(["test", "done", "abc", "--output-format", "json"])
            .unwrap();
        let sub_matches = matches.subcommand_matches("done").unwrap();
        let parsed = CmdTodoDone::from(sub_matches);
        assert_eq!(parsed.ids, vec![Id::ShortIdOrUid("abc".to_string())]);
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
    }

    #[test]
    fn test_parse_todo_done_multi() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoDone::command());

        let matches = cmd
            .try_get_matches_from(["test", "done", "a", "b", "c", "--output-format", "json"])
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
    }

    #[test]
    fn test_parse_todo_undo() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoUndo::command());

        let matches = cmd
            .try_get_matches_from(["test", "undo", "abc", "--output-format", "json"])
            .unwrap();

        let sub_matches = matches.subcommand_matches("undo").unwrap();
        let parsed = CmdTodoUndo::from(sub_matches);
        assert_eq!(parsed.ids, vec![Id::ShortIdOrUid("abc".to_string())]);
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
    }

    #[test]
    fn test_parse_todo_undo_multi() {
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
    fn test_parse_todo_list() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdTodoList::command());

        let matches = cmd
            .try_get_matches_from(["test", "list", "--output-format", "json"])
            .unwrap();

        let sub_matches = matches.subcommand_matches("list").unwrap();
        let parsed = CmdTodoList::from(sub_matches);
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
    }
}
