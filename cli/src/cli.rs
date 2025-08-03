// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{error::Error, ffi::OsString, path::PathBuf};

use aimcal_core::{APP_NAME, Aim};
use clap::{ArgMatches, Command, ValueHint, arg, builder::styling, crate_version, value_parser};
use colored::Colorize;
use futures::{FutureExt, future::BoxFuture};

use crate::cmd_dashboard::CmdDashboard;
use crate::cmd_event::CmdEventList;
use crate::cmd_generate_completion::CmdGenerateCompletion;
use crate::cmd_todo::{CmdTodoDone, CmdTodoEdit, CmdTodoList, CmdTodoNew, CmdTodoUndo};
use crate::cmd_tui::{CmdEdit, CmdNew};
use crate::config::parse_config;

/// Run the AIM command-line interface.
pub async fn run() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    match Cli::parse() {
        Ok(cli) => {
            if let Err(e) = cli.run().await {
                println!("{} {}", "Error:".red(), e);
            }
        }
        Err(e) => println!("{} {}", "Error:".red(), e),
    };
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
            .subcommand(CmdNew::command())
            .subcommand(CmdEdit::command())
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
            .subcommand(CmdTodoUndo::command().hide(true)) // TODO: remove in v0.4.0
            .subcommand(CmdGenerateCompletion::command())
    }

    /// Parse the command-line arguments
    pub fn parse() -> Result<Self, Box<dyn Error>> {
        let commands = Self::command();
        let matches = commands.get_matches();
        Self::from(matches)
    }

    /// Parse the specified arguments
    pub fn try_parse_from<I, T>(args: I) -> Result<Self, Box<dyn Error>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let commands = Self::command();
        let matches = commands.try_get_matches_from(args)?;
        Self::from(matches)
    }

    /// Create a CLI instance from the `ArgMatches`
    pub fn from(matches: ArgMatches) -> Result<Self, Box<dyn Error>> {
        use Commands::*;
        let command = match matches.subcommand() {
            Some((CmdDashboard::NAME, matches)) => Dashboard(CmdDashboard::from(matches)),
            Some((CmdNew::NAME, matches)) => New(CmdNew::from(matches)),
            Some((CmdEdit::NAME, matches)) => Edit(CmdEdit::from(matches)),
            Some(("event", matches)) => match matches.subcommand() {
                Some(("list", matches)) => EventList(CmdEventList::from(matches)),
                _ => unreachable!(),
            },
            Some(("todo", matches)) => match matches.subcommand() {
                Some((CmdTodoNew::NAME, matches)) => TodoNew(CmdTodoNew::from(matches)?),
                Some((CmdTodoEdit::NAME, matches)) => TodoEdit(CmdTodoEdit::from(matches)),
                Some((CmdTodoDone::NAME, matches)) => TodoDone(CmdTodoDone::from(matches)),
                Some((CmdTodoUndo::NAME, matches)) => TodoUndo(CmdTodoUndo::from(matches)),
                Some((CmdTodoList::NAME, matches)) => TodoList(CmdTodoList::from(matches)),
                _ => unreachable!(),
            },
            Some((CmdTodoDone::NAME, matches)) => TodoDone(CmdTodoDone::from(matches)),
            Some((CmdTodoUndo::NAME, matches)) => Undo(CmdTodoUndo::from(matches)),
            Some((CmdGenerateCompletion::NAME, matches)) => {
                GenerateCompletion(CmdGenerateCompletion::from(matches))
            }
            None => Dashboard(CmdDashboard),
            _ => unreachable!(),
        };

        let config = matches.get_one("config").cloned();
        Ok(Cli { config, command })
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

    /// New a event or todo
    New(CmdNew),

    /// Edit a event or todo
    Edit(CmdEdit),

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

    /// Mark a todo as undone
    Undo(CmdTodoUndo),

    /// Generate shell completion
    GenerateCompletion(CmdGenerateCompletion),
}

impl Commands {
    /// Run the command with the given configuration
    #[rustfmt::skip]
    pub async fn run(self, config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        use Commands::*;
        match self {
            Dashboard(a) => Self::run_with(config, |x| a.run(x).boxed()).await,
            New(a)       => Self::run_with(config, |x| a.run(x).boxed()).await,
            Edit(a)      => Self::run_with(config, |x| a.run(x).boxed()).await,
            EventList(a) => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoNew(a)   => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoEdit(a)  => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoDone(a)  => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoUndo(a)  => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoList(a)  => Self::run_with(config, |x| a.run(x).boxed()).await,
            Undo(a) => {
                println!(
                    "{} `aim undo` is now `aim todo undo`, the shortcut will be removed in v0.4.0",
                    "Deprecated:".yellow(),
                );
                Self::run_with(config, |x| a.run(x).boxed()).await
            },
            GenerateCompletion(a) => a.run(),
        }
    }

