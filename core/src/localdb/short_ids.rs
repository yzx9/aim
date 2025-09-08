// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
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

    #[tracing::instrument(skip(self))]
    pub async fn get_by_short_id(
        &self,
        short_id: NonZeroU32,
    ) -> Result<Option<UidAndShortId>, sqlx::Error> {
        let row: Option<(String, String)> =
            sqlx::query_as("SELECT uid, kind FROM short_ids WHERE short_id = ?;")
                .bind(short_id.get() as i64)
                .fetch_optional(&self.pool)
                .await?;

        match row {
            Some((uid, kind)) => {
                let parsed_kind = kind_from_str(&kind);
                if parsed_kind.is_none() {
                    tracing::warn!(kind, "unknown short_id kind");
                    return Ok(None);
                }

                Ok(Some(UidAndShortId {
                    uid,
                    short_id,
                    kind: parsed_kind.unwrap(),
                }))
            }
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
            .bind(kind_to_str(kind))
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
}

fn kind_to_str(kind: Kind) -> &'static str {
    match kind {
        Kind::Todo => "todo",
        Kind::Event => "event",
    }
}

fn kind_from_str(kind: &str) -> Option<Kind> {
    match kind {
        "todo" => Some(Kind::Todo),
        "event" => Some(Kind::Event),
        _ => None,
    }
}
