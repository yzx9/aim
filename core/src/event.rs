// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Display, num::NonZeroU32, str::FromStr};

use chrono::{DateTime, Duration, Local, Timelike, Utc};
use icalendar::{Component, EventLike};

use crate::{DateTimeAnchor, LooseDateTime};

/// Trait representing a calendar event.
pub trait Event {
    /// The short identifier for the event.
    /// It will be `None` if the event does not have a short ID.
    /// It is used for display purposes and may not be unique.
    fn short_id(&self) -> Option<NonZeroU32> {
        None
    }

    /// The unique identifier for the event.
    fn uid(&self) -> &str;

    /// The description of the event, if available.
    fn description(&self) -> Option<&str>;

    /// The location of the event, if available.
    fn start(&self) -> Option<LooseDateTime>;

    /// The start date and time of the event, if available.
    fn end(&self) -> Option<LooseDateTime>;

    /// The status of the event, if available.
    fn status(&self) -> Option<EventStatus>;

    /// The summary of the event.
    fn summary(&self) -> &str;
}

impl Event for icalendar::Event {
    fn uid(&self) -> &str {
        self.get_uid().unwrap_or_default()
    }

    fn description(&self) -> Option<&str> {
        self.get_description()
    }

    fn start(&self) -> Option<LooseDateTime> {
        self.get_start().map(Into::into)
    }

    fn end(&self) -> Option<LooseDateTime> {
        self.get_end().map(Into::into)
    }

    fn status(&self) -> Option<EventStatus> {
        self.get_status().map(EventStatus::from)
    }

    fn summary(&self) -> &str {
        self.get_summary().unwrap_or_default()
    }
}

/// Darft for an event, used for creating new events.
#[derive(Debug, Clone)]
pub struct EventDraft {
    /// The description of the event, if available.
    pub description: Option<String>,

    /// The start date and time of the event, if available.
    pub start: Option<LooseDateTime>,

    /// The end date and time of the event, if available.
    pub end: Option<LooseDateTime>,

    /// The status of the event.
    pub status: EventStatus,

    /// The summary of the event.
    pub summary: String,
}

impl EventDraft {
    /// Creates a new empty patch.
    pub(crate) fn default(now: &DateTime<Local>) -> Self {
        // next 00 or 30 minute
        let start = if now.minute() < 30 {
            now.with_minute(30).unwrap().with_second(0).unwrap()
        } else {
            (*now + Duration::hours(1))
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
        };

        Self {
            description: None,
            start: Some(start.into()),
            end: Some((start + Duration::hours(1)).into()),
            status: EventStatus::default(),
            summary: String::new(),
        }
    }

    pub(crate) fn resolve<'a>(&'a self, now: &'a DateTime<Local>) -> ResolvedEventDraft<'a> {
        let default_duration = Duration::hours(1);
        let (start, end) = match (self.start, self.end) {
            (Some(start), Some(end)) => (start, end),
            (None, Some(end)) => {
                // If start is not specified, but end is, set start to end - duration
                let start = match end {
                    LooseDateTime::DateOnly(d) => d.into(),
                    LooseDateTime::Floating(dt) => (dt - default_duration).into(),
                    LooseDateTime::Local(dt) => (dt - default_duration).into(),
                };
                (start, end)
            }
            (Some(start), None) => {
                // If end is not specified, but start is, set it to start + duration
                let end = match start {
                    LooseDateTime::DateOnly(d) => d.into(),
                    LooseDateTime::Floating(dt) => (dt + default_duration).into(),
                    LooseDateTime::Local(dt) => (dt + default_duration).into(),
                };
                (start, end)
            }
            (None, None) => {
                let start = *now;
                let end = (start + default_duration).into();
                (start.into(), end)
            }
        };

        ResolvedEventDraft {
            description: self.description.as_deref(),
            start,
            end,
            status: self.status,
            summary: &self.summary,

            now,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedEventDraft<'a> {
    pub description: Option<&'a str>,
    pub start: LooseDateTime,
    pub end: LooseDateTime,
    pub status: EventStatus,
    pub summary: &'a str,

    pub now: &'a DateTime<Local>,
}

impl ResolvedEventDraft<'_> {
    /// Converts the draft into a icalendar Event component.
    pub(crate) fn into_ics(self, uid: &str) -> icalendar::Event {
        let mut event = icalendar::Event::with_uid(uid);

        if let Some(description) = self.description {
            Component::description(&mut event, description);
        }

        EventLike::starts(&mut event, self.start);
        EventLike::ends(&mut event, self.end);

        icalendar::Event::status(&mut event, self.status.into());

        Component::summary(&mut event, self.summary);

        // Set the creation time to now
        Component::created(&mut event, self.now.with_timezone(&Utc));
        event
    }
}

/// Patch for an event, allowing partial updates.
#[derive(Debug, Default, Clone)]
pub struct EventPatch {
    /// The description of the event, if available.
    pub description: Option<Option<String>>,

