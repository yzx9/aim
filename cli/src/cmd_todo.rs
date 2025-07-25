// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli::ArgOutputFormat,
    short_id::{ShortIdMap, TodoWithShortId},
    todo_formatter::TodoFormatter,
};
use aimcal_core::{
    Aim, LooseDateTime, Priority, SortOrder, TodoConditions, TodoDraft, TodoPatch, TodoSort,
    TodoStatus,
};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, offset::LocalResult};
use clap::{Arg, ArgMatches, Command, arg, value_parser};
use clap_num::number_range;
use colored::Colorize;
use std::error::Error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CmdTodoNew {
    pub description: Option<String>,
    pub due: Option<String>,
    pub priority: Priority,
    pub summary: String,
}

impl CmdTodoNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new todo item")
            .arg(TodoEdit::arg_summary(true).required(true))
            .arg(TodoEdit::arg_due())
            .arg(TodoEdit::arg_description())
            .arg(ArgPriority::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            description: TodoEdit::parse_description(matches),
            due: TodoEdit::parse_due(matches),
            priority: ArgPriority::parse(matches)
                .unwrap_or(&ArgPriority::None)
                .into(),
            summary: TodoEdit::parse_summary(matches).expect("summary is required"),
        }
    }

    pub async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        log::debug!("Add todos...");
        let due = self
            .due
            .as_ref()
            .map(|a| parse_datetime(a))
            .transpose()?
            .flatten();

        let draft = TodoDraft {
            uid: Uuid::new_v4().to_string(), // TODO: better uid
            description: self.description,
            due,
            priority: self.priority,
            summary: self.summary,
        };
        let todo = aim.new_todo(draft).await?;

        let todo = TodoWithShortId::with(map, todo);
        let formatter = TodoFormatter::new(Local::now().naive_local());
        println!("{}", formatter.format(&[todo]));

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoEdit {
    pub uid_or_short_id: String,
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
            .arg(TodoEdit::arg_id())
            .arg(TodoEdit::arg_summary(false))
            .arg(TodoEdit::arg_due())
            .arg(TodoEdit::arg_description())
            .arg(TodoEdit::arg_percent_complete())
            .arg(ArgPriority::arg())
            .arg(ArgStatus::arg())
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            uid_or_short_id: TodoEdit::parse_id(matches),
            output_format: ArgOutputFormat::parse(matches),

            description: TodoEdit::parse_description(matches),
            due: TodoEdit::parse_due(matches),
            percent_complete: TodoEdit::parse_percent_complete(matches),
            priority: ArgPriority::parse(matches).map(Into::into),
            status: ArgStatus::parse(matches).map(Into::into),
            summary: TodoEdit::parse_summary(matches),
        }
    }

    pub async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        log::debug!("Editing todo...");
        let due = self.due.as_ref().map(|a| parse_datetime(a)).transpose()?;

        TodoEdit {
            uid_or_short_id: self.uid_or_short_id,
            output_format: self.output_format,
            patch: TodoPatch {
                description: self.description.map(|d| (!d.is_empty()).then_some(d)),
                due,
                priority: self.priority,
                percent_complete: None,
                status: self.status,
                summary: self.summary,
                ..Default::default()
            },
        }
        .run(aim, map)
        .await
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoDone {
    pub uid_or_short_id: String,
    pub output_format: ArgOutputFormat,
}

