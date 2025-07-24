// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::Cli;
use clap::{ArgMatches, Command, ValueEnum, arg, value_parser};
use clap_complete::generate;
use std::io;

#[derive(Debug, Clone, Copy)]
pub struct CmdGenerateCompletion {
    shell: Shell,
}

impl CmdGenerateCompletion {
    pub fn command() -> Command {
        Command::new("generate-completion")
            .about("Generate shell completion for the specified shell")
            .hide(true)
            .arg(
                arg!(shell: <SHELL> "The shell generator to use")
                    .value_parser(value_parser!(Shell)),
            )
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        match matches.get_one::<Shell>("shell") {
            Some(shell) => Self { shell: *shell },
            _ => unreachable!(),
        }
    }

    pub fn run(self) {
        use clap_complete::Shell as ClapShell;

        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        match self.shell {
            Shell::Bash => generate(ClapShell::Bash, &mut cmd, name, &mut io::stdout()),
            Shell::Elvish => generate(ClapShell::Elvish, &mut cmd, name, &mut io::stdout()),
            Shell::Fish => generate(ClapShell::Fish, &mut cmd, name, &mut io::stdout()),
            Shell::PowerShell => generate(ClapShell::PowerShell, &mut cmd, name, &mut io::stdout()),
            Shell::Zsh => generate(ClapShell::Zsh, &mut cmd, name, &mut io::stdout()),
            Shell::Nushell => generate(
                clap_complete_nushell::Nushell {},
                &mut cmd,
                name,
                &mut io::stdout(),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    #[clap(name = "powershell")]
    #[allow(clippy::enum_variant_names)]
    PowerShell,
    Zsh,
}
