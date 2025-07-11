// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod todo_formatter;

use crate::todo_formatter::TodoFormatter;
use aim_core::{
    Aim, Config, Event, EventQuery, Pager, SortOrder, TodoQuery, TodoSortKey, TodoStatus,
};
use chrono::{Duration, Local};
use clap::Parser;
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
        Some(Commands::Events) => list_events(&aim).await?,
        Some(Commands::Todos) => list_todos(&aim).await?,
        None => {
            println!("--- Events ---");
            list_events(&aim).await?;

            println!("\n--- Todos ---");
            list_todos(&aim).await?;
        }
    }

    Ok(())
}

async fn list_events(aim: &Aim) -> Result<(), Box<dyn Error>> {
    log::debug!("Listing events...");
    const MAX: i64 = 100;

    let query = EventQuery::new();
    if aim.count_events(&query).await? > MAX {
        println!("Displaying only the first {} todos", MAX);
    }

    let pager: Pager = (MAX, 0).into();
    let mut events = aim.list_events(&query, &pager).await?;
    events.reverse();
    for event in events {
        println!(
            "- Event #{}: {} (Starts: {})",
            event.id(),
            event.summary(),
            event.start_at().unwrap_or("N/A")
        );
    }

    Ok(())
}

pub async fn list_todos(aim: &Aim) -> Result<(), Box<dyn Error>> {
    log::debug!("Listing todos...");
    const MAX: i64 = 100;
    let now = Local::now().naive_local();

    let query = TodoQuery {
        now,
        status: Some(TodoStatus::NeedsAction),
        due: Some(Duration::days(2)),
    };

    let pager = (MAX, 0).into();
    let sort = vec![
        (TodoSortKey::Priority, SortOrder::Desc).into(),
        (TodoSortKey::Due, SortOrder::Desc).into(),
    ];
    let todos = aim.list_todos(&query, &sort, &pager).await?;

    if todos.len() == (MAX as usize) && aim.count_todos(&query).await? > MAX {
        println!("Displaying only the first {} todos", MAX);
    }
    let formatter = TodoFormatter::new(now);
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
