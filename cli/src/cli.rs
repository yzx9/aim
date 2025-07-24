// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cmd_dashboard::CmdDashboard,
    cmd_event::CmdEventList,
    cmd_generate_completion::CmdGenerateCompletion,
    cmd_todo::{CmdTodoDone, CmdTodoList, CmdTodoNew, CmdTodoUndo},
    config::APP_NAME,
};
use clap::{Arg, ArgMatches, Command, ValueEnum, ValueHint, arg, crate_version, value_parser};
use std::{error::Error, path::PathBuf};

/// Run the AIM command-line interface.
pub async fn run() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    Cli::parse().run().await
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
        Command::new(APP_NAME)
            .about("Analyze. Interact. Manage Your Time, with calendar support.")
            .author("Zexin Yuan <aim@yzx9.xyz>")
            .version(crate_version!())
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
                    .subcommand(CmdTodoList::command())
                    .subcommand(CmdTodoNew::command())
                    .subcommand(CmdTodoDone::command())
                    .subcommand(CmdTodoUndo::command()),
            )
            .subcommand(CmdTodoDone::command())
            .subcommand(CmdTodoUndo::command())
            .subcommand(CmdGenerateCompletion::command())
    }

    /// Parse the command-line arguments
    pub fn parse() -> Self {
        let matches = Self::command().get_matches();

        let command = match matches.subcommand() {
            Some(("dashboard", _)) => Commands::Dashboard(CmdDashboard::parse()),
            Some(("event", matches)) => match matches.subcommand() {
                Some(("list", matches)) => Commands::EventList(CmdEventList::parse(matches)),
                _ => unreachable!(),
            },
            Some(("todo", matches)) => match matches.subcommand() {
                Some(("list", matches)) => Commands::TodoList(CmdTodoList::parse(matches)),
                Some(("new", matches)) => Commands::TodoNew(CmdTodoNew::parse(matches)),
                Some(("done", matches)) => Commands::TodoDone(CmdTodoDone::parse(matches)),
                Some(("undo", matches)) => Commands::TodoUndo(CmdTodoUndo::parse(matches)),
                _ => unreachable!(),
            },
            Some(("done", matches)) => Commands::TodoDone(CmdTodoDone::parse(matches)),
            Some(("undo", matches)) => Commands::TodoUndo(CmdTodoUndo::parse(matches)),
            Some(("generate-completion", matches)) => {
                Commands::GenerateCompletion(CmdGenerateCompletion::parse(matches))
            }
            None => Commands::Dashboard(CmdDashboard::parse()),
            _ => unreachable!(),
        };

        let config = matches.get_one::<PathBuf>("config").cloned();
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
        match self {
            Commands::Dashboard(a) => a.run(config).await,
            Commands::EventList(a) => a.run(config).await,
            Commands::TodoNew(a) => a.run(config).await,
            Commands::TodoDone(a) => a.run(config).await,
            Commands::TodoUndo(a) => a.run(config).await,
            Commands::TodoList(a) => a.run(config).await,
            Commands::GenerateCompletion(a) => {
                a.run();
                Ok(())
            }
        }
    }
}

/// The output format for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ArgOutputFormat {
    Json,
    Table,
}

impl ArgOutputFormat {
    pub fn arg() -> Arg {
        arg!(--"output-format" <FORMAT> "Output format")
            .value_parser(value_parser!(ArgOutputFormat))
            .default_value("table")
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        matches
            .get_one::<ArgOutputFormat>("output-format")
            .copied()
            .unwrap_or(ArgOutputFormat::Table)
    }
}
