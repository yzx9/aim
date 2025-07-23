// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use aimcal_core::{Event, Todo};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs, io,
    path::PathBuf,
    sync::{Arc, RwLock},
};

/// A thread-safe structure for mapping UIDs to display numbers.
///
/// If a UID is not found, a new display number (1, 2, 3, ...) is allocated.
#[derive(Debug, Clone)]
pub struct ShortIdMap {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Inner {
    map: HashMap<String, i64>,
    next: i64,
    last_modified: DateTime<Local>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            next: 1,
            last_modified: Local::now(),
        }
    }
}

impl<'a> ShortIdMap {
    fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::default())),
        }
    }

    /// Load the map from disk.
    ///
    /// If the file does not exist or is empty, a new map is returned.
    pub fn load_or_new(config: &'a Config) -> Result<Self, Box<dyn std::error::Error>> {
        let path = match Self::get_map_path(config) {
            Some(a) => a,
            None => {
                log::warn!("No state directory configured, using empty map");
                return Ok(Self::new());
            }
        };

        match fs::read_to_string(&path) {
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
    pub fn dump(&self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_map_path(config).ok_or("No state directory configured")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let inner = self.inner.read().unwrap();
        let content = serde_json::to_string_pretty(&*inner)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get or allocate a display number for the given uid.
    ///
    /// If the UID is not already in the map, a new number is assigned and returned.
    pub fn get_or_assign(&self, uid: &str) -> i64 {
        // First try read-only access
        {
            let inner = self.inner.read().unwrap();
            if let Some(&id) = inner.map.get(uid) {
                return id;
            }
        }

        // Upgrade to write lock when needed
        let mut inner = self.inner.write().unwrap();
        if let Some(&id) = inner.map.get(uid) {
            return id;
        }

        let id = inner.next;
        inner.next += 1;
        inner.map.insert(uid.to_string(), id);
        id
    }

    pub fn find(&self, short_id: i64) -> Option<String> {
        // TODO: prefer using a more efficient data structure if performance becomes an issue
        self.inner
            .read()
            .unwrap()
            .map
            .iter()
            .find_map(|(uid, &id)| (id == short_id).then_some(uid.clone()))
    }

    fn get_map_path(config: &Config) -> Option<PathBuf> {
        config.state_dir.as_ref().map(|a| a.join("short_id.json"))
    }
}

#[derive(Debug)]
pub struct EventWithShortId<E: Event> {
    pub inner: E,
    pub short_id: i64,
}

impl<E: Event> EventWithShortId<E> {
    pub fn with(map: &ShortIdMap, event: E) -> Self {
        let short_id = map.get_or_assign(event.uid());
        Self {
            inner: event,
            short_id,
        }
    }
}

#[derive(Debug)]
pub struct TodoWithShortId<T: Todo> {
    pub inner: T,
    pub short_id: i64,
}

impl<T: Todo> TodoWithShortId<T> {
    pub fn with(map: &ShortIdMap, todo: T) -> Self {
        let short_id = map.get_or_assign(todo.uid());
        Self {
            inner: todo,
            short_id,
        }
    }
}
