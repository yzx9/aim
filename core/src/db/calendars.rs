// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use jiff::Zoned;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct Calendars {
    pool: SqlitePool,
}

impl Calendars {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, calendar: CalendarRecord) -> Result<(), sqlx::Error> {
        const SQL: &str = "\
INSERT INTO calendars (id, name, kind, priority, enabled, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(id) DO UPDATE SET
    name = excluded.name,
    kind = excluded.kind,
    priority = excluded.priority,
    enabled = excluded.enabled,
    updated_at = excluded.updated_at;
";

        sqlx::query(SQL)
            .bind(&calendar.id)
            .bind(&calendar.name)
            .bind(&calendar.kind)
            .bind(calendar.priority)
            .bind(calendar.enabled)
            .bind(&calendar.created_at)
            .bind(&calendar.updated_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<CalendarRecord>, sqlx::Error> {
        const SQL: &str = "\
SELECT id, name, kind, priority, enabled, created_at, updated_at
FROM calendars
WHERE id = ?;
";

        sqlx::query_as(SQL)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list(&self) -> Result<Vec<CalendarRecord>, sqlx::Error> {
        const SQL: &str = "\
SELECT id, name, kind, priority, enabled, created_at, updated_at
FROM calendars
ORDER BY priority ASC;
";

        sqlx::query_as(SQL).fetch_all(&self.pool).await
    }

    pub async fn list_enabled(&self) -> Result<Vec<CalendarRecord>, sqlx::Error> {
        const SQL: &str = "\
SELECT id, name, kind, priority, enabled, created_at, updated_at
FROM calendars
WHERE enabled = 1
ORDER BY priority ASC;
";

        sqlx::query_as(SQL).fetch_all(&self.pool).await
    }

    pub async fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), sqlx::Error> {
        const SQL: &str = "\
UPDATE calendars
SET enabled = ?, updated_at = ?
WHERE id = ?;
";

        let now = Zoned::now().strftime("%Y-%m-%dT%H:%M:%S%.f%:z").to_string();

        sqlx::query(SQL)
            .bind(enabled)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        const SQL: &str = "DELETE FROM calendars WHERE id = ?;";

        sqlx::query(SQL).bind(id).execute(&self.pool).await?;

        Ok(())
    }
}

/// Calendar record stored in database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CalendarRecord {
    /// Unique calendar identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Backend kind.
    pub kind: String,
    /// Lower numbers sort first.
    pub priority: i32,
    /// Whether the calendar is enabled for queries and backend initialization.
    pub enabled: bool,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

impl CalendarRecord {
    /// Creates a new calendar record with the given parameters.
    #[must_use]
    pub fn new(id: String, name: String, kind: String, priority: i32, enabled: bool) -> Self {
        let now = Zoned::now().strftime("%Y-%m-%dT%H:%M:%S%.f%:z").to_string();
        Self {
            id,
            name,
            kind,
            priority,
            enabled,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> crate::db::Db {
        crate::db::Db::open(None)
            .await
            .expect("Failed to create test database")
    }

    #[tokio::test]
    async fn calendars_insert_inserts_new_calendar() {
        let db = setup_test_db().await;

        let calendar = CalendarRecord::new(
            "test-calendar".to_string(),
            "Test Calendar".to_string(),
            "local".to_string(),
            0,
            true,
        );

        db.calendars
            .upsert(calendar)
            .await
            .expect("Failed to insert calendar");

        let retrieved = db
            .calendars
            .get("test-calendar")
            .await
            .expect("Failed to get calendar")
            .expect("Calendar not found");

        assert_eq!(retrieved.id, "test-calendar");
        assert_eq!(retrieved.name, "Test Calendar");
        assert_eq!(retrieved.kind, "local");
        assert_eq!(retrieved.priority, 0);
        assert!(retrieved.enabled);
    }

    #[tokio::test]
    async fn calendars_upsert_updates_existing_calendar() {
        let db = setup_test_db().await;

        let calendar = CalendarRecord::new(
            "test-calendar".to_string(),
            "Original Name".to_string(),
            "local".to_string(),
            0,
            true,
        );
        db.calendars.upsert(calendar).await.unwrap();

        let updated = CalendarRecord::new(
            "test-calendar".to_string(),
            "Updated Name".to_string(),
            "caldav".to_string(),
            5,
            false,
        );
        db.calendars.upsert(updated).await.unwrap();

        let retrieved = db.calendars.get("test-calendar").await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Name");
        assert_eq!(retrieved.kind, "caldav");
        assert_eq!(retrieved.priority, 5);
        assert!(!retrieved.enabled);
    }

    #[tokio::test]
    async fn calendars_list_returns_all_calendars() {
        let db = setup_test_db().await;

        // Default calendar exists from migration
        let calendars = db.calendars.list().await.unwrap();
        assert!(!calendars.is_empty());

        // Add more calendars
        for i in 1..=3 {
            let calendar = CalendarRecord::new(
                format!("calendar-{i}"),
                format!("Calendar {i}"),
                "local".to_string(),
                i,
                true,
            );
            db.calendars.upsert(calendar).await.unwrap();
        }

        let calendars = db.calendars.list().await.unwrap();
        assert!(calendars.len() >= 4); // default + 3 new
    }

    #[tokio::test]
    async fn calendars_list_enabled_returns_only_enabled() {
        let db = setup_test_db().await;

        // Add enabled and disabled calendars
        let enabled = CalendarRecord::new(
            "enabled-cal".to_string(),
            "Enabled".to_string(),
            "local".to_string(),
            0,
            true,
        );
        db.calendars.upsert(enabled).await.unwrap();

        let disabled = CalendarRecord::new(
            "disabled-cal".to_string(),
            "Disabled".to_string(),
            "local".to_string(),
            1,
            false,
        );
        db.calendars.upsert(disabled).await.unwrap();

        let enabled_calendars = db.calendars.list_enabled().await.unwrap();
        let ids: Vec<&str> = enabled_calendars.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"enabled-cal"));
        assert!(!ids.contains(&"disabled-cal"));
    }

