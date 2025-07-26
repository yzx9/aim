// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aimcal_core::Priority;
use clap::{Arg, ArgMatches, arg, value_parser};

#[derive(Debug, Clone, Copy, clap::ValueEnum, serde::Deserialize)]
pub enum ParsedPriority {
    #[clap(name = "none", aliases = ["n", "0"])]
    #[serde(rename = "none", alias = "n", alias = "0")]
    None,

    #[clap(name = "1", hide = true)]
    #[serde(rename = "1")]
    P1,

    #[clap(name = "high", aliases = ["h" ,"2"])]
    #[serde(rename = "2", alias = "high", alias = "h")]
    P2,

    #[clap(name = "3", hide = true)]
    #[serde(rename = "3")]
    P3,

    #[clap(name = "4", hide = true)]
    #[serde(rename = "4")]
    P4,

    #[clap(name = "middle", aliases = ["mid", "m", "5"])]
    #[serde(rename = "5", alias = "middle", alias = "mid", alias = "m")]
    P5,

    #[clap(name = "6", hide = true)]
    #[serde(rename = "6")]
    P6,

    #[clap(name = "7", hide = true)]
    #[serde(rename = "7")]
    P7,

    #[clap(name = "low", aliases = ["l", "8"])]
    #[serde(rename = "8", alias = "low", alias = "l")]
    P8,

    #[clap(name = "9", hide = true)]
    #[serde(rename = "9")]
    P9,
}

impl ParsedPriority {
    pub fn arg() -> Arg {
        arg!(-p --priority <PRIORITY> "Priority of the todo")
            .value_parser(value_parser!(ParsedPriority))
    }

    pub fn arg_parse(matches: &ArgMatches) -> Option<&Self> {
        matches.get_one::<ParsedPriority>("priority")
    }
}

impl From<&ParsedPriority> for Priority {
    fn from(priority: &ParsedPriority) -> Self {
        match priority {
            ParsedPriority::None => Priority::None,
            ParsedPriority::P1 => Priority::P1,
            ParsedPriority::P2 => Priority::P2,
            ParsedPriority::P3 => Priority::P3,
            ParsedPriority::P4 => Priority::P4,
            ParsedPriority::P5 => Priority::P5,
            ParsedPriority::P6 => Priority::P6,
            ParsedPriority::P7 => Priority::P7,
            ParsedPriority::P8 => Priority::P8,
            ParsedPriority::P9 => Priority::P9,
        }
    }
}
