// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{LooseDateTime, Priority, SortOrder};
use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};
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
    fn completed(&self) -> Option<DateTime<Local>>;

    /// Returns the description of the todo item, if available.
    fn description(&self) -> Option<&str>;

    /// Returns the due date and time of the todo item, if available.
    fn due(&self) -> Option<LooseDateTime>;

    /// The percent complete, from 0 to 100.
    fn percent_complete(&self) -> Option<u8>;

    /// The priority from 1 to 9, where 1 is the highest priority.
    fn priority(&self) -> Priority;

    /// Returns the status of the todo item.
    fn status(&self) -> TodoStatus;

    /// Returns the summary of the todo item.
    fn summary(&self) -> &str;
}

impl Todo for icalendar::Todo {
    fn uid(&self) -> &str {
        self.get_uid().unwrap_or("")
    }

    fn completed(&self) -> Option<DateTime<Local>> {
        self.get_completed().map(|dt| dt.with_timezone(&Local))
    }

    fn description(&self) -> Option<&str> {
        self.get_description()
    }

    fn due(&self) -> Option<LooseDateTime> {
        self.get_due().map(Into::into)
    }

    fn percent_complete(&self) -> Option<u8> {
        self.get_percent_complete()
    }

    fn priority(&self) -> Priority {
        self.get_priority()
            .map(|p| Priority::from(p as u8))
            .unwrap_or_default()
    }

    fn status(&self) -> TodoStatus {
        self.get_status().map(Into::into).unwrap_or_default()
    }

    fn summary(&self) -> &str {
        self.get_summary().unwrap_or("")
    }
}

/// Darft for a todo item, used for creating new todos.
#[derive(Debug)]
pub struct TodoDraft {
    /// The description of the todo item, if available.
    pub description: Option<String>,

    /// The due date and time of the todo item, if available.
    pub due: Option<LooseDateTime>,

    /// The priority of the todo item.
    pub priority: Priority,

    /// The summary of the todo item.
    pub summary: String,
}

impl TodoDraft {
    /// Converts the draft into a icalendar Todo component.
    pub(crate) fn into_todo(self, uid: &str) -> icalendar::Todo {
        let mut todo = icalendar::Todo::with_uid(uid);

        icalendar::Todo::status(&mut todo, icalendar::TodoStatus::NeedsAction);
        Component::priority(&mut todo, self.priority.into());
        Component::summary(&mut todo, &self.summary);

        if let Some(description) = self.description {
            Component::description(&mut todo, &description);
        }
        if let Some(due) = self.due {
            icalendar::Todo::due(&mut todo, due);
        }

        todo
    }
}

/// Patch for a todo item, allowing partial updates.
#[derive(Debug, Default, Clone)]
pub struct TodoPatch {
    /// The unique identifier for the todo item.
    pub uid: String,

    /// The description of the todo item, if available.
    pub description: Option<Option<String>>,

    /// The due date and time of the todo item, if available.
    pub due: Option<Option<LooseDateTime>>,

    /// The percent complete, from 0 to 100.
    pub percent_complete: Option<Option<u8>>,

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
        self.description.is_none()
            && self.due.is_none()
            && self.percent_complete.is_none()
            && self.priority.is_none()
            && self.status.is_none()
            && self.summary.is_none()
    }

    /// Applies the patch to a mutable todo item, modifying it in place.
    pub(crate) fn apply_to<'a>(&self, t: &'a mut icalendar::Todo) -> &'a mut icalendar::Todo {
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

        if let Some(percent) = self.percent_complete {
            t.percent_complete(percent.unwrap_or(0));
        }

        if let Some(priority) = self.priority {
            t.priority(priority.into());
        }

        if let Some(status) = self.status {
            t.status(status.into());

            match status {
                TodoStatus::Completed => t.completed(Utc::now()),
                _ if t.get_completed().is_some() => t.remove_property(KEY_COMPLETED),
                _ => t,
            };
        }

        if let Some(summary) = &self.summary {
            t.summary(summary);
        }

        t
    }
}

/// The status of a todo item, which can be one of several predefined states.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum TodoStatus {
    /// The todo item needs action.
    #[default]
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

impl From<icalendar::TodoStatus> for TodoStatus {
    fn from(status: icalendar::TodoStatus) -> Self {
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

/// The default sort key for todo items, which is by due date.
#[derive(Debug, Clone, Copy)]
pub enum TodoSort {
    /// Sort by the due date and time of the todo item.
    Due(SortOrder),

    /// Sort by the priority of the todo item.
    Priority {
        /// Sort order, either ascending or descending.
        order: SortOrder,

        /// Put items with no priority first or last.
        none_first: bool,
    },
}
