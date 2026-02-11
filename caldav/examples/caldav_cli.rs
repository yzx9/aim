// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! CalDAV client validation tool.
//!
//! This is a standalone CLI example for testing the CalDAV client implementation
//! against real CalDAV servers. It serves as both a validation tool and example
//! code for using the CalDavClient API.

use std::error::Error;
use std::io::Write as _;

use aimcal_caldav::{
    AuthMethod, CalDavClient, CalDavConfig, CalDavError, CalendarQueryRequest, Href,
};
use aimcal_ical::formatter;
use aimcal_ical::{CalendarComponent, ICalendar};
use clap::{Parser, Subcommand};
use colored::Colorize as _;

/// CalDAV client validation tool.
#[derive(Parser)]
#[command(name = "caldav_cli")]
#[command(about = "CalDAV client validation tool", long_about = None)]
#[command(version)]
struct Cli {
    /// CalDAV server URL
    #[arg(long)]
    server: Option<String>,
    /// Calendar home path
    #[arg(long, default_value = "/")]
    home: String,
    /// Username for basic auth
    #[arg(long)]
    username: Option<String>,
    /// Password for basic auth
    #[arg(long)]
    password: Option<String>,
    /// Bearer token for OAuth
    #[arg(long)]
    token: Option<String>,
    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,
    #[command(subcommand)]
    command: Commands,
}

/// Available commands.
#[derive(Subcommand)]
enum Commands {
    /// Test server discovery
    Discover,
    /// List all calendar collections
    ListCals,
    /// List events in a time range
    ListEvents {
        /// Calendar href
        calendar: String,
        /// Start date (e.g., "2025-01-01" or "today")
        #[arg(long)]
        start: String,
        /// End date
        #[arg(long)]
        end: Option<String>,
    },
    /// List todos
    ListTodos {
        /// Calendar href
        calendar: String,
        /// Filter by status: pending, completed, all
        #[arg(long, default_value = "pending")]
        status: String,
    },
    /// Get a specific calendar object
    Get {
        /// Resource href
        href: String,
    },
    /// Add a new event or todo
    Add {
        /// Resource href (full path to the .ics file on the server)
        href: String,
        /// iCalendar file path (or "-" for stdin)
        input: String,
    },
    /// Edit an existing event or todo
    Edit {
        /// Resource href
        href: String,
        /// iCalendar file path (or "-" for stdin)
        input: String,
    },
    /// Delete an event or todo
    Delete {
        /// Resource href
        href: String,
    },
}

impl Cli {
    fn build_config(&self) -> Result<CalDavConfig, Box<dyn std::error::Error>> {
        // Read from environment variables first
        let server = self
            .server
            .clone()
            .or_else(|| std::env::var("AIM_CALDAV_SERVER").ok())
            .ok_or_else(|| {
                "AIM_CALDAV_SERVER must be provided via --server or AIM_CALDAV_SERVER env var"
                    .to_string()
            })?;

        let username = self
            .username
            .clone()
            .or_else(|| std::env::var("AIM_CALDAV_USERNAME").ok());

        let password = self
            .password
            .clone()
            .or_else(|| std::env::var("AIM_CALDAV_PASSWORD").ok());

        let token = self
            .token
            .clone()
            .or_else(|| std::env::var("AIM_CALDAV_TOKEN").ok());

        let auth = if let Some(token) = token {
            AuthMethod::Bearer { token }
        } else if let (Some(username), Some(password)) = (username, password) {
            AuthMethod::Basic { username, password }
        } else {
            AuthMethod::None
        };

        Ok(CalDavConfig {
            base_url: server,
            calendar_home: self.home.clone(),
            auth,
            timeout_secs: self.timeout,
            user_agent: "aimcal-caldav-cli/0.1.0".to_string(),
        })
    }
}

async fn cmd_discover(client: &CalDavClient) -> Result<(), CalDavError> {
    let result = client.discover().await?;

    if result.supports_calendars {
        println!("{}", "✓ CalDAV support detected".green());
        println!("Calendar home: {}", result.calendar_home.as_str());
    } else {
        println!("{}", "⚠ Server doesn't appear to support CalDAV".yellow());
        println!("Calendar home: {}", result.calendar_home.as_str());
    }

    let capabilities = client.capabilities();
    println!("\nServer capabilities:");
    println!("  calendar-access: {}", capabilities.supports_calendars);
    println!("  calendar-query: {}", capabilities.can_query());
    println!("  calendar-multiget: {}", capabilities.can_multiget());
    println!("  mkcalendar: {}", capabilities.can_mkcalendar());
    println!("  free-busy-query: {}", capabilities.can_free_busy());

    Ok(())
}

