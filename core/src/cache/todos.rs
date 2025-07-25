// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{LooseDateTime, Pager, Priority, Todo, TodoConditions, TodoSort, TodoStatus};
use chrono::{DateTime, Local, NaiveDateTime};
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
    async fn create_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        const SQL: &str = "
CREATE TABLE todos (
    uid         TEXT PRIMARY KEY,
    path        TEXT NOT NULL,
    completed   TEXT NOT NULL,
    description TEXT NOT NULL,
    percent     INTEGER,
    priority    INTEGER NOT NULL,
    status      TEXT NOT NULL,
    summary     TEXT NOT NULL,
    due         TEXT NOT NULL
);
";

        sqlx::query(SQL).execute(pool).await?;
        Ok(())
    }

    pub async fn upsert(&self, todo: &TodoRecord) -> Result<(), sqlx::Error> {
        const SQL: &str = "
INSERT INTO todos (uid, path, completed, description, percent, priority, status, summary, due)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(uid) DO UPDATE SET
    path        = excluded.path,
    completed   = excluded.completed,
    description = excluded.description,
    percent     = excluded.percent,
    priority    = excluded.priority,
    status      = excluded.status,
    summary     = excluded.summary,
    due         = excluded.due;
";

        sqlx::query(SQL)
            .bind(&todo.uid)
            .bind(&todo.path)
            .bind(&todo.completed)
            .bind(&todo.description)
            .bind(todo.percent)
            .bind(todo.priority)
            .bind(&todo.status)
            .bind(&todo.summary)
            .bind(&todo.due)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get(&self, uid: &str) -> Result<Option<TodoRecord>, sqlx::Error> {
        const SQL: &str = "
SELECT uid, path, completed, description, percent, priority, status, summary, due
FROM todos
WHERE uid = ?;
";

        sqlx::query_as(SQL)
            .bind(uid)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list(
        &self,
        conds: &TodoConditions,
        sort: &[TodoSort],
        pager: &Pager,
    ) -> Result<Vec<TodoRecord>, sqlx::Error> {
        let mut where_clauses = Vec::new();
        if conds.status.is_some() {
            where_clauses.push("status = ?");
        }
        let due_before = conds.due_before();
        if due_before.is_some() {
            where_clauses.push("due <= ?");
        }

        let mut sql = "SELECT * FROM todos".to_string();
        if !where_clauses.is_empty() {
            sql += " WHERE ";
            sql += &where_clauses.join(" AND ");
        }

        if !sort.is_empty() {
            sql += " ORDER BY ";
            for (i, s) in sort.iter().enumerate() {
                match s {
                    TodoSort::Due(order) => {
                        sql += "due ";
                        sql += order.sql_keyword();
                    }
                    TodoSort::Priority { order, none_first } => {
                        sql += match none_first {
                            true => "priority ",
                            false => "((priority + 9) % 10) ",
                        };
                        sql += order.sql_keyword();
                    }
                }

                if i < sort.len() - 1 {
                    sql += ", ";
                }
            }
        }
        sql += " LIMIT ? OFFSET ?";

        let mut executable = sqlx::query_as(&sql);
        if let Some(status) = &conds.status {
            executable = executable.bind(AsRef::<str>::as_ref(status));
        }
        if let Some(due) = due_before {
            executable = executable.bind(format_ndt(due));
        }

        executable
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count(&self, conds: &TodoConditions) -> Result<i64, sqlx::Error> {
        let mut sql = "SELECT COUNT(*) FROM todos".to_string();

        let mut where_clauses = Vec::new();
        if conds.status.is_some() {
            where_clauses.push("status = ?");
        }
        let due_before = conds.due_before();
        if due_before.is_some() {
            where_clauses.push("due <= ?");
        }
        if !where_clauses.is_empty() {
            sql += " WHERE ";
            sql += &where_clauses.join(" AND ");
        }

        let mut executable = sqlx::query_as(&sql);
        if let Some(status) = &conds.status {
            let status: &str = status.as_ref();
            executable = executable.bind(status);
        }
        if let Some(due) = conds.due_before() {
            executable = executable.bind(format_ndt(due));
        }
        let row: (i64,) = executable.fetch_one(&self.pool).await?;
        Ok(row.0)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TodoRecord {
    uid: String,
    path: String,
    completed: String,
    description: String,
    percent: Option<u8>,
    priority: u8,
    status: String,
    summary: String,
    due: String,
}

impl TodoRecord {
    pub fn from<T: Todo>(path: String, todo: &T) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            uid: todo.uid().to_string(),
            path,
            summary: todo.summary().to_string(),
            description: todo.description().unwrap_or("").to_string(),
            due: todo
                .due()
                .map(|a| a.format_stable())
                .unwrap_or("".to_string()),
            completed: todo.completed().map(format_dt).unwrap_or("".to_string()),
            percent: todo.percent_complete(),
            priority: todo.priority().into(),
            status: todo
                .status()
                .as_ref()
                .map_or("".to_string(), ToString::to_string),
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Todo for TodoRecord {
    fn uid(&self) -> &str {
        &self.uid
    }

    fn completed(&self) -> Option<DateTime<Local>> {
        (!self.completed.is_empty())
            .then(|| DateTime::parse_from_rfc3339(&self.completed).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Local))
    }

    fn description(&self) -> Option<&str> {
        (!self.description.is_empty()).then_some(self.description.as_str())
    }

    fn due(&self) -> Option<LooseDateTime> {
        LooseDateTime::parse_stable(&self.due)
    }

    fn percent_complete(&self) -> Option<u8> {
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

// NOTE: The format strings used here are stable and should not change across different runs.
const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

fn format_ndt(ndt: NaiveDateTime) -> String {
    ndt.format(DATETIME_FORMAT).to_string()
}

fn format_dt(dt: DateTime<Local>) -> String {
    dt.format(DATETIME_FORMAT).to_string()
}
