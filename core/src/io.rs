// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::path::{Path, PathBuf};

use aimcal_ical::{CalendarComponent, ICalendar, formatter::format, parse};
use tokio::fs;

use crate::localdb::LocalDb;

#[tracing::instrument(skip(db))]
pub async fn add_calendar(db: &LocalDb, calendar_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut reader = fs::read_dir(calendar_path)
        .await
        .map_err(|e| format!("Failed to read directory: {e}"))?;

    let mut handles = vec![];
    while let Some(entry) = reader.next_entry().await? {
        let path = entry.path();
        match path.extension() {
            Some(ext) if ext == "ics" => {
                let db = db.clone();
                handles.push(tokio::spawn(async move {
                    if let Err(err) = add_ics(db, &path).await {
                        tracing::error!(path = %path.display(), err, "failed to process file");
                    }
                }));
            }
            _ => {}
        }
    }

    let count = handles.len();
    for handle in handles {
        handle.await?;
    }

    tracing::debug!(count = count, "total .ics files processed");
    Ok(())
}

// TODO: support multiple calendars in one file
pub async fn parse_ics(path: &Path) -> Result<ICalendar<String>, Box<dyn Error>> {
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    let calendars = parse(&content)
        .map_err(|e| -> Box<dyn Error> { format!("Failed to parse calendar: {e:?}").into() })?;

    if calendars.is_empty() {
        return Err("No calendars found in file".into());
    }

    // Hybrid: parse borrowed, convert to owned for storage
    Ok(calendars.into_iter().next().unwrap().to_owned())
}

pub async fn write_ics(path: &Path, calendar: &ICalendar<String>) -> Result<(), String> {
    let ics_content = format(calendar).map_err(|e| format!("Failed to format calendar: {e}"))?;

    fs::write(path, ics_content)
        .await
        .map_err(|e| format!("Failed to write calendar file: {e}"))
}

async fn add_ics(db: LocalDb, path: &Path) -> Result<(), Box<dyn Error>> {
    tracing::debug!(path = %path.display(), "parsing file");
    let calendar = parse_ics(path).await?;

    tracing::debug!(path = %path.display(), components = calendar.components.len(), "found components");
    for component in calendar.components {
        tracing::debug!(?component, "processing component");
        match component {
            CalendarComponent::Event(event) => db.upsert_event(path, &event).await?,
            CalendarComponent::Todo(todo) => db.upsert_todo(path, &todo).await?,
            _ => tracing::warn!(?component, "ignoring unsupported component type"),
        }
    }

    Ok(())
}
