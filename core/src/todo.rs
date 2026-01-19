// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, fmt::Display, num::NonZeroU32, str::FromStr};

use aimcal_ical::{
    self as ical, Completed, Description, DtStamp, Due, PercentComplete, Summary, TodoStatusValue,
    Uid, VTodo,
};
use jiff::Zoned;

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
    fn uid(&self) -> Cow<'_, str>;

    /// The description of the todo item.
    fn completed(&self) -> Option<Zoned>;

    /// The description of the todo item, if available.
    fn description(&self) -> Option<Cow<'_, str>>;

    /// The due date and time of the todo item, if available.
    fn due(&self) -> Option<LooseDateTime>;

    /// The percent complete, from 0 to 100.
    fn percent_complete(&self) -> Option<u8>;

    /// The priority from 1 to 9, where 1 is the highest priority.
    fn priority(&self) -> Priority;

    /// The status of the todo item.
    fn status(&self) -> TodoStatus;

    /// The summary of the todo item.
    fn summary(&self) -> Cow<'_, str>;
}

impl Todo for VTodo<String> {
    fn uid(&self) -> Cow<'_, str> {
        self.uid.content.to_string().into()
    }

    fn completed(&self) -> Option<Zoned> {
        self.completed.as_ref().map(|c| c.zoned())
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.description
            .as_ref()
            .map(|a| a.content.to_string().into()) // PERF: avoid allocation
    }

    fn due(&self) -> Option<LooseDateTime> {
        self.due.as_ref().map(|d| d.0.clone().into())
    }

    fn percent_complete(&self) -> Option<u8> {
        self.percent_complete.as_ref().map(|p| p.value)
    }

    fn priority(&self) -> Priority {
        match self.priority.as_ref() {
            Some(p) => p.value.into(),
            None => Priority::default(),
        }
    }

    fn status(&self) -> TodoStatus {
        self.status
            .as_ref()
            .map(|s| s.value.into())
            .unwrap_or_default()
    }

    fn summary(&self) -> Cow<'_, str> {
        self.summary
            .as_ref()
            .map_or_else(|| "".into(), |s| s.content.to_string().into()) // PERF: avoid allocation
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
    pub(crate) fn default(config: &Config, now: &Zoned) -> Result<Self, String> {
        Ok(Self {
            description: None,
            due: config
                .default_due
                .as_ref()
                .map(|d| d.clone().resolve_since_zoned(now))
                .transpose()?,
            percent_complete: None,
            priority: Some(config.default_priority),
            status: TodoStatus::default(),
            summary: String::default(),
        })
    }

    /// Converts the draft into a icalendar Todo component.
    pub(crate) fn resolve<'a>(&'a self, config: &Config, now: &'a Zoned) -> ResolvedTodoDraft<'a> {
        let due = self.due.clone().or_else(|| {
            config
                .default_due
                .as_ref()
                .map(|d| d.clone().resolve_since_zoned(now))
                .and_then(Result::ok)
        });

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

#[derive(Debug, Clone)]
pub struct ResolvedTodoDraft<'a> {
    pub description: Option<&'a str>,
    pub due: Option<LooseDateTime>,
    pub percent_complete: Option<u8>,
    pub priority: Option<Priority>,
    pub status: TodoStatus,
    pub summary: &'a str,

    pub now: &'a Zoned,
}

