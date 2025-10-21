// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, error::Error, ops::Deref, rc::Rc};

use aimcal_core::{Aim, Priority, Todo, TodoDraft, TodoPatch, TodoStatus};

use crate::tui::dispatcher::{Action, Dispatcher};
use crate::util::{format_datetime, parse_datetime};

pub trait TodoStoreLike {
    type Output<'a>: Deref<Target = TodoStore>
    where
        Self: 'a;

    fn todo(&self) -> Self::Output<'_>;
}

#[derive(Debug)]
pub struct TodoStore {
    pub data: TodoData,
    pub dirty: TodoMarker,

    /// Whether to show verbose priority options
    pub verbose_priority: bool,

    /// Whether the user submit the changes
    pub submit: bool,
}

impl TodoStore {
    pub fn from_draft(draft: TodoDraft) -> Self {
        Self::new(TodoData {
            description: draft.description.unwrap_or_default(),
            due: draft.due.map(format_datetime).unwrap_or_default(),
            percent_complete: draft.percent_complete,
            priority: draft.priority.unwrap_or_default(),
            status: draft.status,
            summary: draft.summary,
        })
    }

    pub fn from_patch(todo: &impl Todo, patch: TodoPatch) -> Self {
        Self::new(TodoData {
            description: match patch.description {
                Some(v) => v.unwrap_or_default(),
                None => todo.description().unwrap_or_default().to_owned(),
            },
            due: match patch.due {
                Some(v) => v.map(format_datetime).unwrap_or_default(),
                None => todo.due().map(format_datetime).unwrap_or_default(),
            },
            percent_complete: patch
                .percent_complete
                .unwrap_or_else(|| todo.percent_complete()),
            priority: patch.priority.unwrap_or_else(|| todo.priority()),
            status: patch.status.unwrap_or_else(|| todo.status()),
            summary: patch.summary.unwrap_or_else(|| todo.summary().to_owned()),
        })
    }

    fn new(data: TodoData) -> Self {
        use Priority::{P1, P3, P4, P6, P7, P9};
        let verbose_priority = matches!(data.priority, P1 | P3 | P4 | P6 | P7 | P9);
        Self {
            data,
            dirty: TodoMarker::default(),
            verbose_priority,
            submit: false,
        }
    }

    pub fn submit_draft(self, aim: &Aim) -> Result<TodoDraft, Box<dyn Error>> {
        Ok(TodoDraft {
            description: self.dirty.description.then_some(self.data.description),
            due: parse_datetime(&aim.now(), &self.data.due)?,
            percent_complete: self
                .dirty
                .percent_complete
                .then_some(self.data.percent_complete)
                .flatten(),
            priority: Some(self.data.priority), // Always commit since it was confirmed by the user
            status: self.data.status,
            summary: if self.data.summary.is_empty() {
                "New todo".to_string()
            } else {
                self.data.summary
            },
        })
    }

    pub fn submit_patch(self, aim: &Aim) -> Result<TodoPatch, Box<dyn Error>> {
        Ok(TodoPatch {
            description: match self.dirty.description {
                true if self.data.description.is_empty() => Some(None),
                true => Some(Some(self.data.description.clone())),
                false => None,
            },
            due: match self.dirty.due {
                true => Some(parse_datetime(&aim.now(), &self.data.due)?),
                false => None,
            },
            percent_complete: self
                .dirty
                .percent_complete
                .then_some(self.data.percent_complete),
            priority: self.dirty.priority.then_some(self.data.priority),
            status: self.dirty.status.then_some(self.data.status),
            summary: self.dirty.summary.then(|| self.data.summary.clone()),
        })
    }

    pub fn register_to(that: Rc<RefCell<Self>>, dispatcher: &mut Dispatcher) {
        let callback = Rc::new(RefCell::new(move |action: &Action| match action {
            Action::UpdateTodoDescription(v) => {
                let mut that = that.borrow_mut();
                that.data.description.clone_from(v);
                that.dirty.description = true;
            }
            Action::UpdateTodoDue(v) => {
                let mut that = that.borrow_mut();
                that.data.due.clone_from(v);
                that.dirty.due = true;
            }
            Action::UpdateTodoPercentComplete(v) => {
                let mut that = that.borrow_mut();
                that.data.percent_complete = *v;
                that.dirty.percent_complete = true;
            }
            Action::UpdateTodoPriority(v) => {
                let mut that = that.borrow_mut();
                that.data.priority = *v;
                that.dirty.priority = true;
            }
            Action::UpdateTodoStatus(v) => {
                let mut that = that.borrow_mut();
                that.data.status = *v;
                that.dirty.status = true;
            }
            Action::UpdateTodoSummary(v) => {
                let mut that = that.borrow_mut();
                that.data.summary.clone_from(v);
                that.dirty.summary = true;
            }
            Action::SubmitChanges => {
                let mut that = that.borrow_mut();
                that.submit = true;
            }
            _ => {}
        }));
        dispatcher.register(callback);
    }
}

impl TodoStoreLike for TodoStore {
    type Output<'a> = &'a Self;

    fn todo(&self) -> &Self {
        self
    }
}

#[derive(Debug, Default)]
pub struct TodoData {
    pub description: String,
    pub due: String,
    pub percent_complete: Option<u8>,
    pub priority: Priority,
    pub status: TodoStatus,
    pub summary: String,
}

#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct TodoMarker {
    description: bool,
    due: bool,
    percent_complete: bool,
    priority: bool,
    status: bool,
    summary: bool,
}
