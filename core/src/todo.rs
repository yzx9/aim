// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, fmt::Display, num::NonZeroU32, str::FromStr};

use aimcal_ical::{
    Completed, DateTime as IcalDateTime, Description, DtStamp, Due, PercentComplete,
    Priority as IcalPriority, Summary, TodoStatus as IcalTodoStatus, TodoStatusValue, Uid, VTodo,
};
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

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
    fn completed(&self) -> Option<DateTime<Local>>;

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

    fn completed(&self) -> Option<DateTime<Local>> {
        #[expect(clippy::cast_sign_loss)]
        self.completed.as_ref().and_then(|c| match &**c {
            IcalDateTime::Utc { date, time, .. } => {
                let naive_dt = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        i32::from(date.year),
                        date.month as u32,
                        date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(
                        u32::from(time.hour),
                        u32::from(time.minute),
                        u32::from(time.second),
                    )
                    .unwrap(),
                );
                Some(
                    DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).with_timezone(&Local),
                )
            }
            IcalDateTime::Floating { date, time, .. } => {
                let naive_dt = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        i32::from(date.year),
                        date.month as u32,
                        date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(
                        u32::from(time.hour),
                        u32::from(time.minute),
                        u32::from(time.second),
                    )
                    .unwrap(),
                );
                Some(
                    Local
                        .from_local_datetime(&naive_dt)
                        .single()
                        .unwrap_or_else(|| {
                            tracing::warn!("invalid local time, using UTC");
                            DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc)
                                .with_timezone(&Local)
                        }),
                )
            }
            IcalDateTime::Zoned {
                date, time, tz_id, ..
            } => {
                let naive_dt = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        i32::from(date.year),
                        date.month as u32,
                        date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(
                        u32::from(time.hour),
                        u32::from(time.minute),
                        u32::from(time.second),
                    )
                    .unwrap(),
                );
                match tz_id.as_str().parse::<chrono_tz::Tz>() {
                    Ok(tz) => match tz.from_local_datetime(&naive_dt) {
                        chrono::LocalResult::Single(dt) => Some(dt.with_timezone(&Local)),
                        _ => None,
                    },
                    Err(_) => None,
                }
            }
            IcalDateTime::Date { .. } => None,
        })
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.description
            .as_ref()
            .map(|a| a.content.to_string().into()) // PERF: avoid allocation
    }

    fn due(&self) -> Option<LooseDateTime> {
        self.due.as_ref().map(|d| d.inner.clone().into())
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
    /// Converts the draft into an aimcal-ical `VTodo` component.
    pub(crate) fn into_ics(self, uid: &str) -> VTodo<String> {
        VTodo {
            uid: Uid::new(uid.to_string()),
            dt_stamp: DtStamp::new(IcalDateTime::from(LooseDateTime::Local(*self.now))),
            dt_start: None,
            due: self.due.map(|d| Due::new(d.into())),
            completed: None,
            duration: None,
            summary: Some(Summary::new(self.summary.to_string())),
            description: self.description.map(|d| Description::new(d.to_string())),
            status: Some(IcalTodoStatus::new(match self.status {
                TodoStatus::NeedsAction => TodoStatusValue::NeedsAction,
                TodoStatus::Completed => TodoStatusValue::Completed,
                TodoStatus::InProcess => TodoStatusValue::InProcess,
                TodoStatus::Cancelled => TodoStatusValue::Cancelled,
            })),
            percent_complete: self
                .percent_complete
                .map(|p| PercentComplete::new(p.min(100))),
            priority: self
                .priority
                .map(|p| IcalPriority::new(Into::<u8>::into(p))),
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
    pub fn apply_to<'a>(&self, t: &'a mut VTodo<String>) -> &'a mut VTodo<String> {
        if let Some(Some(desc)) = &self.description {
            t.description = Some(Description::new((*desc).to_string()));
        } else if self.description.is_some() {
            t.description = None;
        }

        if let Some(Some(due)) = self.due {
            t.due = Some(Due::new(due.into()));
        } else if self.due.is_some() {
            t.due = None;
        }

        if let Some(Some(v)) = self.percent_complete {
            t.percent_complete = Some(PercentComplete::new(v.min(100)));
        } else if self.percent_complete.is_some() {
            t.percent_complete = None;
        }

        if let Some(priority) = self.priority {
            t.priority = Some(IcalPriority::new(Into::<u8>::into(priority)));
        }

        if let Some(status) = self.status {
            t.status = Some(IcalTodoStatus::new(status.into()));

            // Handle COMPLETED property
            if status == TodoStatus::Completed && t.completed.is_none() {
                t.completed = Some(Completed::new(IcalDateTime::from(LooseDateTime::Local(
                    *self.now,
                ))));
            } else if status != TodoStatus::Completed {
                t.completed = None;
            }
        }

        if let Some(summary) = &self.summary {
            t.summary = Some(Summary::new((*summary).to_string()));
        }

        // Set the creation time to now if it is not already set
        if t.dt_stamp.inner.date().year == 1970 {
            t.dt_stamp = DtStamp::new(IcalDateTime::from(LooseDateTime::Local(*self.now)));
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
