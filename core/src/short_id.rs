// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{Config, Event, EventStatus, Id, LooseDateTime, Priority, Todo, TodoStatus};
use bimap::BiBTreeMap;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{
    io,
    num::NonZeroU32,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tokio::fs;

/// A thread-safe structure for mapping UIDs to display numbers.
///
/// If a UID is not found, a new display number (1, 2, 3, ...) is allocated.
#[derive(Debug, Clone)]
pub struct ShortIdMap {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Inner {
    map: BiBTreeMap<String, NonZeroU32>,
    next: NonZeroU32,
    last_modified: DateTime<Local>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            map: BiBTreeMap::new(),
            next: NonZeroU32::new(1).expect("Failed to create NonZeroU32"),
            last_modified: Local::now(),
        }
    }
}

impl ShortIdMap {
    fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::default())),
        }
    }

    /// Load the map from disk.
    ///
    /// If the file does not exist or is empty, a new map is returned.
    pub async fn load_or_new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let path = match Self::get_map_path(config) {
            Some(a) => a,
            None => {
                log::warn!("No state directory configured, using empty map");
                return Ok(Self::new());
            }
        };

        match fs::read_to_string(&path).await {
            Ok(content) => match serde_json::from_str::<Inner>(&content) {
                Ok(inner) => {
                    log::debug!("Loaded existing map from disk: {:?}", path.display());
                    let mut inner = inner;
                    inner.last_modified = Local::now(); // Update last_modified to current time
                    Ok(Self {
                        inner: Arc::new(RwLock::new(inner)),
                    })
                }
                Err(e) => {
                    log::warn!("Failed to parse existing map: {e}");
                    Ok(Self::new())
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::new()),
            Err(e) => Err(e.into()),
        }
    }

    /// Dump the map to disk.
    pub async fn dump(&self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_map_path(config).ok_or("No state directory configured")?;
        let content = {
            let inner = self.inner.read().unwrap();
            serde_json::to_string(&*inner)?
        };
        fs::write(path, content).await?;
        Ok(())
    }

    /// Get or allocate a display number for the given uid.
    ///
    /// If the UID is not already in the map, a new number is assigned and returned.
    pub fn get_or_assign_short_id(&self, uid: &str) -> NonZeroU32 {
        // First try read-only access
        {
            let inner = self.inner.read().unwrap();
            if let Some(&id) = inner.map.get_by_left(uid) {
                return id;
            }
        }

        // Upgrade to write lock when needed
        let mut inner = self.inner.write().unwrap();
        if let Some(&id) = inner.map.get_by_left(uid) {
            return id; // Check again in the write lock
        }

        let id = inner.next;
        inner.next = inner.next.saturating_add(1);
        inner.map.insert(uid.to_string(), id);
        id
    }

    pub fn get_uid(&self, id: Id) -> String {
        if let Id::ShortIdOrUid(uid) = &id {
            if let Ok(short_id) = uid.parse::<NonZeroU32>() {
                let uid = self
                    .inner
                    .read()
                    .unwrap()
                    .map
                    .get_by_right(&short_id)
                    .cloned();

                if let Some(uid) = uid {
                    return uid;
                }
            }
        }

        match id {
            Id::Uid(uid) => uid,
            Id::ShortIdOrUid(uid) => uid,
        }
    }

    fn get_map_path(config: &Config) -> Option<PathBuf> {
        config.state_dir.as_ref().map(|a| a.join("short_id.json"))
    }
}

#[derive(Debug)]
pub struct EventWithShortId<E: Event> {
    pub inner: E,
    pub short_id: NonZeroU32,
}

impl<E: Event> EventWithShortId<E> {
    pub fn with(map: &ShortIdMap, event: E) -> Result<Self, String> {
        let short_id = if let Some(short_id) = event.short_id() {
            short_id // If the todo already has a short ID, use it directly
        } else {
            map.get_or_assign_short_id(event.uid())
        };

        Ok(Self {
            inner: event,
            short_id,
        })
    }
}

impl<E: Event> Event for EventWithShortId<E> {
    fn short_id(&self) -> Option<NonZeroU32> {
        Some(self.short_id)
    }

    fn uid(&self) -> &str {
        self.inner.uid()
    }

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn start(&self) -> Option<LooseDateTime> {
        self.inner.start()
    }

    fn end(&self) -> Option<LooseDateTime> {
        self.inner.end()
    }

    fn status(&self) -> Option<EventStatus> {
        self.inner.status()
    }

    fn summary(&self) -> &str {
        self.inner.summary()
    }
}

#[derive(Debug)]
pub struct TodoWithShortId<T: Todo> {
    pub inner: T,
    pub short_id: NonZeroU32,
}

impl<T: Todo> TodoWithShortId<T> {
    pub fn with(map: &ShortIdMap, todo: T) -> Result<Self, String> {
        let short_id = if let Some(short_id) = todo.short_id() {
            short_id // If the todo already has a short ID, use it directly
        } else {
            map.get_or_assign_short_id(todo.uid())
        };

        Ok(Self {
            inner: todo,
            short_id,
        })
    }
}

impl<T: Todo> Todo for TodoWithShortId<T> {
    fn short_id(&self) -> Option<NonZeroU32> {
        Some(self.short_id)
    }
    fn uid(&self) -> &str {
        self.inner.uid()
    }
    fn completed(&self) -> Option<DateTime<Local>> {
        self.inner.completed()
    }
    fn description(&self) -> Option<&str> {
        self.inner.description()
    }
    fn due(&self) -> Option<LooseDateTime> {
        self.inner.due()
    }
    fn percent_complete(&self) -> Option<u8> {
        self.inner.percent_complete()
    }
    fn priority(&self) -> Priority {
        self.inner.priority()
    }
    fn status(&self) -> TodoStatus {
        self.inner.status()
    }
    fn summary(&self) -> &str {
        self.inner.summary()
    }
}
