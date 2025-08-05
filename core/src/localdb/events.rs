// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, Local};
use sqlx::SqlitePool;

use crate::{Event, EventStatus, LooseDateTime, Pager, event::ParsedEventConditions};

#[derive(Debug, Clone)]
pub struct Events {
    pool: SqlitePool,
}

impl Events {
    pub async fn new(pool: SqlitePool) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { pool })
    }

    pub async fn insert(&self, event: EventRecord) -> Result<(), sqlx::Error> {
        const SQL: &str = "
INSERT INTO events (uid, path, summary, description, status, start, end)
VALUES (?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(uid) DO UPDATE SET
    path        = excluded.path,
    summary     = excluded.summary,
    description = excluded.description,
    status      = excluded.status,
    start       = excluded.start,
    end         = excluded.end;
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

    pub async fn get(&self, uid: &str) -> Result<Option<EventRecord>, sqlx::Error> {
        const SQL: &str = "
SELECT uid, path, summary, description, status, start, end
FROM events
WHERE uid = ?;
";

        sqlx::query_as(SQL)
            .bind(uid)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list(
        &self,
        conds: &ParsedEventConditions,
        pager: &Pager,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        let mut sql = "
SELECT uid, path, summary, description, status, start, end
FROM events
"
        .to_string();

        let mut where_clauses = Vec::new();
        if conds.end_after.is_some() {
            where_clauses.push("end >= ?");
        }
        if !where_clauses.is_empty() {
            sql += " WHERE ";
            sql += &where_clauses.join(" AND ");
        }

        sql += "ORDER BY start ASC LIMIT ? OFFSET ?";

        let mut executable = sqlx::query_as(&sql);
        if let Some(end_after) = conds.end_after {
            executable = executable.bind(format_dt(end_after));
        }

        executable
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count(&self, conds: &ParsedEventConditions) -> Result<i64, sqlx::Error> {
        let mut sql = "SELECT COUNT(*) FROM events".to_string();
        let mut where_clauses = Vec::new();
        if conds.end_after.is_some() {
            where_clauses.push("end >= ?");
        }
        if !where_clauses.is_empty() {
            sql += " WHERE ";
            sql += &where_clauses.join(" AND ");
        }

        let row: (i64,) = sqlx::query_as(&sql).fetch_one(&self.pool).await?;
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

    pub fn path(&self) -> &str {
        &self.path
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

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

fn format_dt(dt: DateTime<Local>) -> String {
    dt.format(DATETIME_FORMAT).to_string()
}