async fn cmd_list_cals(client: &CalDavClient) -> Result<(), Box<dyn std::error::Error>> {
    let calendars = client.list_calendars().await?;

    if calendars.is_empty() {
        println!("No calendars found");
        return Ok(());
    }

    println!("{:-<100}", "");
    println!("{:<50} {:<20} {:<20}", "Href", "Name", "Components");
    println!("{:-<100}", "");

    for cal in &calendars {
        let name = cal.display_name.as_deref().unwrap_or("Unnamed");
        let components = cal.supported_components.join(", ");
        println!("{:<50} {:<20} {}", cal.href.as_str(), name, components);
    }

    Ok(())
}

/// Helper to extract events from an ICalendar
fn extract_events(ical: &ICalendar<String>) -> Vec<&aimcal_ical::VEvent<String>> {
    ical.components
        .iter()
        .filter_map(|c| match c {
            CalendarComponent::Event(e) => Some(e),
            _ => None,
        })
        .collect()
}

/// Helper to extract todos from an ICalendar
fn extract_todos(ical: &ICalendar<String>) -> Vec<&aimcal_ical::VTodo<String>> {
    ical.components
        .iter()
        .filter_map(|c| match c {
            CalendarComponent::Todo(t) => Some(t),
            _ => None,
        })
        .collect()
}

async fn cmd_list_events(
    client: &CalDavClient,
    calendar: &str,
    start: &str,
    end: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let href = Href::new(calendar.to_string());

    // Parse start date to UTC format
    let start_utc = parse_date_to_utc(start)?;
    let end_utc = match end {
        Some(e) => Some(parse_date_to_utc(e)?),
        None => None,
    };

    let request = CalendarQueryRequest::new()
        .component("VEVENT".to_string())
        .time_range(start_utc, end_utc);

    let events = client.query(&href, &request).await?;

    if events.is_empty() {
        println!("No events found");
        return Ok(());
    }

    println!("{:-<80}", "");
    println!("{:<20} {:<50} {:<10}", "Start", "Summary", "Href");
    println!("{:-<80}", "");

    for resource in events {
        let extracted = extract_events(&resource.data);
        if let Some(event) = extracted.first() {
            let start_str = format_datetime(&event.dt_start.value);
            let summary = event
                .summary
                .as_ref()
                .map(|s| s.content.to_string())
                .unwrap_or_default();
            let href_short = resource
                .href
                .as_str()
                .rsplit('/')
                .next()
                .unwrap_or(resource.href.as_str());
            println!("{:<20} {:<50} {:<10}", start_str, summary, href_short);
        }
    }

    Ok(())
}

/// Format a DateTime value for display
fn format_datetime(dt: &aimcal_ical::DateTime) -> String {
    match dt {
        aimcal_ical::DateTime::Utc { date, time } => {
            format!(
                "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
                date.year, date.month, date.day, time.hour, time.minute, time.second
            )
        }
        aimcal_ical::DateTime::Floating { date, time }
        | aimcal_ical::DateTime::Zoned { date, time, .. } => {
            format!(
                "{:04}{:02}{:02}T{:02}{:02}{:02}",
                date.year, date.month, date.day, time.hour, time.minute, time.second
            )
        }
        aimcal_ical::DateTime::Date(date) => {
            format!("{:04}{:02}{:02}", date.year, date.month, date.day)
        }
    }
}

async fn cmd_list_todos(
    client: &CalDavClient,
    calendar: &str,
    status: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let href = Href::new(calendar.to_string());

    let todos = match status {
        "completed" => client.get_completed_todos(&href).await?,
        "all" => {
            let mut pending = client.get_pending_todos(&href).await?;
            let mut completed = client.get_completed_todos(&href).await?;
            pending.append(&mut completed);
            pending
        }
        _ => client.get_pending_todos(&href).await?,
    };

    if todos.is_empty() {
        println!("No todos found");
        return Ok(());
    }

    println!("{:-<80}", "");
    println!("{:<20} {:<50} {:<10}", "Due", "Summary", "Status");
    println!("{:-<80}", "");

    for resource in todos {
        let extracted = extract_todos(&resource.data);
        if let Some(todo) = extracted.first() {
            let due = todo
                .due
                .as_ref()
                .map(|d| {
                    format_datetime(&d.value)
                        .chars()
                        .take(10)
                        .collect::<String>()
                })
                .unwrap_or_else(|| "-".to_string());
            let summary = todo
                .summary
                .as_ref()
                .map(|s| s.content.to_string())
                .unwrap_or_default();
            let status_str = todo
                .status
                .as_ref()
                .map(format_todo_status)
                .unwrap_or_else(|| "NEEDS-ACTION".to_string());
            println!("{:<20} {:<50} {:<10}", due, summary, status_str);
        }
    }

    Ok(())
}

