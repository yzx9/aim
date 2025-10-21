// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;

use aimcal_core::EventStatus;

use crate::tui::component_form::{Access, Form, FormItem, Input, RadioGroup};
use crate::tui::component_page::SinglePage;
use crate::tui::dispatcher::{Action, Dispatcher};
use crate::tui::event_store::EventStoreLike;

pub fn new_event_editor<S: EventStoreLike + 'static>()
-> SinglePage<S, Form<S, Box<dyn FormItem<S>>>> {
    SinglePage::new(&"Event Editor", new_event_form())
}

pub fn new_event_form<S: EventStoreLike + 'static>() -> Form<S, Box<dyn FormItem<S>>> {
    Form::new(vec![
        Box::new(new_summary()),
        Box::new(new_start()),
        Box::new(new_end()),
        Box::new(new_status()),
        Box::new(new_description()),
    ])
}

macro_rules! new_input {
    ($fn: ident, $title:expr, $acc: ident, $field: ident, $action: ident) => {
        fn $fn<S: EventStoreLike>() -> Input<S, $acc> {
            Input::new($title)
        }

        struct $acc;

        impl<S: EventStoreLike> Access<S, String> for $acc {
            fn get(store: &RefCell<S>) -> String {
                store.borrow().event().data.$field.clone()
            }

            fn set(dispatcher: &mut Dispatcher, value: String) -> bool {
                dispatcher.dispatch(&Action::$action(value));
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
    use EventStatus::{Cancelled, Confirmed, Tentative};
    let values = vec![Tentative, Confirmed, Cancelled];
    let options = values.iter().map(ToString::to_string).collect();
    RadioGroup::new("Status", values, options)
}

struct StatusAccess;

impl<S: EventStoreLike> Access<S, EventStatus> for StatusAccess {
    fn get(store: &RefCell<S>) -> EventStatus {
        store.borrow().event().data.status
    }

    fn set(dispatcher: &mut Dispatcher, value: EventStatus) -> bool {
        dispatcher.dispatch(&Action::UpdateEventStatus(value));
        true
    }
}
