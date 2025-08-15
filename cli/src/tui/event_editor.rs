// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use aimcal_core::EventStatus;
use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::*;

use crate::tui::component::{Component, Message};
use crate::tui::component_form::{Access, Form, Input, RadioGroup};
use crate::tui::component_page::SinglePage;
use crate::tui::dispatcher::{Action, Dispatcher};
use crate::tui::event_store::EventStoreLike;

pub struct EventEditor<S: EventStoreLike>(SinglePage<S, EventForm<S>>);

impl<S: EventStoreLike + 'static> EventEditor<S> {
    pub fn new() -> Self {
        Self(SinglePage::new("Event Editor".to_owned(), EventForm::new()))
    }
}

impl<S: EventStoreLike> Component<S> for EventEditor<S> {
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

pub struct EventForm<S: EventStoreLike>(Form<S>);

impl<S: EventStoreLike + 'static> EventForm<S> {
    pub fn new() -> Self {
        Self(Form::new(vec![
            Box::new(new_summary()),
            Box::new(new_start()),
            Box::new(new_end()),
            Box::new(new_status()),
            Box::new(new_description()),
        ]))
    }
}

impl<S: EventStoreLike> Component<S> for EventForm<S> {
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

macro_rules! new_input {
    ($fn: ident, $title:expr, $acc: ident, $field: ident, $action: ident) => {
        fn $fn<S: EventStoreLike>() -> Input<S, $acc> {
            Input::new($title.to_string())
        }

        struct $acc;

        impl<S: EventStoreLike> Access<S, String> for $acc {
            fn get(store: &Rc<RefCell<S>>) -> String {
                store.borrow().event().data.$field.clone()
            }

            fn set(dispatcher: &mut Dispatcher, value: String) -> bool {
                dispatcher.dispatch(Action::$action(value));
                true
            }
        }
    };
}

new_input!(
    new_summary,
    "Summary",
    SummaryAccess,
    summary,
    UpdateEventSummary
);
new_input!(
    new_description,
    "Description",
    DescriptionAccess,
    description,
    UpdateEventDescription
);
new_input!(new_start, "Start", StartAccess, start, UpdateEventStart);
new_input!(new_end, "End", EndAccess, end, UpdateEventEnd);

fn new_status<S: EventStoreLike>() -> RadioGroup<S, EventStatus, StatusAccess> {
    use EventStatus::*;
    let values = vec![Tentative, Confirmed, Cancelled];
    let options = values.iter().map(ToString::to_string).collect();
    RadioGroup::new("Status".to_string(), values, options)
}

struct StatusAccess;

impl<S: EventStoreLike> Access<S, EventStatus> for StatusAccess {
    fn get(store: &Rc<RefCell<S>>) -> EventStatus {
        store.borrow().event().data.status
    }

    fn set(dispatcher: &mut Dispatcher, value: EventStatus) -> bool {
        dispatcher.dispatch(Action::UpdateEventStatus(value));
        true
    }
}
