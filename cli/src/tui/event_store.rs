// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, error::Error, ops::Deref, rc::Rc};

use aimcal_core::{Event, EventDraft, EventPatch, EventStatus};

use crate::tui::dispatcher::{Action, Dispatcher};
use crate::util::{format_datetime, parse_datetime};

pub trait EventStoreLike {
    type Output<'a>: Deref<Target = EventStore>
    where
        Self: 'a;

    fn event<'a>(&'a self) -> Self::Output<'a>;
}

#[derive(Debug)]
pub struct EventStore {
    pub data: EventData,
    pub dirty: EventMarker,

    /// Whether the user submit the changes
    pub submit: bool,
}

impl EventStore {
    pub fn new_by_draft(draft: EventDraft) -> Self {
        Self::new(EventData {
            description: draft.description.unwrap_or_default(),
            end: draft.end.map(format_datetime).unwrap_or_default(),
            start: draft.start.map(format_datetime).unwrap_or_default(),
            status: draft.status,
            summary: draft.summary,
        })
    }

    pub fn new_by_event(event: &impl Event) -> Self {
        Self::new(EventData {
            description: event.description().unwrap_or_default().to_owned(),
            start: event.start().map(format_datetime).unwrap_or_default(),
            end: event.end().map(format_datetime).unwrap_or_default(),
            status: event.status().unwrap_or_default(),
            summary: event.summary().to_string(),
        })
    }

    fn new(data: EventData) -> Self {
        Self {
            data,
            dirty: EventMarker::default(),
            submit: false,
        }
    }

    pub fn submit_draft(self) -> Result<EventDraft, Box<dyn Error>> {
        Ok(EventDraft {
            description: self.dirty.description.then_some(self.data.description),
            start: parse_datetime(&self.data.start)?,
            end: parse_datetime(&self.data.end)?,
            status: self.data.status,
            summary: if self.data.summary.is_empty() {
                "New event".to_string()
            } else {
                self.data.summary
            },
        })
    }

    pub fn submit_patch(self) -> Result<EventPatch, Box<dyn Error>> {
        Ok(EventPatch {
            description: match self.dirty.description {
                true if self.data.description.is_empty() => Some(None),
                true => Some(Some(self.data.description.clone())),
                false => None,
            },
            start: match self.dirty.start {
                true => Some(parse_datetime(&self.data.start)?),
                false => None,
            },
            end: match self.dirty.end {
                true => Some(parse_datetime(&self.data.end)?),
                false => None,
            },
            status: self.dirty.status.then_some(self.data.status),
            summary: self.dirty.summary.then(|| self.data.summary.clone()),
        })
    }

    pub fn register_to(that: Rc<RefCell<Self>>, dispatcher: &mut Dispatcher) {
        let callback = Rc::new(RefCell::new(move |action: &Action| match action {
            Action::UpdateEventDescription(v) => {
                let mut that = that.borrow_mut();
                that.data.description = v.clone();
                that.dirty.description = true;
            }
            Action::UpdateEventStart(v) => {
                let mut that = that.borrow_mut();
                that.data.start = v.clone();
                that.dirty.start = true;
            }
            Action::UpdateEventEnd(v) => {
                let mut that = that.borrow_mut();
                that.data.end = v.clone();
                that.dirty.end = true;
            }
            Action::UpdateEventStatus(v) => {
                let mut that = that.borrow_mut();
                that.data.status = *v;
                that.dirty.status = true;
            }
            Action::UpdateEventSummary(v) => {
                let mut that = that.borrow_mut();
                that.data.summary = v.clone();
                that.dirty.summary = true;
            }
            Action::SubmitChanges => {
                let mut that = that.borrow_mut();
                that.submit = true;
            }
            _ => (),
        }));
        dispatcher.register(callback);
    }
}

impl EventStoreLike for EventStore {
    type Output<'a> = &'a EventStore;

    fn event(&self) -> &EventStore {
        self
    }
}

#[derive(Debug)]
pub struct EventData {
    pub description: String,
    pub start: String,
    pub end: String,
    pub status: EventStatus,
    pub summary: String,
}

impl Default for EventData {
    fn default() -> Self {
        Self {
            description: String::new(),
            start: String::new(),
            end: String::new(),
            status: EventStatus::Confirmed,
            summary: String::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct EventMarker {
    description: bool,
    start: bool,
    end: bool,
    status: bool,
    summary: bool,
}
