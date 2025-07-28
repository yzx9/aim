// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event_formatter::EventFormatter,
    parser::ArgOutputFormat,
    short_id::{EventWithShortId, ShortIdMap},
};
use aimcal_core::{Aim, EventConditions, Pager};
use clap::{ArgMatches, Command};
use colored::Colorize;
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct CmdEventList {
    pub conds: EventConditions,
    pub output_format: ArgOutputFormat,
}

impl CmdEventList {
    pub const NAME: &str = "list";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("List events")
            .arg(ArgOutputFormat::arg())
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        Self {
            conds: EventConditions { startable: true },
            output_format: ArgOutputFormat::parse(matches),
        }
    }

    pub async fn run(self, aim: &Aim, map: &ShortIdMap) -> Result<(), Box<dyn Error>> {
        log::debug!("Listing events...");
        Self::list(aim, map, &self.conds, self.output_format).await
    }

    /// List events with the given conditions and output format.
    pub async fn list(
        aim: &Aim,
        map: &ShortIdMap,
        conds: &EventConditions,
        output_format: ArgOutputFormat,
    ) -> Result<(), Box<dyn Error>> {
        const MAX: i64 = 16;
        let pager: Pager = (MAX, 0).into();
        let events = aim.list_events(conds, &pager).await?;
        if events.len() >= (MAX as usize) {
            let total = aim.count_events(conds).await?;
            if total > MAX {
                println!("Displaying the {total}/{MAX} events");
            }
        } else if events.is_empty() && output_format == ArgOutputFormat::Table {
            println!("{}", "No events found".italic());
            return Ok(());
        }

        let events: Vec<_> = events
            .into_iter()
            .map(|event| EventWithShortId::with(map, event))
            .collect();

        let formatter = EventFormatter::new(aim.now()).with_output_format(output_format);
        println!("{}", formatter.format(&events));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    #[test]
    fn test_parse_event_list() {
        let cmd = Command::new("test")
            .subcommand_required(true)
            .subcommand(CmdEventList::command());

        let matches = cmd
            .try_get_matches_from(["test", "list", "--output-format", "json"])
            .unwrap();

        let sub_matches = matches.subcommand_matches("list").unwrap();
        let parsed = CmdEventList::parse(sub_matches);
        assert_eq!(parsed.output_format, ArgOutputFormat::Json);
    }
}
