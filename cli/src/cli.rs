// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::config::APP_NAME;
use clap::{Arg, Command, ValueEnum, ValueHint, arg, crate_version, value_parser};
use clap_complete::generate;
use std::{io, path::PathBuf, process};

pub struct Cli {
    pub config: Option<PathBuf>,
    pub command: Commands,
}

impl Cli {
    pub fn parse() -> Cli {
        let matches = build_cli().get_matches();

        fn output_format(matches: &clap::ArgMatches) -> OutputFormat {
            matches
                .get_one::<OutputFormat>("output-format")
                .copied()
                .unwrap_or(OutputFormat::Table)
        }

        let command = match matches.subcommand() {
            Some(("events", matches)) => Commands::Events(ListArgs {
                output_format: output_format(matches),
            }),
            Some(("todos", matches)) => Commands::Todos(ListArgs {
                output_format: output_format(matches),
            }),
            Some(("done", matches)) => Commands::Done {
                uid_or_short_id: matches
                    .get_one::<String>("id")
                    .expect("id is required")
                    .clone(),
            },
            Some(("undo", matches)) => Commands::Undo {
                uid_or_short_id: matches
                    .get_one::<String>("id")
                    .expect("id is required")
                    .clone(),
            },
            Some(("generate-completion", matches)) => match matches.get_one::<Shell>("shell") {
                Some(shell) => {
                    shell.generate_completion();
                    process::exit(1)
                }
                _ => unreachable!(),
            },
            None => Commands::Dashboard,
            _ => unreachable!(),
        };

        let config = matches.get_one::<PathBuf>("config").cloned();
        Cli { config, command }
    }
}

#[derive(Debug, Clone)]
pub enum Commands {
    Dashboard,
    Events(ListArgs),
    Todos(ListArgs),
    Done { uid_or_short_id: String },
    Undo { uid_or_short_id: String },
}

#[derive(Debug, Clone)]
pub struct ListArgs {
    pub output_format: OutputFormat,
}

fn build_cli() -> Command {
    fn output_format() -> Arg {
        arg!(--"output-format" <FORMAT> "Output format")
            .value_parser(value_parser!(OutputFormat))
            .default_value("table")
    }

    Command::new(APP_NAME)
        .about("Analyze. Interact. Manage Your Time.")
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
        .subcommand(
            Command::new("events")
                .about("List events")
                .arg(output_format()),
        )
        .subcommand(
            Command::new("todos")
                .about("List todos")
                .arg(output_format()),
        )
        .subcommand(
            Command::new("done")
                .about("Mark a todo as done")
                .arg(arg!(<id> "The short id or uid of the todo to mark as done")),
        )
        .subcommand(
            Command::new("undo")
                .about("Mark a todo as undone")
                .arg(arg!(<id> "The short id or uid of the todo to mark as done")),
        )
        .subcommand(
            Command::new("generate-completion")
                .about("Generate shell completion for the specified shell")
                .hide(true)
                .arg(
                    arg!(shell: <SHELL> "The shell generator to use")
                        .value_parser(value_parser!(Shell)),
                ),
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Shell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    #[clap(name = "powershell")]
    #[allow(clippy::enum_variant_names)]
    PowerShell,
    Zsh,
}

impl Shell {
    fn generate_completion(&self) {
        use clap_complete::Shell as ClapShell;

        let mut cmd = build_cli();
        let name = cmd.get_name().to_string();
        match self {
            Shell::Bash => generate(ClapShell::Bash, &mut cmd, name, &mut io::stdout()),
            Shell::Elvish => generate(ClapShell::Elvish, &mut cmd, name, &mut io::stdout()),
            Shell::Fish => generate(ClapShell::Fish, &mut cmd, name, &mut io::stdout()),
            Shell::PowerShell => generate(ClapShell::PowerShell, &mut cmd, name, &mut io::stdout()),
            Shell::Zsh => generate(ClapShell::Zsh, &mut cmd, name, &mut io::stdout()),

            Shell::Nushell => generate(
                clap_complete_nushell::Nushell {},
                &mut cmd,
                name,
                &mut io::stdout(),
            ),
        }
    }
}
