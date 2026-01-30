// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroU32;

use sqlx::SqlitePool;

use crate::{Kind, short_id::UidAndShortId};

#[derive(Debug, Clone)]
pub struct ShortIds {
    pool: SqlitePool,
}

impl ShortIds {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_by_short_id(
        &self,
        short_id: NonZeroU32,
    ) -> Result<Option<UidAndShortId>, sqlx::Error> {
        let row: Option<(String, String)> =
            sqlx::query_as("SELECT uid, kind FROM short_ids WHERE short_id = ?;")
                .bind(i64::from(short_id.get()))
                .fetch_optional(&self.pool)
                .await?;

        match row {
            Some((uid, kind)) => Ok(if let Some(kind) = Kind::parse_stable(&kind) { Some(UidAndShortId {
                uid,
                short_id,
                kind,
            }) } else {
                tracing::warn!(kind, "unknown short_id kind");
                None
            }),
            None => Ok(None),
        }
    }

    pub async fn get_or_assign_short_id(
        &self,
        uid: &str,
        kind: Kind,
    ) -> Result<NonZeroU32, sqlx::Error> {
        // In SQLite, every table (unless declared WITHOUT ROWID) maintains a hidden ROWID column.
        //
        // When a column is defined as `INTEGER PRIMARY KEY`, it becomes an alias for the ROWID,
        // and SQLite will automatically assign it a value one greater than the current maximum.
        //
        // `AUTOINCREMENT` is an alternative that guarantees IDs are never reused, even after
        // deletions or conflicts. However, unlike ROWID, it may reserve or skip IDs when an insert
        // fails or is ignored due to a conflict.
        //
        // In our case, we prefer `short_id` values to remain as small and compact as possible,
        // so we intentionally avoid using AUTOINCREMENT.
        const SQL: &str = "\
INSERT INTO short_ids (uid, kind) VALUES (?, ?)
ON CONFLICT(uid) DO NOTHING
RETURNING short_id;
";

        if let Some((short_id,)) = sqlx::query_as::<_, (NonZeroU32,)>(SQL)
            .bind(uid)
            .bind(kind.to_str_stable())
            .fetch_optional(&self.pool)
            .await?
        {
            return Ok(short_id);
        }

        // if the insert did not return a short_id, it means the uid already exists
        let (short_id,): (NonZeroU32,) =
            sqlx::query_as("SELECT short_id FROM short_ids WHERE uid = ?")
                .bind(uid)
                .fetch_one(&self.pool)
                .await?;

        Ok(short_id)
    }

    /// Truncate the `short_ids` table, removing all entries.
    pub async fn truncate(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM short_ids;")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::{
        Event as EventTrait, EventStatus, Id, LooseDateTime, Priority, Todo as TodoTrait,
        TodoStatus,
    };

    // Re-export ical types for tests
    use aimcal_ical::{
        Completed, Description, DtEnd, DtStamp, DtStart, Due, PercentComplete,
        Priority as IcalPriority, Summary, Uid, VEvent, VTodo,
    };

    // Re-export EventStatus and TodoStatus from ical
    use aimcal_ical::{
        EventStatus as IcalEventStatus, EventStatusValue, TodoStatus as IcalTodoStatus,
        TodoStatusValue,
    };

    // Re-export ShortIds wrapper types
    use crate::short_id::ShortIds;

    /// Test helper to create a test database
    async fn setup_test_db() -> crate::localdb::LocalDb {
        crate::localdb::LocalDb::open(None)
            .await
            .expect("Failed to create test database")
    }

    /// Helper to create a test `VEvent`
    fn test_event(uid: &str, summary: &str) -> VEvent<String> {
        let now = jiff::Zoned::now();
        VEvent {
            uid: Uid::new(uid.to_string()),
            dt_stamp: DtStamp::new(now.datetime()),
            dt_start: DtStart::new(LooseDateTime::Local(now.clone())),
            dt_end: Some(DtEnd::new(LooseDateTime::Local(now.clone()))),
            duration: None,
            summary: Some(Summary::new(summary.to_string())),
            description: Some(Description::new("Test description".to_string())),
            status: Some(IcalEventStatus::new(EventStatusValue::Confirmed)),
            location: None,
            geo: None,
            url: None,
            organizer: None,
            attendees: Vec::new(),
            last_modified: None,
            transparency: None,
            sequence: None,
            priority: None,
            classification: None,
            resources: None,
            categories: None,
            rrule: None,
            rdates: Vec::new(),
            ex_dates: Vec::new(),
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
            alarms: Vec::new(),
        }
    }

