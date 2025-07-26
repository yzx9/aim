// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Config,
    cmd_dashboard::CmdDashboard,
    cmd_event::CmdEventList,
    cmd_generate_completion::CmdGenerateCompletion,
    cmd_todo::{CmdTodoDone, CmdTodoEdit, CmdTodoList, CmdTodoNew, CmdTodoUndo},
    config::APP_NAME,
    short_id::ShortIdMap,
};
use aimcal_core::Aim;
use clap::{Command, ValueHint, arg, builder::styling, crate_version, value_parser};
use colored::Colorize;
use futures::{FutureExt, future::BoxFuture};
use std::{error::Error, path::PathBuf};
use tokio::try_join;

/// Run the AIM command-line interface.
pub async fn run() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    if let Err(e) = Cli::parse().run().await {
        println!("{} {}", "Error:".red(), e);
    }
    Ok(())
}

/// Command-line interface
#[derive(Debug)]
pub struct Cli {
    /// Path to the configuration file
    pub config: Option<PathBuf>,

    /// The command to execute
    pub command: Commands,
}

impl Cli {
    /// Create the command-line interface
    pub fn command() -> Command {
        const STYLES: styling::Styles = styling::Styles::styled()
            .header(styling::AnsiColor::Green.on_default().bold())
            .usage(styling::AnsiColor::Green.on_default().bold())
            .literal(styling::AnsiColor::Blue.on_default().bold())
            .placeholder(styling::AnsiColor::Cyan.on_default());

        Command::new(APP_NAME)
            .about("Analyze. Interact. Manage Your Time, with calendar support.")
            .author("Zexin Yuan <aim@yzx9.xyz>")
            .version(crate_version!())
            .styles(STYLES)
            .subcommand_required(false) // allow default to dashboard
            .arg_required_else_help(false)
            .arg(
                arg!(-c --config [CONFIG] "Path to the configuration file")
                    .long_help(
                        "\
Path to the configuration file. Defaults to $XDG_CONFIG_HOME/aim/config.toml on Linux and MacOS, \
%LOCALAPPDATA%/aim/config.toml on Windows.",
                    )
                    .value_parser(value_parser!(PathBuf))
                    .value_hint(ValueHint::FilePath),
            )
            .subcommand(CmdDashboard::command())
            .subcommand(
                Command::new("event")
                    .alias("e")
                    .about("Manage your event list")
                    .arg_required_else_help(true)
                    .subcommand_required(true)
                    .subcommand(CmdEventList::command()),
            )
            .subcommand(
                Command::new("todo")
                    .alias("t")
                    .about("Manage your todo list")
                    .arg_required_else_help(true)
                    .subcommand_required(true)
                    .subcommand(CmdTodoNew::command())
                    .subcommand(CmdTodoEdit::command())
                    .subcommand(CmdTodoDone::command())
                    .subcommand(CmdTodoUndo::command())
                    .subcommand(CmdTodoList::command()),
            )
            .subcommand(CmdTodoDone::command())
            .subcommand(CmdTodoUndo::command())
            .subcommand(CmdGenerateCompletion::command())
    }

    /// Parse the command-line arguments
    pub fn parse() -> Self {
        use Commands::*;
        let matches = Self::command().get_matches();
        let command = match matches.subcommand() {
            Some((CmdDashboard::NAME, _)) => Dashboard(CmdDashboard::parse()),
            Some(("event", matches)) => match matches.subcommand() {
                Some(("list", matches)) => EventList(CmdEventList::parse(matches)),
                _ => unreachable!(),
            },
            Some(("todo", matches)) => match matches.subcommand() {
                Some((CmdTodoNew::NAME, matches)) => TodoNew(CmdTodoNew::parse(matches)),
                Some((CmdTodoEdit::NAME, matches)) => TodoEdit(CmdTodoEdit::parse(matches)),
                Some((CmdTodoDone::NAME, matches)) => TodoDone(CmdTodoDone::parse(matches)),
                Some((CmdTodoUndo::NAME, matches)) => TodoUndo(CmdTodoUndo::parse(matches)),
                Some((CmdTodoList::NAME, matches)) => TodoList(CmdTodoList::parse(matches)),
                _ => unreachable!(),
            },
            Some((CmdTodoDone::NAME, matches)) => TodoDone(CmdTodoDone::parse(matches)),
            Some((CmdTodoUndo::NAME, matches)) => TodoUndo(CmdTodoUndo::parse(matches)),
            Some((CmdGenerateCompletion::NAME, matches)) => {
                GenerateCompletion(CmdGenerateCompletion::parse(matches))
            }
            None => Dashboard(CmdDashboard::parse()),
            _ => unreachable!(),
        };

        let config = matches.get_one("config").cloned();
        Cli { config, command }
    }

    /// Run the command
    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        self.command.run(self.config).await
    }
}

/// The commands available in the CLI
#[derive(Debug, Clone)]
pub enum Commands {
    /// Show the dashboard
    Dashboard(CmdDashboard),

    /// List events
    EventList(CmdEventList),

    /// Add a new todo
    TodoNew(CmdTodoNew),

    /// Edit a todo
    TodoEdit(CmdTodoEdit),

    /// Mark a todo as done
    TodoDone(CmdTodoDone),

    /// Mark a todo as undone
    TodoUndo(CmdTodoUndo),

    /// List todos
    TodoList(CmdTodoList),

    /// Generate shell completion
    GenerateCompletion(CmdGenerateCompletion),
}

impl Commands {
    /// Run the command with the given configuration
    pub async fn run(self, config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        use Commands::*;
        match self {
            Dashboard(a) => Self::run_with(config, |_, y, z| a.run(y, z).boxed()).await,
            EventList(a) => Self::run_with(config, |_, y, z| a.run(y, z).boxed()).await,
            TodoNew(a) => Self::run_with(config, |x, y, z| a.run(x, y, z).boxed()).await,
            TodoEdit(a) => Self::run_with(config, |_, y, z| a.run(y, z).boxed()).await,
            TodoDone(a) => Self::run_with(config, |_, y, z| a.run(y, z).boxed()).await,
            TodoUndo(a) => Self::run_with(config, |_, y, z| a.run(y, z).boxed()).await,
            TodoList(a) => Self::run_with(config, |_, y, z| a.run(y, z).boxed()).await,
            GenerateCompletion(a) => a.run(),
        }
    }

    async fn run_with<F>(config: Option<PathBuf>, f: F) -> Result<(), Box<dyn Error>>
    where
        F: for<'a> FnOnce(
            &'a Config,
            &'a Aim,
            &'a ShortIdMap,
        ) -> BoxFuture<'a, Result<(), Box<dyn Error>>>,
    {
        log::debug!("Parsing configuration...");
        let config = Config::parse(config).await?;
        let (aim, map) = try_join!(Aim::new(&config.core), ShortIdMap::load_or_new(&config))?;

        f(&config, &aim, &map).await?;

        map.dump(&config).await?;
        Ok(())
    }
}
