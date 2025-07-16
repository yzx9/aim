// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod cli;
mod config;
mod event_formatter;
mod table;
mod todo_formatter;

use crate::{
    cli::{Cli, Commands, ListArgs, OutputFormat},
    config::parse_config,
    event_formatter::EventFormatter,
    todo_formatter::TodoFormatter,
};
use aim_core::{Aim, EventConditions, Pager, SortOrder, TodoConditions, TodoSortKey, TodoStatus};
use chrono::{Duration, Local};
use colored::Colorize;
use std::{error::Error, io, path::PathBuf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Events(args) => events(cli.config, &args).await?,
        Commands::Todos(args) => todos(cli.config, &args).await?,
        Commands::Dashboard => dashboard(cli.config).await?,
    }
    Ok(())
}

pub async fn events(config: Option<PathBuf>, args: &ListArgs) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = parse_config(config).await?;
    let aim = Aim::new(&config).await?;

    log::debug!("Listing events...");
    let now = Local::now().naive_local();
    let conds = EventConditions { now };
    list_events(&aim, &conds, args).await
}

pub async fn todos(config: Option<PathBuf>, args: &ListArgs) -> Result<(), Box<dyn Error>> {
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
    list_todos(&aim, &conds, args).await
}

pub async fn dashboard(config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = parse_config(config).await?;
    let aim = Aim::new(&config).await?;

    log::debug!("Generating dashboard...");
    let now = Local::now().naive_local();

    println!("🗓️ {}", "Events".bold());
    let conds = EventConditions { now };
    let args = ListArgs {
        output_format: OutputFormat::Table,
    };
    list_events(&aim, &conds, &args).await?;
    println!();

    println!("✅ {}", "Todos".bold());
    let conds = TodoConditions {
        now,
        status: Some(TodoStatus::NeedsAction),
        due: Some(Duration::days(2)),
    };
    let args = ListArgs {
        output_format: OutputFormat::Table,
    };
    list_todos(&aim, &conds, &args).await?;

    Ok(())
}

async fn list_events(
    aim: &Aim,
    conds: &EventConditions,
    args: &ListArgs,
) -> Result<(), Box<dyn Error>> {
    const MAX: i64 = 16;
    let pager: Pager = (MAX, 0).into();
    let events = aim.list_events(&conds, &pager).await?;
    if events.len() == (MAX as usize) {
        let total = aim.count_events(&conds).await?;
        if total > MAX {
            println!("Displaying the {}/{} events", total, MAX);
        }
    }

    let formatter = EventFormatter::new(conds.now).with_format(args.output_format);
    formatter.write_to(&mut io::stdout(), &events)?;
    Ok(())
}

async fn list_todos(
    aim: &Aim,
    conds: &TodoConditions,
    args: &ListArgs,
) -> Result<(), Box<dyn Error>> {
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

    let formatter = TodoFormatter::new(conds.now).with_format(args.output_format);
    formatter.write_to(&mut io::stdout(), &todos)?;
    Ok(())
}
