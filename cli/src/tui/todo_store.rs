// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{Priority, Todo, TodoDraft, TodoPatch, TodoStatus};

use crate::util::{format_datetime, parse_datetime};

#[derive(Debug)]
pub struct TodoStore {
    pub data: Data,
    pub dirty: Marker,

    /// Whether to show verbose priority options
    pub verbose_priority: bool,
}

impl TodoStore {
    pub fn new_by_draft(draft: TodoDraft) -> Self {
        Self::new(Data {
            due: draft.due.map(format_datetime).unwrap_or_default(),
            priority: draft.priority.unwrap_or_default(),
            ..Data::default()
        })
    }

    pub fn new_by_todo(todo: &impl Todo) -> Self {
        Self::new(todo.into())
    }

    fn new(data: Data) -> Self {
        use Priority::*;
        let verbose_priority = matches!(data.priority, P1 | P3 | P4 | P6 | P7 | P9);
        Self {
            data,
            dirty: Marker::default(),
            verbose_priority,
        }
    }

    pub fn submit_draft(self) -> Result<TodoDraft, Box<dyn Error>> {
        Ok(TodoDraft {
            description: self.dirty.description.then_some(self.data.description),
            due: parse_datetime(&self.data.due)?,
            percent_complete: self
                .dirty
                .percent_complete
                .then_some(self.data.percent_complete)
                .flatten(),
            priority: Some(self.data.priority), // Always commit since it was confirmed by the user
            status: Some(self.data.status),     // Always commit since it was confirmed by the user
            summary: if self.data.summary.is_empty() {
                "New todo".to_string()
            } else {
                self.data.summary
            },
        })
    }

    pub fn submit_patch(self) -> Result<TodoPatch, Box<dyn Error>> {
        Ok(TodoPatch {
            description: match self.dirty.description {
                true if self.data.description.is_empty() => Some(None),
                true => Some(Some(self.data.description.clone())),
                false => None,
            },
            due: match self.dirty.due {
                true => Some(parse_datetime(&self.data.due)?),
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
}

#[derive(Debug, Default)]
pub struct Data {
    pub description: String,
    pub due: String,
    pub percent_complete: Option<u8>,
    pub priority: Priority,
    pub status: TodoStatus,
    pub summary: String,
}

impl<T: Todo> From<&T> for Data {
    fn from(todo: &T) -> Self {
        Self {
            description: todo.description().unwrap_or("").to_owned(),
            due: todo.due().map(format_datetime).unwrap_or("".to_string()),
            percent_complete: todo.percent_complete(),
            priority: todo.priority(),
            status: todo.status(),
            summary: todo.summary().to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Marker {
    pub description: bool,
    pub due: bool,
    pub percent_complete: bool,
    pub priority: bool,
    pub status: bool,
    pub summary: bool,
}
