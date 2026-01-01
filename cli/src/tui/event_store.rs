// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, error::Error, ops::Deref, rc::Rc};

use aimcal_core::{Aim, Event, EventDraft, EventPatch, EventStatus};

use crate::tui::dispatcher::{Action, Dispatcher};
use crate::util::{format_datetime, parse_datetime};

pub trait EventStoreLike {
    type Output<'a>: Deref<Target = EventStore>
    where
        Self: 'a;

    fn event(&self) -> Self::Output<'_>;
}

#[derive(Debug)]
pub struct EventStore {
    pub data: EventData,
    pub dirty: EventMarker,

    /// Whether the user submit the changes
    pub submit: bool,
}

impl EventStore {
    pub fn from_draft(draft: EventDraft) -> Self {
        Self::new(EventData {
            description: draft.description.unwrap_or_default(),
            end: draft.end.map(format_datetime).unwrap_or_default(),
            start: draft.start.map(format_datetime).unwrap_or_default(),
            status: draft.status,
            summary: draft.summary,
        })
    }

    pub fn from_patch(event: &impl Event, patch: EventPatch) -> Self {
        Self::new(EventData {
            description: match patch.description {
                Some(v) => v.unwrap_or_default(),
                None => event.description().unwrap_or_default().to_owned(),
            },
            start: match patch.start {
                Some(v) => v.map(format_datetime).unwrap_or_default(),
                None => event.start().map(format_datetime).unwrap_or_default(),
            },
            end: match patch.end {
                Some(v) => v.map(format_datetime).unwrap_or_default(),
                None => event.end().map(format_datetime).unwrap_or_default(),
            },
            status: patch.status.or_else(|| event.status()).unwrap_or_default(),
            summary: patch.summary.unwrap_or_else(|| event.summary().to_string()),
        })
    }

    fn new(data: EventData) -> Self {
        Self {
            data,
            dirty: EventMarker::default(),
            submit: false,
        }
    }

    pub fn submit_draft(self, aim: &Aim) -> Result<EventDraft, Box<dyn Error>> {
        Ok(EventDraft {
            description: self.dirty.description.then_some(self.data.description),
            start: parse_datetime(&aim.now(), &self.data.start)?,
            end: parse_datetime(&aim.now(), &self.data.end)?,
            status: self.data.status,
            summary: if self.data.summary.is_empty() {
                "New event".to_string()
            } else {
                self.data.summary
            },
        })
    }

    pub fn submit_patch(self, aim: &Aim) -> Result<EventPatch, Box<dyn Error>> {
        Ok(EventPatch {
            description: match self.dirty.description {
                true if self.data.description.is_empty() => Some(None),
                true => Some(Some(self.data.description.clone())),
                false => None,
            },
            start: match self.dirty.start {
                true => Some(parse_datetime(&aim.now(), &self.data.start)?),
                false => None,
            },
            end: match self.dirty.end {
                true => Some(parse_datetime(&aim.now(), &self.data.end)?),
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
                that.data.description.clone_from(v);
                that.dirty.description = true;
            }
            Action::UpdateEventStart(v) => {
                let mut that = that.borrow_mut();
                that.data.start.clone_from(v);
                that.dirty.start = true;
            }
            Action::UpdateEventEnd(v) => {
                let mut that = that.borrow_mut();
                that.data.end.clone_from(v);
                that.dirty.end = true;
            }
            Action::UpdateEventStatus(v) => {
                let mut that = that.borrow_mut();
                that.data.status = *v;
                that.dirty.status = true;
            }
            Action::UpdateEventSummary(v) => {
                let mut that = that.borrow_mut();
                that.data.summary.clone_from(v);
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
#[expect(clippy::struct_excessive_bools)]
pub struct EventMarker {
    description: bool,
    start: bool,
    end: bool,
    status: bool,
    summary: bool,
}
