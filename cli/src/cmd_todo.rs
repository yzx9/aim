// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Config,
    parser::{ArgOutputFormat, ArgUidOrShortId, parse_datetime},
    short_id::{ShortIdMap, TodoWithShortId},
    todo_editor::TodoEditor,
    todo_formatter::TodoFormatter,
};
use aimcal_core::{
    Aim, LooseDateTime, Priority, SortOrder, TodoConditions, TodoDraft, TodoPatch, TodoSort,
    TodoStatus,
};
use chrono::Duration;
use clap::{Arg, ArgMatches, Command, arg};
use clap_num::number_range;
use colored::Colorize;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct CmdTodoNew {
    pub output_format: ArgOutputFormat,

    pub description: Option<String>,
    pub due: Option<String>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,
    pub status: Option<TodoStatus>,
    pub summary: Option<String>,
}

impl CmdTodoNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new todo item")
            .arg(TodoEdit::arg_summary(true))
            .arg(TodoEdit::arg_due())
            .arg(TodoEdit::arg_description())
            .arg(TodoEdit::arg_percent_complete())
            .arg(TodoEdit::arg_priority())
            .arg(TodoEdit::arg_status())
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        let description = TodoEdit::parse_description(matches);
        let due = TodoEdit::parse_due(matches);
        let percent_complete = TodoEdit::parse_percent_complete(matches);
        let priority = TodoEdit::parse_priority(matches);
        let status = TodoEdit::parse_status(matches);

        let summary = match matches.get_one::<String>("summary") {
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
            output_format: ArgOutputFormat::parse(matches),

            description,
            due,
            percent_complete,
            priority,
            status,
            summary,
        })
    }

    pub fn new() -> Self {
        Self {
            output_format: ArgOutputFormat::Table,

            description: None,
            due: None,
            percent_complete: None,
            priority: None,
            status: None,
            summary: None,
        }
    }

    pub async fn run(
        self,
        config: &Config,
        aim: &mut Aim,
        map: &ShortIdMap,
    ) -> Result<(), Box<dyn Error>> {
        log::debug!("Adding new todo...");
        let due = match (self.due, config.default_due) {
            (Some(due), _) => parse_datetime(&due)?,
            (None, Some(duration)) => Some(LooseDateTime::Local(aim.now() + duration)),
            (None, None) => None,
        };

        let draft = if let Some(summary) = self.summary {
            TodoDraft {
                description: self.description,
                due,
                percent_complete: self.percent_complete,
                priority: self.priority.unwrap_or(config.default_priority),
                status: self.status,
                summary,
            }
        } else {
            match TodoEditor::run_draft(config, aim)? {
                Some(data) => data,
                None => {
                    log::info!("User canceled the todo edit");
                    return Ok(());
                }
            }
        };
        let todo = aim.new_todo(draft).await?;

        let todo = TodoWithShortId::with(map, todo);
        let formatter = TodoFormatter::new(aim.now()).with_output_format(self.output_format);
        println!("{}", formatter.format(&[todo]));

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoEdit {
    pub uid_or_short_id: ArgUidOrShortId,
    pub output_format: ArgOutputFormat,

    pub description: Option<String>,
    pub due: Option<String>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,
    pub status: Option<TodoStatus>,
    pub summary: Option<String>,
}

impl CmdTodoEdit {
    pub const NAME: &str = "edit";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Edit a todo item")
            .arg(ArgUidOrShortId::arg())
            .arg(TodoEdit::arg_summary(false))
            .arg(TodoEdit::arg_due())
            .arg(TodoEdit::arg_description())
            .arg(TodoEdit::arg_percent_complete())
            .arg(TodoEdit::arg_priority())
            .arg(TodoEdit::arg_status())
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            uid_or_short_id: ArgUidOrShortId::parse(matches),
            output_format: ArgOutputFormat::parse(matches),

            description: TodoEdit::parse_description(matches),
            due: TodoEdit::parse_due(matches),
            percent_complete: TodoEdit::parse_percent_complete(matches),
            priority: TodoEdit::parse_priority(matches),
            status: TodoEdit::parse_status(matches),
            summary: TodoEdit::parse_summary(matches),
        }
    }

    pub fn new(uid_or_short_id: ArgUidOrShortId, output_format: ArgOutputFormat) -> Self {
        Self {
            uid_or_short_id,
            output_format,
            description: None,
            due: None,
            percent_complete: None,
            priority: None,
            status: None,
            summary: None,
        }
    }

    pub async fn run(self, aim: &mut Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        let patch = if self.is_empty() {
            let uid = self.uid_or_short_id.get_id(map);
            match TodoEditor::run_patch(aim, &uid).await? {
                Some(data) => data,
                None => {
                    log::info!("User canceled the todo edit");
                    return Ok(());
                }
            }
        } else {
            TodoPatch {
                uid: self.uid_or_short_id.get_id(map),
                description: self.description.map(|d| (!d.is_empty()).then_some(d)),
                due: self.due.as_ref().map(|a| parse_datetime(a)).transpose()?,
                priority: self.priority,
                percent_complete: None,
                status: self.status,
                summary: self.summary,
            }
        };

        TodoEdit {
            output_format: self.output_format,
            patch,
        }
        .run(aim, map)
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
    pub uid_or_short_id: ArgUidOrShortId,
    pub output_format: ArgOutputFormat,
}