/// Format a TodoStatus value for display
fn format_todo_status(status: &aimcal_ical::TodoStatus<String>) -> String {
    match &status.value {
        aimcal_ical::TodoStatusValue::NeedsAction => "NEEDS-ACTION".to_string(),
        aimcal_ical::TodoStatusValue::Completed => "COMPLETED".to_string(),
        aimcal_ical::TodoStatusValue::InProcess => "IN-PROCESS".to_string(),
        aimcal_ical::TodoStatusValue::Cancelled => "CANCELLED".to_string(),
    }
}

/// Read iCalendar data from a file or stdin.
fn read_icalendar(input: &str) -> Result<ICalendar<String>, Box<dyn std::error::Error>> {
    let content = if input == "-" {
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        std::fs::read_to_string(input)?
    };

    let calendars = aimcal_ical::parse(&content).map_err(|e| {
        format!(
            "iCalendar parsing failed: {}",
            e.iter()
                .map(|err| format!("{err}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;
    Ok(calendars
        .into_iter()
        .next()
        .ok_or_else(|| "No calendar data found in input".to_string())?
        .to_owned())
}

async fn cmd_get(client: &CalDavClient, href: &str) -> Result<(), Box<dyn std::error::Error>> {
    let href_obj = Href::new(href.to_string());
    let resource = client.get_event(&href_obj).await?;

    println!("ETag: {}", resource.etag.as_str());
    println!("Href: {}", resource.href.as_str());
    println!();

    // Output formatted iCalendar
    let ical_str = formatter::format(&resource.data)?;
    println!("{}", ical_str);

    Ok(())
}

async fn cmd_add(
    client: &CalDavClient,
    href: &str,
    input: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let href_obj = Href::new(href.to_string());
    let ical = read_icalendar(input)?;

    // Determine if this is an event or todo based on components
    let has_event = ical
        .components
        .iter()
        .any(|c| matches!(c, CalendarComponent::Event(_)));
    let has_todo = ical
        .components
        .iter()
        .any(|c| matches!(c, CalendarComponent::Todo(_)));

    let etag = if has_event {
        client.create_event(&href_obj, &ical).await?
    } else if has_todo {
        client.create_todo(&href_obj, &ical).await?
    } else {
        return Err("No VEVENT or VTODO component found in iCalendar data".into());
    };

    println!("{}", "✓ Resource created successfully".green());
    println!("Href: {}", href);
    println!("ETag: {}", etag.as_str());

    Ok(())
}

async fn cmd_edit(
    client: &CalDavClient,
    href: &str,
    input: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let href_obj = Href::new(href.to_string());

    // Get current resource to retrieve ETag
    let resource = client.get_event(&href_obj).await?;
    let current_etag = resource.etag;

    // Read new iCalendar data
    let ical = read_icalendar(input)?;

    // Determine if this is an event or todo based on components
    let has_event = ical
        .components
        .iter()
        .any(|c| matches!(c, CalendarComponent::Event(_)));
    let has_todo = ical
        .components
        .iter()
        .any(|c| matches!(c, CalendarComponent::Todo(_)));

    let new_etag = if has_event {
        client.update_event(&href_obj, &current_etag, &ical).await?
    } else if has_todo {
        client.update_todo(&href_obj, &current_etag, &ical).await?
    } else {
        return Err("No VEVENT or VTODO component found in iCalendar data".into());
    };

    println!("{}", "✓ Resource updated successfully".green());
    println!("Href: {}", href);
    println!("Old ETag: {}", current_etag.as_str());
    println!("New ETag: {}", new_etag.as_str());

    Ok(())
}

async fn cmd_delete(client: &CalDavClient, href: &str) -> Result<(), Box<dyn std::error::Error>> {
    let href_obj = Href::new(href.to_string());

    // Get current resource to retrieve ETag
    let resource = client.get_event(&href_obj).await?;
    let etag = resource.etag;

    // Determine if this is an event or todo based on components
    let has_event = resource
        .data
        .components
        .iter()
        .any(|c| matches!(c, CalendarComponent::Event(_)));
    let has_todo = resource
        .data
        .components
        .iter()
        .any(|c| matches!(c, CalendarComponent::Todo(_)));

    if has_event {
        client.delete_event(&href_obj, &etag).await?;
    } else if has_todo {
        client.delete_todo(&href_obj, &etag).await?;
    } else {
        return Err("No VEVENT or VTODO component found in resource".into());
    }

    println!("{}", "✓ Resource deleted successfully".green());
    println!("Href: {}", href);

    Ok(())
}

/// Parse a date string to UTC datetime format for CalDAV.
///
/// Accepts formats like:
/// - "today" → today at 00:00:00 UTC
/// - "2025-01-01" → 2025-01-01T00:00:00Z
/// - "2025-01-01T12:00:00Z" → passed through
fn parse_date_to_utc(date: &str) -> Result<String, String> {
    let now = jiff::Zoned::now();

    if date.eq_ignore_ascii_case("today") {
        let start_of_day = jiff::civil::Date::from(now)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .map_err(|e| format!("Failed to convert to UTC: {e}"))?;
        return Ok(start_of_day.strftime("%Y%m%dT%H%M%SZ").to_string());
    }

    // Try YYYY-MM-DD format
    if let Ok(dt) = jiff::civil::DateTime::strptime(date, "%Y-%m-%d") {
        let zoned = dt
            .to_zoned(jiff::tz::TimeZone::UTC)
            .map_err(|e| format!("Failed to convert to UTC: {e}"))?;
        return Ok(zoned.strftime("%Y%m%dT%H%M%SZ").to_string());
    }

    // Try YYYY-MM-DDTHH:MM:SSZ format (already UTC)
    if date.contains('T') && (date.ends_with('Z') || date.contains('+')) {
        // Assume it's already in a valid format
        return Ok(date.to_string());
    }

    Err(format!(
        "Invalid date format: '{date}'. Use YYYY-MM-DD, today, or full datetime"
    ))
}

/// Format error for user-friendly display.
fn format_error(err: Box<dyn Error>) -> String {
    let err_str = err.to_string();
    if err_str.contains("authentication") || err_str.contains("401") || err_str.contains("403") {
        format!("{} Authentication failed", "Error:".red().bold())
    } else if err_str.contains("404") || err_str.contains("not found") {
        format!("{} Resource not found", "Error:".red().bold())
    } else if err_str.contains("412") || err_str.contains("precondition") {
        format!(
            "{} ETag conflict - resource was modified by another client",
            "Error:".red().bold()
        )
    } else if err_str.contains("CalDAV") {
        format!("{} Server doesn't support CalDAV", "Error:".red().bold())
    } else if err_str.contains("network") || err_str.contains("connection") {
        format!(
            "{} Network error - check server URL and connection",
            "Error:".red().bold()
        )
    } else {
        format!("{} {}", "Error:".red().bold(), err_str)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env files (if they exist)
    // Priority: .env.local (highest) -> .env -> existing environment variables (lowest)
    dotenvy::dotenv().ok(); // Load .env
    dotenvy::from_filename(".env.local").ok(); // Load .env.local (overrides .env)

    let cli = Cli::parse();
    let config = cli.build_config()?;
    let client = CalDavClient::new(config)?;

    // Create a new runtime for the async operations
    let runtime = tokio::runtime::Runtime::new()?;

    let result = runtime.block_on(async {
        match cli.command {
            Commands::Discover => cmd_discover(&client)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
            Commands::ListCals => cmd_list_cals(&client).await,
            Commands::ListEvents {
                calendar,
                start,
                end,
            } => cmd_list_events(&client, &calendar, &start, end.as_deref()).await,
            Commands::ListTodos { calendar, status } => {
                cmd_list_todos(&client, &calendar, &status).await
            }
            Commands::Get { href } => cmd_get(&client, &href).await,
            Commands::Add { href, input } => cmd_add(&client, &href, &input).await,
            Commands::Edit { href, input } => cmd_edit(&client, &href, &input).await,
            Commands::Delete { href } => cmd_delete(&client, &href).await,
        }
    });

    if let Err(e) = result {
        // Flush stdout before printing error
        std::io::stdout().flush().ok();
        eprintln!("{}", format_error(e));
        std::process::exit(1);
    }

    Ok(())
}
