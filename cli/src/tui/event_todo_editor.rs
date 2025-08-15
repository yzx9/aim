// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::cell::{Ref, RefCell};
use std::error::Error;
use std::rc::Rc;

use aimcal_core::{EventDraft, TodoDraft};
use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::*;

use crate::tui::component::{Component, Message};
use crate::tui::component_form::Access;
use crate::tui::component_page::TabPages;
use crate::tui::dispatcher::{Action, Dispatcher, EventOrTodo};
use crate::tui::event_editor::EventForm;
use crate::tui::event_store::{EventStore, EventStoreLike};
use crate::tui::todo_editor::TodoForm;
use crate::tui::todo_store::{TodoStore, TodoStoreLike};

pub trait EventTodoStoreLike: EventStoreLike + TodoStoreLike {
    fn active(&self) -> EventOrTodo;
}

pub enum EventOrTodoDraft {
    Event(EventDraft),
    Todo(TodoDraft),
}

pub struct EventTodoStore {
    pub event: Rc<RefCell<EventStore>>,
    pub todo: Rc<RefCell<TodoStore>>,
    pub active: EventOrTodo,
    pub submit: bool,
}

impl EventTodoStore {
    pub fn new(event: EventDraft, todo: TodoDraft) -> Self {
        Self {
            event: Rc::new(RefCell::new(EventStore::new_by_draft(event))),
            todo: Rc::new(RefCell::new(TodoStore::new_by_draft(todo))),
            active: EventOrTodo::Todo, // active todo by default since it is more common to draft todo
            submit: false,
        }
    }

    pub fn register_to(that: Rc<RefCell<Self>>, dispatcher: &mut Dispatcher) {
        EventStore::register_to(that.borrow().event.clone(), dispatcher);
        TodoStore::register_to(that.borrow().todo.clone(), dispatcher);

        let callback = Rc::new(RefCell::new(move |action: &Action| match action {
            Action::Activate(v) => that.borrow_mut().active = *v,
            Action::SubmitChanges => that.borrow_mut().submit = true,
            _ => {}
        }));
        dispatcher.register(callback);
    }

    pub fn submit_draft(self) -> Result<EventOrTodoDraft, Box<dyn Error>> {
        match self.active {
            EventOrTodo::Event => {
                let store = Rc::try_unwrap(self.event).map_err(|_| "Store still has references")?;
                let event = store.into_inner().submit_draft()?;
                Ok(EventOrTodoDraft::Event(event))
            }
            EventOrTodo::Todo => {
                let store = Rc::try_unwrap(self.todo).map_err(|_| "Store still has references")?;
                let todo = store.into_inner().submit_draft()?;
                Ok(EventOrTodoDraft::Todo(todo))
            }
        }
    }
}

impl EventStoreLike for EventTodoStore {
    type Output<'a> = Ref<'a, EventStore>;

    fn event(&self) -> Ref<'_, EventStore> {
        self.event.borrow()
    }
}

impl TodoStoreLike for EventTodoStore {
    type Output<'a> = Ref<'a, TodoStore>;

    fn todo(&self) -> Ref<'_, TodoStore> {
        self.todo.borrow()
    }
}

impl EventTodoStoreLike for EventTodoStore {
    fn active(&self) -> EventOrTodo {
        self.active
    }
}

struct ActiveAccess<S: EventTodoStoreLike>(std::marker::PhantomData<S>);

impl<S: EventTodoStoreLike> Access<S, EventOrTodo> for ActiveAccess<S> {
    fn get(store: &Rc<RefCell<S>>) -> EventOrTodo {
        store.borrow().active()
    }

    fn set(dispatcher: &mut Dispatcher, value: EventOrTodo) -> bool {
        dispatcher.dispatch(Action::Activate(value));
        true
    }
}

pub struct EventTodoEditor<S: EventTodoStoreLike>(TabPages<S, ActiveAccess<S>>);

impl<S: EventTodoStoreLike + 'static> EventTodoEditor<S> {
    pub fn new() -> Self {
        Self(TabPages::new(vec![
            (
                EventOrTodo::Event,
                "Event".to_owned(),
                Box::new(EventForm::new()),
            ),
            (
                EventOrTodo::Todo,
                "Todo".to_owned(),
                Box::new(TodoForm::new()),
            ),
        ]))
    }
}

impl<S: EventTodoStoreLike + 'static> Component<S> for EventTodoEditor<S> {
    fn render(&self, store: &Rc<RefCell<S>>, area: Rect, buf: &mut Buffer) {
        self.0.render(store, area, buf);
    }
    fn get_cursor_position(&self, store: &Rc<RefCell<S>>, area: Rect) -> Option<(u16, u16)> {
        self.0.get_cursor_position(store, area)
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &Rc<RefCell<S>>,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        self.0.on_key(dispatcher, store, area, key)
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher, store: &Rc<RefCell<S>>) {
        self.0.activate(dispatcher, store);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher, store: &Rc<RefCell<S>>) {
        self.0.deactivate(dispatcher, store);
    }
}