impl CmdTodoDone {
    pub const NAME: &str = "done";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Mark a todo item as done")
            .arg(ArgUidOrShortId::arg())
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            uid_or_short_id: ArgUidOrShortId::parse(matches),
            output_format: ArgOutputFormat::parse(matches),
        }
    }

    pub async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        log::debug!("Marking todo as done...");
        TodoEdit {
            output_format: self.output_format,
            patch: TodoPatch {
                uid: self.uid_or_short_id.get_id(map),
                status: Some(TodoStatus::Completed),
                ..Default::default()
            },
        }
        .run(aim, map)
        .await
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoUndo {
    pub uid_or_short_id: ArgUidOrShortId,
    pub output_format: ArgOutputFormat,
}

impl CmdTodoUndo {
    pub const NAME: &str = "undo";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Mark a todo item as undone")
            .arg(ArgUidOrShortId::arg())
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            uid_or_short_id: ArgUidOrShortId::parse(matches),
            output_format: ArgOutputFormat::parse(matches),
        }
    }

    pub async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        log::debug!("Marking todo as undone...");
        TodoEdit {
            output_format: self.output_format,
            patch: TodoPatch {
                uid: self.uid_or_short_id.get_id(map),
                status: Some(TodoStatus::NeedsAction),
                ..Default::default()
            },
        }
        .run(aim, map)
        .await
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

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            conds: TodoConditions {
                status: Some(TodoStatus::NeedsAction),
                due: Some(Duration::days(2)),
            },
            output_format: ArgOutputFormat::parse(matches),
        }
    }

    pub async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        log::debug!("Listing todos...");
        Self::list(aim, map, &self.conds, self.output_format).await?;
        Ok(())
    }

    pub async fn list(
        aim: &Aim,
        map: &ShortIdMap,
        conds: &TodoConditions,
        output_format: ArgOutputFormat,
    ) -> Result<(), Box<dyn Error>> {
        const MAX: i64 = 16;
        let pager = (MAX, 0).into();
        let sort = vec![
            TodoSort::Priority {
                order: SortOrder::Desc,
                none_first: false, // TODO: add config entry
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

        let todos: Vec<_> = todos
            .into_iter()
            .map(|todo| TodoWithShortId::with(map, todo))
            .collect();

        let formatter = TodoFormatter::new(aim.now()).with_output_format(output_format);
        println!("{}", formatter.format(&todos));
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct TodoEdit {
    output_format: ArgOutputFormat,
    patch: TodoPatch,
}

impl TodoEdit {
    fn arg_description() -> Arg {
        arg!(--description <DESCRIPTION> "Description of the todo")
    }

    fn parse_description(matches: &ArgMatches) -> Option<String> {
        matches.get_one("description").cloned()
    }

    fn arg_due() -> Arg {
        arg!(--due <DUE> "Due date and time of the todo")
    }

    fn parse_due(matches: &ArgMatches) -> Option<String> {
        matches.get_one("due").cloned()
    }

    fn arg_percent_complete() -> Arg {
        fn from_0_to_100(s: &str) -> Result<u8, String> {
            number_range(s, 0, 100)
        }

        arg!(--percent <PERCENT> "Percent complete of the todo (0-100)").value_parser(from_0_to_100)
    }

    fn parse_percent_complete(matches: &ArgMatches) -> Option<u8> {
        matches.get_one("percent").copied()
    }

    fn arg_priority() -> Arg {
        clap::arg!(-p --priority <PRIORITY> "Priority of the todo")
            .value_parser(clap::value_parser!(Priority))
    }

    fn parse_priority(matches: &ArgMatches) -> Option<Priority> {
        matches.get_one("priority").copied()
    }

    fn arg_status() -> Arg {
        clap::arg!(--status <STATUS> "Status of the todo")
            .value_parser(clap::value_parser!(TodoStatus))
    }

    fn parse_status(matches: &ArgMatches) -> Option<TodoStatus> {
        matches.get_one("status").cloned()
    }

    fn arg_summary(positional: bool) -> Arg {
        if positional {
            arg!(summary: <SUMMARY> "Summary of the todo").required(false)
        } else {
            arg!(summary: -s --summary <SUMMARY> "Summary of the todo")
        }
    }

    fn parse_summary(matches: &ArgMatches) -> Option<String> {
        matches.get_one::<String>("summary").cloned()
    }

    async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        log::debug!("Edit todo ...");
        let todo = aim.update_todo(self.patch).await?;
        let todo = TodoWithShortId::with(map, todo);
        let formatter = TodoFormatter::new(aim.now()).with_output_format(self.output_format);
        println!("{}", formatter.format(&[todo]));
        Ok(())
    }
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
        let parsed = CmdTodoNew::parse(sub_matches).unwrap();
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
        let parsed = CmdTodoNew::parse(sub_matches).unwrap();
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
        let parsed = CmdTodoNew::parse(sub_matches);
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
                "--status",
                "needs-action",
                "--summary",
                "Another summary",
            ])
            .unwrap();
        let sub_matches = matches.subcommand_matches("edit").unwrap();
        let parsed = CmdTodoEdit::parse(sub_matches);
        assert_eq!(parsed.uid_or_short_id, "test_id".into());
        assert_eq!(parsed.summary, Some("Another summary".to_string()));
        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.due, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.priority, Some(Priority::P1));
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
        let parsed = CmdTodoDone::parse(sub_matches);
        assert_eq!(parsed.uid_or_short_id, "abc".into());
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
        let parsed = CmdTodoUndo::parse(sub_matches);
        assert_eq!(parsed.uid_or_short_id, "abc".into());
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
        let parsed = CmdTodoList::parse(sub_matches);
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
    }
}
