// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, error::Error, rc::Rc};

use aimcal_core::EventStatus;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;

use crate::tui::component::{Component, Message};
use crate::tui::component_form::{Access, Form, Input, RadioGroup};
use crate::tui::component_page::SinglePage;
use crate::tui::dispatcher::{Action, Dispatcher};
use crate::tui::event_store::EventStore;

type Store = Rc<RefCell<EventStore>>;

pub struct EventEditor {
    dispatcher: Dispatcher,
    area: Rect,
    cursor_pos: Option<(u16, u16)>,
    page: SinglePage<EventStore, EventForm>,
}

impl EventEditor {
    pub fn new<B: Backend>(
        mut dispatcher: Dispatcher,
        store: &Store,
        terminal: &mut Terminal<B>,
    ) -> Self {
        let area = match terminal.size() {
            Ok(size) => Rect::new(0, 0, size.width, size.height),
            Err(_) => Rect::default(),
        };

        let mut page = SinglePage::new("Event Editor".to_owned(), EventForm::new());

        // Activate the first item
        page.activate(&mut dispatcher);

        Self {
            dispatcher,
            area,
            cursor_pos: page.get_cursor_position(store, area),
            page,
        }
    }

    pub fn darw<B: Backend>(
        &mut self,
        store: &Store,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        terminal.draw(|frame| {
            self.area = frame.area();
            self.page.render(store, frame.area(), frame.buffer_mut());

            if let Some(pos) = self.cursor_pos {
                frame.set_cursor_position(pos);
            }
        })?;
        Ok(())
    }

    pub fn read_event(&mut self, store: &Store) -> Result<Option<Message>, Box<dyn Error>> {
        Ok(match event::read()? {
            Event::Key(e) if e.kind == KeyEventKind::Press => {
                // Handle key events for the current component
                let (form, dispatcher, area) = (&mut self.page, &mut self.dispatcher, self.area);
                if let Some(msg) = form.on_key(dispatcher, store, self.area, e.code) {
                    return Ok(match msg {
                        Message::CursorUpdated => {
                            self.cursor_pos = self.page.get_cursor_position(store, area);
                            Some(Message::Handled)
                        }
                        _ => Some(msg),
                    });
                } else {
                    None
                }
            }
            _ => None, // Ignore other kinds of events
        })
    }
}

pub struct EventForm(Form<EventStore>);

impl EventForm {
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

impl Component<EventStore> for EventForm {
    fn render(&self, store: &Store, area: Rect, buf: &mut Buffer) {
        self.0.render(store, area, buf);
    }

    fn get_cursor_position(&self, store: &Store, area: Rect) -> Option<(u16, u16)> {
        self.0.get_cursor_position(store, area)
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &Store,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        self.0.on_key(dispatcher, store, area, key)
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher) {
        self.0.activate(dispatcher);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher) {
        self.0.deactivate(dispatcher);
    }
}

macro_rules! new_input {
    ($fn: ident, $title:expr, $acc: ident, $field: ident, $action: ident) => {
        fn $fn() -> Input<EventStore, $acc> {
            Input::new($title.to_string())
        }

        struct $acc;

        impl Access<EventStore, String> for $acc {
            fn get(store: &Store) -> String {
                store.borrow().data.$field.clone()
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

fn new_status() -> RadioGroup<EventStore, EventStatus, StatusAccess> {
    use EventStatus::*;
    let values = vec![Tentative, Confirmed, Cancelled];
    let options = values.iter().map(ToString::to_string).collect();
    RadioGroup::new("Status".to_string(), values, options)
}

struct StatusAccess;

impl Access<EventStore, EventStatus> for StatusAccess {
    fn get(store: &Store) -> EventStatus {
        store.borrow().data.status
    }

    fn set(dispatcher: &mut Dispatcher, value: EventStatus) -> bool {
        dispatcher.dispatch(Action::UpdateEventStatus(value));
        true
    }
}
