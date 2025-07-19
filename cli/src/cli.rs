// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::config::APP_NAME;
use clap::{Arg, Command, ValueEnum, ValueHint, value_parser};
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
                .get_one::<OutputFormat>("output_format")
                .cloned()
                .unwrap_or(OutputFormat::Table)
        }

        let command = match matches.subcommand() {
            Some(("events", matches)) => Commands::Events(ListArgs {
                output_format: output_format(matches),
            }),
            Some(("todos", matches)) => Commands::Todos(ListArgs {
                output_format: output_format(matches),
            }),
            Some(("generate", matches)) => match matches.subcommand() {
                Some(("completion", matches)) => match matches.get_one::<Shell>("shell").copied() {
                    Some(shell) => {
                        shell.generate_completion();
                        process::exit(1)
                    }
                    _ => unreachable!(),
                },
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
}

#[derive(Debug, Clone)]
pub struct ListArgs {
    pub output_format: OutputFormat,
}

fn build_cli() -> Command {
    fn output_format() -> Arg {
        Arg::new("output_format")
            .long("output-format")
            .value_name("FORMAT")
            .value_parser(value_parser!(OutputFormat))
            .default_value("table")
            .help("Output format for events")
    }

    Command::new(APP_NAME)
        .about("Analyze. Interact. Manage Your Time.")
        .author("Zexin Yuan <aim@yzx9.xyz>")
        .subcommand_required(false) // allow default to dashboard
        .arg_required_else_help(false)
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("CONFIG")
                .value_parser(value_parser!(PathBuf))
                .default_value("$XDG_CONFIG_HOME/aim/config.toml")
                .help("Path to the configuration file")
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
            Command::new("generate")
                .about("Generate various outputs")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("completion")
                        .about("Generate shell completion for the specified shell")
                        .arg(
                            Arg::new("shell")
                                .value_name("SHELL")
                                .value_parser(value_parser!(Shell))
                                .required(true)
                                .help("The shell generator to use"),
                        ),
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
