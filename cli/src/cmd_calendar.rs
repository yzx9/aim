// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::{borrow::Cow, fmt};

use aimcal_core::{Aim, CalendarDetails, CalendarRecord, CalendarStoreDetails};
use clap::{ArgMatches, Command, arg};

use crate::arg::CommonArgs;
use crate::table::{PaddingDirection, Table, TableColumn, TableStyleBasic, TableStyleJson};
use crate::util::OutputFormat;

#[derive(Debug, Clone, Copy)]
pub struct CmdCalendarList {
    pub output_format: OutputFormat,
}

impl CmdCalendarList {
    pub const NAME: &str = "list";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("List all calendars")
            .arg(CommonArgs::output_format())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            output_format: CommonArgs::get_output_format(matches),
        }
    }

    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        let calendars = aim.list_calendars().await?;
        print_calendars(&calendars, self.output_format);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CmdCalendarShow {
    pub id: String,
    pub output_format: OutputFormat,
}

impl CmdCalendarShow {
    pub const NAME: &str = "show";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("Show detailed information for a calendar")
            .arg(arg!(id: <ID> "Calendar identifier"))
            .arg(CommonArgs::output_format())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: matches
                .get_one::<String>("id")
                .expect("id is required")
                .clone(),
            output_format: CommonArgs::get_output_format(matches),
        }
    }

    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        let calendar = aim.get_calendar_details(&self.id).await?;
        print_calendar_details(&calendar, self.output_format)?;
        Ok(())
    }
}

fn print_calendars(calendars: &[CalendarRecord], output_format: OutputFormat) {
    let formatter = CalendarFormatter::new(output_format);
    println!("{}", formatter.format(calendars));
}

fn print_calendar_details(
    calendar: &CalendarDetails,
    output_format: OutputFormat,
) -> Result<(), Box<dyn Error>> {
    match output_format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(calendar)?),
        OutputFormat::Table => println!("{}", CalendarDetailsDisplay(calendar)),
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct CalendarFormatter {
    format: OutputFormat,
}

impl CalendarFormatter {
    fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    fn format<'a>(&'a self, calendars: &'a [CalendarRecord]) -> CalendarDisplay<'a> {
        CalendarDisplay {
            calendars,
            formatter: self,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CalendarDisplay<'a> {
    calendars: &'a [CalendarRecord],
    formatter: &'a CalendarFormatter,
}

impl fmt::Display for CalendarDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let columns = [
            CalendarColumn::Id,
            CalendarColumn::Name,
            CalendarColumn::Kind,
            CalendarColumn::Priority,
            CalendarColumn::Enabled,
        ];
        let metas: Vec<_> = columns.iter().map(ColumnMeta).collect();

        match self.formatter.format {
            OutputFormat::Json => {
                let table = Table::new(TableStyleJson::new(), &metas, self.calendars);
                write!(f, "{table}")
            }
            OutputFormat::Table => {
                let table = Table::new(TableStyleBasic::new(), &metas, self.calendars);
                write!(f, "{table}")
            }
        }
    }
}

struct CalendarDetailsDisplay<'a>(&'a CalendarDetails);

impl fmt::Display for CalendarDetailsDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let calendar = self.0;
        let mut rows: Vec<(&str, Cow<'_, str>)> = vec![
            ("ID", calendar.id.as_str().into()),
            ("Name", calendar.name.as_str().into()),
            ("Kind", calendar.kind.as_str().into()),
            ("Priority", calendar.priority.to_string().into()),
            ("Enabled", yes_no(calendar.enabled).into()),
            ("Default", yes_no(calendar.is_default).into()),
            ("Created At", calendar.created_at.as_str().into()),
            ("Updated At", calendar.updated_at.as_str().into()),
        ];

        let mut backend_rows: Vec<(&str, Cow<'_, str>)> = Vec::new();
        if let Some(backend) = &calendar.store {
            match backend {
                CalendarStoreDetails::Local { calendar_path } => {
                    backend_rows.push(("Store", "local".into()));
                    backend_rows.push((
                        "Calendar Path",
                        calendar_path
                            .as_deref()
                            .unwrap_or("(not configured)")
                            .into(),
                    ));
                }
                CalendarStoreDetails::Caldav {
                    base_url,
                    calendar_home,
                    calendar_href,
                    auth_method,
                    timeout_secs,
                    user_agent,
                } => {
                    backend_rows.push(("Store", "caldav".into()));
                    backend_rows.push(("Base URL", base_url.as_str().into()));
                    backend_rows.push(("Calendar Home", calendar_home.as_str().into()));
                    backend_rows.push(("Calendar Href", calendar_href.as_str().into()));
                    backend_rows.push(("Auth Method", auth_method.as_str().into()));
                    backend_rows.push(("Timeout Seconds", timeout_secs.to_string().into()));
                    backend_rows.push(("User Agent", user_agent.as_str().into()));
                }
            }
        }

        rows.append(&mut backend_rows);

        let width = rows.iter().map(|(key, _)| key.len()).max().unwrap_or(0);
        for (idx, (key, value)) in rows.into_iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            write!(f, "{key:width$}: {value}")?;
        }

        Ok(())
    }
}

const fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

#[derive(Debug, Clone, Copy)]
enum CalendarColumn {
    Id,
    Name,
    Kind,
    Priority,
    Enabled,
}

#[derive(Debug, Clone, Copy)]
struct ColumnMeta<'a>(&'a CalendarColumn);

impl TableColumn<CalendarRecord> for ColumnMeta<'_> {
    fn name(&self) -> Cow<'_, str> {
        match self.0 {
            CalendarColumn::Id => "ID",
            CalendarColumn::Name => "Name",
            CalendarColumn::Kind => "Kind",
            CalendarColumn::Priority => "Priority",
            CalendarColumn::Enabled => "Enabled",
        }
        .into()
    }

    fn format<'a>(&self, calendar: &'a CalendarRecord) -> Cow<'a, str> {
        match self.0 {
            CalendarColumn::Id => calendar.id.as_str().into(),
            CalendarColumn::Name => calendar.name.as_str().into(),
            CalendarColumn::Kind => calendar.kind.as_str().into(),
            CalendarColumn::Priority => calendar.priority.to_string().into(),
            CalendarColumn::Enabled => yes_no(calendar.enabled).into(),
        }
    }

    fn padding_direction(&self) -> PaddingDirection {
        match self.0 {
            CalendarColumn::Priority => PaddingDirection::Right,
            _ => PaddingDirection::Left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_details_table_includes_default_and_store_fields() {
        let calendar = CalendarDetails {
            id: "work".to_string(),
            name: "Work".to_string(),
            kind: "caldav".to_string(),
            priority: 1,
            enabled: true,
            is_default: false,
            created_at: "2026-03-19T10:00:00+08:00".to_string(),
            updated_at: "2026-03-19T10:30:00+08:00".to_string(),
            store: Some(CalendarStoreDetails::Caldav {
                base_url: "https://example.com".to_string(),
                calendar_home: "/dav/calendars/user/".to_string(),
                calendar_href: "/dav/calendars/user/work/".to_string(),
                auth_method: "basic".to_string(),
                timeout_secs: 30,
                user_agent: "aim/0.1.0".to_string(),
            }),
        };

        let rendered = CalendarDetailsDisplay(&calendar).to_string();
        assert!(rendered.contains("Default"));
        assert!(rendered.contains("Base URL"));
        assert!(rendered.contains("Auth Method"));
    }
}
