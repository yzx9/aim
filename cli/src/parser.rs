// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use aimcal_core::{Id, LooseDateTime};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, offset::LocalResult};
use clap::{Arg, ArgMatches, arg, value_parser};

pub fn arg_id() -> Arg {
    arg!(id: <ID> "The short id or uid of the todo to edit")
}

pub fn get_id(matches: &ArgMatches) -> Id {
    let id = matches
        .get_one::<String>("id")
        .expect("id is required")
        .clone();

    Id::ShortIdOrUid(id)
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

    pub fn from(matches: &ArgMatches) -> Self {
        matches
            .get_one("output-format")
            .copied()
            .unwrap_or(ArgOutputFormat::Table)
    }
}

pub fn parse_datetime(dt: &str) -> Result<Option<LooseDateTime>, &'static str> {
    if dt.is_empty() {
        Ok(None)
    } else if let Ok(dt) = NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M") {
        Ok(Some(match Local.from_local_datetime(&dt) {
            LocalResult::Single(dt) => dt.into(),
            LocalResult::Ambiguous(dt1, _) => {
                log::warn!("Ambiguous local time for {dt} in local, picking earliest");
                dt1.into()
            }
            LocalResult::None => {
                log::warn!("Invalid local time for {dt} in local, falling back to floating");
                dt.into()
            }
        }))
    } else if let Ok(time) = NaiveTime::parse_from_str(dt, "%H:%M") {
        // If the input is just a time, we assume it's today
        match Local::now().with_time(time) {
            LocalResult::Single(dt) => Ok(Some(dt.into())),
            LocalResult::Ambiguous(dt1, _) => {
                log::warn!("Ambiguous local time for {dt} in local, picking earliest");
                Ok(Some(dt1.into()))
            }
            LocalResult::None => Err("Invalid local time"),
        }
    } else if let Ok(date) = NaiveDate::parse_from_str(dt, "%Y-%m-%d") {
        Ok(Some(date.into()))
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
