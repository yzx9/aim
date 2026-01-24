// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use jiff::Zoned;
use sqlx::{Sqlite, SqlitePool, query::QueryAs, sqlite::SqliteArguments};
use std::borrow::Cow;

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
        if let Some(ref due) = conds.due {
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
        if let Some(ref due) = conds.due {
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
            completed: todo
                .completed()
                .map(|dt| format_dt(&dt))
                .unwrap_or_default(),
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
    fn uid(&self) -> Cow<'_, str> {
        self.uid.as_str().into()
    }

    fn completed(&self) -> Option<Zoned> {
        (!self.completed.is_empty())
            .then(|| Zoned::strptime(STABLE_FORMAT_LOCAL, &self.completed).ok())
            .flatten()
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        (!self.description.is_empty()).then_some(self.description.as_str().into())
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

    fn summary(&self) -> Cow<'_, str> {
        self.summary.as_str().into()
    }
}

fn format_dt(dt: &Zoned) -> String {
    dt.strftime(STABLE_FORMAT_LOCAL).to_string()
}

#[cfg(test)]
mod tests {
    use jiff::civil;
    use jiff::tz::TimeZone;

    use super::*;

    /// Test helper to create a test database
    async fn setup_test_db() -> crate::localdb::LocalDb {
        crate::localdb::LocalDb::open(None)
            .await
            .expect("Failed to create test database")
    }

    /// Test helper to create a test todo
    fn test_todo(uid: &str, summary: &str) -> crate::localdb::tests_utils::TestTodo {
        crate::localdb::tests_utils::test_todo(uid, summary)
    }

    #[tokio::test]
    async fn todos_upsert_inserts_new_todo() {
        // Arrange
        let db = setup_test_db().await;
        let todo = test_todo("todo-1", "Test Todo");
        let record = TodoRecord::from("/path/to/todo.ics".to_string(), &todo);

        // Act
        db.todos
            .upsert(&record)
            .await
            .expect("Failed to upsert todo");

        // Assert
        let retrieved = db
            .todos
            .get("todo-1")
            .await
            .expect("Failed to get todo")
            .expect("Todo not found");
        assert_eq!(retrieved.uid(), "todo-1");
        assert_eq!(retrieved.summary(), "Test Todo");
    }

    #[tokio::test]
    async fn todos_upsert_updates_existing_todo() {
        // Arrange
        let db = setup_test_db().await;
        let todo = test_todo("todo-1", "Original Summary");
        let record = TodoRecord::from("/path/to/todo.ics".to_string(), &todo);
        db.todos
            .upsert(&record)
            .await
            .expect("Failed to upsert todo");

        // Act
        let updated_todo = test_todo("todo-1", "Updated Summary");
        let updated_record = TodoRecord::from("/new/path/todo.ics".to_string(), &updated_todo);
        db.todos
            .upsert(&updated_record)
            .await
            .expect("Failed to update todo");

        // Assert
        let retrieved = db
            .todos
            .get("todo-1")
            .await
            .expect("Failed to get todo")
            .expect("Todo not found");
        assert_eq!(retrieved.uid(), "todo-1");
        assert_eq!(retrieved.summary(), "Updated Summary");
        assert_eq!(retrieved.path(), "/new/path/todo.ics");
    }