    /// Helper to create a test `VTodo`
    fn test_todo(uid: &str, summary: &str) -> VTodo<String> {
        let now = jiff::Zoned::now();
        let utc_now = now.with_time_zone(jiff::tz::TimeZone::UTC);
        VTodo {
            uid: Uid::new(uid.to_string()),
            dt_stamp: DtStamp::new(now.datetime()),
            dt_start: None,
            due: Some(Due::new(LooseDateTime::Local(now.clone()))),
            completed: Some(Completed::new(utc_now.datetime())),
            duration: None,
            summary: Some(Summary::new(summary.to_string())),
            description: Some(Description::new("Test todo description".to_string())),
            location: None,
            geo: None,
            url: None,
            organizer: None,
            attendees: Vec::new(),
            last_modified: None,
            status: Some(IcalTodoStatus::new(TodoStatusValue::NeedsAction)),
            sequence: None,
            priority: Some(IcalPriority::new(5)),
            percent_complete: Some(PercentComplete::new(0)),
            classification: None,
            resources: None,
            categories: None,
            rrule: None,
            rdates: Vec::new(),
            ex_dates: Vec::new(),
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
            alarms: Vec::new(),
        }
    }

    #[tokio::test]
    async fn short_ids_get_or_assign_short_id_assigns_new_id() {
        // Arrange
        let db = setup_test_db().await;
        let uid = "test-uid-1";

        // Act
        let short_id = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Todo)
            .await
            .expect("Failed to assign short ID");

