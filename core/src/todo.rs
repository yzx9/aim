// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{DatePerhapsTime, Priority, SortOrder};
use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, Utc};
use icalendar::Component;
use std::{fmt::Display, str::FromStr};

const KEY_COMPLETED: &str = "COMPLETED";
const KEY_DESCRIPTION: &str = "DESCRIPTION";
const KEY_DUE: &str = "DUE";

/// Trait representing a todo item.
pub trait Todo {
    /// Returns the unique identifier for the todo item.
    fn uid(&self) -> &str;
    /// Returns the description of the todo item.
    fn completed(&self) -> Option<DateTime<FixedOffset>>;
    /// Returns the description of the todo item, if available.
    fn description(&self) -> Option<&str>;
    /// Returns the due date and time of the todo item, if available.
    fn due(&self) -> Option<DatePerhapsTime>;
    /// The percent complete, from 0 to 100.
    fn percent(&self) -> Option<u8>;
    /// The priority from 1 to 9, where 1 is the highest priority.
    fn priority(&self) -> Priority;
    /// Returns the status of the todo item, if available.
    fn status(&self) -> Option<TodoStatus>;
    /// Returns the summary of the todo item.
    fn summary(&self) -> &str;
}

/// Patch for a todo item, allowing partial updates.
#[derive(Debug, Default, Clone)]
pub struct TodoPatch {
    /// The unique identifier for the todo item.
    pub uid: String,

    /// The completion date and time of the todo item, if available.
    pub completed: Option<Option<DateTime<FixedOffset>>>,

    /// The description of the todo item, if available.
    pub description: Option<Option<String>>,

    /// The due date and time of the todo item, if available.
    pub due: Option<Option<DatePerhapsTime>>,

    /// The percent complete, from 0 to 100.
    pub percent: Option<Option<u8>>,

    /// The priority of the todo item, from 1 to 9, where 1 is the highest priority.
    pub priority: Option<Priority>,

    /// The status of the todo item, if available.
    pub status: Option<TodoStatus>,

    /// The summary of the todo item, if available.
    pub summary: Option<String>,
}

impl TodoPatch {
    /// Is this patch empty, meaning no fields are set
    pub fn is_empty(&self) -> bool {
        self.completed.is_none()
            && self.description.is_none()
            && self.due.is_none()
            && self.percent.is_none()
            && self.priority.is_none()
            && self.status.is_none()
            && self.summary.is_none()
    }

    /// Applies the patch to a mutable todo item, modifying it in place.
    pub fn apply_to<'a>(&self, t: &'a mut icalendar::Todo) -> &'a mut icalendar::Todo {
        if let Some(completed) = self.completed {
            match completed {
                Some(dt) => t.completed(dt.with_timezone(&Utc)),
                None => t.remove_property(KEY_COMPLETED),
            };
        }

        if let Some(description) = &self.description {
            match description {
                Some(desc) => t.description(desc),
                None => t.remove_property(KEY_DESCRIPTION),
            };
        }

        if let Some(due) = &self.due {
            match due {
                Some(d) => t.due(*d),
                None => t.remove_property(KEY_DUE),
            };
        }

        if let Some(percent) = self.percent {
            t.percent_complete(percent.unwrap_or(0));
        }

        if let Some(priority) = self.priority {
            t.priority(priority.into());
        }

        if let Some(status) = self.status {
            t.status(status.into());
        }

        if let Some(summary) = &self.summary {
            t.summary(summary);
        }

        t
    }
}

/// The status of a todo item, which can be one of several predefined states.
#[derive(Debug, Clone, Copy)]
pub enum TodoStatus {
    /// The todo item needs action.
    NeedsAction,
    /// The todo item has been completed.
    Completed,
    /// The todo item is currently in process.
    InProcess,
    /// The todo item has been cancelled.
    Cancelled,
}

const STATUS_NEEDS_ACTION: &str = "NEEDS-ACTION";
const STATUS_COMPLETED: &str = "COMPLETED";
const STATUS_IN_PROCESS: &str = "IN-PROGRESS";
const STATUS_CANCELLED: &str = "CANCELLED";

impl AsRef<str> for TodoStatus {
    fn as_ref(&self) -> &str {
        match self {
            TodoStatus::NeedsAction => STATUS_NEEDS_ACTION,
            TodoStatus::Completed => STATUS_COMPLETED,
            TodoStatus::InProcess => STATUS_IN_PROCESS,
            TodoStatus::Cancelled => STATUS_CANCELLED,
        }
    }
}

impl Display for TodoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl FromStr for TodoStatus {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            STATUS_NEEDS_ACTION => Ok(TodoStatus::NeedsAction),
            STATUS_COMPLETED => Ok(TodoStatus::Completed),
            STATUS_IN_PROCESS => Ok(TodoStatus::InProcess),
            STATUS_CANCELLED => Ok(TodoStatus::Cancelled),
            _ => Err(()),
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

/// Conditions for filtering todo items, such as current time, status, and due date.
#[derive(Debug, Clone, Copy)]
pub struct TodoConditions {
    /// The current time, used for filtering todos.
    pub now: NaiveDateTime,

    /// The status of the todo item to filter by, if any.
    pub status: Option<TodoStatus>,

    /// The priority of the todo item to filter by, if any.
    pub due: Option<Duration>,
}

impl TodoConditions {
    /// The due datetime.
    pub fn due_before(&self) -> Option<NaiveDateTime> {
        self.due.map(|a| self.now + a)
    }
}

/// The key by which todo items can be sorted.
#[derive(Debug, Clone, Copy)]
pub enum TodoSortKey {
    /// Sort by the due date and time of the todo item.
    Due,
    /// Sort by the priority of the todo item.
    Priority,
}

/// The default sort key for todo items, which is by due date.
#[derive(Debug, Clone, Copy)]
pub struct TodoSort {
    /// The key by which to sort the todo items.
    pub key: TodoSortKey,
    /// The order in which to sort the todo items (ascending or descending).
    pub order: SortOrder,
}

impl From<(TodoSortKey, SortOrder)> for TodoSort {
    fn from((key, order): (TodoSortKey, SortOrder)) -> Self {
        Self { key, order }
    }
}
