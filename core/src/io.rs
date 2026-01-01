// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::path::{Path, PathBuf};

use icalendar::{Calendar, CalendarComponent};
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

pub async fn parse_ics(path: &Path) -> Result<Calendar, Box<dyn Error>> {
    fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?
        .parse()
        .map_err(|e| format!("Failed to parse calendar: {e}").into())
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
