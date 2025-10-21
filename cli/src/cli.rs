// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{error::Error, ffi::OsString, path::PathBuf};

use aimcal_core::{APP_NAME, Aim};
use clap::{ArgMatches, Command, ValueHint, arg, builder::styling, crate_version, value_parser};
use colored::Colorize;
use futures::{FutureExt, future::BoxFuture};
use tracing_subscriber::{EnvFilter, Registry, prelude::*};

use crate::cmd_event::{
    CmdEventDelay, CmdEventEdit, CmdEventList, CmdEventNew, CmdEventReschedule,
};
use crate::cmd_generate_completion::CmdGenerateCompletion;
use crate::cmd_todo::{
    CmdTodoCancel, CmdTodoDelay, CmdTodoDone, CmdTodoEdit, CmdTodoList, CmdTodoNew,
    CmdTodoReschedule, CmdTodoUndo,
};
use crate::cmd_toplevel::{CmdDashboard, CmdDelay, CmdFlush, CmdReschedule};
use crate::cmd_tui::{CmdEdit, CmdNew};
use crate::config::parse_config;

/// Run the AIM command-line interface.
///
/// # Errors
/// If an error occurs while running the CLI
pub async fn run() -> Result<(), Box<dyn Error>> {
    init_tracing()?;

    let err = match Cli::parse() {
        Ok(cli) => match cli.run().await {
            Ok(()) => return Ok(()),
            Err(e) => e,
        },
        Err(e) => e,
    };
    println!("{} {}", "Error:".red(), err);
    Ok(())
}

