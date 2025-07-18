// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod cli;
mod config;
mod event_formatter;
mod table;
mod todo_formatter;

pub use crate::cli::{Cli, Commands};
use crate::{
    cli::{ListArgs, OutputFormat},
    config::parse_config,
    event_formatter::EventFormatter,
    todo_formatter::TodoFormatter,
};
use aim_core::{Aim, EventConditions, Pager, SortOrder, TodoConditions, TodoSortKey, TodoStatus};
use chrono::{Duration, Local};
use colored::Colorize;
use std::{error::Error, path::PathBuf};

pub async fn cmd_events(config: Option<PathBuf>, args: &ListArgs) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = parse_config(config).await?;
    let aim = Aim::new(&config).await?;

    log::debug!("Listing events...");
    let now = Local::now().naive_local();
    let conds = EventConditions { now };
    list_events(&aim, &conds, args).await
}

pub async fn cmd_todos(config: Option<PathBuf>, args: &ListArgs) -> Result<(), Box<dyn Error>> {
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

pub async fn cmd_dashboard(config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
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
    let events = aim.list_events(conds, &pager).await?;
    if events.len() >= (MAX as usize) {
        let total = aim.count_events(conds).await?;
        if total > MAX {
            println!("Displaying the {total}/{MAX} events");
        }
    }

    let formatter = EventFormatter::new(conds.now).with_output_format(args.output_format);
    println!("{}", formatter.format(&events));
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
    let todos = aim.list_todos(conds, &sort, &pager).await?;
    if todos.len() >= (MAX as usize) {
        let total = aim.count_todos(conds).await?;
        if total > MAX {
            println!("Displaying the {total}/{MAX} todos");
        }
    }

    let formatter = TodoFormatter::new(conds.now).with_output_format(args.output_format);
    println!("{}", formatter.format(&todos));
    Ok(())
}
