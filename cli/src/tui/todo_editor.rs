// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, error::Error, rc::Rc};

use aimcal_core::{Priority, TodoStatus};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;

use crate::tui::component::{Component, Message};
use crate::tui::component_form::{Access, Form, Input, PositiveIntegerAccess, RadioGroup};
use crate::tui::component_page::SinglePage;
use crate::tui::dispatcher::{Action, Dispatcher};
use crate::tui::todo_store::TodoStore;

type Store = Rc<RefCell<TodoStore>>;

pub struct TodoEditor {
    dispatcher: Dispatcher,
    area: Rect, // TODO: support resize
    cursor_pos: Option<(u16, u16)>,
    page: SinglePage<TodoStore, TodoForm>,
}

impl TodoEditor {
    pub fn new<B: Backend>(
        mut dispatcher: Dispatcher,
        store: &Store,
        terminal: &mut Terminal<B>,
    ) -> Self {
        let area = match terminal.size() {
            Ok(size) => Rect::new(0, 0, size.width, size.height),
            Err(_) => Rect::default(),
        };

        let mut page = SinglePage::new("Todo Editor".to_owned(), TodoForm::new());

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
                match form.on_key(dispatcher, store, self.area, e.code) {
                    Some(msg) => match msg {
                        Message::CursorUpdated => {
                            self.cursor_pos = self.page.get_cursor_position(store, area);
                            Some(Message::Handled)
                        }
                        _ => Some(msg),
                    },
                    None => None,
                }
            }
            _ => None, // Ignore other kinds of events
        })
    }
}

pub struct TodoForm(Form<TodoStore>);

impl TodoForm {
    pub fn new() -> Self {
        Self(Form::new(vec![
            Box::new(new_summary()),
            Box::new(new_due()),
            Box::new(new_percent_complete()),
            Box::new(FieldPriority::new()),
            Box::new(new_status()),
            Box::new(new_description()),
        ]))
    }
}

impl Component<TodoStore> for TodoForm {
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
        fn $fn() -> Input<TodoStore, $acc> {
            Input::new($title.to_string())
        }

        struct $acc;

        impl Access<TodoStore, String> for $acc {
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
    UpdateTodoSummary
);
new_input!(
    new_description,
    "Description",
    DescriptionAccess,
    description,
    UpdateTodoDescription
);
new_input!(new_due, "Due", DueAccess, due, UpdateTodoDue);

struct PercentCompleteAccess;

impl Access<TodoStore, Option<u8>> for PercentCompleteAccess {
    fn get(store: &Store) -> Option<u8> {
        store.borrow().data.percent_complete
    }

    fn set(dispatcher: &mut Dispatcher, value: Option<u8>) -> bool {
        dispatcher.dispatch(Action::UpdateTodoPercentComplete(value));
        true
    }
}

fn new_percent_complete()
-> Input<TodoStore, PositiveIntegerAccess<TodoStore, u8, PercentCompleteAccess>> {
    Input::new("Percent complete".to_string())
}

fn new_status() -> RadioGroup<TodoStore, TodoStatus, StatusAccess> {
    use TodoStatus::*;
    let values = vec![NeedsAction, Completed, InProcess, Cancelled];
    let options = values.iter().map(ToString::to_string).collect();
    RadioGroup::new("Status".to_string(), values, options)
}

struct StatusAccess;

impl Access<TodoStore, TodoStatus> for StatusAccess {
    fn get(store: &Store) -> TodoStatus {
        store.borrow().data.status
    }

    fn set(dispatcher: &mut Dispatcher, value: TodoStatus) -> bool {
        dispatcher.dispatch(Action::UpdateTodoStatus(value));
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
        let values_verb = vec![P1, P2, P3, P4, P5, P6, P7, P8, P9, None];
        let values = vec![P2, P5, P8, None];

        let options_verb = values_verb
            .iter()
            .map(|a| Self::fmt(a, true).to_string())
            .collect();

        let options = values
            .iter()
            .map(|a| Self::fmt(a, false).to_string())
            .collect();

        Self {
            verbose: RadioGroup::new("Priority".to_string(), values_verb, options_verb),
            concise: RadioGroup::new("Priority".to_string(), values, options),
        }
    }

    fn get(&self, store: &Store) -> &RadioGroup<TodoStore, Priority, PriorityAccess> {
        match store.borrow().verbose_priority {
            true => &self.verbose,
            false => &self.concise,
        }
    }

    fn get_mut(&mut self, store: &Store) -> &mut RadioGroup<TodoStore, Priority, PriorityAccess> {
        match store.borrow().verbose_priority {
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
    fn render(&self, store: &Store, area: Rect, buf: &mut Buffer) {
        self.get(store).render(store, area, buf)
    }

    fn get_cursor_position(&self, store: &Store, area: Rect) -> Option<(u16, u16)> {
        self.get(store).get_cursor_position(store, area)
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &Store,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        self.get_mut(store).on_key(dispatcher, store, area, key)
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher) {
        self.verbose.activate(dispatcher);
        self.concise.activate(dispatcher);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher) {
        self.verbose.deactivate(dispatcher);
        self.concise.deactivate(dispatcher);
    }
}

struct PriorityAccess;

impl Access<TodoStore, Priority> for PriorityAccess {
    fn get(store: &Store) -> Priority {
        store.borrow().data.priority
    }

    fn set(dispatcher: &mut Dispatcher, value: Priority) -> bool {
        dispatcher.dispatch(Action::UpdateTodoPriority(value));
        true
    }
}
