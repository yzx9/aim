// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod config;
mod event_formatter;
mod table;
mod todo_formatter;

use crate::{
    config::{APP_NAME, parse_config},
    event_formatter::EventFormatter,
    todo_formatter::TodoFormatter,
};
use aim_core::{Aim, EventConditions, Pager, SortOrder, TodoConditions, TodoSortKey, TodoStatus};
use chrono::{Duration, Local};
use clap::{Arg, Command, ValueHint, value_parser};
use clap_complete::aot::generate;
use colored::Colorize;
use std::{error::Error, io, path::PathBuf, str::FromStr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut cmd = build_cli();
    let matches = build_cli().get_matches();

    let config = matches.get_one::<PathBuf>("config");

    match matches.subcommand() {
        Some(("events", _)) => events(config).await?,
        Some(("todos", _)) => todos(config).await?,
        Some(("generate", generate_matches)) => match generate_matches.subcommand() {
            Some(("completion", matches)) => match matches.get_one::<Shell>("shell").copied() {
                Some(generator) => generate_completion(generator, &mut cmd),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        None => dashboard(config).await?,
        _ => unreachable!(),
    }
    Ok(())
}

fn build_cli() -> Command {
    Command::new(APP_NAME)
        .about("An Information Management tool")
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
        .subcommand(Command::new("events").about("List events"))
        .subcommand(Command::new("todos").about("List todos"))
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
                            .help("The shell generator to use (e.g., bash, zsh, fish, powershell)"),
                    ),
            ),
        )
}

pub async fn events(config: Option<&PathBuf>) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = parse_config(config).await?;
    let aim = Aim::new(&config).await?;

    log::debug!("Listing events...");
    let now = Local::now().naive_local();
    let conds = EventConditions { now };
    list_events(&aim, &conds).await
}

pub async fn todos(config: Option<&PathBuf>) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = parse_config(config).await?;
    let aim = Aim::new(&config).await?;

    log::debug!("Listing todos...");
    let now = Local::now().naive_local();
    let conds = TodoConditions {
        now,
        status: Some(TodoStatus::NeedsAction),
        due: Some(Duration::days(2)),
    };
    list_todos(&aim, &conds).await
}

pub async fn dashboard(config: Option<&PathBuf>) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = parse_config(config).await?;
    let aim = Aim::new(&config).await?;

    log::debug!("Generating dashboard...");
    let now = Local::now().naive_local();

    println!("🗓️ {}", "Events".bold());
    let conds = EventConditions { now };
    list_events(&aim, &conds).await?;
    println!();

    println!("✅ {}", "Todos".bold());
    let conds = TodoConditions {
        now,
        status: Some(TodoStatus::NeedsAction),
        due: Some(Duration::days(2)),
    };
    list_todos(&aim, &conds).await?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    PowerShell,
    Zsh,
}

impl FromStr for Shell {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "elvish" => Ok(Shell::Elvish),
            "fish" => Ok(Shell::Fish),
            "nushell" => Ok(Shell::Nushell),
            "powershell" => Ok(Shell::PowerShell),
            "zsh" => Ok(Shell::Zsh),
            _ => Err(format!("Invalid shell: {}", s)),
        }
    }
}

pub fn generate_completion(shell: Shell, cmd: &mut Command) {
    use clap_complete::Shell as ClapShell;

    let name = cmd.get_name().to_string();
    match shell {
        Shell::Bash => generate(ClapShell::Bash, cmd, name, &mut io::stdout()),
        Shell::Elvish => generate(ClapShell::Elvish, cmd, name, &mut io::stdout()),
        Shell::Fish => generate(ClapShell::Fish, cmd, name, &mut io::stdout()),
        Shell::PowerShell => generate(ClapShell::PowerShell, cmd, name, &mut io::stdout()),
        Shell::Zsh => generate(ClapShell::Zsh, cmd, name, &mut io::stdout()),

        Shell::Nushell => {
            generate(
                clap_complete_nushell::Nushell {},
                cmd,
                name,
                &mut io::stdout(),
            );
        }
    }
}

async fn list_events(aim: &Aim, conds: &EventConditions) -> Result<(), Box<dyn Error>> {
    const MAX: i64 = 16;
    let pager: Pager = (MAX, 0).into();
    let events = aim.list_events(&conds, &pager).await?;
    if events.len() == (MAX as usize) {
        let total = aim.count_events(&conds).await?;
        if total > MAX {
            println!("Displaying the {}/{} events", total, MAX);
        }
    }

    let formatter = EventFormatter::new(conds.now);
    formatter.write_to(&mut io::stdout(), &events)?;
    Ok(())
}

async fn list_todos(aim: &Aim, conds: &TodoConditions) -> Result<(), Box<dyn Error>> {
    const MAX: i64 = 16;
    let pager = (MAX, 0).into();
    let sort = vec![
        (TodoSortKey::Priority, SortOrder::Desc).into(),
        (TodoSortKey::Due, SortOrder::Desc).into(),
    ];
    let todos = aim.list_todos(&conds, &sort, &pager).await?;
    if todos.len() == (MAX as usize) {
        let total = aim.count_todos(&conds).await?;
        if total > MAX {
            println!("Displaying the {}/{} todos", total, MAX);
        }
    }

    let formatter = TodoFormatter::new(conds.now);
    formatter.write(&mut io::stdout(), &todos)?;
    Ok(())
}
