// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;

use aimcal_core::{Priority, TodoStatus};

use crate::tui::component_form::{Access, Form, FormItem, Input, RadioGroup};
use crate::tui::component_form_util::{
    FormItemSwitch, PositiveIntegerAccess, SwitchPredicate, VisibleIf, VisiblePredicate,
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
        Box::new(new_priority()),
        Box::new(new_status()),
        Box::new(new_percent_complete()),
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

struct PercentCompleteVisiblePredicate<S> {
    _marker: std::marker::PhantomData<S>,
}

impl<S: TodoStoreLike> VisiblePredicate<S> for PercentCompleteVisiblePredicate<S> {
    fn is_visible(store: &RefCell<S>) -> bool {
        let s = store.borrow();
        let data = &s.todo().data;
        data.percent_complete.is_some() || matches!(data.status, TodoStatus::InProcess)
    }
}

type ComponentPercentComplete<S> = VisibleIf<
    S,
    Input<S, PositiveIntegerAccess<S, u8, PercentCompleteAccess>>,
    PercentCompleteVisiblePredicate<S>,
>;

fn new_percent_complete<S: TodoStoreLike>() -> ComponentPercentComplete<S> {
    VisibleIf::new(Input::new("Percent complete"))
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

struct PriorityVerbosePredicate<S> {
    _marker: std::marker::PhantomData<S>,
}

impl<S: TodoStoreLike> SwitchPredicate<S> for PriorityVerbosePredicate<S> {
    fn is(store: &RefCell<S>) -> bool {
        store.borrow().todo().verbose_priority
    }
}

type ComponentPriority<S> = FormItemSwitch<
    S,
    RadioGroup<S, Priority, PriorityAccess>,
    RadioGroup<S, Priority, PriorityAccess>,
    PriorityVerbosePredicate<S>,
>;

fn new_priority<S: TodoStoreLike>() -> ComponentPriority<S> {
    use Priority::*;
    let values_verb = vec![P1, P2, P3, P4, P5, P6, P7, P8, P9, None];
    let values = vec![P2, P5, P8, None];

    let options_verb = values_verb
        .iter()
        .map(|a| fmt_priority(a, true).to_string())
        .collect();

    let options = values
        .iter()
        .map(|a| fmt_priority(a, false).to_string())
        .collect();

    const TITLE: &str = "Priority";
    let verbose = RadioGroup::new(TITLE, values_verb, options_verb);
    let concise = RadioGroup::new(TITLE, values, options);
    FormItemSwitch::new(verbose, concise)
}

const fn fmt_priority(priority: &Priority, verbose: bool) -> &'static str {
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