pub fn init_tracing() -> Result<(), Box<dyn Error>> {
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    let filter = EnvFilter::builder().try_from_env()?;

    let subscriber = Registry::default().with(filter).with(stdout_log);

    tracing::subscriber::set_global_default(subscriber)?;
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
    #[must_use]
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
            .subcommand(CmdDelay::command())
            .subcommand(CmdReschedule::command())
            .subcommand(
                Command::new("event")
                    .alias("e")
                    .about("Manage your event list")
                    .arg_required_else_help(true)
                    .subcommand_required(true)
                    .subcommand(CmdEventNew::command())
                    .subcommand(CmdEventEdit::command())
                    .subcommand(CmdEventDelay::command())
                    .subcommand(CmdEventReschedule::command())
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
                    .subcommand(CmdTodoCancel::command())
                    .subcommand(CmdTodoDelay::command())
                    .subcommand(CmdTodoReschedule::command())
                    .subcommand(CmdTodoList::command()),
            )
            .subcommand(CmdTodoDone::command())
            .subcommand(CmdFlush::command())
            .subcommand(CmdGenerateCompletion::command())
    }

    /// Parse the command-line arguments
    ///
    /// # Errors
    /// If an error occurs while parsing the arguments
    pub fn parse() -> Result<Self, Box<dyn Error>> {
        let commands = Self::command();
        let matches = commands.get_matches();
        Self::from(&matches)
    }

    /// Parse the specified arguments
    ///
    /// # Errors
    /// If an error occurs while parsing the arguments
    pub fn try_parse_from<I, T>(args: I) -> Result<Self, Box<dyn Error>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        #[allow(clippy::enum_glob_use)]
        let commands = Self::command();
        let matches = commands.try_get_matches_from(args)?;
        Self::from(&matches)
    }

    /// Create a CLI instance from the `ArgMatches`
    ///
    /// # Errors
    /// If an error occurs while parsing the arguments
    #[allow(clippy::enum_glob_use)]
    pub fn from(matches: &ArgMatches) -> Result<Self, Box<dyn Error>> {
        use Commands::*;
        let command = match matches.subcommand() {
            Some((CmdDashboard::NAME, matches)) => Dashboard(CmdDashboard::from(matches)),
            Some((CmdNew::NAME, matches)) => New(CmdNew::from(matches)),
            Some((CmdEdit::NAME, matches)) => Edit(CmdEdit::from(matches)),
            Some((CmdDelay::NAME, matches)) => Delay(CmdDelay::from(matches)),
            Some((CmdReschedule::NAME, matches)) => Reschedule(CmdReschedule::from(matches)),
            Some((CmdFlush::NAME, matches)) => Flush(CmdFlush::from(matches)),
            Some(("event", matches)) => match matches.subcommand() {
                Some((CmdEventNew::NAME, matches)) => EventNew(CmdEventNew::from(matches)),
                Some((CmdEventEdit::NAME, matches)) => EventEdit(CmdEventEdit::from(matches)),
                Some((CmdEventDelay::NAME, matches)) => EventDelay(CmdEventDelay::from(matches)),
                Some((CmdEventReschedule::NAME, matches)) => {
                    EventReschedule(CmdEventReschedule::from(matches))
                }
                Some((CmdEventList::NAME, matches)) => EventList(CmdEventList::from(matches)),
                _ => unreachable!(),
            },
            Some(("todo", matches)) => match matches.subcommand() {
                Some((CmdTodoNew::NAME, matches)) => TodoNew(CmdTodoNew::from(matches)),
                Some((CmdTodoEdit::NAME, matches)) => TodoEdit(CmdTodoEdit::from(matches)),
                Some((CmdTodoUndo::NAME, matches)) => TodoUndo(CmdTodoUndo::from(matches)),
                Some((CmdTodoDone::NAME, matches)) => TodoDone(CmdTodoDone::from(matches)),
                Some((CmdTodoCancel::NAME, matches)) => TodoCancel(CmdTodoCancel::from(matches)),
                Some((CmdTodoDelay::NAME, matches)) => TodoDelay(CmdTodoDelay::from(matches)),
                Some((CmdTodoReschedule::NAME, matches)) => {
                    TodoReschedule(CmdTodoReschedule::from(matches))
                }
                Some((CmdTodoList::NAME, matches)) => TodoList(CmdTodoList::from(matches)),
                _ => unreachable!(),
            },
            Some((CmdTodoDone::NAME, matches)) => TodoDone(CmdTodoDone::from(matches)),
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
    ///
    /// # Errors
    /// If an error occurs while running the command
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

    /// Delay an event or todo based on original time
    Delay(CmdDelay),

    /// Reschedule an event or todo based on current time
    Reschedule(CmdReschedule),

    /// Flush the short IDs
    Flush(CmdFlush),

    /// Add a new event
    EventNew(CmdEventNew),

    /// Edit an event
    EventEdit(CmdEventEdit),

    /// Delay an event based on original start
    EventDelay(CmdEventDelay),

    /// Reschedule an event based on current time
    EventReschedule(CmdEventReschedule),

    /// List events
    EventList(CmdEventList),

    /// Add a new todo
    TodoNew(CmdTodoNew),

    /// Edit a todo
    TodoEdit(CmdTodoEdit),

    /// Mark a todo as needs-action
    TodoUndo(CmdTodoUndo),

    /// Mark a todo as completed
    TodoDone(CmdTodoDone),

    /// Mark a todo as cancelled
    TodoCancel(CmdTodoCancel),

    /// Delay a todo based on original due
    TodoDelay(CmdTodoDelay),

    /// Reschedule a todo based on current time
    TodoReschedule(CmdTodoReschedule),

    /// List todos
    TodoList(CmdTodoList),

    /// Generate shell completion
    GenerateCompletion(CmdGenerateCompletion),
}

impl Commands {
    /// Run the command with the given configuration
    ///
    /// # Errors
    /// If an error occurs while running the command
    #[allow(clippy::enum_glob_use)]
    #[rustfmt::skip]
    #[tracing::instrument(skip_all, fields(trace_id = %uuid::Uuid::new_v4()))]
    pub async fn run(self, config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        use Commands::*;
        tracing::info!(?self, "running command");
        match self {
            Dashboard(a)       => Self::run_with(config, |x| a.run(x).boxed()).await,
            New(a)             => Self::run_with(config, |x| a.run(x).boxed()).await,
            Edit(a)            => Self::run_with(config, |x| a.run(x).boxed()).await,
            Delay(a)           => Self::run_with(config, |x| a.run(x).boxed()).await,
            Reschedule(a)      => Self::run_with(config, |x| a.run(x).boxed()).await,
            Flush(a)           => Self::run_with(config, |x| a.run(x).boxed()).await,
            EventNew(a)        => Self::run_with(config, |x| a.run(x).boxed()).await,
            EventEdit(a)       => Self::run_with(config, |x| a.run(x).boxed()).await,
            EventDelay(a)      => Self::run_with(config, |x| a.run(x).boxed()).await,
            EventReschedule(a) => Self::run_with(config, |x| a.run(x).boxed()).await,
            EventList(a)       => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoNew(a)         => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoEdit(a)        => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoUndo(a)        => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoDone(a)        => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoCancel(a)      => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoDelay(a)       => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoReschedule(a)  => Self::run_with(config, |x| a.run(x).boxed()).await,
            TodoList(a)        => Self::run_with(config, |x| a.run(x).boxed()).await,
            GenerateCompletion(a) => { a.run(); Ok(()) }
        }
    }

    async fn run_with<F>(config: Option<PathBuf>, f: F) -> Result<(), Box<dyn Error>>
    where
        F: for<'a> FnOnce(&'a mut Aim) -> BoxFuture<'a, Result<(), Box<dyn Error>>>,
    {
        tracing::debug!("parsing configuration...");
        let (core_config, _config) = parse_config(config).await?;

        tracing::debug!("instantiating...");
        let mut aim = Aim::new(core_config).await?;

        tracing::debug!("running command...");
        f(&mut aim).await?;

        tracing::debug!("closing...");
        aim.close().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use aimcal_core::Id;

    use crate::{cmd_generate_completion::Shell, util::OutputFormat};

    use super::*;

    #[test]
    fn test_parse_config() {
        let args = ["test", "-c", "/tmp/config.toml"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/tmp/config.toml")));
        assert!(matches!(cli.command, Commands::Dashboard(_)));
    }

    #[test]
    fn test_parse_default_dashboard() {
        let args = ["test"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Dashboard(_)));
    }

    #[test]
    fn test_parse_dashboard() {
        let args = ["test", "dashboard"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Dashboard(_)));
    }

    #[test]
    fn test_parse_new() {
        let args = ["test", "new"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::New(_)));
    }

    #[test]
    fn test_parse_add() {
        let args = ["test", "add"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::New(_)));
    }

    #[test]
    fn test_parse_edit() {
        let args = ["test", "edit", "id1"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Edit(_)));
    }

    #[test]
    fn test_parse_flush() {
        let args = ["test", "flush"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Flush(_)));
    }

    #[test]
    fn test_parse_event_new() {
        let cli = Cli::try_parse_from([
            "test",
            "event",
            "new",
            "a new event",
            "--start",
            "2025-01-01 10:00",
            "--end",
            "2025-01-01 12:00",
        ])
        .unwrap();
        assert!(matches!(cli.command, Commands::EventNew(_)));
    }

    #[test]
    fn test_parse_event_add() {
        let args = [
            "test",
            "event",
            "add",
            "a new event",
            "--start",
            "2025-01-01 10:00",
            "--end",
            "2025-01-01 12:00",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::EventNew(_)));
    }

    #[test]
    fn test_parse_event_edit() {
        let args = ["test", "event", "edit", "some_id", "-s", "new summary"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::EventEdit(cmd) => {
                assert_eq!(cmd.id, Id::ShortIdOrUid("some_id".to_string()));
                assert_eq!(cmd.summary, Some("new summary".to_string()));
            }
            _ => panic!("Expected EventEdit command"),
        }
    }

    #[test]
    fn test_parse_event_delay() {
        let args = ["test", "event", "delay", "id1", "id2"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::EventDelay(cmd) => {
                let expected_ids = [
                    Id::ShortIdOrUid("id1".to_string()),
                    Id::ShortIdOrUid("id2".to_string()),
                ];
                assert_eq!(cmd.ids, expected_ids);
            }
            _ => panic!("Expected EventDelay command"),
        }
    }

    #[test]
    fn test_parse_event_reschedule() {
        let args = ["test", "event", "reschedule", "id1", "id2"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::EventReschedule(cmd) => {
                let expected_ids = [
                    Id::ShortIdOrUid("id1".to_string()),
                    Id::ShortIdOrUid("id2".to_string()),
                ];
                assert_eq!(cmd.ids, expected_ids);
            }
            _ => panic!("Expected EventReschedule command"),
        }
    }

    #[test]
    fn test_parse_event_list() {
        let args = ["test", "event", "list", "--output-format", "json"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::EventList(cmd) => assert_eq!(cmd.output_format, OutputFormat::Json),
            _ => panic!("Expected EventList command"),
        }
    }

    #[test]
    fn test_parse_todo_new() {
        let args = ["test", "todo", "new", "a new todo"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::TodoNew(_)));
    }

    #[test]
    fn test_parse_todo_add() {
        let args = ["test", "todo", "add", "a new todo"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::TodoNew(_)));
    }

    #[test]
    fn test_parse_todo_edit() {
        let args = ["test", "todo", "edit", "some_id", "-s", "new summary"];
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
    fn test_parse_todo_undo() {
        let args = ["test", "todo", "undo", "id1", "id2"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TodoUndo(cmd) => {
                let expected_ids = [
                    Id::ShortIdOrUid("id1".to_string()),
                    Id::ShortIdOrUid("id2".to_string()),
                ];
                assert_eq!(cmd.ids, expected_ids);
            }
            _ => panic!("Expected TodoUndo command"),
        }
    }

    #[test]
    fn test_parse_todo_done() {
        let args = ["test", "todo", "done", "id1", "id2"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TodoDone(cmd) => {
                let expected_ids = [
                    Id::ShortIdOrUid("id1".to_string()),
                    Id::ShortIdOrUid("id2".to_string()),
                ];
                assert_eq!(cmd.ids, expected_ids);
            }
            _ => panic!("Expected TodoDone command"),
        }
    }

    #[test]
    fn test_parse_todo_cancel() {
        let args = ["test", "todo", "cancel", "id1", "id2"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TodoCancel(cmd) => {
                let expected_ids = [
                    Id::ShortIdOrUid("id1".to_string()),
                    Id::ShortIdOrUid("id2".to_string()),
                ];
                assert_eq!(cmd.ids, expected_ids);
            }
            _ => panic!("Expected TodoDone command"),
        }
    }

    #[test]
    fn test_parse_todo_delay() {
        let args = ["test", "todo", "delay", "id1", "id2", "id3"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TodoDelay(cmd) => {
                let expected_ids = [
                    Id::ShortIdOrUid("id1".to_string()),
                    Id::ShortIdOrUid("id2".to_string()),
                    Id::ShortIdOrUid("id3".to_string()),
                ];
                assert_eq!(cmd.ids, expected_ids);
            }
            _ => panic!("Expected TodoDelay command"),
        }
    }

    #[test]
    fn test_parse_todo_reschedule() {
        let args = ["test", "todo", "reschedule", "id1", "id2"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TodoReschedule(cmd) => {
                let expected_ids = [
                    Id::ShortIdOrUid("id1".to_string()),
                    Id::ShortIdOrUid("id2".to_string()),
                ];
                assert_eq!(cmd.ids, expected_ids);
            }
            _ => panic!("Expected TodoReschedule command"),
        }
    }

    #[test]
    fn test_parse_todo_list() {
        let args = ["test", "todo", "list", "--output-format", "json"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TodoList(cmd) => assert_eq!(cmd.output_format, OutputFormat::Json),
            _ => panic!("Expected TodoList command"),
        }
    }

    #[test]
    fn test_parse_done() {
        let args = ["test", "done", "id1", "id2"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TodoDone(cmd) => {
                let expected_ids = [
                    Id::ShortIdOrUid("id1".to_string()),
                    Id::ShortIdOrUid("id2".to_string()),
                ];
                assert_eq!(cmd.ids, expected_ids);
            }
            _ => panic!("Expected TodoDone command"),
        }
    }

    #[test]
    fn test_parse_generate_completions() {
        let args = ["test", "generate-completion", "zsh"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::GenerateCompletion(cmd) => assert_eq!(cmd.shell, Shell::Zsh),
            _ => panic!("Expected GenerateCompletion command"),
        }
    }
}
