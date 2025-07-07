use aim_core::{Aim, Event, Todo};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aim")]
#[command(about = "An Information Management tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// List events and todos from a directory of .ics files
    List,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();
    let config = parse_config().await?;

    match &cli.command {
        Commands::List => {
            let calendar = Aim::new(&config).await?;

            // Query and print results to verify
            println!("\n--- {} events found ---", calendar.count_events().await?);
            for event in calendar.list_events().await? {
                print_event(&event);
            }

            println!("\n--- {} todos found ---", calendar.count_todos().await?);
            for todo in calendar.list_todos().await? {
                print_todo(&todo);
            }
        }
    }

    Ok(())
}

fn print_event<E: Event>(event: &E) {
    println!(
        "- Event #{}: {} (Starts: {})",
        event.id(),
        event.summary(),
        event.start_at().unwrap_or("N/A")
    )
}

fn print_todo<T: Todo>(todo: &T) {
    println!(
        "- Todo #{}: {} (Due: {})",
        todo.id(),
        todo.summary(),
        todo.due_at().unwrap_or("N/A")
    )
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
