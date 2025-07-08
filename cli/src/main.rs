use aim_core::{Aim, Event, EventQuery, Pager, Todo, TodoQuery, TodoStatus};
use chrono::Duration;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aim")]
#[command(about = "An Information Management tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// List events from a directory of .ics files
    Events,

    /// List todos from a directory of .ics files
    Todos,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let config = parse_config().await?;
    let aim = Aim::new(&config).await?;

    match cli.command {
        Some(Commands::Events) => print_events(&aim).await?,
        Some(Commands::Todos) => print_todos(&aim).await?,
        None => {
            println!("--- Events ---");
            print_events(&aim).await?;

            println!("\n--- Todos ---");
            print_todos(&aim).await?;
        }
    }

    Ok(())
}

async fn print_events(aim: &Aim) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Listing events...");
    const MAX: i64 = 100;

    let query = EventQuery::new();
    if aim.count_events(&query).await? > MAX {
        println!("Displaying only the first {} todos", MAX);
    }

    let pager: Pager = (MAX, 0).into();
    let mut todos = aim.list_events(&query, &pager).await?;
    todos.reverse();
    for event in todos {
        println!(
            "- Event #{}: {} (Starts: {})",
            event.id(),
            event.summary(),
            event.start_at().unwrap_or("N/A")
        );
    }

    Ok(())
}

async fn print_todos(aim: &Aim) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Listing todos...");
    const MAX: i64 = 100;

    let query = TodoQuery::new()
        .with_status(TodoStatus::NeedsAction)
        .with_due(Duration::days(2));
    if aim.count_todos(&query).await? > MAX {
        println!("Displaying only the first {} todos", MAX);
    }

    let pager: Pager = (MAX, 0).into();
    let mut todos = aim.list_todos(&query, &pager).await?;
    todos.reverse();
    for todo in todos {
        println!(
            "[ ] {} {} {} ",
            todo.id(),
            todo.due_at().unwrap_or("N/A"),
            todo.summary(),
        );
    }

    Ok(())
}

async fn parse_config() -> Result<aim_core::Config, Box<dyn std::error::Error>> {
    let path = xdg::BaseDirectories::with_prefix("aim")
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

    Ok(aim_core::Config::new(calendar_path))
}

fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let home = home::home_dir().expect("Failed to get home directory");
        home.join(&path[2..])
    } else {
        path.into()
    }
}
