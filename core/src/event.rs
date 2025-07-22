// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::DatePerhapsTime;
use chrono::NaiveDateTime;
use std::{fmt::Display, str::FromStr};

/// Trait representing a calendar event.
pub trait Event {
    /// Returns the unique identifier for the event.
    fn uid(&self) -> &str;
    /// Returns the summary of the event.
    fn summary(&self) -> &str;
    /// Returns the description of the event, if available.
    fn description(&self) -> Option<&str>;
    /// Returns the location of the event, if available.
    fn start(&self) -> Option<DatePerhapsTime>;
    /// Returns the start date and time of the event, if available.
    fn end(&self) -> Option<DatePerhapsTime>;
    /// Returns the status of the event, if available.
    fn status(&self) -> Option<&str>;
}

/// Represents the status of an event, which can be tentative, confirmed, or cancelled.
#[derive(Debug, Clone, Copy)]
pub enum EventStatus {
    Tentative,
    Confirmed,
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

/// Represents conditions for filtering events in a calendar.
#[derive(Debug, Clone, Copy)]
pub struct EventConditions {
    /// The current time to filter events.
    pub now: NaiveDateTime,
}
