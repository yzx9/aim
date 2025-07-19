// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aim_cli::{Cli, Commands, cmd_dashboard, cmd_events, cmd_todos};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Dashboard => cmd_dashboard(cli.config).await?,
        Commands::Events(args) => cmd_events(cli.config, &args).await?,
        Commands::Todos(args) => cmd_todos(cli.config, &args).await?,
    }
    Ok(())
}
