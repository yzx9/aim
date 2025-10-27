// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Display, num::NonZeroU32, str::FromStr};

use chrono::{DateTime, Local, Utc};
use icalendar::Component;

use crate::{Config, DateTimeAnchor, LooseDateTime, Priority, SortOrder};

/// Trait representing a todo item.
pub trait Todo {
    /// The short identifier for the todo.
    /// It will be `None` if the event does not have a short ID.
    /// It is used for display purposes and may not be unique.
    fn short_id(&self) -> Option<NonZeroU32> {
        None
    }

    /// The unique identifier for the todo item.
    fn uid(&self) -> &str;

    /// The description of the todo item.
    fn completed(&self) -> Option<DateTime<Local>>;

    /// The description of the todo item, if available.
    fn description(&self) -> Option<&str>;

    /// The due date and time of the todo item, if available.
    fn due(&self) -> Option<LooseDateTime>;

    /// The percent complete, from 0 to 100.
    fn percent_complete(&self) -> Option<u8>;

    /// The priority from 1 to 9, where 1 is the highest priority.
    fn priority(&self) -> Priority;

    /// The status of the todo item.
    fn status(&self) -> TodoStatus;

    /// The summary of the todo item.
    fn summary(&self) -> &str;
}

impl Todo for icalendar::Todo {
    fn uid(&self) -> &str {
        self.get_uid().unwrap_or_default()
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
        match self.get_priority() {
            Some(p) => Priority::from(u8::try_from(p.min(9)).unwrap_or_default()),
            _ => Priority::default(),
        }
    }

    fn status(&self) -> TodoStatus {
        self.get_status().map(Into::into).unwrap_or_default()
    }

    fn summary(&self) -> &str {
        self.get_summary().unwrap_or_default()
    }
}

/// Darft for a todo item, used for creating new todos.
#[derive(Debug)]
pub struct TodoDraft {
    /// The description of the todo item, if available.
    pub description: Option<String>,

    /// The due date and time of the todo item, if available.
    pub due: Option<LooseDateTime>,

    /// The percent complete, from 0 to 100, if available.
    pub percent_complete: Option<u8>,

    /// The priority of the todo item, if available.
    pub priority: Option<Priority>,

    /// The status of the todo item.
    pub status: TodoStatus,

    /// The summary of the todo item.
    pub summary: String,
}

impl TodoDraft {
    /// Creates a new empty patch.
    pub(crate) fn default(config: &Config, now: &DateTime<Local>) -> Self {
        Self {
            description: None,
            due: config.default_due.map(|d| d.resolve_since_datetime(now)),
            percent_complete: None,
            priority: Some(config.default_priority),
            status: TodoStatus::default(),
            summary: String::default(),
        }
    }

    /// Converts the draft into a icalendar Todo component.
    pub(crate) fn resolve<'a>(
        &'a self,
        config: &Config,
        now: &'a DateTime<Local>,
    ) -> ResolvedTodoDraft<'a> {
        let due = self
            .due
            .or_else(|| config.default_due.map(|d| d.resolve_since_datetime(now)));

        let percent_complete = self.percent_complete.map(|a| a.max(100));

        let priority = self.priority.or(Some(config.default_priority));

        ResolvedTodoDraft {
            description: self.description.as_deref(),
            due,
            percent_complete,
            priority,
            status: self.status,
            summary: &self.summary,

            now,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedTodoDraft<'a> {
    pub description: Option<&'a str>,
    pub due: Option<LooseDateTime>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,
    pub status: TodoStatus,
    pub summary: &'a str,

    pub now: &'a DateTime<Local>,
}

impl ResolvedTodoDraft<'_> {
    /// Converts the draft into a icalendar Todo component.
    pub(crate) fn into_ics(self, uid: &str) -> icalendar::Todo {
        let mut todo = icalendar::Todo::with_uid(uid);

        if let Some(description) = self.description {
            Component::description(&mut todo, description);
        }

        if let Some(due) = self.due {
            icalendar::Todo::due(&mut todo, due);
        }

        if let Some(percent) = self.percent_complete {
            icalendar::Todo::percent_complete(&mut todo, percent);
        }

        if let Some(priority) = self.priority {
            Component::priority(&mut todo, priority.into());
        }

        icalendar::Todo::status(&mut todo, self.status.into());

        Component::summary(&mut todo, self.summary);

        // Set the creation time to now
        Component::created(&mut todo, self.now.with_timezone(&Utc));
        todo
    }
}

