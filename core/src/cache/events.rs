// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{Event, EventConditions, EventStatus, LooseDateTime, Pager};
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
    async fn create_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        const SQL: &str = "
CREATE TABLE events (
    uid         TEXT PRIMARY KEY,
    path        TEXT NOT NULL,
    summary     TEXT NOT NULL,
    description TEXT NOT NULL,
    status      TEXT NOT NULL,
    start       TEXT NOT NULL,
    end         TEXT NOT NULL
);
";

        sqlx::query(SQL).execute(pool).await?;
        Ok(())
    }

    pub async fn insert(&self, event: EventRecord) -> Result<(), sqlx::Error> {
        const SQL: &str = "
INSERT INTO events (uid, path, summary, description, status, start, end)
VALUES (?, ?, ?, ?, ?, ?, ?);
";

        sqlx::query(SQL)
            .bind(&event.uid)
            .bind(&event.path)
            .bind(&event.summary)
            .bind(&event.description)
            .bind(&event.status)
            .bind(&event.start)
            .bind(&event.end)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list(
        &self,
        _conds: &EventConditions,
        pager: &Pager,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM events ORDER BY start ASC LIMIT ? OFFSET ?")
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
    path: String,
    uid: String,
    summary: String,
    description: String,
    status: String,
    start: String,
    end: String,
}

impl EventRecord {
    pub fn from<E: Event>(path: String, event: &E) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            uid: event.uid().to_string(),
            path,
            summary: event.summary().to_string(),
            description: event
                .description()
                .map(ToString::to_string)
                .unwrap_or("".to_string()),
            status: event
                .status()
                .map(|s| s.to_string())
                .unwrap_or("".to_string()),
            start: event
                .start()
                .map(|a| a.format_stable())
                .unwrap_or("".to_string()),
            end: event
                .end()
                .map(|a| a.format_stable())
                .unwrap_or("".to_string()),
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
        (!self.description.is_empty()).then_some(self.description.as_str())
    }

    fn start(&self) -> Option<LooseDateTime> {
        LooseDateTime::parse_stable(&self.start)
    }

    fn end(&self) -> Option<LooseDateTime> {
        LooseDateTime::parse_stable(&self.end)
    }

    fn status(&self) -> Option<EventStatus> {
        self.status.as_str().parse().ok()
    }
}