    #[tokio::test]
    async fn calendars_set_enabled_toggles_enabled_flag() {
        let db = setup_test_db().await;

        let calendar = CalendarRecord::new(
            "test-cal".to_string(),
            "Test".to_string(),
            "local".to_string(),
            0,
            true,
        );
        db.calendars.upsert(calendar).await.unwrap();

        db.calendars.set_enabled("test-cal", false).await.unwrap();

        let retrieved = db.calendars.get("test-cal").await.unwrap().unwrap();
        assert!(!retrieved.enabled);

        db.calendars.set_enabled("test-cal", true).await.unwrap();

        let retrieved = db.calendars.get("test-cal").await.unwrap().unwrap();
        assert!(retrieved.enabled);
    }

    #[tokio::test]
    async fn calendars_delete_removes_calendar() {
        let db = setup_test_db().await;

        let calendar = CalendarRecord::new(
            "to-delete".to_string(),
            "To Delete".to_string(),
            "local".to_string(),
            0,
            true,
        );
        db.calendars.upsert(calendar).await.unwrap();

        db.calendars.delete("to-delete").await.unwrap();

        let retrieved = db.calendars.get("to-delete").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn calendars_list_orders_by_priority() {
        let db = setup_test_db().await;

        // Clear default calendar first for clean test
        db.calendars.delete("default").await.ok();

        // Add calendars with different priorities
        for (priority, name) in [(2, "Second"), (0, "First"), (1, "Middle")] {
            let calendar = CalendarRecord::new(
                name.to_lowercase(),
                name.to_string(),
                "local".to_string(),
                priority,
                true,
            );
            db.calendars.upsert(calendar).await.unwrap();
        }

        let calendars = db.calendars.list().await.unwrap();
        let names: Vec<&str> = calendars.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["First", "Middle", "Second"]);
    }
}
