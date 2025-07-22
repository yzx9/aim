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

pub trait Todo {
    fn uid(&self) -> &str;
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

#[derive(Debug, Default, Clone)]
pub struct TodoPatch {
    pub uid: String,
    pub completed: Option<Option<DateTime<FixedOffset>>>,
    pub description: Option<Option<String>>,
    pub due: Option<Option<DatePerhapsTime>>,
    pub percent: Option<Option<u8>>,
    pub priority: Option<Priority>,
    pub status: Option<TodoStatus>,
    pub summary: Option<String>,
}

impl TodoPatch {
    pub fn is_empty(&self) -> bool {
        self.completed.is_none()
            && self.description.is_none()
            && self.due.is_none()
            && self.percent.is_none()
            && self.priority.is_none()
            && self.status.is_none()
            && self.summary.is_none()
    }

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

#[derive(Debug)]
pub struct TodoConditions {
    pub now: NaiveDateTime,
    pub status: Option<TodoStatus>,
    pub due: Option<Duration>,
}

impl TodoConditions {
    pub fn due_before(&self) -> Option<NaiveDateTime> {
        self.due.map(|a| self.now + a)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TodoSortKey {
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
