// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, Local};
use sqlx::{Sqlite, SqlitePool, query::QueryAs, sqlite::SqliteArguments};

use crate::datetime::STABLE_FORMAT_LOCAL;
use crate::todo::{ResolvedTodoConditions, ResolvedTodoSort};
use crate::{LooseDateTime, Pager, Priority, Todo, TodoStatus};

#[derive(Debug, Clone)]
pub struct Todos {
    pool: SqlitePool,
}

impl Todos {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, todo: &TodoRecord) -> Result<(), sqlx::Error> {
        const SQL: &str = "\
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
        const SQL: &str = "\
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
        conds: &ResolvedTodoConditions,
        sort: &[ResolvedTodoSort],
        pager: &Pager,
    ) -> Result<Vec<TodoRecord>, sqlx::Error> {
        let mut sql = "\
SELECT uid, path, completed, description, percent, priority, status, summary, due
FROM todos
"
        .to_string();
        sql += &Self::build_where(conds);

        if !sort.is_empty() {
            sql += "ORDER BY ";
            for (i, s) in sort.iter().enumerate() {
                match s {
                    ResolvedTodoSort::Due(order) => {
                        sql += "due ";
                        sql += order.sql_keyword();
                    }
                    ResolvedTodoSort::Priority { order, none_first } => {
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
        sql += " LIMIT ? OFFSET ?;";

        let mut executable = sqlx::query_as(&sql);
        if let Some(status) = &conds.status {
            executable = executable.bind(AsRef::<str>::as_ref(status));
        }
        if let Some(due) = conds.due {
            executable = executable.bind(format_dt(due));
        }

        executable
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count(&self, conds: &ResolvedTodoConditions) -> Result<i64, sqlx::Error> {
        let mut sql = "SELECT COUNT(*) FROM todos".to_string();
        sql += &Self::build_where(conds);
        sql += ";";

        let mut query = sqlx::query_as(&sql);
        query = Self::bind_conditions(conds, query);
        let row: (i64,) = query.fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    fn build_where(conds: &ResolvedTodoConditions) -> String {
        let mut where_clauses = Vec::new();
        if conds.status.is_some() {
            where_clauses.push("status = ?");
        }
        if conds.due.is_some() {
            where_clauses.push("due <= ?");
        }

        if where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {} ", where_clauses.join(" AND "))
        }
    }

    fn bind_conditions<'a, O>(
        conds: &'a ResolvedTodoConditions,
        mut query: QueryAs<'a, Sqlite, O, SqliteArguments<'a>>,
    ) -> QueryAs<'a, Sqlite, O, SqliteArguments<'a>> {
        if let Some(status) = &conds.status {
            let status: &str = status.as_ref();
            query = query.bind(status);
        }
        if let Some(due) = conds.due {
            query = query.bind(format_dt(due));
        }
        query
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
    pub fn from<T: Todo>(path: String, todo: &T) -> Self {
        Self {
            uid: todo.uid().to_string(),
            path,
            summary: todo.summary().to_string(),
            description: todo.description().unwrap_or_default().to_string(),
            due: todo.due().map(|a| a.format_stable()).unwrap_or_default(),
            completed: todo.completed().map(format_dt).unwrap_or_default(),
            percent: todo.percent_complete(),
            priority: todo.priority().into(),
            status: todo.status().to_string(),
        }
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

    fn status(&self) -> TodoStatus {
        self.status.as_str().parse().unwrap_or_default()
    }

    fn summary(&self) -> &str {
        &self.summary
    }
}

fn format_dt(dt: DateTime<Local>) -> String {
    dt.format(STABLE_FORMAT_LOCAL).to_string()
}
