use aim_core::Database;
use clap::Parser;
use log::debug;

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
            debug!("Scanning directory: {}", path);

            let db = Database::new(path).await?;

            // Query and print results to verify
            println!("\n--- {} events found ---", db.count_events().await?);
            for event in db.list_events().await? {
                println!("- {}", event,);
            }

            println!("\n--- {} todos found ---", db.count_todos().await?);
            for todo in db.list_todos().await? {
                println!("- {}", todo);
            }
        }
    }

    Ok(())
}
