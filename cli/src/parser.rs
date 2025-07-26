// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::short_id::ShortIdMap;
use aimcal_core::{LooseDateTime, Priority};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, offset::LocalResult};
use clap::{Arg, ArgMatches, arg, value_parser};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgUidOrShortId(String);

impl ArgUidOrShortId {
    /// Get the ID of the todo, either by parsing the UID or looking up the short ID.
    pub fn get_id(self, map: &ShortIdMap) -> String {
        self.0
            .parse()
            .ok()
            .and_then(|a| map.find(a))
            .unwrap_or_else(|| self.0.to_string())
    }
}

impl From<&str> for ArgUidOrShortId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl FromStr for ArgUidOrShortId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("ID cannot be empty")
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

pub fn parse_datetime(dt: &str) -> Result<Option<LooseDateTime>, &str> {
    if dt.is_empty() {
        Ok(None)
    } else if let Ok(dt) = NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M") {
        Ok(Some(match Local.from_local_datetime(&dt) {
            LocalResult::Single(dt) => LooseDateTime::Local(dt),
            LocalResult::Ambiguous(dt1, _) => {
                log::warn!("Ambiguous local time for {dt} in local, picking earliest");
                LooseDateTime::Local(dt1)
            }
            LocalResult::None => {
                log::warn!("Invalid local time for {dt} in local, falling back to floating");
                LooseDateTime::Floating(dt)
            }
        }))
    } else if let Ok(time) = NaiveTime::parse_from_str(dt, "%H:%M") {
        // If the input is just a time, we assume it's today
        match Local::now().with_time(time) {
            LocalResult::Single(dt) => Ok(Some(LooseDateTime::Local(dt))),
            LocalResult::Ambiguous(dt1, _) => {
                log::warn!("Ambiguous local time for {dt} in local, picking earliest");
                Ok(Some(LooseDateTime::Local(dt1)))
            }
            LocalResult::None => Err("Invalid local time"),
        }
    } else if let Ok(date) = NaiveDate::parse_from_str(dt, "%Y-%m-%d") {
        Ok(Some(LooseDateTime::DateOnly(date)))
    } else {
        Err("Invalid date format. Expected format: YYYY-MM-DD, HH:MM and YYYY-MM-DD HH:MM")
    }
}

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

impl From<ParsedPriority> for Priority {
    fn from(priority: ParsedPriority) -> Self {
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

impl From<&ParsedPriority> for Priority {
    fn from(priority: &ParsedPriority) -> Self {
        (*priority).into()
    }
}

impl From<Priority> for ParsedPriority {
    fn from(priority: Priority) -> Self {
        match priority {
            Priority::None => ParsedPriority::None,
            Priority::P1 => ParsedPriority::P1,
            Priority::P2 => ParsedPriority::P2,
            Priority::P3 => ParsedPriority::P3,
            Priority::P4 => ParsedPriority::P4,
            Priority::P5 => ParsedPriority::P5,
            Priority::P6 => ParsedPriority::P6,
            Priority::P7 => ParsedPriority::P7,
            Priority::P8 => ParsedPriority::P8,
            Priority::P9 => ParsedPriority::P9,
        }
    }
}

impl From<&Priority> for ParsedPriority {
    fn from(priority: &Priority) -> Self {
        (*priority).into()
    }
}

impl fmt::Display for ParsedPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ParsedPriority::None => "none",
            ParsedPriority::P1 => "1",
            ParsedPriority::P2 => "2",
            ParsedPriority::P3 => "3",
            ParsedPriority::P4 => "4",
            ParsedPriority::P5 => "3",
            ParsedPriority::P6 => "6",
            ParsedPriority::P7 => "7",
            ParsedPriority::P8 => "8",
            ParsedPriority::P9 => "9",
        };
        write!(f, "{s}")
    }
}
