// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Display, num::NonZeroU32, str::FromStr};

use chrono::{DateTime, Duration, Local, Timelike};
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
#[derive(Debug)]
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
    pub(crate) fn default(now: DateTime<Local>) -> Self {
        // next 00 or 30 minute
        let start = if now.minute() < 30 {
            now.with_minute(30).unwrap().with_second(0).unwrap()
        } else {
            (now + Duration::hours(1))
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

    /// Converts the draft into a icalendar Event component.
    pub(crate) fn into_ics(self, now: &DateTime<Local>, uid: &str) -> icalendar::Event {
        let mut event = icalendar::Event::with_uid(uid);

        if let Some(description) = self.description {
            Component::description(&mut event, &description);
        }

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
        EventLike::starts(&mut event, start);
        EventLike::ends(&mut event, end);

        icalendar::Event::status(&mut event, self.status.into());

        Component::summary(&mut event, &self.summary);

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
    pub fn is_empty(&self) -> bool {
        self.description.is_none()
            && self.start.is_none()
            && self.end.is_none()
            && self.status.is_none()
            && self.summary.is_none()
    }

    /// Applies the patch to a mutable event, modifying it in place.
    pub(crate) fn apply_to<'a>(&self, e: &'a mut icalendar::Event) -> &'a mut icalendar::Event {
        if let Some(description) = &self.description {
            match description {
                Some(desc) => e.description(desc),
                None => e.remove_description(),
            };
        }

        if let Some(start) = &self.start {
            match start {
                Some(s) => e.starts(*s),
                None => e.remove_starts(),
            };
        }

        if let Some(end) = &self.end {
            match end {
                Some(ed) => e.ends(*ed),
                None => e.remove_ends(),
            };
        }

        if let Some(status) = self.status {
            e.status(status.into());
        }

        if let Some(summary) = &self.summary {
            e.summary(summary);
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
        write!(f, "{}", self.as_ref())
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

#[derive(Debug)]
pub(crate) struct ParsedEventConditions {
    /// The date and time after which the event must start
    pub start_before: Option<DateTime<Local>>,

    /// The date and time after which the event must end
    pub end_after: Option<DateTime<Local>>,
}

impl ParsedEventConditions {
    pub fn parse(now: &DateTime<Local>, conds: &EventConditions) -> Self {
        Self {
            start_before: conds.cutoff.map(|w| w.resolve_at_end_of_day(now)),
            end_after: conds.startable.map(|w| w.resolve_at_start_of_day(now)),
        }
    }
}
