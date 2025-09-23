// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;

use aimcal_core::{Priority, TodoStatus};
use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::*;

use crate::tui::component::{Component, Message};
use crate::tui::component_form::{
    Access, Form, FormItem, FormItemState, Input, PositiveIntegerAccess, RadioGroup,
};
use crate::tui::component_page::SinglePage;
use crate::tui::dispatcher::{Action, Dispatcher};
use crate::tui::todo_store::TodoStoreLike;

pub fn new_todo_editor<S: TodoStoreLike + 'static>() -> SinglePage<S, Form<S, Box<dyn FormItem<S>>>>
{
    SinglePage::new("Todo Editor", new_todo_form())
}

pub fn new_todo_form<S: TodoStoreLike + 'static>() -> Form<S, Box<dyn FormItem<S>>> {
    Form::new(vec![
        Box::new(new_summary()),
        Box::new(new_due()),
        Box::new(FieldPriority::new()),
        Box::new(new_status()),
        Box::new(ConditionalPercentComplete::new()),
        Box::new(new_description()),
    ])
}

macro_rules! new_input {
    ($fn: ident, $title:expr, $acc: ident, $field: ident, $action: ident) => {
        fn $fn<S: TodoStoreLike>() -> Input<S, $acc> {
            Input::new($title)
        }

        struct $acc;

        impl<S: TodoStoreLike> Access<S, String> for $acc {
            fn get(store: &RefCell<S>) -> String {
                store.borrow().todo().data.$field.clone()
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

impl<S: TodoStoreLike> Access<S, Option<u8>> for PercentCompleteAccess {
    fn get(store: &RefCell<S>) -> Option<u8> {
        store.borrow().todo().data.percent_complete
    }

    fn set(dispatcher: &mut Dispatcher, value: Option<u8>) -> bool {
        dispatcher.dispatch(Action::UpdateTodoPercentComplete(value));
        true
    }
}

fn new_percent_complete<S: TodoStoreLike>()
-> Input<S, PositiveIntegerAccess<S, u8, PercentCompleteAccess>> {
    Input::new("Percent complete")
}

/// A conditional component that only renders when the todo status is `InProcess`
struct ConditionalPercentComplete<S: TodoStoreLike>(
    Input<S, PositiveIntegerAccess<S, u8, PercentCompleteAccess>>,
);

impl<S: TodoStoreLike> ConditionalPercentComplete<S> {
    fn new() -> Self {
        Self(new_percent_complete())
    }
}

impl<S: TodoStoreLike> Component<S> for ConditionalPercentComplete<S> {
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        self.0.render(store, area, buf);
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        self.0.get_cursor_position(store, area)
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        self.0.on_key(dispatcher, store, area, key)
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        self.0.activate(dispatcher, store);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        self.0.deactivate(dispatcher, store);
    }
}

impl<S: TodoStoreLike> FormItem<S> for ConditionalPercentComplete<S> {
    fn item_title(&self) -> &str {
        self.0.item_title()
    }

    fn item_state(&self, store: &RefCell<S>) -> FormItemState {
        match store.borrow().todo().data.status {
            TodoStatus::InProcess => self.0.item_state(store), // Only visible if the status is InProcess
            _ => FormItemState::Invisible,
        }
    }
}

fn new_status<S: TodoStoreLike>() -> RadioGroup<S, TodoStatus, StatusAccess> {
    use TodoStatus::*;
    let values = vec![NeedsAction, Completed, InProcess, Cancelled];
    let options = values.iter().map(ToString::to_string).collect();
    RadioGroup::new("Status", values, options)
}

struct StatusAccess;

impl<S: TodoStoreLike> Access<S, TodoStatus> for StatusAccess {
    fn get(store: &RefCell<S>) -> TodoStatus {
        store.borrow().todo().data.status
    }

    fn set(dispatcher: &mut Dispatcher, value: TodoStatus) -> bool {
        dispatcher.dispatch(Action::UpdateTodoStatus(value));
        true
    }
}

struct FieldPriority<S: TodoStoreLike> {
    verbose: RadioGroup<S, Priority, PriorityAccess>,
    concise: RadioGroup<S, Priority, PriorityAccess>,
}

impl<S: TodoStoreLike> FieldPriority<S> {
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

        const TITLE: &str = "Priority";
        Self {
            verbose: RadioGroup::new(TITLE, values_verb, options_verb),
            concise: RadioGroup::new(TITLE, values, options),
        }
    }

    fn get(&self, store: &RefCell<S>) -> &RadioGroup<S, Priority, PriorityAccess> {
        match store.borrow().todo().verbose_priority {
            true => &self.verbose,
            false => &self.concise,
        }
    }

    fn get_mut(&mut self, store: &RefCell<S>) -> &mut RadioGroup<S, Priority, PriorityAccess> {
        match store.borrow().todo().verbose_priority {
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

impl<S: TodoStoreLike> Component<S> for FieldPriority<S> {
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        self.get(store).render(store, area, buf)
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        self.get(store).get_cursor_position(store, area)
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        self.get_mut(store).on_key(dispatcher, store, area, key)
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        self.verbose.activate(dispatcher, store);
        self.concise.activate(dispatcher, store);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        self.verbose.deactivate(dispatcher, store);
        self.concise.deactivate(dispatcher, store);
    }
}

impl<S: TodoStoreLike> FormItem<S> for FieldPriority<S> {
    fn item_title(&self) -> &str {
        self.verbose.item_title()
    }

    fn item_state(&self, store: &RefCell<S>) -> FormItemState {
        self.get(store).item_state(store)
    }
}

struct PriorityAccess;

impl<S: TodoStoreLike> Access<S, Priority> for PriorityAccess {
    fn get(store: &RefCell<S>) -> Priority {
        store.borrow().todo().data.priority
    }

    fn set(dispatcher: &mut Dispatcher, value: Priority) -> bool {
        dispatcher.dispatch(Action::UpdateTodoPriority(value));
        true
    }
}
