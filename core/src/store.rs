// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

pub mod caldav;
pub mod local;

pub use caldav::CaldavStore;
pub use local::LocalStore;

use std::error::Error;

use aimcal_ical::{VEvent, VTodo};
use async_trait::async_trait;

use crate::{EventPatch, TodoPatch};

/// Error type for store operations that is Send + Sync.
pub type StoreError = Box<dyn Error + Send + Sync>;

/// Result of a backend synchronization operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncResult {
    /// Number of items created during synchronization.
    pub created: usize,
    /// Number of items updated during synchronization.
    pub updated: usize,
    /// Number of items deleted during synchronization.
    pub deleted: usize,
}

/// Store trait for storing and synchronizing events and todos.
///
/// This trait abstracts different storage backends (local ICS files, `CalDAV` servers, etc.)
/// providing a unified interface for CRUD operations on calendar items.
#[async_trait]
pub trait Store: Send + Sync {
    /// Creates a new event in the store.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier for the event
    /// * `event` - The event to create
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be created in the store.
    async fn create_event(&self, uid: &str, event: &VEvent<String>) -> Result<String, StoreError>;

    /// Retrieves an event from the store by UID.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the event to retrieve
    ///
    /// # Errors
    ///
    /// Returns an error if the event is not found or cannot be retrieved.
    async fn get_event(&self, uid: &str) -> Result<VEvent<String>, StoreError>;

    /// Updates an existing event in the store.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the event to update
    /// * `patch` - The patch to apply to the event
    ///
    /// # Errors
    ///
    /// Returns an error if the event is not found or cannot be updated.
    async fn update_event(
        &self,
        uid: &str,
        patch: &EventPatch,
    ) -> Result<VEvent<String>, StoreError>;

    /// Deletes an event from the store.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the event to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the event is not found or cannot be deleted.
    async fn delete_event(&self, uid: &str) -> Result<(), StoreError>;

    /// Creates a new todo in the store.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier for the todo
    /// * `todo` - The todo to create
    ///
    /// # Errors
    ///
    /// Returns an error if the todo cannot be created in the store.
    async fn create_todo(&self, uid: &str, todo: &VTodo<String>) -> Result<String, StoreError>;

    /// Retrieves a todo from the store by UID.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the todo to retrieve
    ///
    /// # Errors
    ///
    /// Returns an error if the todo is not found or cannot be retrieved.
    async fn get_todo(&self, uid: &str) -> Result<VTodo<String>, StoreError>;

    /// Updates an existing todo in the store.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the todo to update
    /// * `patch` - The patch to apply to the todo
    ///
    /// # Errors
    ///
    /// Returns an error if the todo is not found or cannot be updated.
    async fn update_todo(&self, uid: &str, patch: &TodoPatch) -> Result<VTodo<String>, StoreError>;

    /// Deletes a todo from the store.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the todo to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the todo is not found or cannot be deleted.
    async fn delete_todo(&self, uid: &str) -> Result<(), StoreError>;

    /// Lists all events in the store.
    ///
    /// # Errors
    ///
    /// Returns an error if the events cannot be listed.
    async fn list_events(&self) -> Result<Vec<(String, VEvent<String>)>, StoreError>;

    /// Lists all todos in the store.
    ///
    /// # Errors
    ///
    /// Returns an error if the todos cannot be listed.
    async fn list_todos(&self) -> Result<Vec<(String, VTodo<String>)>, StoreError>;

    /// Checks if a UID exists in the store.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier to check
    ///
    /// # Returns
    ///
    /// `true` if the UID exists, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the check cannot be performed.
    async fn uid_exists(&self, uid: &str) -> Result<bool, StoreError>;

    /// Returns the calendar identifier for this store.
    ///
    /// This identifies which calendar in the database items from this store belong to.
    fn calendar_id(&self) -> &str;

    /// Synchronizes the store with the local cache (database).
    ///
    /// This operation scans the store for changes and updates the local
    /// database accordingly.
    ///
    /// # Returns
    ///
    /// A `SyncResult` containing counts of created, updated, and deleted items.
    ///
    /// # Errors
    ///
    /// Returns an error if synchronization fails.
    async fn sync_cache(&self) -> Result<SyncResult, StoreError>;
}
