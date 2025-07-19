// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    DatePerhapsTime, Pager, Priority, SortOrder, Todo, TodoConditions, TodoSort, TodoSortKey,
    TodoStatus,
};
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Utc};
use icalendar::Component;
use sqlx::sqlite::SqlitePool;

#[derive(sqlx::FromRow)]
pub struct TodoRecord {
    #[allow(dead_code)]
    id: i64,
    uid: String,
    completed: String,
    description: String,
    percent: Option<u8>,
    priority: u8,
    status: String,
    summary: String,
    due_at: String,
    due_tz: String,
}

impl TodoRecord {
    /// See RFC-5545 Sect. 3.6.2
    ///
    /// ## max lengths
    /// - completed (25): 2023-10-01T12:00:00+14:00
    /// - status (12): needs-action
    /// - due_at (19): 2023-10-01T12:00:00
    /// - due_tz (32): America/Argentina/ComodRivadavia
    pub async fn create_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
CREATE TABLE todos (
    id INTEGER PRIMARY KEY,
    uid TEXT NOT NULL UNIQUE,
    completed CHAR(25) NOT NULL,
    description TEXT NOT NULL,
    percent INTEGER,
    priority INTEGER NOT NULL,
    status CHAR(12) NOT NULL,
    summary TEXT NOT NULL,
    due_at CHAR(19) NOT NULL,
    due_tz CHAR(32) NOT NULL
);
        ",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "
CREATE UNIQUE INDEX idx_todos_uid ON todos (uid);
            ",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn insert(self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
INSERT INTO todos (uid, completed, description, percent, priority, status, summary, due_at, due_tz)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
        ",
        )
        .bind(self.uid)
        .bind(self.completed)
        .bind(self.description)
        .bind(self.percent)
        .bind(self.priority)
        .bind(self.status)
        .bind(self.summary)
        .bind(self.due_at)
        .bind(self.due_tz)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list(
        pool: &SqlitePool,
        query: &TodoConditions,
        sort: &[TodoSort],
        pager: &Pager,
    ) -> Result<Vec<TodoRecord>, sqlx::Error> {
        let due_before = query.due_before();

        let mut where_clauses: Vec<&str> = Vec::new();
        if query.status.is_some() {
            where_clauses.push("status = ?");
        }
        if due_before.is_some() {
            where_clauses.push("due_at <= ?");
        }

        let mut sql = "SELECT * FROM todos".to_string();
        if !where_clauses.is_empty() {
            sql += " WHERE ";
            sql += &where_clauses.join(" AND ");
        }

        if !sort.is_empty() {
            sql += " ORDER BY ";
            for (i, s) in sort.iter().enumerate() {
                sql += match s.key {
                    TodoSortKey::Due => "due_at",
                    TodoSortKey::Priority => "priority",
                };
                sql += match s.order {
                    SortOrder::Asc => " ASC",
                    SortOrder::Desc => " DESC",
                };
                if i < sort.len() - 1 {
                    sql += ", ";
                }
            }
        }
        sql += " LIMIT ? OFFSET ?";

        let mut executable = sqlx::query_as(&sql);
        if let Some(status) = &query.status {
            let status: &str = status.as_ref();
            executable = executable.bind(status);
        }
        if let Some(due_at) = due_before {
            executable = executable.bind(format_ndt(due_at));
        }

        executable
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(pool)
            .await
    }

    pub async fn count(pool: &SqlitePool, query: &TodoConditions) -> Result<i64, sqlx::Error> {
        let due_before = query.due_before();
        let mut where_clauses = Vec::new();
        if query.status.is_some() {
            where_clauses.push("status = ?");
        }
        if due_before.is_some() {
            where_clauses.push("due_at <= ?");
        }

        let mut sql = "SELECT COUNT(*) FROM todos".to_string();
        if !where_clauses.is_empty() {
            sql += " WHERE ";
            sql += &where_clauses.join(" AND ");
        }

        let mut executable = sqlx::query_as(&sql);
        if let Some(status) = &query.status {
            let status: &str = status.as_ref();
            executable = executable.bind(status);
        }
        if let Some(due_at) = query.due_before() {
            executable = executable.bind(format_ndt(due_at));
        }
        let row: (i64,) = executable.fetch_one(pool).await?;
        Ok(row.0)
    }
}

impl Todo for TodoRecord {
    fn uid(&self) -> &str {
        &self.uid
    }

    fn completed(&self) -> Option<DateTime<FixedOffset>> {
        if self.completed.is_empty() {
            None
        } else {
            DateTime::parse_from_rfc3339(&self.completed).ok()
        }
    }

    fn description(&self) -> Option<&str> {
        if self.completed.is_empty() {
            None
        } else {
            Some(&self.description)
        }
    }

    fn due(&self) -> Option<DatePerhapsTime> {
        from_dt_tz(&self.due_at, &self.due_tz)
    }

    fn percent(&self) -> Option<u8> {
        self.percent
    }

    fn priority(&self) -> Priority {
        self.priority.into()
    }

    fn status(&self) -> Option<TodoStatus> {
        self.status.as_str().parse().ok()
    }

    fn summary(&self) -> &str {
        &self.summary
    }
}

impl From<icalendar::Todo> for TodoRecord {
    fn from(todo: icalendar::Todo) -> Self {
        let (due_at, due_tz) = to_dt_tz(todo.get_due());
        Self {
            id: 0,                                         // Placeholder, will be set by the database
            uid: todo.get_uid().unwrap_or("").to_string(), // TODO: Handle missing UID
            summary: todo.get_summary().unwrap_or("").to_string(),
            description: todo.get_description().unwrap_or("").to_string(),
            due_at,
            due_tz,
            completed: todo
                .get_completed()
                .map(format_dt)
                .unwrap_or("".to_string()),
            percent: todo.get_percent_complete(),
            priority: todo.get_priority().map(|v| v as u8).unwrap_or(0),
            status: todo
                .get_status()
                .as_ref()
                .map(|s| {
                    let s: TodoStatus = s.into();
                    s.to_string()
                })
                .unwrap_or("".to_string()),
        }
    }
}

const DATE_FORMAT: &str = "%Y-%m-%d";
const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

fn format_ndt(ndt: NaiveDateTime) -> String {
    ndt.format(DATETIME_FORMAT).to_string()
}

fn format_dt(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format(DATETIME_FORMAT).to_string()
}

fn to_dt_tz(dt: Option<icalendar::DatePerhapsTime>) -> (String, String) {
    match dt {
        Some(dt) => DatePerhapsTime::to_dt_tz(&dt.into(), DATE_FORMAT, DATETIME_FORMAT),
        None => ("".to_string(), "".to_string()),
    }
}

fn from_dt_tz(dt: &str, tz: &str) -> Option<DatePerhapsTime> {
    DatePerhapsTime::from_dt_tz(dt, tz, DATE_FORMAT, DATETIME_FORMAT)
}
