// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use aimcal_core::{EventStatus, Kind, Priority, TodoStatus};

type Callback = Rc<RefCell<dyn FnMut(&Action)>>;

pub struct Dispatcher {
    subscribers: Vec<Callback>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn register(&mut self, callback: Callback) {
        self.subscribers.push(callback);
    }

    pub fn dispatch(&mut self, action: &Action) {
        for sub in &self.subscribers {
            (sub.borrow_mut())(action);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Activate(Kind),
    UpdateTodoDescription(String),
    UpdateTodoDue(String),
    UpdateTodoPercentComplete(Option<u8>),
    UpdateTodoPriority(Priority),
    UpdateTodoStatus(TodoStatus),
    UpdateTodoSummary(String),
    UpdateEventDescription(String),
    UpdateEventStart(String),
    UpdateEventEnd(String),
    UpdateEventStatus(EventStatus),
    UpdateEventSummary(String),
    SubmitChanges,
}
