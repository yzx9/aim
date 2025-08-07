// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::error::Error;

use aimcal_core::{Priority, TodoStatus};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;

use crate::tui::component::{
    Access, Component, Form, FormItem, Input, Message, PositiveIntegerAccess, RadioGroup,
};
use crate::tui::todo_store::TodoStore;

pub struct TodoEditor {
    area: Rect, // TODO: support resize
    cursor_pos: Option<(u16, u16)>,
    form: Form<TodoStore>,
}

impl TodoEditor {
    pub fn new<B: Backend>(store: &mut TodoStore, terminal: &mut Terminal<B>) -> Self {
        let area = match terminal.size() {
            Ok(size) => Rect::new(0, 0, size.width, size.height),
            Err(_) => Rect::default(),
        };

        let mut form = Form::new(
            "Todo Editor".to_owned(),
            vec![
                Box::new(new_summary()),
                Box::new(new_due()),
                Box::new(new_percent_complete()),
                Box::new(FieldPriority::new()),
                Box::new(new_status()),
                Box::new(new_description()),
            ],
        );

        // Activate the first item
        form.activate(store);

        Self {
            area,
            cursor_pos: form.get_cursor_position(store, area),
            form,
        }
    }

    pub fn darw<B: Backend>(
        &mut self,
        store: &mut TodoStore,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        terminal.draw(|frame| {
            self.area = frame.area();
            self.render(store, frame.area(), frame.buffer_mut());

            if let Some(pos) = self.cursor_pos {
                frame.set_cursor_position(pos);
            }
        })?;
        Ok(())
    }

    pub fn read_event(&mut self, store: &mut TodoStore) -> Result<Option<Message>, Box<dyn Error>> {
        Ok(match event::read()? {
            Event::Key(e) if e.kind == KeyEventKind::Press => self.on_key(store, self.area, e.code),
            _ => None, // Ignore other kinds of events
        })
    }
}

impl Component<TodoStore> for TodoEditor {
    fn render(&self, store: &TodoStore, area: Rect, buf: &mut Buffer) {
        self.form.render(store, area, buf);
    }

    fn on_key(&mut self, store: &mut TodoStore, area: Rect, key: KeyCode) -> Option<Message> {
        // Handle key events for the current component
        if let Some(msg) = self.form.on_key(store, area, key) {
            return match msg {
                Message::CursorUpdated => {
                    self.cursor_pos = self.form.get_cursor_position(store, area);
                    Some(Message::Handled)
                }
                _ => Some(msg),
            };
        }

        match key {
            KeyCode::Esc => Some(Message::Exit),
            _ => None,
        }
    }
}

macro_rules! new_input {
    ($name: ident, $title:expr, $field: ident, $acc: ident) => {
        struct $acc;

        impl Access<TodoStore, String> for $acc {
            fn get(store: &TodoStore) -> Cow<'_, String> {
                Cow::Borrowed(&store.data.$field)
            }

            fn set(store: &mut TodoStore, value: String) -> bool {
                store.data.$field = value;
                store.dirty.$field = true;
                true
            }
        }

        fn $name() -> Input<TodoStore, $acc> {
            Input::new($title.to_string())
        }
    };
}

new_input!(new_summary, "Summary", summary, SummaryAccess);
new_input!(new_description, "Description", description, DesAcc);
new_input!(new_due, "Due", due, DueAccess);

struct PercentCompleteAccess;

impl Access<TodoStore, Option<u8>> for PercentCompleteAccess {
    fn get(store: &TodoStore) -> Cow<'_, Option<u8>> {
        Cow::Borrowed(&store.data.percent_complete)
    }

    fn set(store: &mut TodoStore, value: Option<u8>) -> bool {
        store.data.percent_complete = value;
        store.dirty.percent_complete = true;
        true
    }
}

fn new_percent_complete()
-> Input<TodoStore, PositiveIntegerAccess<TodoStore, u8, PercentCompleteAccess>> {
    Input::new("Percent complete".to_string())
}

