// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroU32;

use sqlx::SqlitePool;

use crate::short_id::{ShortIdKind, UidAndShortId};

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
            sqlx::query_as("SELECT uid, kind FROM short_ids WHERE short_id = ?")
                .bind(short_id.get() as i64)
                .fetch_optional(&self.pool)
                .await?;

        match row {
            Some((uid, kind)) => {
                let parsed_kind = kind_from_str(&kind);
                if parsed_kind.is_none() {
                    log::warn!("Unknown short_id kind: {kind}");
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
        kind: ShortIdKind,
    ) -> Result<NonZeroU32, sqlx::Error> {
        const SQL: &str = "
INSERT INTO short_ids (uid, kind) VALUES (?, ?)
ON CONFLICT(uid) DO NOTHING
RETURNING short_id
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

fn kind_to_str(kind: ShortIdKind) -> &'static str {
    match kind {
        ShortIdKind::Todo => "todo",
        ShortIdKind::Event => "event",
    }
}

fn kind_from_str(kind: &str) -> Option<ShortIdKind> {
    match kind {
        "todo" => Some(ShortIdKind::Todo),
        "event" => Some(ShortIdKind::Event),
        _ => None,
    }
}
