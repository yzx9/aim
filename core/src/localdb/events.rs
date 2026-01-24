// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;

use jiff::{Zoned, civil::Date};
use sqlx::{Sqlite, SqlitePool, query::QueryAs, sqlite::SqliteArguments};

use crate::datetime::{STABLE_FORMAT_DATEONLY, STABLE_FORMAT_LOCAL};
use crate::event::ResolvedEventConditions;
use crate::{Event, EventStatus, LooseDateTime, Pager};

#[derive(Debug, Clone)]
pub struct Events {
    pool: SqlitePool,
}

impl Events {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
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
        conds: &ResolvedEventConditions,
        pager: &Pager,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        let mut sql = "\
SELECT uid, path, summary, description, status, start, end
FROM events
"
        .to_string();
        sql += &Self::build_where(conds);
        sql += "ORDER BY start ASC LIMIT ? OFFSET ?;";

        let mut executable = sqlx::query_as(&sql);
        executable = Self::bind_conditions(conds, executable);

        executable
            .bind(pager.limit)
            .bind(pager.offset)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count(&self, conds: &ResolvedEventConditions) -> Result<i64, sqlx::Error> {
        let mut sql = "SELECT COUNT(*) FROM events".to_string();
        sql += &Self::build_where(conds);
        sql += ";";

        let mut executable = sqlx::query_as(&sql);
        executable = Self::bind_conditions(conds, executable);

        let row: (i64,) = executable.fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    fn build_where(conds: &ResolvedEventConditions) -> String {
        let mut where_clauses = Vec::new();
        if conds.start_before.is_some() {
            where_clauses.push("start <= ?");
        }
        if conds.end_after.is_some() {
            where_clauses.push("(end >= ? OR end = ?)");
        }

        if where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {} ", where_clauses.join(" AND "))
        }
    }

