// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod events;
mod short_ids;
mod todos;

use std::error::Error;
use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

use crate::localdb::events::{EventRecord, Events};
use crate::localdb::short_ids::ShortIds;
use crate::localdb::todos::{TodoRecord, Todos};
use crate::{Event, Todo};

#[derive(Debug, Clone)]
pub struct LocalDb {
    pool: SqlitePool,

    pub events: Events,
    pub todos: Todos,
    pub short_ids: ShortIds,
}

impl LocalDb {
    /// Opens a sqlite database connection.
    /// If `state_dir` is `None`, it opens an in-memory database.
    pub async fn open(filename: Option<&Path>) -> Result<Self, Box<dyn Error>> {
        let options = if let Some(filename) = filename {
            tracing::info!(dir = %filename.display(), "connecting to SQLite database");
            SqliteConnectOptions::new()
                .filename(filename.to_str().ok_or("Invalid path encoding")?)
                .create_if_missing(true)
        } else {
            tracing::info!("connecting to in-memory SQLite database");
            SqliteConnectOptions::new().in_memory(true)
        };

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| format!("Failed to connect to SQLite database: {e}"))?;

        sqlx::migrate!("src/localdb/migrations") // relative path from the crate root
            .run(&pool)
            .await
            .map_err(|e| format!("Failed to run migrations: {e}"))?;

        tracing::debug!("ensuring tables in the database");
        let events = Events::new(pool.clone());
        let todos = Todos::new(pool.clone());
        let short_ids = ShortIds::new(pool.clone());
        Ok(LocalDb {
            pool,
            events,
            todos,
            short_ids,
        })
    }

    pub async fn upsert_event(
        &self,
        path: &Path,
        event: &impl Event,
    ) -> Result<(), Box<dyn Error>> {
        let path = path.to_str().ok_or("Invalid path encoding")?.to_string();
        let record = EventRecord::from(path.clone(), event);
        self.events
            .insert(record)
            .await
            .map_err(|e| format!("Failed to upsert event: {e}").into())
    }

    pub async fn upsert_todo(&self, path: &Path, todo: &impl Todo) -> Result<(), Box<dyn Error>> {
        let path = path.to_str().ok_or("Invalid path encoding")?.to_string();
        let record = TodoRecord::from(path.clone(), todo);
        self.todos
            .upsert(&record)
            .await
            .map_err(|e| format!("Failed to upsert todo: {e}").into())
    }

    pub async fn close(self) -> Result<(), Box<dyn Error>> {
        tracing::debug!("closing database connection");
        self.pool.close().await;
        Ok(())
    }
}
