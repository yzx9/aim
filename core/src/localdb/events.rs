// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, Local, NaiveDate};
use sqlx::{Sqlite, SqlitePool, query::QueryAs, sqlite::SqliteArguments};

use crate::{
    Event, EventStatus, LooseDateTime, Pager,
    datetime::{STABLE_FORMAT_DATEONLY, STABLE_FORMAT_LOCAL},
    event::ParsedEventConditions,
};

#[derive(Debug, Clone)]
pub struct Events {
    pool: SqlitePool,
}

impl Events {
    pub async fn new(pool: SqlitePool) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { pool })
    }

    pub async fn insert(&self, event: EventRecord) -> Result<(), sqlx::Error> {
        const SQL: &str = "\
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
        const SQL: &str = "\
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
        let mut sql = "\
SELECT uid, path, summary, description, status, start, end
FROM events
"
        .to_string();
        sql += &self.build_where(conds);
        sql += "ORDER BY start ASC LIMIT ? OFFSET ?;";

        let mut executable = sqlx::query_as(&sql);
        executable = self.bind_conditions(conds, executable);

        executable
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count(&self, conds: &ParsedEventConditions) -> Result<i64, sqlx::Error> {
        let mut sql = "SELECT COUNT(*) FROM events".to_string();
        sql += &self.build_where(conds);
        sql += ";";

        let mut executable = sqlx::query_as(&sql);
        executable = self.bind_conditions(conds, executable);

        let row: (i64,) = executable.fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    fn build_where(&self, conds: &ParsedEventConditions) -> String {
        let mut where_clauses = Vec::new();
        if conds.start_before.is_some() {
            where_clauses.push("start <= ?");
        }
        if conds.end_after.is_some() {
            where_clauses.push("(end >= ? OR end = ?)");
        }

        if !where_clauses.is_empty() {
            format!(" WHERE {} ", where_clauses.join(" AND "))
        } else {
            String::new()
        }
    }

    fn bind_conditions<'a, O>(
        &self,
        conds: &'a ParsedEventConditions,
        mut query: QueryAs<'a, Sqlite, O, SqliteArguments<'a>>,
    ) -> QueryAs<'a, Sqlite, O, SqliteArguments<'a>> {
        if let Some(start_before) = conds.start_before {
            query = query.bind(format_dt(start_before));
        }
        if let Some(end_after) = conds.end_after {
            query = query
                .bind(format_dt(end_after))
                .bind(format_date(end_after.date_naive()));
        }
        query
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
                .unwrap_or_default(),
            status: event.status().map(|s| s.to_string()).unwrap_or_default(),
            start: event.start().map(|a| a.format_stable()).unwrap_or_default(),
            end: event.end().map(|a| a.format_stable()).unwrap_or_default(),
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

fn format_date(date: NaiveDate) -> String {
    date.format(STABLE_FORMAT_DATEONLY).to_string()
}

fn format_dt(dt: DateTime<Local>) -> String {
    dt.format(STABLE_FORMAT_LOCAL).to_string()
}
