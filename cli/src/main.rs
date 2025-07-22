// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aim_cli::{Cli, Commands, cmd_dashboard, cmd_done, cmd_events, cmd_todos, cmd_undo};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Dashboard => cmd_dashboard(cli.config).await?,
        Commands::Events(args) => cmd_events(cli.config, &args).await?,
        Commands::Todos(args) => cmd_todos(cli.config, &args).await?,
        Commands::Done { uid_or_short_id } => cmd_done(cli.config, &uid_or_short_id).await?,
        Commands::Undo { uid_or_short_id } => cmd_undo(cli.config, &uid_or_short_id).await?,
    }
    Ok(())
}
