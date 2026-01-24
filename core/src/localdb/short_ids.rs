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
            Some((uid, kind)) => Ok(match Kind::parse_stable(&kind) {
                Some(kind) => Some(UidAndShortId {
                    uid,
                    short_id,
                    kind,
                }),
                None => {
                    tracing::warn!(kind, "unknown short_id kind");
                    None
                }
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
    use super::*;

    /// Test helper to create a test database
    async fn setup_test_db() -> crate::localdb::LocalDb {
        crate::localdb::LocalDb::open(None)
            .await
            .expect("Failed to create test database")
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
}
