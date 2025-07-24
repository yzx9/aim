// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Config,
    cli::ArgOutputFormat,
    short_id::{ShortIdMap, TodoWithShortId},
    todo_formatter::TodoFormatter,
};
use aimcal_core::{
    Aim, DatePerhapsTime, Priority, SortOrder, TodoConditions, TodoDraft, TodoPatch, TodoSort,
    TodoStatus,
};
use chrono::{Duration, Local, NaiveDateTime, Utc};
use chrono_tz::UTC;
use clap::{Arg, ArgMatches, Command, arg, value_parser};
use std::{error::Error, path::PathBuf, str::FromStr};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CmdTodoDraft {
    pub summary: String,
    pub description: Option<String>,
    pub due: Option<String>,
    pub priority: Priority,
}

impl CmdTodoDraft {
    pub fn command() -> Command {
        Command::new("new")
            .about("Add a new todo item")
            .arg(arg!(summary: <SUMMARY> "Summary of the todo").required(true))
            .arg(arg!(-d --description <DESCRIPTION> "Description of the todo"))
            .arg(arg!(-u --due <DUE> "Due date and time of the todo"))
            .arg(ArgPriority::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            summary: matches
                .get_one::<String>("summary")
                .expect("summary is required")
                .clone(),
            description: matches.get_one::<String>("description").cloned(),
            due: matches.get_one::<String>("due").cloned(),
            priority: ArgPriority::parse(matches).into(),
        }
    }

    pub async fn run(&self, config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        log::debug!("Parsing configuration...");
        let config = Config::parse(config).await?;
        let aim = Aim::new(&config.core).await?;
        let map = ShortIdMap::load_or_new(&config)?;

        log::debug!("Add todos...");
        let due = self
            .due
            .as_ref()
            .and_then(|a| NaiveDateTime::parse_from_str(a, "%Y-%m-%d %H:%M:%S").ok())
            .map(|a| DatePerhapsTime {
                date: a.date(),
                time: Some(a.time()),
                tz: Some(UTC),
            });

        let draft = TodoDraft {
            uid: Uuid::new_v4().to_string(), // TODO: better uid
            description: self.description.clone(),
            due,
            priority: self.priority,
            summary: self.summary.clone(),
        };
        let todo = aim.new_todo(draft).await?;

        let todo = TodoWithShortId::with(&map, todo);
        let formatter = TodoFormatter::new(Local::now().naive_local());
        println!("{}", formatter.format(&[todo]));

        map.dump(&config)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoDone(TodoEdit);

impl CmdTodoDone {
    pub fn command() -> Command {
        Command::new("done")
            .about("Edit a todo item")
            .arg(arg!(<id> "The short id or uid of the todo to edit"))
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self(TodoEdit {
            uid_or_short_id: matches
                .get_one::<String>("id")
                .expect("id is required")
                .clone(),
            output_format: ArgOutputFormat::parse(matches),
        })
    }

    /// Mark a todo as done.
    pub async fn run(self, config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        log::debug!("Marking todo as done...");
        let patch = TodoPatch {
            completed: Some(Some(Utc::now().into())),
            status: Some(TodoStatus::Completed),
            ..Default::default()
        };
        self.0.run(config, patch).await
    }
}

#[derive(Debug, Clone)]
pub struct CmdTodoUndo(TodoEdit);

impl CmdTodoUndo {
    pub fn command() -> Command {
        Command::new("undo")
            .about("Edit a todo item")
            .arg(arg!(<id> "The short id or uid of the todo to edit"))
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self(TodoEdit {
            uid_or_short_id: matches
                .get_one::<String>("id")
                .expect("id is required")
                .clone(),
            output_format: ArgOutputFormat::parse(matches),
        })
    }

    /// Mark a todo as undone.
    pub async fn run(self, config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        log::debug!("Marking todo as undone...");
        let patch = TodoPatch {
            completed: Some(None),
            status: Some(TodoStatus::NeedsAction),
            ..Default::default()
        };
        self.0.run(config, patch).await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CmdTodoList {
    pub conds: TodoConditions,
    pub output_format: ArgOutputFormat,
}

impl CmdTodoList {
    pub fn command() -> Command {
        Command::new("todo")
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

    pub async fn run(self, config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        log::debug!("Parsing configuration...");
        let config = Config::parse(config).await?;
        let aim = Aim::new(&config.core).await?;
        let map = ShortIdMap::load_or_new(&config)?;

        log::debug!("Listing todos...");
        Self::list(&aim, &map, &self.conds, self.output_format).await?;

        map.dump(&config)?;
        Ok(())
    }

    /// List todos with the given conditions and output format.
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
    pub uid_or_short_id: String,
    pub output_format: ArgOutputFormat,
}

impl TodoEdit {
    async fn run(
        &self,
        config: Option<PathBuf>,
        mut patch: TodoPatch,
    ) -> Result<(), Box<dyn Error>> {
        let now = Local::now().naive_local();

        log::debug!("Parsing configuration...");
        let config = Config::parse(config).await?;
        let aim = Aim::new(&config.core).await?;
        let map = ShortIdMap::load_or_new(&config)?;

        log::debug!("Edit todo ...");
        patch.uid = self
            .uid_or_short_id
            .parse()
            .ok()
            .and_then(|a| map.find(a))
            .unwrap_or_else(|| self.uid_or_short_id.to_string()); // treat it as a UID if is not a short ID
        let todo = aim.update_todo(patch.clone()).await?;

        let todo = TodoWithShortId::with(&map, todo);
        let formatter = TodoFormatter::new(now).with_output_format(self.output_format);
        println!("{}", formatter.format(&[todo]));
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
pub enum ArgPriority { P1, P2, P3, P4, P5, P6, P7, P8, P9, None }

impl ArgPriority {
    pub fn arg() -> Arg {
        arg!(-p --priority <PRIORITY> "Priority of the todo")
            .value_parser(value_parser!(ArgPriority))
    }

    pub fn parse(matches: &ArgMatches) -> &Self {
        matches
            .get_one::<ArgPriority>("priority")
            .unwrap_or(&ArgPriority::None)
    }
}

impl FromStr for ArgPriority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1" => Ok(ArgPriority::P1),
            "2" | "high" => Ok(ArgPriority::P2),
            "3" => Ok(ArgPriority::P3),
            "4" => Ok(ArgPriority::P4),
            "5" | "middle" => Ok(ArgPriority::P5),
            "6" => Ok(ArgPriority::P6),
            "7" => Ok(ArgPriority::P7),
            "8" | "low" => Ok(ArgPriority::P8),
            "9" => Ok(ArgPriority::P9),
            _ => Err(format!("Invalid priority: {s}")),
        }
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
