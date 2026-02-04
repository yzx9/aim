// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct Resources {
    pool: SqlitePool,
}

impl Resources {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        uid: &str,
        backend_kind: u8,
        resource_id: &str,
        metadata: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        const SQL: &str = "
INSERT INTO resources (uid, backend_kind, resource_id, metadata)
VALUES (?, ?, ?, ?)
ON CONFLICT(uid, backend_kind) DO UPDATE SET
    resource_id = excluded.resource_id,
    metadata = excluded.metadata;
";

        sqlx::query(SQL)
            .bind(uid)
            .bind(backend_kind)
            .bind(resource_id)
            .bind(metadata)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get(
        &self,
        uid: &str,
        backend_kind: u8,
    ) -> Result<Option<ResourceRecord>, sqlx::Error> {
        const SQL: &str = "
SELECT uid, backend_kind, resource_id, metadata
FROM resources
WHERE uid = ? AND backend_kind = ?;
";

        sqlx::query_as(SQL)
            .bind(uid)
            .bind(backend_kind)
            .fetch_optional(&self.pool)
            .await
    }

    #[allow(dead_code)]
    pub async fn delete(&self, uid: &str, backend_kind: u8) -> Result<(), sqlx::Error> {
        const SQL: &str = "DELETE FROM resources WHERE uid = ? AND backend_kind = ?;";

        sqlx::query(SQL)
            .bind(uid)
            .bind(backend_kind)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
pub struct ResourceRecord {
    pub uid: String,
    pub backend_kind: u8,
    pub resource_id: String,
    pub metadata: Option<String>,
}

impl ResourceRecord {
    #[allow(dead_code)]
    pub fn metadata_json<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        let metadata = self.metadata.as_ref()?;
        serde_json::from_str(metadata).ok()
    }

    #[allow(dead_code)]
    pub fn backend_kind(&self) -> u8 {
        self.backend_kind
    }
}

#[cfg(test)]
mod tests {
    async fn setup_test_db() -> crate::localdb::LocalDb {
        crate::localdb::LocalDb::open(None)
            .await
            .expect("Failed to create test database")
    }

    #[tokio::test]
    async fn resources_insert_inserts_new_resource() {
        let db = setup_test_db().await;

        // Insert a todo first to satisfy FK constraint
        let todo = crate::localdb::tests_utils::test_todo("test-uid", "Test Todo");
        let todo_record = crate::localdb::todos::TodoRecord::from_todo("test-uid", &todo, 0);
        let upsert_result = db.todos.upsert(&todo_record).await;
        if let Err(e) = upsert_result {
            panic!("Failed to upsert todo: {e:?}");
        }

        // NOTE: Due to migration design with dual FK constraints,
        // we also need to insert into events table to satisfy both FKs
        let event = crate::localdb::tests_utils::test_event("test-uid", "Test Event");
        let event_record = crate::localdb::events::EventRecord::from_event("test-uid", &event, 0);
        db.events.upsert(event_record).await.unwrap();

        let insert_result = db
            .resources
            .insert("test-uid", 0, "file:///path/test.ics", None)
            .await;

        if let Err(e) = insert_result {
            panic!("Failed to insert resource: {e:?}");
        }

        let retrieved = db.resources.get("test-uid", 0).await.unwrap();
        assert!(retrieved.is_some());
        let resource = retrieved.unwrap();
        assert_eq!(resource.uid, "test-uid");
        assert_eq!(resource.backend_kind, 0);
        assert_eq!(resource.resource_id, "file:///path/test.ics");
        assert!(resource.metadata.is_none());
    }

    #[tokio::test]
    async fn resources_get_returns_none_for_nonexistent() {
        let db = setup_test_db().await;

        let retrieved = db.resources.get("nonexistent", 0).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn resources_delete_removes_resource() {
        let db = setup_test_db().await;

        // Insert a todo first to satisfy FK constraint
        let todo = crate::localdb::tests_utils::test_todo("test-uid", "Test Todo");
        let todo_record = crate::localdb::todos::TodoRecord::from_todo("test-uid", &todo, 0);
        db.todos.upsert(&todo_record).await.unwrap();

        // NOTE: Due to migration design with dual FK constraints,
        // we also need to insert into events table to satisfy both FKs
        let event = crate::localdb::tests_utils::test_event("test-uid", "Test Event");
        let event_record = crate::localdb::events::EventRecord::from_event("test-uid", &event, 0);
        db.events.upsert(event_record).await.unwrap();

        db.resources
            .insert("test-uid", 0, "file:///path/test.ics", None)
            .await
            .unwrap();

        db.resources.delete("test-uid", 0).await.unwrap();

        let retrieved = db.resources.get("test-uid", 0).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn resources_insert_updates_existing_resource() {
        let db = setup_test_db().await;

        // Insert a todo first to satisfy FK constraint
        let todo = crate::localdb::tests_utils::test_todo("test-uid", "Test Todo");
        let todo_record = crate::localdb::todos::TodoRecord::from_todo("test-uid", &todo, 0);
        db.todos.upsert(&todo_record).await.unwrap();

        // NOTE: Due to migration design with dual FK constraints,
        // we also need to insert into events table to satisfy both FKs
        let event = crate::localdb::tests_utils::test_event("test-uid", "Test Event");
        let event_record = crate::localdb::events::EventRecord::from_event("test-uid", &event, 0);
        db.events.upsert(event_record).await.unwrap();

        db.resources
            .insert("test-uid", 0, "file:///path/test.ics", None)
            .await
            .unwrap();

        db.resources
            .insert("test-uid", 0, "file:///path/new.ics", Some("{}"))
            .await
            .unwrap();

        let retrieved = db.resources.get("test-uid", 0).await.unwrap();
        assert!(retrieved.is_some());
        let resource = retrieved.unwrap();
        assert_eq!(resource.resource_id, "file:///path/new.ics");
        assert_eq!(resource.metadata, Some("{}".to_string()));
    }

    #[tokio::test]
    async fn resources_metadata_json_parses_json() {
        #[derive(serde::Deserialize)]
        struct TestMetadata {
            etag: String,
            version: i32,
        }

        let db = setup_test_db().await;

        // Insert a todo first to satisfy FK constraint
        let todo = crate::localdb::tests_utils::test_todo("test-uid", "Test Todo");
        let todo_record = crate::localdb::todos::TodoRecord::from_todo("test-uid", &todo, 1);
        db.todos.upsert(&todo_record).await.unwrap();

        // NOTE: Due to migration design with dual FK constraints,
        // we also need to insert into events table to satisfy both FKs
        let event = crate::localdb::tests_utils::test_event("test-uid", "Test Event");
        let event_record = crate::localdb::events::EventRecord::from_event("test-uid", &event, 0);
        db.events.upsert(event_record).await.unwrap();

        let json = r#"{"etag":"\"abc123\"","version":1}"#;
        db.resources
            .insert("test-uid", 1, "/dav/test.ics", Some(json))
            .await
            .unwrap();

        let retrieved = db.resources.get("test-uid", 1).await.unwrap();
        assert!(retrieved.is_some());
        let resource = retrieved.unwrap();
        let metadata: TestMetadata = resource.metadata_json().unwrap();
        assert_eq!(metadata.etag, "\"abc123\"");
        assert_eq!(metadata.version, 1);
    }
}