    #[tokio::test]
    async fn todos_get_returns_todo_by_uid() {
        // Arrange
        let db = setup_test_db().await;
        let todo = test_todo("todo-1", "Test Todo");
        let record = TodoRecord::from("/path/to/todo.ics".to_string(), &todo);
        db.todos
            .upsert(&record)
            .await
            .expect("Failed to upsert todo");

        // Act
        let retrieved = db.todos.get("todo-1").await.expect("Failed to get todo");

        // Assert
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().uid(), "todo-1");
    }

    #[tokio::test]
    async fn todos_get_returns_none_for_missing_uid() {
        // Arrange
        let db = setup_test_db().await;

        // Act
        let retrieved = db
            .todos
            .get("nonexistent")
            .await
            .expect("Failed to get todo");

        // Assert
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn todos_handles_optional_percent_complete() {
        // Arrange
        let db = setup_test_db().await;

        // Test with None
        let todo1 = test_todo("todo-1", "Test None");
        let record1 = TodoRecord::from("/path/to/todo1.ics".to_string(), &todo1);
        db.todos
            .upsert(&record1)
            .await
            .expect("Failed to upsert todo");

        // Test with 0
        let todo2 = test_todo("todo-2", "Test 0").with_percent_complete(0);
        let record2 = TodoRecord::from("/path/to/todo2.ics".to_string(), &todo2);
        db.todos
            .upsert(&record2)
            .await
            .expect("Failed to upsert todo");

        // Test with 100
        let todo3 = test_todo("todo-3", "Test 100").with_percent_complete(100);
        let record3 = TodoRecord::from("/path/to/todo3.ics".to_string(), &todo3);
        db.todos
            .upsert(&record3)
            .await
            .expect("Failed to upsert todo");

        // Assert
        let retrieved1 = db
            .todos
            .get("todo-1")
            .await
            .expect("Failed to get todo")
            .expect("Todo not found");
        assert_eq!(retrieved1.percent_complete(), None);

        let retrieved2 = db
            .todos
            .get("todo-2")
            .await
            .expect("Failed to get todo")
            .expect("Todo not found");
        assert_eq!(retrieved2.percent_complete(), Some(0));

        let retrieved3 = db
            .todos
            .get("todo-3")
            .await
            .expect("Failed to get todo")
            .expect("Todo not found");
        assert_eq!(retrieved3.percent_complete(), Some(100));
    }

    #[tokio::test]
    async fn todos_handles_all_priority_levels() {
        // Arrange
        let db = setup_test_db().await;

        // Test various priority levels
        for (i, priority) in [
            Priority::None,
            Priority::P1,
            Priority::P2,
            Priority::P5,
            Priority::P8,
            Priority::P9,
        ]
        .iter()
        .enumerate()
        {
            let uid = format!("todo-{}", i + 1);
            let todo = test_todo(&uid, "Test Todo").with_priority(*priority);
            let record = TodoRecord::from(format!("/path/to/todo{}.ics", i + 1), &todo);
            db.todos
                .upsert(&record)
                .await
                .expect("Failed to upsert todo");

            let retrieved = db
                .todos
                .get(&uid)
                .await
                .expect("Failed to get todo")
                .expect("Todo not found");
            assert_eq!(retrieved.priority(), *priority);
        }
    }

    #[tokio::test]
    async fn todos_list_returns_all_todos() {
        // Arrange
        let db = setup_test_db().await;
        let todo1 = test_todo("todo-1", "Todo 1");
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo1))
            .await
            .unwrap();
        let todo2 = test_todo("todo-2", "Todo 2");
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo2))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: None,
        };
        let sort = vec![];
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn todos_list_filters_by_status() {
        // Arrange
        let db = setup_test_db().await;
        let todo_needs_action =
            test_todo("todo-1", "Needs Action").with_status(TodoStatus::NeedsAction);
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo_needs_action))
            .await
            .unwrap();

        let todo_completed = test_todo("todo-2", "Completed").with_status(TodoStatus::Completed);
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo_completed))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: Some(TodoStatus::NeedsAction),
            due: None,
        };
        let sort = vec![];
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uid(), "todo-1");
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn todos_list_filters_by_due_date() {
        // Arrange
        let db = setup_test_db().await;
        let cutoff = civil::date(2025, 1, 15)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let todo_before = test_todo("todo-1", "Before Due").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 10)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo_before))
            .await
            .unwrap();

        let todo_after = test_todo("todo-2", "After Due").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 20)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo_after))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: Some(cutoff),
        };
        let sort = vec![];
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uid(), "todo-1");
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn todos_list_filters_by_both_conditions() {
        // Arrange
        let db = setup_test_db().await;
        let cutoff = civil::date(2025, 1, 15)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let todo_matching = test_todo("todo-1", "Matching")
            .with_status(TodoStatus::NeedsAction)
            .with_due(LooseDateTime::Local(
                civil::date(2025, 1, 10)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo_matching))
            .await
            .unwrap();

        let todo_wrong_status = test_todo("todo-2", "Wrong Status")
            .with_status(TodoStatus::Completed)
            .with_due(LooseDateTime::Local(
                civil::date(2025, 1, 10)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo_wrong_status))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: Some(TodoStatus::NeedsAction),
            due: Some(cutoff),
        };
        let sort = vec![];
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uid(), "todo-1");
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn todos_list_sorts_by_due_asc() {
        // Arrange
        let db = setup_test_db().await;
        let todo1 = test_todo("todo-1", "Third").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 30)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo1))
            .await
            .unwrap();

        let todo2 = test_todo("todo-2", "First").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 10)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo2))
            .await
            .unwrap();

        let todo3 = test_todo("todo-3", "Second").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 20)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path3.ics".into(), &todo3))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: None,
        };
        let sort = vec![ResolvedTodoSort::Due(crate::SortOrder::Asc)];
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].uid(), "todo-2");
        assert_eq!(results[1].uid(), "todo-3");
        assert_eq!(results[2].uid(), "todo-1");
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn todos_list_sorts_by_due_desc() {
        // Arrange
        let db = setup_test_db().await;
        let todo1 = test_todo("todo-1", "Third").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 30)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo1))
            .await
            .unwrap();

        let todo2 = test_todo("todo-2", "First").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 10)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo2))
            .await
            .unwrap();

        let todo3 = test_todo("todo-3", "Second").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 20)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path3.ics".into(), &todo3))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: None,
        };
        let sort = vec![ResolvedTodoSort::Due(crate::SortOrder::Desc)];
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].uid(), "todo-1");
        assert_eq!(results[1].uid(), "todo-3");
        assert_eq!(results[2].uid(), "todo-2");
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn todos_list_sorts_by_priority_with_none_first() {
        // Arrange
        let db = setup_test_db().await;
        let todo1 = test_todo("todo-1", "None Priority").with_priority(Priority::None);
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo1))
            .await
            .unwrap();

        let todo2 = test_todo("todo-2", "High Priority").with_priority(Priority::P2);
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo2))
            .await
            .unwrap();

        let todo3 = test_todo("todo-3", "Low Priority").with_priority(Priority::P8);
        db.todos
            .upsert(&TodoRecord::from("/path3.ics".into(), &todo3))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: None,
        };
        let sort = vec![ResolvedTodoSort::Priority {
            order: crate::SortOrder::Asc,
            none_first: true,
        }];
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert - None (0) first, then P2 (2), then P8 (8)
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].uid(), "todo-1");
        assert_eq!(results[1].uid(), "todo-2");
        assert_eq!(results[2].uid(), "todo-3");
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn todos_list_sorts_by_priority_with_none_last() {
        // Arrange
        let db = setup_test_db().await;
        let todo1 = test_todo("todo-1", "None Priority").with_priority(Priority::None);
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo1))
            .await
            .unwrap();

        let todo2 = test_todo("todo-2", "High Priority").with_priority(Priority::P2);
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo2))
            .await
            .unwrap();

        let todo3 = test_todo("todo-3", "Low Priority").with_priority(Priority::P8);
        db.todos
            .upsert(&TodoRecord::from("/path3.ics".into(), &todo3))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: None,
        };
        let sort = vec![ResolvedTodoSort::Priority {
            order: crate::SortOrder::Asc,
            none_first: false,
        }];
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert - With (priority + 9) % 10: P2 (1), P8 (7), None (9)
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].uid(), "todo-2");
        assert_eq!(results[1].uid(), "todo-3");
        assert_eq!(results[2].uid(), "todo-1");
    }

    #[tokio::test]
    async fn todos_list_respects_limit() {
        // Arrange
        let db = setup_test_db().await;
        for i in 1..=5 {
            let todo = test_todo(&format!("todo-{i}"), &format!("Todo {i}"));
            db.todos
                .upsert(&TodoRecord::from(format!("/path{i}.ics"), &todo))
                .await
                .unwrap();
        }

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: None,
        };
        let sort = vec![];
        let pager = Pager {
            limit: 3,
            offset: 0,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn todos_list_respects_offset() {
        // Arrange
        let db = setup_test_db().await;
        for i in 1..=5 {
            let todo = test_todo(&format!("todo-{i}"), &format!("Todo {i}"));
            db.todos
                .upsert(&TodoRecord::from(format!("/path{i}.ics"), &todo))
                .await
                .unwrap();
        }

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: None,
        };
        let sort = vec![];
        let pager = Pager {
            limit: 10,
            offset: 2,
        };
        let results = db.todos.list(&conds, &sort, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn todos_count_returns_total_count() {
        // Arrange
        let db = setup_test_db().await;
        for i in 1..=5 {
            let todo = test_todo(&format!("todo-{i}"), &format!("Todo {i}"));
            db.todos
                .upsert(&TodoRecord::from(format!("/path{i}.ics"), &todo))
                .await
                .unwrap();
        }

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: None,
        };
        let count = db.todos.count(&conds).await.unwrap();

        // Assert
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn todos_count_filters_by_status() {
        // Arrange
        let db = setup_test_db().await;
        let todo_needs_action =
            test_todo("todo-1", "Needs Action").with_status(TodoStatus::NeedsAction);
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo_needs_action))
            .await
            .unwrap();

        let todo_completed = test_todo("todo-2", "Completed").with_status(TodoStatus::Completed);
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo_completed))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: Some(TodoStatus::NeedsAction),
            due: None,
        };
        let count = db.todos.count(&conds).await.unwrap();

        // Assert
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn todos_count_filters_by_due_date() {
        // Arrange
        let db = setup_test_db().await;
        let cutoff = civil::date(2025, 1, 15)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let todo_before = test_todo("todo-1", "Before Due").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 10)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo_before))
            .await
            .unwrap();

        let todo_after = test_todo("todo-2", "After Due").with_due(LooseDateTime::Local(
            civil::date(2025, 1, 20)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo_after))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: None,
            due: Some(cutoff),
        };
        let count = db.todos.count(&conds).await.unwrap();

        // Assert
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn todos_count_filters_by_both_conditions() {
        // Arrange
        let db = setup_test_db().await;
        let cutoff = civil::date(2025, 1, 15)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let todo_matching = test_todo("todo-1", "Matching")
            .with_status(TodoStatus::NeedsAction)
            .with_due(LooseDateTime::Local(
                civil::date(2025, 1, 10)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.todos
            .upsert(&TodoRecord::from("/path1.ics".into(), &todo_matching))
            .await
            .unwrap();

        let todo_wrong_status = test_todo("todo-2", "Wrong Status")
            .with_status(TodoStatus::Completed)
            .with_due(LooseDateTime::Local(
                civil::date(2025, 1, 10)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.todos
            .upsert(&TodoRecord::from("/path2.ics".into(), &todo_wrong_status))
            .await
            .unwrap();

        // Act
        let conds = ResolvedTodoConditions {
            status: Some(TodoStatus::NeedsAction),
            due: Some(cutoff),
        };
        let count = db.todos.count(&conds).await.unwrap();

        // Assert
        assert_eq!(count, 1);
    }
}
