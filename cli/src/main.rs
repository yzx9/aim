// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aim_cli::{
    Cli, Commands, command_dashboard, command_done, command_events, command_todos, command_undo,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Dashboard => command_dashboard(cli.config).await?,
        Commands::Events(args) => command_events(cli.config, &args).await?,
        Commands::Todos(args) => command_todos(cli.config, &args).await?,
        Commands::Done { uid_or_short_id } => command_done(cli.config, &uid_or_short_id).await?,
        Commands::Undo { uid_or_short_id } => command_undo(cli.config, &uid_or_short_id).await?,
    }
    Ok(())
}