    fn bind_conditions<'a, O>(
        conds: &'a ResolvedEventConditions,
        mut query: QueryAs<'a, Sqlite, O, SqliteArguments<'a>>,
    ) -> QueryAs<'a, Sqlite, O, SqliteArguments<'a>> {
        if let Some(ref start_before) = conds.start_before {
            query = query.bind(format_dt(start_before));
        }
        if let Some(ref end_after) = conds.end_after {
            query = query
                .bind(format_dt(end_after))
                .bind(format_date(end_after.date()));
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
    pub fn from<E: Event>(path: String, event: &E) -> Self {
        Self {
            uid: event.uid().to_string(),
            path,
            summary: event.summary().to_string(),
            description: event
                .description()
                .map(|a| a.to_string())
                .unwrap_or_default(),
            status: event.status().map(|s| s.to_string()).unwrap_or_default(),
            start: event.start().map(|a| a.format_stable()).unwrap_or_default(),
            end: event.end().map(|a| a.format_stable()).unwrap_or_default(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Event for EventRecord {
    fn uid(&self) -> Cow<'_, str> {
        (&self.uid).into()
    }

    fn summary(&self) -> Cow<'_, str> {
        (&self.summary).into()
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        (!self.description.is_empty()).then_some(self.description.as_str().into())
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

fn format_date(date: Date) -> String {
    date.strftime(STABLE_FORMAT_DATEONLY).to_string()
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

    /// Test helper to create a test event
    fn test_event(uid: &str, summary: &str) -> crate::localdb::tests_utils::TestEvent {
        crate::localdb::tests_utils::test_event(uid, summary)
    }

    #[tokio::test]
    async fn events_insert_inserts_new_event() {
        // Arrange
        let db = setup_test_db().await;
        let event = test_event("event-1", "Test Event");
        let record = EventRecord::from("/path/to/event.ics".to_string(), &event);

        // Act
        db.events
            .insert(record)
            .await
            .expect("Failed to insert event");

        // Assert
        let retrieved = db
            .events
            .get("event-1")
            .await
            .expect("Failed to get event")
            .expect("Event not found");
        assert_eq!(retrieved.uid(), "event-1");
        assert_eq!(retrieved.summary(), "Test Event");
    }

    #[tokio::test]
    async fn events_insert_updates_existing_event() {
        // Arrange
        let db = setup_test_db().await;
        let event = test_event("event-1", "Original Summary");
        let record = EventRecord::from("/path/to/event.ics".to_string(), &event);
        db.events
            .insert(record)
            .await
            .expect("Failed to insert event");

        // Act
        let updated_event = test_event("event-1", "Updated Summary");
        let updated_record = EventRecord::from("/new/path/event.ics".to_string(), &updated_event);
        db.events
            .insert(updated_record)
            .await
            .expect("Failed to update event");

        // Assert
        let retrieved = db
            .events
            .get("event-1")
            .await
            .expect("Failed to get event")
            .expect("Event not found");
        assert_eq!(retrieved.uid(), "event-1");
        assert_eq!(retrieved.summary(), "Updated Summary");
        assert_eq!(retrieved.path(), "/new/path/event.ics");
    }

    #[tokio::test]
    async fn events_get_returns_event_by_uid() {
        // Arrange
        let db = setup_test_db().await;
        let event = test_event("event-1", "Test Event");
        let record = EventRecord::from("/path/to/event.ics".to_string(), &event);
        db.events
            .insert(record)
            .await
            .expect("Failed to insert event");

        // Act
        let retrieved = db.events.get("event-1").await.expect("Failed to get event");

        // Assert
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().uid(), "event-1");
    }

    #[tokio::test]
    async fn events_get_returns_none_for_missing_uid() {
        // Arrange
        let db = setup_test_db().await;

        // Act
        let retrieved = db
            .events
            .get("nonexistent")
            .await
            .expect("Failed to get event");

        // Assert
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn events_handles_empty_optional_fields() {
        // Arrange
        let db = setup_test_db().await;
        let event = test_event("event-1", "Test Event");
        let record = EventRecord::from("/path/to/event.ics".to_string(), &event);

        // Act
        db.events
            .insert(record)
            .await
            .expect("Failed to insert event");

        // Assert
        let retrieved = db
            .events
            .get("event-1")
            .await
            .expect("Failed to get event")
            .expect("Event not found");
        assert_eq!(retrieved.description(), None);
        assert_eq!(retrieved.status(), None);
        assert_eq!(retrieved.start(), None);
        assert_eq!(retrieved.end(), None);
    }

    #[tokio::test]
    async fn events_list_returns_all_events() {
        // Arrange
        let db = setup_test_db().await;
        let event1 = test_event("event-1", "Event 1");
        db.events
            .insert(EventRecord::from("/path1.ics".into(), &event1))
            .await
            .unwrap();
        let event2 = test_event("event-2", "Event 2");
        db.events
            .insert(EventRecord::from("/path2.ics".into(), &event2))
            .await
            .unwrap();

        // Act
        let conds = ResolvedEventConditions {
            start_before: None,
            end_after: None,
        };
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.events.list(&conds, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn events_list_filters_by_start_before() {
        // Arrange
        let db = setup_test_db().await;
        let cutoff = civil::date(2025, 1, 15)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let event_before = test_event("event-1", "Before Event").with_start(LooseDateTime::Local(
            civil::date(2025, 1, 10)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.events
            .insert(EventRecord::from("/path1.ics".into(), &event_before))
            .await
            .unwrap();

        let event_after = test_event("event-2", "After Event").with_start(LooseDateTime::Local(
            civil::date(2025, 1, 20)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.events
            .insert(EventRecord::from("/path2.ics".into(), &event_after))
            .await
            .unwrap();

        // Act
        let conds = ResolvedEventConditions {
            start_before: Some(cutoff),
            end_after: None,
        };
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.events.list(&conds, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uid(), "event-1");
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn events_list_filters_by_both_conditions() {
        // Arrange
        let db = setup_test_db().await;
        let start_cutoff = civil::date(2025, 1, 15)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let end_after = civil::date(2025, 1, 10)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let matching_event = test_event("event-1", "Matching Event")
            .with_start(LooseDateTime::Local(
                civil::date(2025, 1, 12)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ))
            .with_end(LooseDateTime::Local(
                civil::date(2025, 1, 14)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.events
            .insert(EventRecord::from("/path1.ics".into(), &matching_event))
            .await
            .unwrap();

        let non_matching_event =
            test_event("event-2", "Non-Matching Event").with_start(LooseDateTime::Local(
                civil::date(2025, 1, 20)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.events
            .insert(EventRecord::from("/path2.ics".into(), &non_matching_event))
            .await
            .unwrap();

        // Act
        let conds = ResolvedEventConditions {
            start_before: Some(start_cutoff),
            end_after: Some(end_after),
        };
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.events.list(&conds, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uid(), "event-1");
    }

    #[tokio::test]
    async fn events_list_respects_limit() {
        // Arrange
        let db = setup_test_db().await;
        for i in 1..=5 {
            let event = test_event(&format!("event-{i}"), &format!("Event {i}"));
            db.events
                .insert(EventRecord::from(format!("/path{i}.ics"), &event))
                .await
                .unwrap();
        }

        // Act
        let conds = ResolvedEventConditions {
            start_before: None,
            end_after: None,
        };
        let pager = Pager {
            limit: 3,
            offset: 0,
        };
        let results = db.events.list(&conds, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn events_list_respects_offset() {
        // Arrange
        let db = setup_test_db().await;
        for i in 1..=5 {
            let event = test_event(&format!("event-{i}"), &format!("Event {i}"));
            db.events
                .insert(EventRecord::from(format!("/path{i}.ics"), &event))
                .await
                .unwrap();
        }

        // Act
        let conds = ResolvedEventConditions {
            start_before: None,
            end_after: None,
        };
        let pager = Pager {
            limit: 10,
            offset: 2,
        };
        let results = db.events.list(&conds, &pager).await.unwrap();

        // Assert
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    #[expect(clippy::indexing_slicing)]
    async fn events_list_orders_by_start_time() {
        // Arrange
        let db = setup_test_db().await;
        let event1 = test_event("event-1", "Third Event").with_start(LooseDateTime::Local(
            civil::date(2025, 1, 30)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.events
            .insert(EventRecord::from("/path1.ics".into(), &event1))
            .await
            .unwrap();

        let event2 = test_event("event-2", "First Event").with_start(LooseDateTime::Local(
            civil::date(2025, 1, 10)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.events
            .insert(EventRecord::from("/path2.ics".into(), &event2))
            .await
            .unwrap();

        let event3 = test_event("event-3", "Second Event").with_start(LooseDateTime::Local(
            civil::date(2025, 1, 20)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.events
            .insert(EventRecord::from("/path3.ics".into(), &event3))
            .await
            .unwrap();

        // Act
        let conds = ResolvedEventConditions {
            start_before: None,
            end_after: None,
        };
        let pager = Pager {
            limit: 10,
            offset: 0,
        };
        let results = db.events.list(&conds, &pager).await.unwrap();

        // Assert - results should be ordered by start time ASC
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].uid(), "event-2");
        assert_eq!(results[1].uid(), "event-3");
        assert_eq!(results[2].uid(), "event-1");
    }

    #[tokio::test]
    async fn events_count_returns_total_count() {
        // Arrange
        let db = setup_test_db().await;
        for i in 1..=5 {
            let event = test_event(&format!("event-{i}"), &format!("Event {i}"));
            db.events
                .insert(EventRecord::from(format!("/path{i}.ics"), &event))
                .await
                .unwrap();
        }

        // Act
        let conds = ResolvedEventConditions {
            start_before: None,
            end_after: None,
        };
        let count = db.events.count(&conds).await.unwrap();

        // Assert
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn events_count_filters_by_start_before() {
        // Arrange
        let db = setup_test_db().await;
        let cutoff = civil::date(2025, 1, 15)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let event_before = test_event("event-1", "Before Event").with_start(LooseDateTime::Local(
            civil::date(2025, 1, 10)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.events
            .insert(EventRecord::from("/path1.ics".into(), &event_before))
            .await
            .unwrap();

        let event_after = test_event("event-2", "After Event").with_start(LooseDateTime::Local(
            civil::date(2025, 1, 20)
                .at(0, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        ));
        db.events
            .insert(EventRecord::from("/path2.ics".into(), &event_after))
            .await
            .unwrap();

        // Act
        let conds = ResolvedEventConditions {
            start_before: Some(cutoff),
            end_after: None,
        };
        let count = db.events.count(&conds).await.unwrap();

        // Assert
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn events_count_filters_by_end_after() {
        // Arrange
        let db = setup_test_db().await;
        let end_after = civil::date(2025, 1, 15)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let event_matching = test_event("event-1", "Matching Event")
            .with_start(LooseDateTime::Local(
                civil::date(2025, 1, 10)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ))
            .with_end(LooseDateTime::Local(
                civil::date(2025, 1, 20)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.events
            .insert(EventRecord::from("/path1.ics".into(), &event_matching))
            .await
            .unwrap();

        let event_non_matching = test_event("event-2", "Non-Matching Event")
            .with_start(LooseDateTime::Local(
                civil::date(2025, 1, 10)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ))
            .with_end(LooseDateTime::Local(
                civil::date(2025, 1, 12)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.events
            .insert(EventRecord::from("/path2.ics".into(), &event_non_matching))
            .await
            .unwrap();

        // Act
        let conds = ResolvedEventConditions {
            start_before: None,
            end_after: Some(end_after),
        };
        let count = db.events.count(&conds).await.unwrap();

        // Assert
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn events_count_filters_by_both_conditions() {
        // Arrange
        let db = setup_test_db().await;
        let start_cutoff = civil::date(2025, 1, 15)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let end_after = civil::date(2025, 1, 10)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let matching_event = test_event("event-1", "Matching Event")
            .with_start(LooseDateTime::Local(
                civil::date(2025, 1, 12)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ))
            .with_end(LooseDateTime::Local(
                civil::date(2025, 1, 14)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.events
            .insert(EventRecord::from("/path1.ics".into(), &matching_event))
            .await
            .unwrap();

        let non_matching_event =
            test_event("event-2", "Non-Matching Event").with_start(LooseDateTime::Local(
                civil::date(2025, 1, 20)
                    .at(0, 0, 0, 0)
                    .to_zoned(TimeZone::UTC)
                    .unwrap(),
            ));
        db.events
            .insert(EventRecord::from("/path2.ics".into(), &non_matching_event))
            .await
            .unwrap();

        // Act
        let conds = ResolvedEventConditions {
            start_before: Some(start_cutoff),
            end_after: Some(end_after),
        };
        let count = db.events.count(&conds).await.unwrap();

        // Assert
        assert_eq!(count, 1);
    }
}