    async fn run_with<F>(config: Option<PathBuf>, f: F) -> Result<(), Box<dyn Error>>
    where
        F: for<'a> FnOnce(&'a mut Aim) -> BoxFuture<'a, Result<(), Box<dyn Error>>>,
    {
        log::debug!("Parsing configuration...");
        let (core_config, _config) = parse_config(config).await?;
        let mut aim = Aim::new(core_config).await?;

        f(&mut aim).await?;

        aim.close().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cmd_generate_completion::Shell, parser::ArgOutputFormat};
    use aimcal_core::Id;

    #[test]
    fn test_parse_config() {
        let cli = Cli::try_parse_from(vec!["test", "-c", "/tmp/config.toml"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/tmp/config.toml")));
        assert!(matches!(cli.command, Commands::Dashboard(_)));
    }

    #[test]
    fn test_parse_default_dashboard() {
        let cli = Cli::try_parse_from(vec!["test"]).unwrap();
        assert!(matches!(cli.command, Commands::Dashboard(_)));
    }

    #[test]
    fn test_parse_dashboard() {
        let cli = Cli::try_parse_from(vec!["test", "dashboard"]).unwrap();
        assert!(matches!(cli.command, Commands::Dashboard(_)));
    }

    #[test]
    fn test_parse_new() {
        let cli = Cli::try_parse_from(vec!["test", "new"]).unwrap();
        assert!(matches!(cli.command, Commands::New(_)));
    }

    #[test]
    fn test_parse_add() {
        let cli = Cli::try_parse_from(vec!["test", "add"]).unwrap();
        assert!(matches!(cli.command, Commands::New(_)));
    }

    #[test]
    fn test_parse_edit() {
        let cli = Cli::try_parse_from(vec!["test", "edit", "id1"]).unwrap();
        assert!(matches!(cli.command, Commands::Edit(_)));
    }

    #[test]
    fn test_parse_event_list() {
        let args = vec!["test", "event", "list", "--output-format", "json"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::EventList(cmd) => {
                assert_eq!(cmd.output_format, ArgOutputFormat::Json);
            }
            _ => panic!("Expected EventList command"),
        }
    }

    #[test]
    fn test_parse_todo_new() {
        let cli = Cli::try_parse_from(vec!["test", "todo", "new", "a new todo"]).unwrap();
        assert!(matches!(cli.command, Commands::TodoNew(_)));
    }

    #[test]
    fn test_parse_todo_add() {
        let cli = Cli::try_parse_from(vec!["test", "todo", "add", "a new todo"]).unwrap();
        assert!(matches!(cli.command, Commands::TodoNew(_)));
    }

    #[test]
    fn test_parse_todo_edit() {
        let args = vec!["test", "todo", "edit", "some_id", "-s", "new summary"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TodoEdit(cmd) => {
                assert_eq!(cmd.id, Id::ShortIdOrUid("some_id".to_string()));
                assert_eq!(cmd.summary, Some("new summary".to_string()));
            }
            _ => panic!("Expected TodoEdit command"),
        }
    }

    #[test]
    fn test_parse_todo_done() {
        let cli = Cli::try_parse_from(vec!["test", "todo", "done", "id1", "id2"]).unwrap();
        match cli.command {
            Commands::TodoDone(cmd) => {
                assert_eq!(
                    cmd.ids,
                    vec![
                        Id::ShortIdOrUid("id1".to_string()),
                        Id::ShortIdOrUid("id2".to_string())
                    ]
                );
            }
            _ => panic!("Expected TodoDone command"),
        }
    }

    #[test]
    fn test_parse_todo_undo() {
        let cli = Cli::try_parse_from(vec!["test", "todo", "undo", "id1"]).unwrap();
        match cli.command {
            Commands::TodoUndo(cmd) => {
                assert_eq!(cmd.ids, vec![Id::ShortIdOrUid("id1".to_string())]);
            }
            _ => panic!("Expected TodoUndo command"),
        }
    }

    #[test]
    fn test_parse_todo_list() {
        let args = vec!["test", "todo", "list", "--output-format", "json"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TodoList(cmd) => {
                assert_eq!(cmd.output_format, ArgOutputFormat::Json);
            }
            _ => panic!("Expected TodoList command"),
        }
    }

    #[test]
    fn test_parse_done() {
        let cli = Cli::try_parse_from(vec!["test", "done", "id1", "id2"]).unwrap();
        match cli.command {
            Commands::TodoDone(cmd) => {
                assert_eq!(
                    cmd.ids,
                    vec![
                        Id::ShortIdOrUid("id1".to_string()),
                        Id::ShortIdOrUid("id2".to_string())
                    ]
                );
            }
            _ => panic!("Expected TodoDone command"),
        }
    }

    #[test]
    fn test_parse_undo() {
        let cli = Cli::try_parse_from(vec!["test", "undo", "id1"]).unwrap();
        match cli.command {
            Commands::Undo(cmd) => {
                assert_eq!(cmd.ids, vec![Id::ShortIdOrUid("id1".to_string())]);
            }
            _ => panic!("Expected Undo command"),
        }
    }

    #[test]
    fn test_parse_generate_completions() {
        let args = vec!["test", "generate-completion", "zsh"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::GenerateCompletion(cmd) => {
                assert_eq!(cmd.shell, Shell::Zsh);
            }
            _ => panic!("Expected GenerateCompletion command"),
        }
    }
}
