// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, error::Error, num::NonZeroU32};

use jiff::Zoned;

use crate::localdb::LocalDb;
use crate::{Event, EventStatus, Id, Kind, LooseDateTime, Priority, Todo, TodoStatus};

#[derive(Debug, Clone)]
pub struct ShortIds {
    db: LocalDb,
}

impl ShortIds {
    pub fn new(db: LocalDb) -> Self {
        Self { db }
    }

    pub async fn get(&self, id: &Id) -> Result<Option<UidAndShortId>, Box<dyn Error>> {
        Ok(match id.maybe_short_id() {
            Some(short_id) => self.db.short_ids.get_by_short_id(short_id).await?,
            None => None,
        })
    }

    /// Converts the Id to a UID.
    pub async fn get_uid(&self, id: &Id) -> Result<String, Box<dyn Error>> {
        if let Some(short_id) = id.maybe_short_id()
            && let Some(data) = self.db.short_ids.get_by_short_id(short_id).await?
        {
            return Ok(data.uid);
        }

        Ok(id.as_uid().to_owned())
    }

    pub async fn event<E: Event>(&self, event: E) -> Result<EventWithShortId<E>, Box<dyn Error>> {
        let short_id = match event.short_id() {
            Some(short_id) => short_id, // If the todo already has a short ID, use it directly
            None => {
                self.db
                    .short_ids
                    .get_or_assign_short_id(&event.uid(), Kind::Event)
                    .await?
            }
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
        let short_id = match todo.short_id() {
            Some(short_id) => short_id, // If the todo already has a short ID, use it directly
            None => {
                self.db
                    .short_ids
                    .get_or_assign_short_id(&todo.uid(), Kind::Todo)
                    .await?
            }
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

    pub async fn flush(&self) -> Result<(), Box<dyn Error>> {
        self.db.short_ids.truncate().await?;
        Ok(())
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

    fn uid(&self) -> Cow<'_, str> {
        self.inner.uid()
    }

    fn description(&self) -> Option<Cow<'_, str>> {
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

    fn summary(&self) -> Cow<'_, str> {
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

    fn uid(&self) -> Cow<'_, str> {
        self.inner.uid()
    }

    fn completed(&self) -> Option<Zoned> {
        self.inner.completed()
    }

    fn description(&self) -> Option<Cow<'_, str>> {
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

    fn summary(&self) -> Cow<'_, str> {
        self.inner.summary()
    }
}

#[derive(Debug, Clone)]
pub struct UidAndShortId {
    pub uid: String,

    #[expect(dead_code)]
    pub short_id: NonZeroU32,

    pub kind: Kind,
}
