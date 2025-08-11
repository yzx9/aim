// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{error::Error, io};

use clap::{ArgMatches, Command, ValueEnum, arg, value_parser};
use clap_complete::generate;

use crate::Cli;

#[derive(Debug, Clone, Copy)]
pub struct CmdGenerateCompletion {
    pub shell: Shell,
}

impl CmdGenerateCompletion {
    pub const NAME: &str = "generate-completion";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Generate shell completion for the specified shell")
            .hide(true)
            .arg(
                arg!(shell: <SHELL> "The shell generator to use")
                    .value_parser(value_parser!(Shell)),
            )
    }

    pub fn from(matches: &ArgMatches) -> Self {
        match matches.get_one::<Shell>("shell") {
            Some(shell) => Self { shell: *shell },
            _ => unreachable!(),
        }
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "generating shell completion...");
        self.generate(&mut io::stdout());
        Ok(())
    }

    pub fn generate(self, buf: &mut impl io::Write) {
        use clap_complete::Shell as ClapShell;

        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        match self.shell {
            Shell::Bash => generate(ClapShell::Bash, &mut cmd, name, buf),
            Shell::Elvish => generate(ClapShell::Elvish, &mut cmd, name, buf),
            Shell::Fish => generate(ClapShell::Fish, &mut cmd, name, buf),
            Shell::PowerShell => generate(ClapShell::PowerShell, &mut cmd, name, buf),
            Shell::Zsh => generate(ClapShell::Zsh, &mut cmd, name, buf),
            Shell::Nushell => generate(clap_complete_nushell::Nushell {}, &mut cmd, name, buf),
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    #[test]
    fn test_parse_generate_completion() {
        let cmd = Command::new("test").subcommand(CmdGenerateCompletion::command());

        let matches = cmd
            .try_get_matches_from(["aim", "generate-completion", "bash"])
            .unwrap();

        let sub_matches = matches.subcommand_matches("generate-completion").unwrap();
        let parsed = CmdGenerateCompletion::from(sub_matches);
        assert_eq!(parsed.shell, Shell::Bash);

        let mut output = vec![];
        parsed.generate(&mut output);
        assert!(!output.is_empty())
    }

    #[test]
    fn test_parse_shell_variants() {
        fn test_shell_parsing(shell_str: &str, expected_shell: Shell) {
            let cmd = Cli::command();
            let matches = cmd
                .try_get_matches_from(["aim", "generate-completion", shell_str])
                .unwrap_or_else(|e| panic!("Failed to parse for shell '{shell_str}': {e}"));
            let sub_matches = matches.subcommand_matches("generate-completion").unwrap();
            let parsed = CmdGenerateCompletion::from(sub_matches);
            assert_eq!(parsed.shell, expected_shell);
        }

        test_shell_parsing("bash", Shell::Bash);
        test_shell_parsing("elvish", Shell::Elvish);
        test_shell_parsing("fish", Shell::Fish);
        test_shell_parsing("nushell", Shell::Nushell);
        test_shell_parsing("powershell", Shell::PowerShell);
        test_shell_parsing("zsh", Shell::Zsh);
    }
}