        // Assert
        assert_eq!(short_id.get(), 1);
    }

    #[tokio::test]
    async fn short_ids_get_or_assign_short_id_returns_existing_id() {
        // Arrange
        let db = setup_test_db().await;
        let uid = "test-uid-1";
        let first_id = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Todo)
            .await
            .expect("Failed to assign short ID");

        // Act
        let second_id = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Todo)
            .await
            .expect("Failed to get short ID");

        // Assert
        assert_eq!(first_id, second_id);
        assert_eq!(first_id.get(), 1);
    }

    #[tokio::test]
    async fn short_ids_get_by_short_id_returns_correct_data() {
        // Arrange
        let db = setup_test_db().await;
        let uid = "test-uid-1";
        let short_id = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Event)
            .await
            .expect("Failed to assign short ID");

        // Act
        let result = db
            .short_ids
            .get_by_short_id(short_id)
            .await
            .expect("Failed to get by short ID");

        // Assert
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.uid, uid);
        assert_eq!(data.short_id, short_id);
        assert_eq!(data.kind, Kind::Event);
    }

    #[tokio::test]
    async fn short_ids_get_by_short_id_returns_none_for_missing_id() {
        // Arrange
        let db = setup_test_db().await;

        // Act
        let result = db
            .short_ids
            .get_by_short_id(NonZeroU32::new(999).unwrap())
            .await
            .expect("Failed to get by short ID");

        // Assert
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn short_ids_get_or_assign_short_id_increments_for_new_uids() {
        // Arrange
        let db = setup_test_db().await;

        // Act
        let id1 = db
            .short_ids
            .get_or_assign_short_id("uid-1", Kind::Todo)
            .await
            .expect("Failed to assign short ID");
        let id2 = db
            .short_ids
            .get_or_assign_short_id("uid-2", Kind::Todo)
            .await
            .expect("Failed to assign short ID");
        let id3 = db
            .short_ids
            .get_or_assign_short_id("uid-3", Kind::Event)
            .await
            .expect("Failed to assign short ID");

        // Assert
        assert_eq!(id1.get(), 1);
        assert_eq!(id2.get(), 2);
        assert_eq!(id3.get(), 3);
    }

    #[tokio::test]
    async fn short_ids_handles_same_uid_with_same_kind() {
        // Arrange
        let db = setup_test_db().await;
        let uid = "test-uid-1";

        // Act - assign the same UID with the same kind
        let id1 = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Todo)
            .await
            .expect("Failed to assign short ID");
        let id2 = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Todo)
            .await
            .expect("Failed to get short ID");

        // Assert - should return the same ID (ON CONFLICT DO NOTHING)
        assert_eq!(id1, id2);
        assert_eq!(id1.get(), 1);
    }

    #[tokio::test]
    async fn short_ids_truncate_removes_all_entries() {
        // Arrange
        let db = setup_test_db().await;
        for i in 1..=5 {
            let uid = format!("uid-{i}");
            db.short_ids
                .get_or_assign_short_id(&uid, Kind::Todo)
                .await
                .expect("Failed to assign short ID");
        }

        // Act
        db.short_ids
            .truncate()
            .await
            .expect("Failed to truncate short_ids");

        // Assert - all entries should be removed
        let result = db
            .short_ids
            .get_by_short_id(NonZeroU32::new(1).unwrap())
            .await
            .expect("Failed to get by short ID");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn short_ids_truncate_resets_id_generation() {
        // Arrange
        let db = setup_test_db().await;
        let id1 = db
            .short_ids
            .get_or_assign_short_id("uid-1", Kind::Todo)
            .await
            .expect("Failed to assign short ID");
        assert_eq!(id1.get(), 1);

        // Act - truncate and then assign a new ID
        db.short_ids
            .truncate()
            .await
            .expect("Failed to truncate short_ids");

        let id2 = db
            .short_ids
            .get_or_assign_short_id("uid-2", Kind::Todo)
            .await
            .expect("Failed to assign short ID");

        // Assert - ID generation should restart from 1
        assert_eq!(id2.get(), 1);
    }

    #[tokio::test]
    async fn short_ids_assign_sequential_ids_starting_from_one() {
        // Arrange
        let db = setup_test_db().await;

        // Act
        let id1 = db
            .short_ids
            .get_or_assign_short_id("uid-1", Kind::Event)
            .await
            .expect("Failed to assign short ID");
        let id2 = db
            .short_ids
            .get_or_assign_short_id("uid-2", Kind::Event)
            .await
            .expect("Failed to assign short ID");
        let id3 = db
            .short_ids
            .get_or_assign_short_id("uid-3", Kind::Event)
            .await
            .expect("Failed to assign short ID");

        // Assert
        assert_eq!(id1.get(), 1);
        assert_eq!(id2.get(), 2);
        assert_eq!(id3.get(), 3);
    }

    #[tokio::test]
    async fn short_ids_increment_across_different_kinds() {
        // Arrange
        let db = setup_test_db().await;

        // Act
        let id1 = db
            .short_ids
            .get_or_assign_short_id("event-1", Kind::Event)
            .await
            .expect("Failed to assign short ID");
        let id2 = db
            .short_ids
            .get_or_assign_short_id("todo-1", Kind::Todo)
            .await
            .expect("Failed to assign short ID");
        let id3 = db
            .short_ids
            .get_or_assign_short_id("event-2", Kind::Event)
            .await
            .expect("Failed to assign short ID");

        // Assert
        // IDs are sequential across different kinds (no separate sequences)
        assert_eq!(id1.get(), 1);
        assert_eq!(id2.get(), 2);
        assert_eq!(id3.get(), 3);
    }

    #[tokio::test]
    async fn short_ids_flush_removes_all_mappings() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db.clone());

        // Create some mappings
        db.short_ids
            .get_or_assign_short_id("uid-1", Kind::Event)
            .await
            .expect("Failed to assign short ID");
        db.short_ids
            .get_or_assign_short_id("uid-2", Kind::Todo)
            .await
            .expect("Failed to assign short ID");

        // Act
        short_ids.flush().await.expect("Failed to flush short IDs");

        // Assert - all mappings should be removed
        let result = db
            .short_ids
            .get_by_short_id(1.try_into().unwrap())
            .await
            .expect("Failed to check short ID");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn short_ids_restart_from_one_after_flush() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db.clone());

        // Create some mappings
        let id1 = db
            .short_ids
            .get_or_assign_short_id("uid-1", Kind::Event)
            .await
            .expect("Failed to assign short ID");
        assert_eq!(id1.get(), 1);
        let id2 = db
            .short_ids
            .get_or_assign_short_id("uid-2", Kind::Event)
            .await
            .expect("Failed to assign short ID");
        assert_eq!(id2.get(), 2);

        // Act - flush and assign new IDs
        short_ids.flush().await.expect("Failed to flush short IDs");

        let id3 = db
            .short_ids
            .get_or_assign_short_id("uid-3", Kind::Event)
            .await
            .expect("Failed to assign short ID");

        // Assert - ID generation should restart from 1
        assert_eq!(id3.get(), 1);
    }

    #[tokio::test]
    async fn short_ids_preserve_existing_mapping_on_reassign() {
        // Arrange
        let db = setup_test_db().await;
        let uid = "persistent-uid";

        // Act - assign the same UID twice
        let id1 = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Event)
            .await
            .expect("Failed to assign short ID");
        let id2 = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Event)
            .await
            .expect("Failed to get short ID");

        // Assert - should return the same ID
        assert_eq!(id1, id2);
        assert_eq!(id1.get(), 1);
    }

    #[tokio::test]
    async fn short_ids_continue_sequentially_after_flush_with_new_uids() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db.clone());

        // Create initial mappings
        db.short_ids
            .get_or_assign_short_id("uid-1", Kind::Event)
            .await
            .expect("Failed to assign short ID");
        db.short_ids
            .get_or_assign_short_id("uid-2", Kind::Event)
            .await
            .expect("Failed to assign short ID");
        db.short_ids
            .get_or_assign_short_id("uid-3", Kind::Event)
            .await
            .expect("Failed to assign short ID");

        // Flush
        short_ids.flush().await.expect("Failed to flush short IDs");

        // Act - create new mappings after flush
        let id1 = db
            .short_ids
            .get_or_assign_short_id("new-uid-1", Kind::Event)
            .await
            .expect("Failed to assign short ID");
        let id2 = db
            .short_ids
            .get_or_assign_short_id("new-uid-2", Kind::Event)
            .await
            .expect("Failed to assign short ID");

        // Assert - should restart from 1
        assert_eq!(id1.get(), 1);
        assert_eq!(id2.get(), 2);
    }

    #[tokio::test]
    async fn short_ids_get_returns_uid_and_short_id_for_short_id() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db.clone());
        let uid = "test-uid-event-1";

        // Assign a short ID first
        let assigned_short_id = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Event)
            .await
            .expect("Failed to assign short ID");

        let id = Id::ShortIdOrUid(assigned_short_id.get().to_string());

        // Act
        let result = short_ids
            .get(&id)
            .await
            .expect("Failed to get short ID mapping");

        // Assert
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.uid, uid);
        assert_eq!(data.short_id, assigned_short_id);
        assert_eq!(data.kind, Kind::Event);
    }

    #[tokio::test]
    async fn short_ids_get_returns_none_for_uid_variant() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let uid = "test-uid-123";
        let id = Id::Uid(uid.to_string());

        // Act
        let result = short_ids
            .get(&id)
            .await
            .expect("Failed to get short ID mapping");

        // Assert
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn short_ids_get_returns_none_for_non_existent_short_id() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let id = Id::ShortIdOrUid("999".to_string()); // Non-existent short ID

        // Act
        let result = short_ids
            .get(&id)
            .await
            .expect("Failed to get short ID mapping");

        // Assert
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn short_ids_get_uid_resolves_short_id_to_uid() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db.clone());
        let uid = "test-uid-todo-1";

        // Assign a short ID
        let assigned_short_id = db
            .short_ids
            .get_or_assign_short_id(uid, Kind::Todo)
            .await
            .expect("Failed to assign short ID");

        let id = Id::ShortIdOrUid(assigned_short_id.get().to_string());

        // Act
        let resolved_uid = short_ids.get_uid(&id).await.expect("Failed to resolve UID");

        // Assert
        assert_eq!(resolved_uid, uid);
    }

    #[tokio::test]
    async fn short_ids_get_uid_returns_uid_string_for_uid_variant() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let uid = "direct-uid-string";
        let id = Id::Uid(uid.to_string());

        // Act
        let resolved_uid = short_ids.get_uid(&id).await.expect("Failed to resolve UID");

        // Assert
        assert_eq!(resolved_uid, uid);
    }

    #[tokio::test]
    async fn short_ids_get_uid_returns_string_for_short_id_or_uid_lookalike() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        // A UUID-like string that could be a UID
        let uid_string = "abc-123-def-456";
        let id = Id::ShortIdOrUid(uid_string.to_string());

        // Act
        let resolved_uid = short_ids.get_uid(&id).await.expect("Failed to resolve UID");

        // Assert
        // Should return the string as-is since it's not a valid short ID
        assert_eq!(resolved_uid, uid_string);
    }

    #[tokio::test]
    async fn short_ids_get_returns_none_for_invalid_short_id_zero() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        // "0" is not a valid NonZeroU32
        let id = Id::ShortIdOrUid("0".to_string());

        // Act
        let result = short_ids
            .get(&id)
            .await
            .expect("Failed to get short ID mapping");

        // Assert
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn event_with_short_id_delegates_short_id() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let event = test_event("event-1", "Test Event");

        // Act
        let wrapped = short_ids
            .event(event)
            .await
            .expect("Failed to wrap event with short ID");

        // Assert
        assert_eq!(wrapped.short_id(), Some(wrapped.short_id));
        assert!(wrapped.short_id.get() > 0);
    }

    #[tokio::test]
    async fn event_with_short_id_delegates_uid() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let uid = "event-delegates-uid";
        let event = test_event(uid, "Test Event");

        // Act
        let wrapped = short_ids
            .event(event)
            .await
            .expect("Failed to wrap event with short ID");

        // Assert
        assert_eq!(wrapped.uid(), Cow::Borrowed(uid));
    }

    #[tokio::test]
    async fn event_with_short_id_delegates_summary() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let summary = "Event Summary Test";
        let event = test_event("event-1", summary);

        // Act
        let wrapped = short_ids
            .event(event)
            .await
            .expect("Failed to wrap event with short ID");

        // Assert
        assert_eq!(wrapped.summary(), Cow::Borrowed(summary));
    }

    #[tokio::test]
    async fn event_with_short_id_delegates_description() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let event = test_event("event-1", "Test");

        // Act
        let wrapped = short_ids
            .event(event)
            .await
            .expect("Failed to wrap event with short ID");

        // Assert
        assert_eq!(
            wrapped.description(),
            Some(Cow::Borrowed("Test description"))
        );
    }

    #[tokio::test]
    async fn event_with_short_id_delegates_status() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let event = test_event("event-1", "Test");

        // Act
        let wrapped = short_ids
            .event(event)
            .await
            .expect("Failed to wrap event with short ID");

        // Assert
        assert_eq!(wrapped.status(), Some(EventStatus::Confirmed));
    }

    #[tokio::test]
    async fn event_with_short_id_delegates_start_end() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let event = test_event("event-1", "Test");

        // Act
        let wrapped = short_ids
            .event(event)
            .await
            .expect("Failed to wrap event with short ID");

        // Assert
        assert!(wrapped.start().is_some());
        assert!(wrapped.end().is_some());
    }

    #[tokio::test]
    async fn todo_with_short_id_delegates_short_id() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let todo = test_todo("todo-1", "Test Todo");

        // Act
        let wrapped = short_ids
            .todo(todo)
            .await
            .expect("Failed to wrap todo with short ID");

        // Assert
        assert_eq!(wrapped.short_id(), Some(wrapped.short_id));
        assert!(wrapped.short_id.get() > 0);
    }

    #[tokio::test]
    async fn todo_with_short_id_delegates_uid() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let uid = "todo-delegates-uid";
        let todo = test_todo(uid, "Test Todo");

        // Act
        let wrapped = short_ids
            .todo(todo)
            .await
            .expect("Failed to wrap todo with short ID");

        // Assert
        assert_eq!(wrapped.uid(), Cow::Borrowed(uid));
    }

    #[tokio::test]
    async fn todo_with_short_id_delegates_summary() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let summary = "Todo Summary Test";
        let todo = test_todo("todo-1", summary);

        // Act
        let wrapped = short_ids
            .todo(todo)
            .await
            .expect("Failed to wrap todo with short ID");

        // Assert
        assert_eq!(wrapped.summary(), Cow::Borrowed(summary));
    }

    #[tokio::test]
    async fn todo_with_short_id_delegates_description() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let todo = test_todo("todo-1", "Test");

        // Act
        let wrapped = short_ids
            .todo(todo)
            .await
            .expect("Failed to wrap todo with short ID");

        // Assert
        assert_eq!(
            wrapped.description(),
            Some(Cow::Borrowed("Test todo description"))
        );
    }

    #[tokio::test]
    async fn todo_with_short_id_delegates_status() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let todo = test_todo("todo-1", "Test");

        // Act
        let wrapped = short_ids
            .todo(todo)
            .await
            .expect("Failed to wrap todo with short ID");

        // Assert
        assert_eq!(wrapped.status(), TodoStatus::NeedsAction);
    }

    #[tokio::test]
    async fn todo_with_short_id_delegates_priority() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let todo = test_todo("todo-1", "Test");

        // Act
        let wrapped = short_ids
            .todo(todo)
            .await
            .expect("Failed to wrap todo with short ID");

        // Assert
        assert_eq!(wrapped.priority(), Priority::P5);
    }

    #[tokio::test]
    async fn todo_with_short_id_delegates_percent_complete() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let todo = test_todo("todo-1", "Test");

        // Act
        let wrapped = short_ids
            .todo(todo)
            .await
            .expect("Failed to wrap todo with short ID");

        // Assert
        assert_eq!(wrapped.percent_complete(), Some(0));
    }

    #[tokio::test]
    async fn todo_with_short_id_delegates_due_and_completed() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let todo = test_todo("todo-1", "Test");

        // Act
        let wrapped = short_ids
            .todo(todo)
            .await
            .expect("Failed to wrap todo with short ID");

        // Assert
        assert!(wrapped.due().is_some());
        assert!(wrapped.completed().is_some());
    }

    #[tokio::test]
    async fn short_ids_events_wraps_multiple_events() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let events = vec![
            test_event("event-1", "Event 1"),
            test_event("event-2", "Event 2"),
            test_event("event-3", "Event 3"),
        ];

        // Act
        let wrapped = short_ids
            .events(events)
            .await
            .expect("Failed to wrap events with short IDs");

        // Assert
        assert_eq!(wrapped.len(), 3);
        let first = wrapped.first().expect("should have first element");
        let second = wrapped.get(1).expect("should have second element");
        let third = wrapped.get(2).expect("should have third element");
        assert!(first.short_id.get() > 0);
        assert!(second.short_id.get() > first.short_id.get());
        assert!(third.short_id.get() > second.short_id.get());
    }

    #[tokio::test]
    async fn short_ids_todos_wraps_multiple_todos() {
        // Arrange
        let db = setup_test_db().await;
        let short_ids = ShortIds::new(db);
        let todos = vec![
            test_todo("todo-1", "Todo 1"),
            test_todo("todo-2", "Todo 2"),
            test_todo("todo-3", "Todo 3"),
        ];

        // Act
        let wrapped = short_ids
            .todos(todos)
            .await
            .expect("Failed to wrap todos with short IDs");

        // Assert
        assert_eq!(wrapped.len(), 3);
        let first = wrapped.first().expect("should have first element");
        let second = wrapped.get(1).expect("should have second element");
        let third = wrapped.get(2).expect("should have third element");
        assert!(first.short_id.get() > 0);
        assert!(second.short_id.get() > first.short_id.get());
        assert!(third.short_id.get() > second.short_id.get());
    }
}
