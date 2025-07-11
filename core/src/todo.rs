// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{DatePerhapsTime, Priority, SortOrder};
use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime};

pub trait Todo {
    fn id(&self) -> i64;
    fn completed(&self) -> Option<DateTime<FixedOffset>>;
    fn description(&self) -> Option<&str>;
    fn due(&self) -> Option<DatePerhapsTime>;
    /// The percent complete, from 0 to 100.
    fn percent(&self) -> Option<u8>;
    /// The priority from 1 to 9, where 1 is the highest priority.
    fn priority(&self) -> Priority;
    fn status(&self) -> Option<TodoStatus>;
    fn summary(&self) -> &str;
}

#[derive(Debug, Clone, Copy)]
pub enum TodoStatus {
    NeedsAction,
    Completed,
    InProcess,
    Cancelled,
}

const STATUS_NEEDS_ACTION: &str = "NEEDS-ACTION";
const STATUS_COMPLETED: &str = "COMPLETED";
const STATUS_IN_PROCESS: &str = "IN-PROGRESS";
const STATUS_CANCELLED: &str = "CANCELLED";

impl From<TodoStatus> for &str {
    fn from(item: TodoStatus) -> &'static str {
        match item {
            TodoStatus::NeedsAction => STATUS_NEEDS_ACTION,
            TodoStatus::Completed => STATUS_COMPLETED,
            TodoStatus::InProcess => STATUS_IN_PROCESS,
            TodoStatus::Cancelled => STATUS_CANCELLED,
        }
    }
}

impl From<TodoStatus> for String {
    fn from(item: TodoStatus) -> String {
        let s: &str = item.into();
        s.to_string()
    }
}

impl TryFrom<&str> for TodoStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            STATUS_NEEDS_ACTION => Ok(TodoStatus::NeedsAction),
            STATUS_COMPLETED => Ok(TodoStatus::Completed),
            STATUS_IN_PROCESS => Ok(TodoStatus::InProcess),
            STATUS_CANCELLED => Ok(TodoStatus::Cancelled),
            _ => Err(format!("Unknown TodoStatus: {}", value)),
        }
    }
}

impl From<TodoStatus> for icalendar::TodoStatus {
    fn from(item: TodoStatus) -> icalendar::TodoStatus {
        match item {
            TodoStatus::NeedsAction => icalendar::TodoStatus::NeedsAction,
            TodoStatus::Completed => icalendar::TodoStatus::Completed,
            TodoStatus::InProcess => icalendar::TodoStatus::InProcess,
            TodoStatus::Cancelled => icalendar::TodoStatus::Cancelled,
        }
    }
}

impl From<&icalendar::TodoStatus> for TodoStatus {
    fn from(status: &icalendar::TodoStatus) -> Self {
        match status {
            icalendar::TodoStatus::NeedsAction => TodoStatus::NeedsAction,
            icalendar::TodoStatus::Completed => TodoStatus::Completed,
            icalendar::TodoStatus::InProcess => TodoStatus::InProcess,
            icalendar::TodoStatus::Cancelled => TodoStatus::Cancelled,
        }
    }
}

#[derive(Debug)]
pub struct TodoQuery {
    pub now: NaiveDateTime,
    pub status: Option<TodoStatus>,
    pub due: Option<Duration>,
}

impl TodoQuery {
    pub fn due_before(&self) -> Option<NaiveDateTime> {
        self.due.map(|a| self.now + a)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TodoSortKey {
    Id,
    Due,
    Priority,
}

pub struct TodoSort {
    pub key: TodoSortKey,
    pub order: SortOrder,
}

impl From<(TodoSortKey, SortOrder)> for TodoSort {
    fn from((key, order): (TodoSortKey, SortOrder)) -> Self {
        Self { key, order }
    }
}
