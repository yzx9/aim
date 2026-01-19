// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Synchronization utilities for `CalDAV`.

use std::collections::HashMap;

use crate::client::CalDavClient;
use crate::error::CalDavError;
use crate::types::{CalendarResource, ETag, Href};

/// Synchronization state.
#[derive(Debug, Clone, Default)]
#[expect(dead_code)]
pub struct SyncState {
    pub resource_etags: HashMap<Href, ETag>,
    pub calendar_ctag: Option<ETag>,
}

/// Changes detected during sync.
#[derive(Debug, Clone, Default)]
#[expect(dead_code)]
pub struct SyncChanges {
    pub added: Vec<CalendarResource>,
    pub modified: Vec<CalendarResource>,
    pub deleted: Vec<Href>,
}

/// Performs two-way sync with `CalDAV` server.
///
/// # Errors
///
/// Returns an error if sync fails.
#[expect(dead_code, clippy::unnecessary_wraps)]
pub fn sync_calendar(
    _client: &CalDavClient,
    _calendar_href: &Href,
    _local_state: &SyncState,
) -> Result<SyncChanges, CalDavError> {
    // TODO: Implement sync logic
    Ok(SyncChanges::default())
}
