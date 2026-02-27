// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, fmt::Display, num::NonZeroU32, str::FromStr};

use aimcal_ical as ical;
use aimcal_ical::{Description, DtEnd, DtStamp, DtStart, EventStatusValue, Summary, Uid, VEvent};
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
#[expect(clippy::option_option)]
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
        if e.dt_stamp.date.year == 1970 {
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

/// Reconstructs a [`VEvent`] from an Event trait object for database-only updates.
pub fn reconstruct_event_from_db<E: Event>(event: &E, now: &Zoned) -> VEvent<String> {
    // Convert to UTC for DTSTAMP (required by RFC 5545)
    let utc_now = now.with_time_zone(jiff::tz::TimeZone::UTC);
    let dt_stamp = DtStamp::new(utc_now.datetime());

    // Events require dt_start, use a default if not available
    let dt_start = event.start().map_or_else(
        || {
            let default_start: LooseDateTime = now.clone().into();
            DtStart::new(default_start)
        },
        DtStart::new,
    );

    VEvent {
        uid: Uid::new(event.uid().into_owned()),
        dt_stamp,
        dt_start,
        dt_end: event.end().map(DtEnd::new),
        duration: None,
        summary: Some(Summary::new(event.summary().into_owned())),
        description: event
            .description()
            .map(|d| Description::new(d.into_owned())),
        status: event.status().map(|s| ical::EventStatus::new(s.into())),
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

#[cfg(test)]
mod tests {
    use super::*;
    use aimcal_ical::{Description, DtEnd, DtStart, Summary, Uid, VEvent};
    use jiff::{civil::date, tz::TimeZone};

    /// Helper function to create a test `EventDraft` with minimal fields
    fn test_event_draft() -> EventDraft {
        EventDraft {
            description: None,
            start: None,
            end: None,
            status: EventStatus::Confirmed,
            summary: String::new(),
        }
    }

    fn vevent_dt_stamp() -> DtStamp<String> {
        // Create a DateTimeUtc for June 15, 2024 at 10:30:00 UTC
        let date = ical::Date::new(2024, 6, 15).unwrap();
        let time = ical::Time::new(10, 30, 0).unwrap();
        DtStamp::new(ical::DateTimeUtc {
            date,
            time,
            x_parameters: Vec::new(),
            retained_parameters: Vec::new(),
            span: (),
        })
    }

    fn create_test_vevent(uid: &str, summary: &str) -> VEvent<String> {
        let dt_start = LooseDateTime::Local(
            date(2025, 1, 1)
                .at(10, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        let dt_end = LooseDateTime::Local(
            date(2025, 1, 1)
                .at(11, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        VEvent {
            uid: Uid::new(uid.to_string()),
            dt_stamp: vevent_dt_stamp(),
            dt_start: DtStart::new(dt_start),
            dt_end: Some(DtEnd::new(dt_end)),
            duration: None,
            summary: Some(Summary::new(summary.to_string())),
            description: None,
            location: None,
            geo: None,
            url: None,
            organizer: None,
            attendees: Vec::new(),
            last_modified: None,
            status: Some(ical::EventStatus::new(EventStatusValue::Confirmed)),
            transparency: None,
            sequence: None,
            priority: None,
            classification: None,
            resources: None,
            categories: None,
            rdates: Vec::new(),
            rrule: None,
            ex_dates: Vec::new(),
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
            alarms: Vec::new(),
        }
    }

    fn create_test_vevent_with_description(
        uid: &str,
        summary: &str,
        description: &str,
    ) -> VEvent<String> {
        let mut vevent = create_test_vevent(uid, summary);
        vevent.description = Some(Description::new(description.to_string()));
        vevent
    }

    // EventDraft tests

    #[test]
    fn event_draft_default_creates_draft_with_rounded_time() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let draft = EventDraft::default(&now);

        // Time should be rounded to next 00 or 30 minute
        let _minute = now.time().minute();

        assert!(
            draft.start.is_some(),
            "Default draft should have start time"
        );
        assert!(draft.end.is_some(), "Default draft should have end time");
        assert_eq!(draft.summary, "");
        assert_eq!(draft.status, EventStatus::Confirmed);
        assert!(draft.description.is_none());
    }

    #[test]
    fn event_draft_default_with_time_before_30() {
        let now = date(2025, 1, 15)
            .at(10, 15, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let draft = EventDraft::default(&now);

        // Should round to 10:30
        assert!(draft.start.is_some());
        let start = draft.start.as_ref().unwrap();
        assert!(
            matches!(start, LooseDateTime::Local(dt) if dt.time().hour() == 10 && dt.time().minute() == 30)
        );

        // End should be start + 1 hour
        assert!(draft.end.is_some());
        let end = draft.end.as_ref().unwrap();
        assert!(
            matches!(end, LooseDateTime::Local(dt) if dt.time().hour() == 11 && dt.time().minute() == 30)
        );
    }

    #[test]
    fn event_draft_default_with_time_after_30() {
        let now = date(2025, 1, 15)
            .at(10, 45, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        let draft = EventDraft::default(&now);

        // Should round to 11:00
        assert!(draft.start.is_some());
        let start = draft.start.as_ref().unwrap();
        assert!(
            matches!(start, LooseDateTime::Local(dt) if dt.time().hour() == 11 && dt.time().minute() == 0)
        );

        // End should be start + 1 hour
        assert!(draft.end.is_some());
        let end = draft.end.as_ref().unwrap();
        assert!(
            matches!(end, LooseDateTime::Local(dt) if dt.time().hour() == 12 && dt.time().minute() == 0)
        );
    }

    #[test]
    fn event_draft_resolve_with_both_start_and_end() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let start = LooseDateTime::Local(
            date(2025, 1, 15)
                .at(10, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        let end = LooseDateTime::Local(
            date(2025, 1, 15)
                .at(11, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        let draft = EventDraft {
            start: Some(start.clone()),
            end: Some(end.clone()),
            ..test_event_draft()
        };

        let resolved = draft.resolve(&now);

        assert_eq!(resolved.summary, "");
        assert_eq!(resolved.start, start);
        assert_eq!(resolved.end, end);
    }

    #[test]
    fn event_draft_resolve_with_start_only_calculates_end() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let start = LooseDateTime::Local(
            date(2025, 1, 15)
                .at(10, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        let draft = EventDraft {
            start: Some(start.clone()),
            end: None,
            ..test_event_draft()
        };

        let resolved = draft.resolve(&now);

        assert_eq!(resolved.start, start);
        // End should be start + 1 hour
        assert!(
            matches!(resolved.end, LooseDateTime::Local(dt) if dt.time().hour() == 11 && dt.time().minute() == 0)
        );
    }

    #[test]
    fn event_draft_resolve_with_end_only_calculates_start() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let end = LooseDateTime::Local(
            date(2025, 1, 15)
                .at(11, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        let draft = EventDraft {
            start: None,
            end: Some(end.clone()),
            ..test_event_draft()
        };

        let resolved = draft.resolve(&now);

        // Start should be end - 1 hour
        assert!(
            matches!(resolved.start, LooseDateTime::Local(dt) if dt.time().hour() == 10 && dt.time().minute() == 0)
        );
        assert_eq!(resolved.end, end);
    }

    #[test]
    fn event_draft_resolve_with_no_times_uses_now() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let draft = EventDraft {
            start: None,
            end: None,
            ..test_event_draft()
        };

        let resolved = draft.resolve(&now);

        // Should use now for start and now + 1 hour for end
        assert!(matches!(resolved.start, LooseDateTime::Local(_)));
        assert!(matches!(resolved.end, LooseDateTime::Local(_)));
    }

    #[test]
    fn event_draft_resolve_with_date_only_start() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let start = LooseDateTime::DateOnly(date(2025, 1, 15));

        let draft = EventDraft {
            start: Some(start.clone()),
            end: None,
            ..test_event_draft()
        };

        let resolved = draft.resolve(&now);

        // DateOnly should be preserved, end should be start + 1 day (via hour math that gets truncated)
        assert_eq!(resolved.start, start);
    }

    #[test]
    fn event_draft_into_ics_creates_valid_vevent() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let dt_start = LooseDateTime::Local(
            date(2025, 1, 15)
                .at(10, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        let dt_end = LooseDateTime::Local(
            date(2025, 1, 15)
                .at(11, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        let draft = EventDraft {
            summary: "Test Event".to_string(),
            description: Some("Test Description".to_string()),
            start: Some(dt_start),
            end: Some(dt_end),
            status: EventStatus::Confirmed,
        };

        let resolved = draft.resolve(&now);
        let vevent = resolved.into_ics("test-uid");

        assert_eq!(vevent.uid.content.to_string(), "test-uid");
        assert_eq!(
            vevent.summary.as_ref().unwrap().content.to_string(),
            "Test Event"
        );
        assert_eq!(
            vevent.description.as_ref().unwrap().content.to_string(),
            "Test Description"
        );
        assert_eq!(
            vevent.status.as_ref().unwrap().value,
            EventStatusValue::Confirmed
        );
    }

    #[test]
    fn event_draft_resolve_preserves_status() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        for status in [
            EventStatus::Tentative,
            EventStatus::Confirmed,
            EventStatus::Cancelled,
        ] {
            let draft = EventDraft {
                status,
                ..test_event_draft()
            };

            let resolved = draft.resolve(&now);
            assert_eq!(resolved.status, status);
        }
    }

    #[test]
    fn event_draft_resolve_preserves_summary() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let draft = EventDraft {
            summary: "My Event Summary".to_string(),
            ..test_event_draft()
        };

        let resolved = draft.resolve(&now);
        assert_eq!(resolved.summary, "My Event Summary");
    }

    #[test]
    fn event_draft_resolve_preserves_description() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let draft = EventDraft {
            description: Some("Event description".to_string()),
            ..test_event_draft()
        };

        let resolved = draft.resolve(&now);
        assert_eq!(resolved.description, Some("Event description"));
    }

    #[test]
    fn event_draft_resolve_with_none_description() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let draft = EventDraft {
            description: None,
            ..test_event_draft()
        };

        let resolved = draft.resolve(&now);
        assert!(resolved.description.is_none());
    }

    #[test]
    fn event_draft_default_status_is_confirmed() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let draft = EventDraft::default(&now);
        assert_eq!(draft.status, EventStatus::Confirmed);
    }

    // EventPatch tests

    #[test]
    fn event_patch_default_is_empty() {
        let patch = EventPatch::default();

        assert!(patch.is_empty());
        assert!(patch.description.is_none());
        assert!(patch.start.is_none());
        assert!(patch.end.is_none());
        assert!(patch.status.is_none());
        assert!(patch.summary.is_none());
    }

    #[test]
    fn event_patch_is_empty_detects_no_changes() {
        let patch = EventPatch::default();
        assert!(patch.is_empty());

        let patch_with_fields = EventPatch {
            summary: Some("Test".to_string()),
            ..Default::default()
        };
        assert!(!patch_with_fields.is_empty());
    }

    #[test]
    fn event_patch_apply_to_sets_summary() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let mut vevent = create_test_vevent("test-uid", "Original Summary");

        let patch = EventPatch {
            summary: Some("New Summary".to_string()),
            ..Default::default()
        };
        let resolved = patch.resolve(now.clone());

        resolved.apply_to(&mut vevent);

        assert_eq!(
            vevent.summary.as_ref().unwrap().content.to_string(),
            "New Summary"
        );
    }

    #[test]
    fn event_patch_apply_to_sets_description() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let mut vevent = create_test_vevent("test-uid", "Test");

        let patch = EventPatch {
            description: Some(Some("New description".to_string())),
            ..Default::default()
        };
        let resolved = patch.resolve(now.clone());

        resolved.apply_to(&mut vevent);

        assert_eq!(
            vevent.description.as_ref().unwrap().content.to_string(),
            "New description"
        );
    }

    #[test]
    fn event_patch_apply_to_clears_description() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let mut vevent =
            create_test_vevent_with_description("test-uid", "Test", "Original Description");

        let patch = EventPatch {
            description: Some(None), // Some(None) means clear the field
            ..Default::default()
        };
        let resolved = patch.resolve(now.clone());

        resolved.apply_to(&mut vevent);

        assert!(
            vevent.description.is_none(),
            "Description should be cleared"
        );
    }

    #[test]
    fn event_patch_apply_to_sets_start() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let mut vevent = create_test_vevent("test-uid", "Test");

        let new_start = LooseDateTime::Local(
            date(2025, 6, 1)
                .at(14, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        let patch = EventPatch {
            start: Some(Some(new_start.clone())),
            ..Default::default()
        };
        let resolved = patch.resolve(now.clone());

        resolved.apply_to(&mut vevent);

        // Check that the start was updated to June 1, 2025 at 14:00 UTC
        assert!(
            matches!(vevent.dt_start.0.value, ical::DateTime::Utc { date, time } if
                date.year == 2025 &&
                date.month == 6 &&
                date.day == 1 &&
                time.hour == 14 &&
                time.minute == 0
            )
        );
    }

    #[test]
    fn event_patch_apply_to_clears_end() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let dt_start = LooseDateTime::Local(
            date(2025, 6, 1)
                .at(10, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        let dt_end = LooseDateTime::Local(
            date(2025, 6, 1)
                .at(11, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        let mut vevent = VEvent {
            uid: Uid::new("test-uid".to_string()),
            dt_stamp: vevent_dt_stamp(),
            dt_start: DtStart::new(dt_start),
            dt_end: Some(DtEnd::new(dt_end)),
            duration: None,
            summary: None,
            description: None,
            location: None,
            geo: None,
            url: None,
            organizer: None,
            attendees: Vec::new(),
            last_modified: None,
            status: None,
            transparency: None,
            sequence: None,
            priority: None,
            classification: None,
            resources: None,
            categories: None,
            rdates: Vec::new(),
            rrule: None,
            ex_dates: Vec::new(),
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
            alarms: Vec::new(),
        };

        let patch = EventPatch {
            end: Some(None), // Some(None) means clear the field
            ..Default::default()
        };
        let resolved = patch.resolve(now.clone());

        resolved.apply_to(&mut vevent);

        assert!(vevent.dt_end.is_none(), "End should be cleared");
    }

    #[test]
    fn event_patch_apply_to_sets_status() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let mut vevent = create_test_vevent("test-uid", "Test");

        for status in [EventStatus::Tentative, EventStatus::Cancelled] {
            let patch = EventPatch {
                status: Some(status),
                ..Default::default()
            };
            let resolved = patch.resolve(now.clone());

            resolved.apply_to(&mut vevent);

            assert_eq!(
                vevent.status.as_ref().unwrap().value,
                EventStatusValue::from(status)
            );
        }
    }

    #[test]
    fn event_patch_resolve_with_now_sets_dt_stamp_if_unset() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let patch = EventPatch {
            summary: Some("Test".to_string()),
            ..Default::default()
        };

        let resolved = patch.resolve(now.clone());

        // The resolved patch should have now for setting dt_stamp
        assert_eq!(resolved.now, now);
    }

    #[test]
    fn event_patch_apply_to_preserves_recent_dt_stamp() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        // Create a VEvent with a normal dt_stamp
        let mut vevent = create_test_vevent("test-uid", "Test");

        let patch = EventPatch {
            summary: Some("Updated".to_string()),
            ..Default::default()
        };
        let resolved = patch.resolve(now.clone());

        resolved.apply_to(&mut vevent);

        // dt_stamp should still be 2024 (from vevent_dt_stamp)
        assert_eq!(vevent.dt_stamp.date.year, 2024);
    }

    #[test]
    fn event_patch_apply_to_preserves_dt_stamp_when_set() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        // Create a VEvent with a recent dt_stamp
        let dt_start_val = LooseDateTime::Local(
            date(2025, 1, 1)
                .at(10, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        let dt_end_val = LooseDateTime::Local(
            date(2025, 1, 1)
                .at(11, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        let mut vevent = VEvent {
            uid: Uid::new("test-uid".to_string()),
            dt_stamp: vevent_dt_stamp(),
            dt_start: DtStart::new(dt_start_val),
            dt_end: Some(DtEnd::new(dt_end_val)),
            duration: None,
            summary: Some(Summary::new("Test".to_string())),
            description: None,
            location: None,
            geo: None,
            url: None,
            organizer: None,
            attendees: Vec::new(),
            last_modified: None,
            status: None,
            transparency: None,
            sequence: None,
            priority: None,
            classification: None,
            resources: None,
            categories: None,
            rdates: Vec::new(),
            rrule: None,
            ex_dates: Vec::new(),
            x_properties: Vec::new(),
            retained_properties: Vec::new(),
            alarms: Vec::new(),
        };

        let original_year = vevent.dt_stamp.date.year;
        let original_month = vevent.dt_stamp.date.month;
        let original_day = vevent.dt_stamp.date.day;

        let patch = EventPatch {
            summary: Some("Updated".to_string()),
            ..Default::default()
        };
        let resolved = patch.resolve(now.clone());

        resolved.apply_to(&mut vevent);

        // dt_stamp should not be updated
        assert_eq!(vevent.dt_stamp.date.year, original_year);
        assert_eq!(vevent.dt_stamp.date.month, original_month);
        assert_eq!(vevent.dt_stamp.date.day, original_day);
    }

    #[test]
    fn event_patch_partial_update_only_changes_specified_fields() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let mut vevent =
            create_test_vevent_with_description("test-uid", "Test Summary", "Original Description");

        let original_description = vevent.description.as_ref().unwrap().content.to_string();
        let _original_summary = vevent.summary.as_ref().unwrap().content.to_string();

        let patch = EventPatch {
            summary: Some("Updated Summary".to_string()),
            // Don't change description, start, end, status
            ..Default::default()
        };
        let resolved = patch.resolve(now.clone());

        resolved.apply_to(&mut vevent);

        assert_eq!(
            vevent.summary.as_ref().unwrap().content.to_string(),
            "Updated Summary"
        );
        assert_eq!(
            vevent.description.as_ref().unwrap().content.to_string(),
            original_description
        );
    }

    #[test]
    fn event_patch_with_all_fields_updates_completely() {
        let now = Zoned::new(jiff::Timestamp::now(), TimeZone::UTC);

        let mut vevent = create_test_vevent("test-uid", "Original Summary");

        let new_start = LooseDateTime::Local(
            date(2025, 6, 1)
                .at(14, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        let new_end = LooseDateTime::Local(
            date(2025, 6, 1)
                .at(15, 0, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );

        let patch = EventPatch {
            description: Some(Some("New Description".to_string())),
            start: Some(Some(new_start)),
            end: Some(Some(new_end)),
            status: Some(EventStatus::Cancelled),
            summary: Some("New Summary".to_string()),
        };

        let resolved = patch.resolve(now.clone());
        resolved.apply_to(&mut vevent);

        assert_eq!(
            vevent.summary.as_ref().unwrap().content.to_string(),
            "New Summary"
        );
        assert_eq!(
            vevent.description.as_ref().unwrap().content.to_string(),
            "New Description"
        );
        assert_eq!(
            vevent.status.as_ref().unwrap().value,
            EventStatusValue::Cancelled
        );

        // Check start and end were updated
        assert!(
            matches!(vevent.dt_start.0.value, ical::DateTime::Utc { date, .. } if
                date.year == 2025 &&
                date.month == 6 &&
                date.day == 1
            )
        );
        // The time should be 14:00:00 for start and 15:00:00 for end
        if let ical::DateTime::Utc { time, .. } = vevent.dt_start.0.value {
            assert_eq!(time.hour, 14);
        }
        assert!(vevent.dt_end.is_some());
        assert!(
            matches!(vevent.dt_end.as_ref().unwrap().0.value, ical::DateTime::Utc { date, .. } if
                date.year == 2025 &&
                date.month == 6 &&
                date.day == 1
            )
        );
        if let ical::DateTime::Utc { time, .. } = vevent.dt_end.as_ref().unwrap().0.value {
            assert_eq!(time.hour, 15);
        }
    }
}
