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

#[derive(Debug, Clone)]
pub struct Todos {
    pool: SqlitePool,
}

impl Todos {
    pub async fn new(pool: SqlitePool) -> Result<Self, Box<dyn std::error::Error>> {
        Self::create_table(&pool)
            .await
            .map_err(|e| format!("Failed to create todos table: {e}"))?;

        Ok(Self { pool })
    }

    /// See RFC-5545 Sect. 3.6.2
    ///
    /// ## max lengths
    /// - completed (25): 2023-10-01T12:00:00+14:00
    /// - status (12): needs-action
    /// - due_at (19): 2023-10-01T12:00:00
    /// - due_tz (32): America/Argentina/ComodRivadavia
    async fn create_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
CREATE TABLE todos (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL,
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

    pub async fn insert(&self, todo: TodoRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
INSERT INTO todos (path, uid, completed, description, percent, priority, status, summary, due_at, due_tz)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        ",
        )
        .bind(&todo.path)
        .bind(&todo.uid)
        .bind(&todo.completed)
        .bind(&todo.description)
        .bind(todo.percent)
        .bind(todo.priority)
        .bind(&todo.status)
        .bind(&todo.summary)
        .bind(&todo.due_at)
        .bind(&todo.due_tz)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list(
        &self,
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
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count(&self, query: &TodoConditions) -> Result<i64, sqlx::Error> {
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
        let row: (i64,) = executable.fetch_one(&self.pool).await?;
        Ok(row.0)
    }
}

#[derive(sqlx::FromRow)]
pub struct TodoRecord {
    #[allow(dead_code)]
    id: i64,
    path: String,
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
    pub fn from(path: String, todo: icalendar::Todo) -> Result<Self, Box<dyn std::error::Error>> {
        let uid = todo.get_uid().ok_or("Todo must have a UID")?.to_string();
        let (due_at, due_tz) = to_dt_tz(todo.get_due());
        Ok(Self {
            id: 0, // Placeholder, will be set by the database
            path,
            uid,
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
        })
    }
}

impl Todo for TodoRecord {
    fn uid(&self) -> &str {
        &self.uid
    }

    fn completed(&self) -> Option<DateTime<FixedOffset>> {
        (!self.completed.is_empty())
            .then(|| DateTime::parse_from_rfc3339(&self.completed).ok())
            .flatten()
    }

    fn description(&self) -> Option<&str> {
        (!self.description.is_empty()).then_some(self.description.as_str())
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
