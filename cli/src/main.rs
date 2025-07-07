use aim_core::{Calendar, Event, Todo};
use clap::Parser;

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
    List {
        /// Path to the directory containing .ics files
        path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::List { path } => {
            log::debug!("Scanning directory: {}", path);

            let db = Calendar::new(path).await?;

            // Query and print results to verify
            println!("\n--- {} events found ---", db.count_events().await?);
            for event in db.list_events().await? {
                print_event(&event);
            }

            println!("\n--- {} todos found ---", db.count_todos().await?);
            for todo in db.list_todos().await? {
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