impl CmdTodoDone {
    pub const NAME: &str = "done";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Mark a todo item as done")
            .arg(TodoEdit::arg_id())
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            uid_or_short_id: TodoEdit::parse_id(matches),
            output_format: ArgOutputFormat::parse(matches),
        }
    }

    pub async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        log::debug!("Marking todo as done...");
        TodoEdit {
            uid_or_short_id: self.uid_or_short_id,
            output_format: self.output_format,
            patch: TodoPatch {
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
    pub uid_or_short_id: String,
    pub output_format: ArgOutputFormat,
}

impl CmdTodoUndo {
    pub const NAME: &str = "undo";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Mark a todo item as undone")
            .arg(TodoEdit::arg_id())
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            uid_or_short_id: TodoEdit::parse_id(matches),
            output_format: ArgOutputFormat::parse(matches),
        }
    }

    pub async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        log::debug!("Marking todo as undone...");
        TodoEdit {
            uid_or_short_id: self.uid_or_short_id,
            output_format: self.output_format,
            patch: TodoPatch {
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
                now: Local::now().naive_local(),
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

        let formatter = TodoFormatter::new(conds.now).with_output_format(output_format);
        println!("{}", formatter.format(&todos));
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct TodoEdit {
    uid_or_short_id: String,
    output_format: ArgOutputFormat,
    patch: TodoPatch,
}

impl TodoEdit {
    fn arg_id() -> Arg {
        arg!(id: <ID> "The short id or uid of the todo to edit")
    }

    fn parse_id(matches: &ArgMatches) -> String {
        matches
            .get_one::<String>("id")
            .expect("id is required")
            .clone()
    }

    fn arg_description() -> Arg {
        arg!(--description <DESCRIPTION> "Description of the todo")
    }

    fn parse_description(matches: &ArgMatches) -> Option<String> {
        matches.get_one::<String>("description").cloned()
    }

    fn arg_due() -> Arg {
        arg!(--due <DUE> "Due date and time of the todo")
    }

    fn parse_due(matches: &ArgMatches) -> Option<String> {
        matches.get_one::<String>("due").cloned()
    }

    fn arg_percent_complete() -> Arg {
        fn from_0_to_100(s: &str) -> Result<u8, String> {
            number_range(s, 0, 100)
        }

        arg!(--percent <PERCENT> "Percent complete of the todo (0-100)").value_parser(from_0_to_100)
    }

    fn parse_percent_complete(matches: &ArgMatches) -> Option<u8> {
        matches.get_one::<u8>("percent").cloned()
    }

    fn arg_summary(positional: bool) -> Arg {
        if positional {
            arg!(summary: <SUMMARY> "Summary of the todo")
        } else {
            arg!(summary: -s --summary <SUMMARY> "Summary of the todo")
        }
    }

    fn parse_summary(matches: &ArgMatches) -> Option<String> {
        matches.get_one::<String>("summary").cloned()
    }

    async fn run(mut self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        let now = Local::now().naive_local();

        log::debug!("Edit todo ...");
        self.patch.uid = self
            .uid_or_short_id
            .parse()
            .ok()
            .and_then(|a| map.find(a))
            .unwrap_or_else(|| self.uid_or_short_id.to_string()); // treat it as a UID if is not a short ID
        let todo = aim.update_todo(self.patch).await?;

        let todo = TodoWithShortId::with(map, todo);
        let formatter = TodoFormatter::new(now).with_output_format(self.output_format);
        println!("{}", formatter.format(&[todo]));
        Ok(())
    }
}

fn parse_datetime(dt: &str) -> Result<Option<LooseDateTime>, &str> {
    if dt.is_empty() {
        Ok(None)
    } else if let Ok(dt) = NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M") {
        Ok(Some(match Local.from_local_datetime(&dt) {
            LocalResult::Single(dt) => LooseDateTime::Local(dt),
            LocalResult::Ambiguous(dt1, _) => {
                log::warn!("Ambiguous local time for {dt} in local, picking earliest");
                LooseDateTime::Local(dt1)
            }
            LocalResult::None => {
                log::warn!("Invalid local time for {dt} in local, falling back to floating");
                LooseDateTime::Floating(dt)
            }
        }))
    } else if let Ok(time) = NaiveTime::parse_from_str(dt, "%H:%M") {
        // If the input is just a time, we assume it's today
        match Local::now().with_time(time) {
            LocalResult::Single(dt) => Ok(Some(LooseDateTime::Local(dt))),
            LocalResult::Ambiguous(dt1, _) => {
                log::warn!("Ambiguous local time for {dt} in local, picking earliest");
                Ok(Some(LooseDateTime::Local(dt1)))
            }
            LocalResult::None => Err("Invalid local time"),
        }
    } else if let Ok(date) = NaiveDate::parse_from_str(dt, "%Y-%m-%d") {
        Ok(Some(LooseDateTime::DateOnly(date)))
    } else {
        Err("Invalid date format. Expected format: YYYY-MM-DD, HH:MM and YYYY-MM-DD HH:MM")
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ArgStatus {
    NeedsAction,
    Completed,
    InProcess,
    Cancelled,
}

impl ArgStatus {
    pub fn arg() -> Arg {
        arg!(--status <STATUS> "Status of the todo").value_parser(value_parser!(ArgStatus))
    }

    pub fn parse(matches: &ArgMatches) -> Option<&Self> {
        matches.get_one::<ArgStatus>("status")
    }
}

impl From<&ArgStatus> for TodoStatus {
    fn from(status: &ArgStatus) -> Self {
        match status {
            ArgStatus::NeedsAction => TodoStatus::NeedsAction,
            ArgStatus::Completed => TodoStatus::Completed,
            ArgStatus::InProcess => TodoStatus::InProcess,
            ArgStatus::Cancelled => TodoStatus::Cancelled,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ArgPriority {
    #[clap(name = "1", hide = true)]
    P1,

    #[clap(name = "high", aliases = ["h" ,"2"])]
    P2,

    #[clap(name = "3", hide = true)]
    P3,

    #[clap(name = "4", hide = true)]
    P4,

    #[clap(name = "middle", aliases = ["m", "mid", "5"])]
    P5,

    #[clap(name = "6", hide = true)]
    P6,

    #[clap(name = "7", hide = true)]
    P7,

    #[clap(name = "low", aliases = ["l", "8"])]
    P8,

    #[clap(name = "9", hide = true)]
    P9,

    #[clap(name = "none", aliases = ["n", "0"])]
    None,
}

impl ArgPriority {
    pub fn arg() -> Arg {
        arg!(-p --priority <PRIORITY> "Priority of the todo")
            .value_parser(value_parser!(ArgPriority))
    }

    pub fn parse(matches: &ArgMatches) -> Option<&Self> {
        matches.get_one::<ArgPriority>("priority")
    }
}

impl From<&ArgPriority> for Priority {
    fn from(priority: &ArgPriority) -> Self {
        match priority {
            ArgPriority::P1 => Priority::P1,
            ArgPriority::P2 => Priority::P2,
            ArgPriority::P3 => Priority::P3,
            ArgPriority::P4 => Priority::P4,
            ArgPriority::P5 => Priority::P5,
            ArgPriority::P6 => Priority::P6,
            ArgPriority::P7 => Priority::P7,
            ArgPriority::P8 => Priority::P8,
            ArgPriority::P9 => Priority::P9,
            ArgPriority::None => Priority::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ArgOutputFormat;
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
                "--priority",
                "1",
            ])
            .unwrap();
        let sub_matches = matches.subcommand_matches("new").unwrap();
        let parsed = CmdTodoNew::parse(sub_matches);
        assert_eq!(parsed.summary, "Another summary");
        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.due, Some("2025-01-01 12:00:00".to_string()));
        assert_eq!(parsed.priority, Priority::P1);
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
        assert_eq!(parsed.uid_or_short_id, "test_id");
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
        assert_eq!(parsed.uid_or_short_id, "abc");
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
        assert_eq!(parsed.uid_or_short_id, "abc");
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
