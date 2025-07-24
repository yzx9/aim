// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{DatePerhapsTime, Event, EventConditions, EventStatus, Pager};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct Events {
    pool: SqlitePool,
}

impl Events {
    pub async fn new(pool: SqlitePool) -> Result<Self, Box<dyn std::error::Error>> {
        Self::create_table(&pool)
            .await
            .map_err(|e| format!("Failed to create events table: {e}"))?;

        Ok(Self { pool })
    }

    /// See RFC-5545 Sect. 3.6.1
    ///
    /// ## max lengths
    /// - `completed`/`due_at` (25): 2023-10-01T12:00:00+14:00
    /// - `status` (12): needs-action
    /// - `start_at`/`end_at` (19): 2023-10-01T12:00:00
    /// - `start_tz`/`end_tz` (32): America/Argentina/ComodRivadavia
    async fn create_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL,
    uid TEXT NOT NULL UNIQUE,
    summary TEXT NOT NULL,
    description TEXT,
    status TEXT,
    start_at CHAR(19) NOT NULL,
    start_tz CHAR(32) NOT NULL,
    end_at CHAR(19) NOT NULL,
    end_tz CHAR(32) NOT NULL
);
        ",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn insert(&self, event: EventRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
INSERT INTO events (path, uid, summary, description, status, start_at, start_tz, end_at, end_tz)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
        ",
        )
        .bind(&event.path)
        .bind(&event.uid)
        .bind(&event.summary)
        .bind(&event.description)
        .bind(&event.status)
        .bind(&event.start_at)
        .bind(&event.start_tz)
        .bind(&event.end_at)
        .bind(&event.end_tz)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list(
        &self,
        _conds: &EventConditions,
        pager: &Pager,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM events ORDER BY id LIMIT ? OFFSET ?")
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count(&self, _conds: &EventConditions) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct EventRecord {
    #[allow(dead_code)]
    id: i64,
    path: String,
    uid: String,
    summary: String,
    description: Option<String>,
    start_at: String,
    start_tz: String,
    end_at: String,
    end_tz: String,
    status: Option<String>,
}

impl EventRecord {
    pub fn from(path: String, event: impl Event) -> Result<Self, Box<dyn std::error::Error>> {
        let (start_at, start_tz) = to_dt_tz(event.start());
        let (end_at, end_tz) = to_dt_tz(event.end());

        Ok(Self {
            id: 0, // Placeholder, will be set by the database
            path,
            uid: event.uid().to_string(),
            summary: event.summary().to_string(),
            description: event.description().map(ToString::to_string),
            start_at,
            start_tz,
            end_at,
            end_tz,
            status: event.status().map(|s| s.to_string()),
        })
    }
}

impl Event for EventRecord {
    fn uid(&self) -> &str {
        &self.uid
    }

    fn summary(&self) -> &str {
        &self.summary
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn start(&self) -> Option<DatePerhapsTime> {
        from_dt_tz(&self.start_at, &self.start_tz)
    }

    fn end(&self) -> Option<DatePerhapsTime> {
        from_dt_tz(&self.end_at, &self.end_tz)
    }

    fn status(&self) -> Option<EventStatus> {
        self.status.as_ref().and_then(|a| a.as_str().parse().ok())
    }
}

const DATE_FORMAT: &str = "%Y-%m-%d";
const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

fn to_dt_tz(dt: Option<DatePerhapsTime>) -> (String, String) {
    match dt {
        Some(dt) => DatePerhapsTime::to_dt_tz(&dt, DATE_FORMAT, DATETIME_FORMAT),
        None => ("".to_string(), "".to_string()),
    }
}

fn from_dt_tz(dt: &str, tz: &str) -> Option<DatePerhapsTime> {
    DatePerhapsTime::from_dt_tz(dt, tz, DATE_FORMAT, DATETIME_FORMAT)
}
