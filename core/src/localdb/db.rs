// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use super::{
    events::{EventRecord, Events},
    todos::{TodoRecord, Todos},
};
use crate::{Event, Todo, localdb::short_ids::ShortIds};
use sqlx::{
    SqlitePool, migrate,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{
    error::Error,
    path::{Path, PathBuf},
};

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
    pub async fn open(state_dir: &Option<PathBuf>) -> Result<Self, Box<dyn Error>> {
        let options = match state_dir {
            Some(dir) => {
                const NAME: &str = "aim.db";

                log::info!("Connecting to SQLite database at {}", dir.display());
                let dir = dir.to_str().ok_or("Invalid path encoding")?;
                SqliteConnectOptions::new()
                    .filename(format!("{dir}/{NAME}"))
                    .create_if_missing(true)
            }
            None => {
                log::info!("Connecting to in-memory SQLite database");
                SqliteConnectOptions::new().in_memory(true)
            }
        };

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| format!("Failed to connect to SQLite database: {e}"))?;

        migrate!("src/localdb/migrations") // relative path from the crate root
            .run(&pool)
            .await
            .map_err(|e| format!("Failed to run migrations: {e}"))?;

        log::debug!("Creating tables in the database");
        let events = Events::new(pool.clone()).await?;
        let todos = Todos::new(pool.clone()).await?;
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
        let record = EventRecord::from(path.clone(), event)?;
        self.events
            .insert(record)
            .await
            .map_err(|e| format!("Failed to upsert event: {e}").into())
    }

    pub async fn upsert_todo(&self, path: &Path, todo: &impl Todo) -> Result<(), Box<dyn Error>> {
        let path = path.to_str().ok_or("Invalid path encoding")?.to_string();
        let record = TodoRecord::from(path.clone(), todo)?;
        self.todos
            .upsert(&record)
            .await
            .map_err(|e| format!("Failed to upsert todo: {e}").into())
    }

    pub async fn close(self) -> Result<(), Box<dyn Error>> {
        log::debug!("Closing database connection");
        self.pool.close().await;
        Ok(())
    }
}
