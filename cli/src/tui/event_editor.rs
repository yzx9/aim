// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use aimcal_core::EventStatus;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;

use crate::tui::component::{Access, Component, Form, Input, Message, RadioGroup};
use crate::tui::dispatcher::{Action, Dispatcher};
use crate::tui::event_store::EventStore;

type Store = Rc<RefCell<EventStore>>;

pub struct EventEditor {
    dispatcher: Dispatcher,
    area: Rect,
    cursor_pos: Option<(u16, u16)>,
    form: Form<EventStore>,
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

        let mut form = Form::new(
            "Event Editor".to_owned(),
            vec![
                Box::new(new_summary()),
                Box::new(new_start()),
                Box::new(new_end()),
                Box::new(new_status()),
                Box::new(new_description()),
            ],
        );

        // Activate the first item
        form.activate(&mut dispatcher);

        Self {
            dispatcher,
            area,
            cursor_pos: form.get_cursor_position(store, area),
            form,
        }
    }

    pub fn darw<B: Backend>(
        &mut self,
        store: &Store,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        terminal.draw(|frame| {
            self.area = frame.area();
            self.form.render(store, frame.area(), frame.buffer_mut());

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
                let (form, dispatcher, area) = (&mut self.form, &mut self.dispatcher, self.area);
                if let Some(msg) = form.on_key(dispatcher, store, self.area, e.code) {
                    return Ok(match msg {
                        Message::CursorUpdated => {
                            self.cursor_pos = self.form.get_cursor_position(store, area);
                            Some(Message::Handled)
                        }
                        _ => Some(msg),
                    });
                }

                match e.code {
                    KeyCode::Esc => Some(Message::Exit),
                    _ => None,
                }
            }
            _ => None, // Ignore other kinds of events
        })
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
