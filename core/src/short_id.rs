// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{error::Error, num::NonZeroU32};

use chrono::{DateTime, Local};

use crate::{Event, EventStatus, Id, LooseDateTime, Priority, Todo, TodoStatus, localdb::LocalDb};

#[derive(Debug, Clone)]
pub struct ShortIds {
    db: LocalDb,
}

impl ShortIds {
    pub fn new(db: LocalDb) -> Self {
        Self { db }
    }

    /// Converts the Id to a UID.
    pub async fn get_uid(&self, id: &Id) -> Result<String, Box<dyn Error>> {
        if let Id::ShortIdOrUid(id) = id
            && let Ok(short_id) = id.parse::<NonZeroU32>()
            && let Some(data) = self.db.short_ids.get_by_short_id(short_id).await?
        {
            return Ok(data.uid);
        };

        let uid = match id {
            Id::Uid(uid) => uid,
            Id::ShortIdOrUid(uid) => uid,
        };
        Ok(uid.clone())
    }

    pub async fn event<E: Event>(&self, event: E) -> Result<EventWithShortId<E>, Box<dyn Error>> {
        let short_id = if let Some(short_id) = event.short_id() {
            short_id // If the todo already has a short ID, use it directly
        } else {
            self.db
                .short_ids
                .get_or_assign_short_id(event.uid(), ShortIdKind::Event)
                .await?
        };

        Ok(EventWithShortId {
            inner: event,
            short_id,
        })
    }

    pub async fn events<E: Event>(
        &self,
        events: Vec<E>,
    ) -> Result<Vec<EventWithShortId<E>>, Box<dyn Error>> {
        let mut with_id = Vec::with_capacity(events.len());
        for event in events {
            with_id.push(self.event(event).await?);
        }
        Ok(with_id)
    }

    pub async fn todo<T: Todo>(&self, todo: T) -> Result<TodoWithShortId<T>, Box<dyn Error>> {
        let short_id = if let Some(short_id) = todo.short_id() {
            short_id // If the todo already has a short ID, use it directly
        } else {
            self.db
                .short_ids
                .get_or_assign_short_id(todo.uid(), ShortIdKind::Todo)
                .await?
        };

        Ok(TodoWithShortId {
            inner: todo,
            short_id,
        })
    }

    pub async fn todos<T: Todo>(
        &self,
        todos: Vec<T>,
    ) -> Result<Vec<TodoWithShortId<T>>, Box<dyn Error>> {
        let mut with_id = Vec::with_capacity(todos.len());
        for todo in todos {
            with_id.push(self.todo(todo).await?);
        }
        Ok(with_id)
    }
}

#[derive(Debug)]
pub struct EventWithShortId<E: Event> {
    pub inner: E,
    pub short_id: NonZeroU32,
}

impl<E: Event> Event for EventWithShortId<E> {
    fn short_id(&self) -> Option<NonZeroU32> {
        Some(self.short_id)
    }
    fn uid(&self) -> &str {
        self.inner.uid()
    }
    fn description(&self) -> Option<&str> {
        self.inner.description()
    }
    fn start(&self) -> Option<LooseDateTime> {
        self.inner.start()
    }
    fn end(&self) -> Option<LooseDateTime> {
        self.inner.end()
    }
    fn status(&self) -> Option<EventStatus> {
        self.inner.status()
    }
    fn summary(&self) -> &str {
        self.inner.summary()
    }
}

#[derive(Debug)]
pub struct TodoWithShortId<T: Todo> {
    pub inner: T,
    pub short_id: NonZeroU32,
}

impl<T: Todo> Todo for TodoWithShortId<T> {
    fn short_id(&self) -> Option<NonZeroU32> {
        Some(self.short_id)
    }
    fn uid(&self) -> &str {
        self.inner.uid()
    }
    fn completed(&self) -> Option<DateTime<Local>> {
        self.inner.completed()
    }
    fn description(&self) -> Option<&str> {
        self.inner.description()
    }
    fn due(&self) -> Option<LooseDateTime> {
        self.inner.due()
    }
    fn percent_complete(&self) -> Option<u8> {
        self.inner.percent_complete()
    }
    fn priority(&self) -> Priority {
        self.inner.priority()
    }
    fn status(&self) -> TodoStatus {
        self.inner.status()
    }
    fn summary(&self) -> &str {
        self.inner.summary()
    }
}

#[derive(Debug, Clone)]
pub struct UidAndShortId {
    pub uid: String,
    #[allow(dead_code)]
    pub short_id: NonZeroU32,
    #[allow(dead_code)]
    pub kind: ShortIdKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortIdKind {
    Event,
    Todo,
}