/// Patch for a todo item, allowing partial updates.
#[derive(Debug, Default, Clone)]
pub struct TodoPatch {
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
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.description.is_none()
            && self.due.is_none()
            && self.percent_complete.is_none()
            && self.priority.is_none()
            && self.status.is_none()
            && self.summary.is_none()
    }

    pub(crate) fn resolve<'a>(&'a self, now: &'a DateTime<Local>) -> ResolvedTodoPatch<'a> {
        let percent_complete = match self.percent_complete {
            Some(Some(v)) => Some(Some(v.min(100))),
            _ => self.percent_complete,
        };

        ResolvedTodoPatch {
            description: self.description.as_ref().map(|opt| opt.as_deref()),
            due: self.due,
            percent_complete,
            priority: self.priority,
            status: self.status,
            summary: self.summary.as_deref(),
            now,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedTodoPatch<'a> {
    pub description: Option<Option<&'a str>>,
    pub due: Option<Option<LooseDateTime>>,
    pub percent_complete: Option<Option<u8>>,
    pub priority: Option<Priority>,
    pub status: Option<TodoStatus>,
    pub summary: Option<&'a str>,

    pub now: &'a DateTime<Local>,
}

impl ResolvedTodoPatch<'_> {
    /// Applies the patch to a mutable todo item, modifying it in place.
    pub fn apply_to<'b>(&self, t: &'b mut icalendar::Todo) -> &'b mut icalendar::Todo {
        match self.description {
            Some(Some(desc)) => t.description(desc),
            Some(None) => t.remove_description(),
            None => t,
        };

        match self.due {
            Some(Some(due)) => t.due(due),
            Some(None) => t.remove_due(),
            None => t,
        };

        match self.percent_complete {
            Some(Some(v)) => t.percent_complete(v),
            Some(None) => t.remove_percent_complete(),
            None => t,
        };

        if let Some(priority) = self.priority {
            t.priority(priority.into());
        }

        if let Some(status) = self.status {
            t.status(status.into());

            match status {
                TodoStatus::Completed => t.completed(self.now.with_timezone(&Utc)),
                _ if t.get_completed().is_some() => t.remove_completed(),
                _ => t,
            };
        }

        if let Some(summary) = &self.summary {
            t.summary(summary);
        }

        // Set the creation time to now if it is not already set
        if t.get_created().is_none() {
            Component::created(t, self.now.with_timezone(&Utc));
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
    /// The status of the todo item to filter by, if any.
    pub status: Option<TodoStatus>,

    /// The priority of the todo item to filter by, if any.
    pub due: Option<DateTimeAnchor>,
}

impl TodoConditions {
    pub(crate) fn resolve(&self, now: &DateTime<Local>) -> ResolvedTodoConditions {
        ResolvedTodoConditions {
            status: self.status,
            due: self.due.map(|a| a.resolve_at_end_of_day(now)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedTodoConditions {
    pub status: Option<TodoStatus>,
    pub due: Option<DateTime<Local>>,
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

        /// Put items with no priority first or last. If none, use the default
        none_first: Option<bool>,
    },
}

impl TodoSort {
    pub(crate) fn resolve(self, config: &Config) -> ResolvedTodoSort {
        match self {
            TodoSort::Due(order) => ResolvedTodoSort::Due(order),
            TodoSort::Priority { order, none_first } => ResolvedTodoSort::Priority {
                order,
                none_first: none_first.unwrap_or(config.default_priority_none_fist),
            },
        }
    }

    pub(crate) fn resolve_vec(sort: &[TodoSort], config: &Config) -> Vec<ResolvedTodoSort> {
        sort.iter().map(|s| s.resolve(config)).collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResolvedTodoSort {
    Due(SortOrder),
    Priority { order: SortOrder, none_first: bool },
}
