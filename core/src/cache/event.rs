// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{Event, EventQuery, Pager};
use icalendar::{CalendarDateTime, Component, DatePerhapsTime, EventStatus};
use sqlx::SqlitePool;

#[derive(sqlx::FromRow)]
pub struct EventRecord {
    id: i64,
    summary: String,
    description: Option<String>,
    start_at: Option<String>,
    start_has_time: bool,
    end_at: Option<String>,
    end_has_time: bool,
    status: Option<String>,
}

impl EventRecord {
    pub async fn create_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    summary TEXT NOT NULL,
    description TEXT,
    start_at TEXT,
    start_has_time BOOLEAN,
    end_at TEXT,
    end_has_time BOOLEAN,
    status TEXT
);
        ",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn insert(self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
INSERT INTO events (summary, description, start_at, start_has_time, end_at, end_has_time, status)
VALUES (?, ?, ?, ?, ?, ?, ?);
        ",
        )
        .bind(self.summary)
        .bind(self.description)
        .bind(self.start_at)
        .bind(self.start_has_time)
        .bind(self.end_at)
        .bind(self.end_has_time)
        .bind(self.status)
        .execute(pool)
        .await?;

        Ok(())
    }
    pub async fn list(
        pool: &SqlitePool,
        _query: &EventQuery,
        pager: &Pager,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM events ORDER BY id LIMIT ? OFFSET ?")
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(pool)
            .await
    }

    pub async fn count(pool: &SqlitePool, _query: &EventQuery) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
            .fetch_one(pool)
            .await?;
        Ok(row.0)
    }
}

impl Event for EventRecord {
    fn id(&self) -> i64 {
        self.id
    }

    fn summary(&self) -> &str {
        &self.summary
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn start_at(&self) -> Option<&str> {
        self.start_at.as_deref()
    }

    fn start_has_time(&self) -> bool {
        self.start_has_time
    }

    fn end_at(&self) -> Option<&str> {
        self.end_at.as_deref()
    }

    fn end_has_time(&self) -> bool {
        self.end_has_time
    }

    fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
}

impl From<icalendar::Event> for EventRecord {
    fn from(event: icalendar::Event) -> Self {
        let status = event.get_status().map(|s| match s {
            EventStatus::Tentative => "TENTATIVE".to_string(),
            EventStatus::Confirmed => "CONFIRMED".to_string(),
            EventStatus::Cancelled => "CANCELLED".to_string(),
        });
        let (start_at, start_has_time) = to_db_time(event.get_start());
        let (end_at, end_has_time) = to_db_time(event.get_start());

        Self {
            id: 0, // Placeholder, will be set by the database
            summary: event.get_summary().unwrap_or("").to_string(),
            description: event.get_description().map(|s| s.to_string()),
            start_at,
            start_has_time,
            end_at,
            end_has_time,
            status,
        }
    }
}

fn to_db_time(date: Option<DatePerhapsTime>) -> (Option<String>, bool) {
    match date {
        Some(DatePerhapsTime::DateTime(dt)) => match dt {
            CalendarDateTime::Floating(dt) => {
                (Some(dt.format("%Y-%m-%dT%H:%M:%S").to_string()), true)
            }
            CalendarDateTime::Utc(dt) => (Some(dt.to_rfc3339()), true),
            CalendarDateTime::WithTimezone { date_time, tzid: _ } => (
                Some(date_time.format("%Y-%m-%dT%H:%M:%S").to_string()),
                true,
            ),
        },
        Some(DatePerhapsTime::Date(d)) => (Some(d.to_string()), false),
        None => (None, false),
    }
}
