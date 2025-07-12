// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod event_formatter;
mod table;
mod todo_formatter;

use crate::{event_formatter::EventFormatter, todo_formatter::TodoFormatter};
use aim_core::{
    Aim, Config, EventConditions, Pager, SortOrder, TodoConditions, TodoSortKey, TodoStatus,
};
use chrono::{Duration, Local};
use clap::Parser;
use colored::Colorize;
use std::{error::Error, io, path::PathBuf};
use xdg::BaseDirectories;

#[derive(Parser)]
#[command(name = "aim")]
#[command(about = "An Information Management tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// List events
    Events,

    /// List todos
    Todos,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let config = parse_config().await?;
    let aim = Aim::new(&config).await?;

    match cli.command {
        Some(Commands::Events) => events(&aim).await?,
        Some(Commands::Todos) => todos(&aim).await?,
        None => dashboard(&aim).await?,
    }

    Ok(())
}

pub async fn events(aim: &Aim) -> Result<(), Box<dyn Error>> {
    log::debug!("Listing events...");
    let now = Local::now().naive_local();
    let conds = EventConditions { now };
    list_events(aim, &conds).await
}

pub async fn todos(aim: &Aim) -> Result<(), Box<dyn Error>> {
    log::debug!("Listing todos...");
    let now = Local::now().naive_local();
    let conds = TodoConditions {
        now,
        status: Some(TodoStatus::NeedsAction),
        due: Some(Duration::days(2)),
    };
    list_todos(aim, &conds).await
}

pub async fn dashboard(aim: &Aim) -> Result<(), Box<dyn Error>> {
    log::debug!("Generating dashboard...");

    println!("🗓️ {}", "Events".bold());
    let now = Local::now().naive_local();
    let conds = EventConditions { now };
    list_events(aim, &conds).await?;

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

async fn parse_config() -> Result<Config, Box<dyn Error>> {
    let path = BaseDirectories::with_prefix("aim")
        .get_config_home()
        .ok_or("Failed to get user-specific config directory")?;

    let config = path.join("config.toml");
    if !config.exists() {
        return Err(format!("No config found at: {}", config.display()).into());
    }

    let content = tokio::fs::read_to_string(path.join("config.toml"))
        .await
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let table: toml::Table = content
        .parse()
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let calendar_path = table
        .get("calendar_path")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid 'calendar_path' in config")?;

    let calendar_path = expand_path(calendar_path);

    Ok(Config::new(calendar_path))
}

fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = home::home_dir() {
            return home.join(&path[2..]);
        }

        log::warn!("Home directory not found");
    }

    path.into()
}