struct StatusAccess;

impl Access<TodoStore, TodoStatus> for StatusAccess {
    fn get(store: &TodoStore) -> Cow<'_, TodoStatus> {
        Cow::Borrowed(&store.data.status)
    }

    fn set(store: &mut TodoStore, value: TodoStatus) -> bool {
        store.data.status = value;
        store.dirty.status = true;
        true
    }
}

fn new_status() -> RadioGroup<TodoStore, TodoStatus, StatusAccess> {
    use TodoStatus::*;
    let values = vec![NeedsAction, Completed, InProcess, Cancelled];
    let options = values.iter().map(ToString::to_string).collect();
    RadioGroup::new("Status".to_string(), values, options)
}

struct PriorityAccess;

impl Access<TodoStore, Priority> for PriorityAccess {
    fn get(store: &TodoStore) -> Cow<'_, Priority> {
        Cow::Borrowed(&store.data.priority)
    }

    fn set(store: &mut TodoStore, value: Priority) -> bool {
        store.data.priority = value;
        store.dirty.priority = true;
        true
    }
}

struct FieldPriority {
    verbose: RadioGroup<TodoStore, Priority, PriorityAccess>,
    concise: RadioGroup<TodoStore, Priority, PriorityAccess>,
}

impl FieldPriority {
    pub fn new() -> Self {
        use Priority::*;
        let values_verbose = vec![P1, P2, P3, P4, P5, P6, P7, P8, P9, None];
        let values_concise = vec![P2, P5, P8, None];

        let options_verbose = values_verbose
            .iter()
            .map(|a| Self::fmt(a, true).to_string())
            .collect();

        let options_concise = values_concise
            .iter()
            .map(|a| Self::fmt(a, false).to_string())
            .collect();

        Self {
            verbose: RadioGroup::new("Priority".to_string(), values_verbose, options_verbose),
            concise: RadioGroup::new("Priority".to_string(), values_concise, options_concise),
        }
    }

    fn get(&self, store: &TodoStore) -> &RadioGroup<TodoStore, Priority, PriorityAccess> {
        match store.verbose_priority {
            true => &self.verbose,
            false => &self.concise,
        }
    }

    fn get_mut(
        &mut self,
        store: &TodoStore,
    ) -> &mut RadioGroup<TodoStore, Priority, PriorityAccess> {
        match store.verbose_priority {
            true => &mut self.verbose,
            false => &mut self.concise,
        }
    }

    fn fmt(priority: &Priority, verbose: bool) -> &'static str {
        match priority {
            Priority::P2 if !verbose => "HIGH",
            Priority::P5 if !verbose => "MID",
            Priority::P8 if !verbose => "LOW",
            Priority::None => "NONE",
            Priority::P1 => "1",
            Priority::P2 => "2",
            Priority::P3 => "3",
            Priority::P4 => "4",
            Priority::P5 => "5",
            Priority::P6 => "6",
            Priority::P7 => "7",
            Priority::P8 => "8",
            Priority::P9 => "9",
        }
    }
}

impl Component<TodoStore> for FieldPriority {
    fn render(&self, store: &TodoStore, area: Rect, buf: &mut Buffer) {
        self.get(store).render(store, area, buf)
    }

    fn on_key(&mut self, store: &mut TodoStore, area: Rect, key: KeyCode) -> Option<Message> {
        self.get_mut(store).on_key(store, area, key)
    }

    fn get_cursor_position(&self, store: &TodoStore, area: Rect) -> Option<(u16, u16)> {
        self.get(store).get_cursor_position(store, area)
    }
}

impl FormItem<TodoStore> for FieldPriority {
    fn activate(&mut self, store: &mut TodoStore) {
        self.verbose.activate(store);
        self.concise.activate(store);
    }

    fn deactivate(&mut self, store: &mut TodoStore) {
        self.verbose.deactivate(store);
        self.concise.deactivate(store);
    }
}