    /// The start date and time of the event, if available.
    pub start: Option<Option<LooseDateTime>>,

    /// The end date and time of the event, if available.
    pub end: Option<Option<LooseDateTime>>,

    /// The status of the event, if available.
    pub status: Option<EventStatus>,

    /// The summary of the event, if available.
    pub summary: Option<String>,
}

impl EventPatch {
    /// Is this patch empty, meaning no fields are set
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.description.is_none()
            && self.start.is_none()
            && self.end.is_none()
            && self.status.is_none()
            && self.summary.is_none()
    }

    pub(crate) fn resolve(&self, now: DateTime<Local>) -> ResolvedEventPatch<'_> {
        ResolvedEventPatch {
            description: self.description.as_ref().map(|opt| opt.as_deref()),
            start: self.start,
            end: self.end,
            status: self.status,
            summary: self.summary.as_deref(),

            now,
        }
    }
}

/// Patch for an event, allowing partial updates.
#[derive(Debug, Default, Clone, Copy)]
pub struct ResolvedEventPatch<'a> {
    pub description: Option<Option<&'a str>>,
    pub start: Option<Option<LooseDateTime>>,
    pub end: Option<Option<LooseDateTime>>,
    pub status: Option<EventStatus>,
    pub summary: Option<&'a str>,

    pub now: DateTime<Local>,
}

impl ResolvedEventPatch<'_> {
    /// Applies the patch to a mutable event, modifying it in place.
    pub fn apply_to<'b>(&self, e: &'b mut icalendar::Event) -> &'b mut icalendar::Event {
        match self.description {
            Some(Some(desc)) => e.description(desc),
            Some(None) => e.remove_description(),
            None => e,
        };

        match self.start {
            Some(Some(start)) => e.starts(start),
            Some(None) => e.remove_starts(),
            None => e,
        };

        match self.end {
            Some(Some(end)) => e.ends(end),
            Some(None) => e.remove_ends(),
            None => e,
        };

        if let Some(status) = self.status {
            e.status(status.into());
        }

        if let Some(summary) = &self.summary {
            e.summary(summary);
        }

        // Set the creation time to now if it is not already set
        if e.get_created().is_none() {
            Component::created(e, self.now.with_timezone(&Utc));
        }
        e
    }
}

/// The status of an event, which can be tentative, confirmed, or cancelled.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum EventStatus {
    /// The event is tentative.
    Tentative,

    /// The event is confirmed.
    #[default]
    Confirmed,

    /// The event is cancelled.
    Cancelled,
}

const STATUS_TENTATIVE: &str = "TENTATIVE";
const STATUS_CONFIRMED: &str = "CONFIRMED";
const STATUS_CANCELLED: &str = "CANCELLED";

impl AsRef<str> for EventStatus {
    fn as_ref(&self) -> &str {
        match self {
            EventStatus::Tentative => STATUS_TENTATIVE,
            EventStatus::Confirmed => STATUS_CONFIRMED,
            EventStatus::Cancelled => STATUS_CANCELLED,
        }
    }
}

impl Display for EventStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl FromStr for EventStatus {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            STATUS_TENTATIVE => Ok(EventStatus::Tentative),
            STATUS_CONFIRMED => Ok(EventStatus::Confirmed),
            STATUS_CANCELLED => Ok(EventStatus::Cancelled),
            _ => Err(()),
        }
    }
}

impl From<EventStatus> for icalendar::EventStatus {
    fn from(status: EventStatus) -> Self {
        match status {
            EventStatus::Tentative => icalendar::EventStatus::Tentative,
            EventStatus::Confirmed => icalendar::EventStatus::Confirmed,
            EventStatus::Cancelled => icalendar::EventStatus::Cancelled,
        }
    }
}

impl From<icalendar::EventStatus> for EventStatus {
    fn from(status: icalendar::EventStatus) -> Self {
        match status {
            icalendar::EventStatus::Tentative => EventStatus::Tentative,
            icalendar::EventStatus::Confirmed => EventStatus::Confirmed,
            icalendar::EventStatus::Cancelled => EventStatus::Cancelled,
        }
    }
}

/// Conditions for filtering events in a calendar.
#[derive(Debug, Default, Clone, Copy)]
pub struct EventConditions {
    /// Whether to include only startable events.
    pub startable: Option<DateTimeAnchor>,

    /// The cutoff date and time, events ending after this will be excluded.
    pub cutoff: Option<DateTimeAnchor>,
}

impl EventConditions {
    pub(crate) fn resolve(&self, now: &DateTime<Local>) -> ResolvedEventConditions {
        ResolvedEventConditions {
            start_before: self.cutoff.map(|w| w.resolve_at_end_of_day(now)),
            end_after: self.startable.map(|w| w.resolve_at_start_of_day(now)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedEventConditions {
    /// The date and time after which the event must start
    pub start_before: Option<DateTime<Local>>,

    /// The date and time after which the event must end
    pub end_after: Option<DateTime<Local>>,
}