impl ResolvedTodoDraft<'_> {
    /// Converts the draft into an aimcal-ical `VTodo` component.
    pub(crate) fn into_ics(self, uid: &str) -> VTodo<String> {
        // Convert to UTC for DTSTAMP (required by RFC 5545)
        let utc_now = self.now.with_time_zone(jiff::tz::TimeZone::UTC);
        let dt_stamp = DtStamp::new(utc_now.datetime());
        VTodo {
            uid: Uid::new(uid.to_string()),
            dt_stamp,
            dt_start: None,
            due: self.due.map(Due::new),
            completed: None,
            duration: None,
            summary: Some(Summary::new(self.summary.to_string())),
            description: self.description.map(|d| Description::new(d.to_string())),
            status: Some(ical::TodoStatus::new(self.status.into())),
            percent_complete: self
                .percent_complete
                .map(|p| PercentComplete::new(p.min(100))),
            priority: self
                .priority
                .map(|p| ical::Priority::new(Into::<u8>::into(p))),
            location: None,
            geo: None,
            url: None,
            organizer: None,
            attendees: Vec::new(),
            last_modified: None,
            sequence: None,
            classification: None,
            resources: None,
            categories: None,
            rrule: None,
            rdates: Vec::new(),
            ex_dates: Vec::new(),
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
            alarms: Vec::new(),
        }
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

    pub(crate) fn resolve<'a>(&'a self, now: &'a Zoned) -> ResolvedTodoPatch<'a> {
        let percent_complete = match self.percent_complete {
            Some(Some(v)) => Some(Some(v.min(100))),
            _ => self.percent_complete,
        };

        ResolvedTodoPatch {
            description: self.description.as_ref().map(|opt| opt.as_deref()),
            due: self.due.clone(),
            percent_complete,
            priority: self.priority,
            status: self.status,
            summary: self.summary.as_deref(),
            now,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedTodoPatch<'a> {
    pub description: Option<Option<&'a str>>,
    pub due: Option<Option<LooseDateTime>>,
    pub percent_complete: Option<Option<u8>>,
    pub priority: Option<Priority>,
    pub status: Option<TodoStatus>,
    pub summary: Option<&'a str>,

    pub now: &'a Zoned,
}

impl ResolvedTodoPatch<'_> {
    /// Applies the patch to a mutable todo item, modifying it in place.
    pub fn apply_to<'a>(&self, t: &'a mut VTodo<String>) -> &'a mut VTodo<String> {
        if let Some(Some(desc)) = self.description {
            t.description = Some(Description::new(desc.to_string()));
        } else if self.description.is_some() {
            t.description = None;
        }

        if let Some(Some(ref due)) = self.due {
            t.due = Some(Due::new(due.clone()));
        } else if self.due.is_some() {
            t.due = None;
        }

        if let Some(Some(v)) = self.percent_complete {
            t.percent_complete = Some(PercentComplete::new(v.min(100)));
        } else if self.percent_complete.is_some() {
            t.percent_complete = None;
        }

        if let Some(priority) = self.priority {
            t.priority = Some(ical::Priority::new(Into::<u8>::into(priority)));
        }

        if let Some(status) = self.status {
            t.status = Some(ical::TodoStatus::new(status.into()));

            // Handle COMPLETED property
            if status == TodoStatus::Completed && t.completed.is_none() {
                let utc_now = self.now.with_time_zone(jiff::tz::TimeZone::UTC);
                t.completed = Some(Completed::new(utc_now.datetime()));
            } else if status != TodoStatus::Completed {
                t.completed = None;
            }
        }

        if let Some(summary) = self.summary {
            t.summary = Some(Summary::new(summary.to_string()));
        }

        // Set the creation time to now if it is not already set
        if t.dt_stamp.date().year == 1970 {
            // TODO: better check for unset
            let utc_now = self.now.with_time_zone(jiff::tz::TimeZone::UTC);
            t.dt_stamp = DtStamp::new(utc_now.datetime());
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
        self.as_ref().fmt(f)
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

impl From<TodoStatusValue> for TodoStatus {
    fn from(value: TodoStatusValue) -> Self {
        match value {
            TodoStatusValue::NeedsAction => TodoStatus::NeedsAction,
            TodoStatusValue::Completed => TodoStatus::Completed,
            TodoStatusValue::InProcess => TodoStatus::InProcess,
            TodoStatusValue::Cancelled => TodoStatus::Cancelled,
        }
    }
}

impl From<TodoStatus> for TodoStatusValue {
    fn from(value: TodoStatus) -> Self {
        match value {
            TodoStatus::NeedsAction => TodoStatusValue::NeedsAction,
            TodoStatus::Completed => TodoStatusValue::Completed,
            TodoStatus::InProcess => TodoStatusValue::InProcess,
            TodoStatus::Cancelled => TodoStatusValue::Cancelled,
        }
    }
}

/// Conditions for filtering todo items, such as current time, status, and due date.
#[derive(Debug, Clone)]
pub struct TodoConditions {
    /// The status of the todo item to filter by, if any.
    pub status: Option<TodoStatus>,

    /// The priority of the todo item to filter by, if any.
    pub due: Option<DateTimeAnchor>,
}

impl TodoConditions {
    pub(crate) fn resolve(&self, now: &Zoned) -> Result<ResolvedTodoConditions, String> {
        Ok(ResolvedTodoConditions {
            status: self.status,
            due: self
                .due
                .as_ref()
                .map(|a| a.resolve_at_end_of_day(now))
                .transpose()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedTodoConditions {
    pub status: Option<TodoStatus>,
    pub due: Option<Zoned>,
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
        sort.iter().map(|s| (*s).resolve(config)).collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResolvedTodoSort {
    Due(SortOrder),
    Priority { order: SortOrder, none_first: bool },
}
