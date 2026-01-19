// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, fmt::Display, num::NonZeroU32, str::FromStr};

use aimcal_ical::{
    self as ical, Description, DtEnd, DtStamp, DtStart, EventStatusValue, Summary, Uid, VEvent,
};
use jiff::{Span, ToSpan, Zoned};

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
    fn uid(&self) -> Cow<'_, str>;

    /// The description of the event, if available.
    fn description(&self) -> Option<Cow<'_, str>>;

    /// The location of the event, if available.
    fn start(&self) -> Option<LooseDateTime>;

    /// The start date and time of the event, if available.
    fn end(&self) -> Option<LooseDateTime>;

    /// The status of the event, if available.
    fn status(&self) -> Option<EventStatus>;

    /// The summary of the event.
    fn summary(&self) -> Cow<'_, str>;
}

impl Event for VEvent<String> {
    fn uid(&self) -> Cow<'_, str> {
        self.uid.content.to_string().into() // PERF: avoid allocation
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.description
            .as_ref()
            .map(|a| a.content.to_string().into()) // PERF: avoid allocation
    }

    fn start(&self) -> Option<LooseDateTime> {
        Some(self.dt_start.0.clone().into())
    }

    fn end(&self) -> Option<LooseDateTime> {
        self.dt_end.as_ref().map(|dt| dt.0.clone().into())
    }

    fn status(&self) -> Option<EventStatus> {
        self.status.as_ref().map(|s| s.value.into())
    }

    fn summary(&self) -> Cow<'_, str> {
        self.summary
            .as_ref()
            .map_or_else(|| "".into(), |s| s.content.to_string().into()) // PERF: avoid allocation
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
    pub(crate) fn default(now: &Zoned) -> Self {
        // next 00 or 30 minute
        let start = if now.time().minute() < 30 {
            now.with()
                .minute(30)
                .second(0)
                .subsec_nanosecond(0)
                .build()
                .unwrap()
        } else {
            (now + Span::new().hours(1))
                .with()
                .minute(0)
                .second(0)
                .subsec_nanosecond(0)
                .build()
                .unwrap()
        };

        Self {
            description: None,
            start: Some(start.clone().into()),
            end: Some((start.checked_add(1.hours()).unwrap()).into()),
            status: EventStatus::default(),
            summary: String::new(),
        }
    }

    pub(crate) fn resolve<'a>(&'a self, now: &'a Zoned) -> ResolvedEventDraft<'a> {
        let default_duration = 1.hours();
        let (start, end) = match (self.start.as_ref(), self.end.as_ref()) {
            (Some(start), Some(end)) => (start.clone(), end.clone()),
            (None, Some(end)) => {
                // If start is not specified, but end is, set start to end - duration
                let neg_duration = Span::new().hours(-1);
                let start = match end {
                    LooseDateTime::DateOnly(d) => (*d).into(),
                    LooseDateTime::Floating(dt) => {
                        LooseDateTime::Floating(dt.checked_add(neg_duration).unwrap())
                    }
                    LooseDateTime::Local(dt) => {
                        LooseDateTime::Local(dt.checked_add(neg_duration).unwrap())
                    }
                };
                (start, end.clone())
            }
            (Some(start), None) => {
                // If end is not specified, but start is, set it to start + duration
                let end = match start {
                    LooseDateTime::DateOnly(d) => (*d).into(),
                    LooseDateTime::Floating(dt) => {
                        LooseDateTime::Floating(dt.checked_add(default_duration).unwrap())
                    }
                    LooseDateTime::Local(dt) => {
                        LooseDateTime::Local(dt.checked_add(default_duration).unwrap())
                    }
                };
                (start.clone(), end)
            }
            (None, None) => {
                let end = now.checked_add(default_duration).unwrap();
                (LooseDateTime::Local(now.clone()), LooseDateTime::Local(end))
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

#[derive(Debug, Clone)]
pub struct ResolvedEventDraft<'a> {
    pub description: Option<&'a str>,
    pub start: LooseDateTime,
    pub end: LooseDateTime,
    pub status: EventStatus,
    pub summary: &'a str,

    pub now: &'a Zoned,
}

impl ResolvedEventDraft<'_> {
    /// Converts the draft into an aimcal-ical `VEvent` component.
    pub(crate) fn into_ics(self, uid: &str) -> VEvent<String> {
        // Convert to UTC for DTSTAMP (required by RFC 5545)
        let utc_now = self.now.with_time_zone(jiff::tz::TimeZone::UTC);
        let dt_stamp = DtStamp::new(utc_now.datetime());
        VEvent {
            uid: Uid::new(uid.to_string()),
            dt_stamp,
            dt_start: DtStart::new(self.start),
            dt_end: Some(DtEnd::new(self.end)),
            duration: None,
            summary: Some(Summary::new(self.summary.to_string())),
            description: self.description.map(|d| Description::new(d.to_string())),
            status: Some(ical::EventStatus::new(self.status.into())),
            location: None,
            geo: None,
            url: None,
            organizer: None,
            attendees: Vec::new(),
            last_modified: None,
            transparency: None,
            sequence: None,
            priority: None,
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

    pub(crate) fn resolve(&self, now: Zoned) -> ResolvedEventPatch<'_> {
        ResolvedEventPatch {
            description: self.description.as_ref().map(|opt| opt.as_deref()),
            start: self.start.clone(),
            end: self.end.clone(),
            status: self.status,
            summary: self.summary.as_deref(),

            now,
        }
    }
}

/// Patch for an event, allowing partial updates.
#[derive(Debug, Default, Clone)]
pub struct ResolvedEventPatch<'a> {
    pub description: Option<Option<&'a str>>,
    pub start: Option<Option<LooseDateTime>>,
    pub end: Option<Option<LooseDateTime>>,
    pub status: Option<EventStatus>,
    pub summary: Option<&'a str>,

    pub now: Zoned,
}

impl ResolvedEventPatch<'_> {
    /// Applies the patch to a mutable event, modifying it in place.
    pub fn apply_to<'a>(&self, e: &'a mut VEvent<String>) -> &'a mut VEvent<String> {
        if let Some(Some(desc)) = self.description {
            e.description = Some(Description::new(desc.to_string()));
        } else if self.description.is_some() {
            e.description = None;
        }

        if let Some(Some(ref start)) = self.start {
            e.dt_start = DtStart::new(start.clone());
        }

        if let Some(Some(ref end)) = self.end {
            e.dt_end = Some(DtEnd::new(end.clone()));
        } else if self.end.is_some() {
            e.dt_end = None;
        }

        if let Some(status) = self.status {
            e.status = Some(ical::EventStatus::new(status.into()));
        }

        if let Some(summary) = self.summary {
            e.summary = Some(Summary::new(summary.to_string()));
        }

        // Set the creation time to now if it is not already set
        if e.dt_stamp.date().year == 1970 {
            // TODO: better check for unset
            let utc_now = self.now.with_time_zone(jiff::tz::TimeZone::UTC);
            e.dt_stamp = DtStamp::new(utc_now.datetime());
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

// TODO: should be removed
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

impl From<EventStatusValue> for EventStatus {
    fn from(value: EventStatusValue) -> Self {
        match value {
            EventStatusValue::Tentative => EventStatus::Tentative,
            EventStatusValue::Confirmed => EventStatus::Confirmed,
            EventStatusValue::Cancelled => EventStatus::Cancelled,
        }
    }
}

impl From<EventStatus> for EventStatusValue {
    fn from(value: EventStatus) -> Self {
        match value {
            EventStatus::Tentative => EventStatusValue::Tentative,
            EventStatus::Confirmed => EventStatusValue::Confirmed,
            EventStatus::Cancelled => EventStatusValue::Cancelled,
        }
    }
}

/// Conditions for filtering events in a calendar.
#[derive(Debug, Default, Clone)]
pub struct EventConditions {
    /// Whether to include only startable events.
    pub startable: Option<DateTimeAnchor>,
    /// The cutoff date and time, events ending after this will be excluded.
    pub cutoff: Option<DateTimeAnchor>,
}

impl EventConditions {
    pub(crate) fn resolve(&self, now: &Zoned) -> Result<ResolvedEventConditions, String> {
        Ok(ResolvedEventConditions {
            start_before: self
                .cutoff
                .as_ref()
                .map(|w| w.resolve_at_end_of_day(now))
                .transpose()?,
            end_after: self
                .startable
                .as_ref()
                .map(|w| w.resolve_at_start_of_day(now))
                .transpose()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedEventConditions {
    /// The date and time after which the event must start
    pub start_before: Option<Zoned>,
    /// The date and time after which the event must end
    pub end_after: Option<Zoned>,
}
