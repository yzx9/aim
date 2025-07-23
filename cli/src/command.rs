// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli::{OutputArgs, OutputFormat, TodoEditArgs},
    config::Config,
    event_formatter::EventFormatter,
    short_id::{EventWithShortId, ShortIdMap, TodoWithShortId},
    todo_formatter::TodoFormatter,
};
use aimcal_core::{
    Aim, EventConditions, Pager, SortOrder, TodoConditions, TodoPatch, TodoSort, TodoStatus,
};
use chrono::{Duration, Local, Utc};
use colored::Colorize;
use std::{error::Error, path::PathBuf};

/// Show the dashboard with events and todos.
pub async fn command_dashboard(config: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    log::debug!("Parsing configuration...");
    let config = Config::parse(config).await?;
    let aim = Aim::new(&config.core).await?;
    let map = ShortIdMap::load_or_new(&config)?;

    log::debug!("Generating dashboard...");
    let now = Local::now().naive_local();

    println!("🗓️ {}", "Events".bold());
    let conds = EventConditions { now };
    let args = OutputArgs {
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
    let args = OutputArgs {
        output_format: OutputFormat::Table,
    };
    list_todos(&aim, &map, &conds, &args).await?;

    map.dump(&config)?;
    Ok(())
}

/// List all events.
pub async fn command_events(
    config: Option<PathBuf>,
    args: &OutputArgs,
) -> Result<(), Box<dyn Error>> {
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

/// List all todos.
pub async fn command_todos(
    config: Option<PathBuf>,
    args: &OutputArgs,
) -> Result<(), Box<dyn Error>> {
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

/// Mark a todo as done.
pub async fn command_done(
    config: Option<PathBuf>,
    args: &TodoEditArgs,
) -> Result<(), Box<dyn Error>> {
    log::debug!("Marking todo as done...");
    let patch = TodoPatch {
        completed: Some(Some(Utc::now().into())),
        status: Some(TodoStatus::Completed),
        ..Default::default()
    };
    edit_todo(config, args, patch).await
}

/// Mark a todo as undone.
pub async fn command_undo(
    config: Option<PathBuf>,
    args: &TodoEditArgs,
) -> Result<(), Box<dyn Error>> {
    log::debug!("Marking todo as undone...");
    let patch = TodoPatch {
        completed: Some(None),
        status: Some(TodoStatus::NeedsAction),
        ..Default::default()
    };
    edit_todo(config, args, patch).await
}

/// List events with the given conditions and output format.
async fn list_events(
    aim: &Aim,
    map: &ShortIdMap,
    conds: &EventConditions,
    args: &OutputArgs,
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

    let events: Vec<_> = events
        .into_iter()
        .map(|event| EventWithShortId::with(map, event))
        .collect();

    let formatter = EventFormatter::new(conds.now).with_output_format(args.output_format);
    println!("{}", formatter.format(&events));
    Ok(())
}

/// List todos with the given conditions and output format.
async fn list_todos(
    aim: &Aim,
    map: &ShortIdMap,
    conds: &TodoConditions,
    args: &OutputArgs,
) -> Result<(), Box<dyn Error>> {
    const MAX: i64 = 16;
    let pager = (MAX, 0).into();
    let sort = vec![
        TodoSort::Priority {
            order: SortOrder::Desc,
            none_first: false, // TODO: add config entry
        },
        TodoSort::Due(SortOrder::Desc),
    ];
    let todos = aim.list_todos(conds, &sort, &pager).await?;
    if todos.len() >= (MAX as usize) {
        let total = aim.count_todos(conds).await?;
        if total > MAX {
            println!("Displaying the {total}/{MAX} todos");
        }
    }

    let todos: Vec<_> = todos
        .into_iter()
        .map(|todo| TodoWithShortId::with(map, todo))
        .collect();

    let formatter = TodoFormatter::new(conds.now).with_output_format(args.output_format);
    println!("{}", formatter.format(&todos));
    Ok(())
}

async fn edit_todo(
    config: Option<PathBuf>,
    args: &TodoEditArgs,
    mut patch: TodoPatch,
) -> Result<(), Box<dyn Error>> {
    let now = Local::now().naive_local();

    log::debug!("Parsing configuration...");
    let config = Config::parse(config).await?;
    let aim = Aim::new(&config.core).await?;
    let map = ShortIdMap::load_or_new(&config)?;

    log::debug!("Edit todo ...");
    patch.uid = args
        .uid_or_short_id
        .parse()
        .ok()
        .and_then(|a| map.find(a))
        .unwrap_or_else(|| args.uid_or_short_id.to_string()); // treat it as a UID if is not a short ID
    let todo = aim.upsert_todo(patch.clone()).await?;
    let todo = TodoWithShortId::with(&map, todo);

    let formatter = TodoFormatter::new(now).with_output_format(args.output_format);
    println!("{}", formatter.format(&[todo]));
    Ok(())
}
