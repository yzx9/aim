// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod cli;
mod config;
mod event_formatter;
mod short_id;
mod table;
mod todo_formatter;

pub use crate::{
    cli::{Cli, Commands},
    config::Config,
};
use crate::{
    cli::{ListArgs, OutputFormat},
    event_formatter::EventFormatter,
    short_id::{EventWithShortId, ShortIdMap, TodoWithShortId},
    todo_formatter::TodoFormatter,
};
use aim_core::{
    Aim, EventConditions, Pager, SortOrder, Todo, TodoConditions, TodoPatch, TodoSortKey,
    TodoStatus,
};
use chrono::{Duration, Local, Utc};
use colored::Colorize;
use std::{error::Error, path::PathBuf};

pub async fn cmd_dashboard(config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = Config::parse(config).await?;
    let aim = Aim::new(&config.core).await?;
    let map = ShortIdMap::load_or_new(&config)?;

    log::debug!("Generating dashboard...");
    let now = Local::now().naive_local();

    println!("🗓️ {}", "Events".bold());
    let conds = EventConditions { now };
    let args = ListArgs {
        output_format: OutputFormat::Table,
    };
    list_events(&aim, &map, &conds, &args).await?;
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
    list_todos(&aim, &map, &conds, &args).await?;

    map.dump(&config)?;
    Ok(())
}

pub async fn cmd_events(config: Option<PathBuf>, args: &ListArgs) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = Config::parse(config).await?;
    let aim = Aim::new(&config.core).await?;
    let map = ShortIdMap::load_or_new(&config)?;

    log::debug!("Listing events...");
    let now = Local::now().naive_local();
    let conds = EventConditions { now };
    list_events(&aim, &map, &conds, args).await?;

    map.dump(&config)?;
    Ok(())
}

pub async fn cmd_todos(config: Option<PathBuf>, args: &ListArgs) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = Config::parse(config).await?;
    let aim = Aim::new(&config.core).await?;
    let map = ShortIdMap::load_or_new(&config)?;

    log::debug!("Listing todos...");
    let now = Local::now().naive_local();
    let conds = TodoConditions {
        now,
        status: Some(TodoStatus::NeedsAction),
        due: Some(Duration::days(2)),
    };
    list_todos(&aim, &map, &conds, args).await?;

    map.dump(&config)?;
    Ok(())
}

pub async fn cmd_done(
    config: Option<PathBuf>,
    uid_or_short_id: &str,
) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = Config::parse(config).await?;
    let aim = Aim::new(&config.core).await?;
    let map = ShortIdMap::load_or_new(&config)?;

    log::debug!("Marking todo as done...");
    let patch = TodoPatch {
        uid: get_uid(&map, uid_or_short_id),
        completed: Some(Some(Utc::now().into())),
        status: Some(TodoStatus::Completed),
        ..Default::default()
    };
    let todo = aim.upsert_todo(patch.clone()).await?;
    print_todo(&map, todo);
    Ok(())
}

pub async fn cmd_undo(
    config: Option<PathBuf>,
    uid_or_short_id: &str,
) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = Config::parse(config).await?;
    let aim = Aim::new(&config.core).await?;
    let map = ShortIdMap::load_or_new(&config)?;

    log::debug!("Marking todo as undone...");
    let patch = TodoPatch {
        uid: get_uid(&map, uid_or_short_id),
        completed: Some(None),
        status: Some(TodoStatus::NeedsAction),
        ..Default::default()
    };
    let todo = aim.upsert_todo(patch.clone()).await?;
    print_todo(&map, todo);
    Ok(())
}

async fn list_events(
    aim: &Aim,
    map: &ShortIdMap,
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

    let events = events
        .into_iter()
        .map(|event| EventWithShortId::with(map, event))
        .collect::<Vec<_>>();

    let formatter = EventFormatter::new(conds.now).with_output_format(args.output_format);
    println!("{}", formatter.format(&events));
    Ok(())
}

async fn list_todos(
    aim: &Aim,
    map: &ShortIdMap,
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

    let todos = todos
        .into_iter()
        .map(|todo| TodoWithShortId::with(map, todo))
        .collect::<Vec<_>>();

    let formatter = TodoFormatter::new(conds.now).with_output_format(args.output_format);
    println!("{}", formatter.format(&todos));
    Ok(())
}

fn get_uid(map: &ShortIdMap, uid_or_short_id: &str) -> String {
    uid_or_short_id
        .parse()
        .ok()
        .and_then(|a| map.find(a))
        .unwrap_or_else(|| uid_or_short_id.to_string()) // treat it as a UID if is not a short ID
}

fn print_todo(map: &ShortIdMap, todo: impl Todo) {
    let todo = TodoWithShortId::with(map, todo);
    let formatter =
        TodoFormatter::new(Local::now().naive_local()).with_output_format(OutputFormat::Json);
    println!("{}", formatter.format(&vec![todo]));
}
