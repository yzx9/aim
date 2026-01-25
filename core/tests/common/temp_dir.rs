// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Temporary directory management for integration tests.
//!
//! This module provides utilities for creating and managing temporary
//! directories with automatic cleanup on drop.

use std::path::PathBuf;
use tokio::fs;

/// Temporary directories used for testing.
///
/// Automatically cleans up all created directories when dropped.
#[derive(Debug)]
pub struct TempDirs {
    /// Calendar directory for .ics files.
    pub calendar_path: PathBuf,
    /// State directory for database files.
    pub state_dir: PathBuf,
}

impl TempDirs {
    /// Creates new temporary directories for testing.
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation fails.
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let base = tempfile::tempdir()?.keep();

        let calendar_path = base.join("calendar");
        let state_dir = base.join("state");

        fs::create_dir_all(&calendar_path).await?;
        fs::create_dir_all(&state_dir).await?;

        Ok(Self {
            calendar_path,
            state_dir,
        })
    }

    /// Gets the base temporary directory.
    #[must_use]
    pub fn base(&self) -> PathBuf {
        // calendar_path and state_dir share the same parent (base)
        self.calendar_path
            .parent()
            .expect("temp directories should have a parent")
            .to_path_buf()
    }

    /// Creates a test .ics file in the calendar directory.
    ///
    /// # Errors
    ///
    /// Returns an error if file writing fails.
    pub async fn create_ics_file(
        &self,
        uid: &str,
        content: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = self.calendar_path.join(format!("{uid}.ics"));
        fs::write(&path, content).await?;
        Ok(path)
    }
}

/// Sets up temporary directories for integration tests.
///
/// This is a convenience wrapper around [`TempDirs::new`].
///
/// # Errors
///
/// Returns an error if directory creation fails.
///
/// # Example
///
/// ```ignore
/// let temp_dirs = setup_temp_dirs().await?;
/// // Use temp_dirs.calendar_path and temp_dirs.state_dir
/// // Automatically cleaned up when dropped
/// ```
pub async fn setup_temp_dirs() -> Result<TempDirs, Box<dyn std::error::Error>> {
    TempDirs::new().await
}

// Implement Drop for automatic cleanup
impl Drop for TempDirs {
    fn drop(&mut self) {
        let base = self.base();
        if let Err(e) = std::fs::remove_dir_all(&base) {
            tracing::warn!(path = %base.display(), err = %e, "failed to clean up temp directory");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn temp_dirs_creates_directories() {
        let dirs = TempDirs::new().await.unwrap();

        assert!(dirs.calendar_path.exists());
        assert!(dirs.state_dir.exists());
        assert!(dirs.calendar_path.is_dir());
        assert!(dirs.state_dir.is_dir());
    }

    #[tokio::test]
    async fn temp_dirs_calendar_and_state_share_parent() {
        let dirs = TempDirs::new().await.unwrap();

        let calendar_parent = dirs.calendar_path.parent();
        let state_parent = dirs.state_dir.parent();

        assert_eq!(calendar_parent, state_parent);
    }

    #[tokio::test]
    async fn temp_dirs_create_ics_file() {
        let dirs = TempDirs::new().await.unwrap();
        let content = "BEGIN:VCALENDAR\nVERSION:2.0\nEND:VCALENDAR";

        let path = dirs.create_ics_file("test-uid", content).await.unwrap();

        assert!(path.exists());
        assert!(path.starts_with(&dirs.calendar_path));
        assert_eq!(path.extension().unwrap().to_str().unwrap(), "ics");
        assert_eq!(fs::read_to_string(&path).await.unwrap(), content);
    }

    #[tokio::test]
    async fn temp_dirs_cleanup_on_drop() {
        let base = {
            let dirs = TempDirs::new().await.unwrap();
            let base = dirs.base();
            assert!(base.exists());
            base
        };

        // After drop, the directory should be removed
        // Give a moment for async cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert!(!base.exists());
    }

    #[tokio::test]
    async fn setup_temp_dirs_convenience_function() {
        let dirs = setup_temp_dirs().await.unwrap();

        assert!(dirs.calendar_path.exists());
        assert!(dirs.state_dir.exists());
    }
}
