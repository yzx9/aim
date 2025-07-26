// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::short_id::ShortIdMap;
use aimcal_core::LooseDateTime;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, offset::LocalResult};
use clap::{Arg, ArgMatches, arg, value_parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgUidOrShortId(String);

impl ArgUidOrShortId {
    pub fn arg() -> Arg {
        arg!(id: <ID> "The short id or uid of the todo to edit")
            .value_parser(value_parser!(ArgUidOrShortId))
    }

    pub fn parse(matches: &ArgMatches) -> ArgUidOrShortId {
        matches
            .get_one::<ArgUidOrShortId>("id")
            .expect("id is required")
            .clone()
    }

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

/// The output format for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ArgOutputFormat {
    Json,
    Table,
}

impl ArgOutputFormat {
    pub fn arg() -> Arg {
        arg!(--"output-format" <FORMAT> "Output format")
            .value_parser(value_parser!(ArgOutputFormat))
            .default_value("table")
    }

    pub fn parse(matches: &ArgMatches) -> Self {
        matches
            .get_one("output-format")
            .copied()
            .unwrap_or(ArgOutputFormat::Table)
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

pub fn format_datetime(t: LooseDateTime) -> String {
    match t {
        LooseDateTime::DateOnly(d) => d.format("%Y-%m-%d"),
        LooseDateTime::Floating(dt) => dt.format("%Y-%m-%d %H:%M"),
        LooseDateTime::Local(dt) => dt.format("%Y-%m-%d %H:%M"),
    }
    .to_string()
}
