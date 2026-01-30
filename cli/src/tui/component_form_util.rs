// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, str::FromStr};

use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect};

use crate::tui::component::{Component, Message};
use crate::tui::component_form::{Access, FormItem, FormItemState};
use crate::tui::dispatcher::Dispatcher;

/// A form item that is only visible if the predicate function returns true.
pub struct VisibleIf<S, T, F>
where
    T: FormItem<S>,
    F: Fn(&RefCell<S>) -> bool,
{
    item: T,
    is_visible: F,
    _s: std::marker::PhantomData<S>,
}

impl<S, T, F> VisibleIf<S, T, F>
where
    T: FormItem<S>,
    F: Fn(&RefCell<S>) -> bool,
{
    pub fn new(item: T, is_visible: F) -> Self {
        Self {
            item,
            is_visible,
            _s: std::marker::PhantomData,
        }
    }
}

impl<S, T, F> Component<S> for VisibleIf<S, T, F>
where
    T: FormItem<S>,
    F: Fn(&RefCell<S>) -> bool,
{
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        self.item.render(store, area, buf);
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        self.item.get_cursor_position(store, area)
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        area: Rect,
        event: KeyEvent,
    ) -> Option<Message> {
        self.item.on_key(dispatcher, store, area, event)
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        self.item.activate(dispatcher, store);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        self.item.deactivate(dispatcher, store);
    }
}

impl<S, T, F> FormItem<S> for VisibleIf<S, T, F>
where
    T: FormItem<S>,
    F: Fn(&RefCell<S>) -> bool,
{
    fn item_title(&self, store: &RefCell<S>) -> &str {
        self.item.item_title(store)
    }

    fn item_state(&self, store: &RefCell<S>) -> FormItemState {
        if (self.is_visible)(store) {
            self.item.item_state(store) // Visible if percent_complete is set or status is InProcess
        } else {
            FormItemState::Invisible
        }
    }
}

pub struct FormItemSwitch<S, T, K, F>
where
    T: FormItem<S>,
    K: FormItem<S>,
    F: Fn(&RefCell<S>) -> bool,
{
    on: T,
    off: K,
    on_or_off: F,
    _s: std::marker::PhantomData<S>,
}

impl<S, T, K, F> FormItemSwitch<S, T, K, F>
where
    T: FormItem<S>,
    K: FormItem<S>,
    F: Fn(&RefCell<S>) -> bool,
{
    pub fn new(on: T, off: K, on_or_off: F) -> Self {
        Self {
            on,
            off,
            on_or_off,
            _s: std::marker::PhantomData,
        }
    }
}

impl<S, T, K, F> Component<S> for FormItemSwitch<S, T, K, F>
where
    T: FormItem<S>,
    K: FormItem<S>,
    F: Fn(&RefCell<S>) -> bool,
{
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        if (self.on_or_off)(store) {
            self.on.render(store, area, buf);
        } else {
            self.off.render(store, area, buf);
        }
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        if (self.on_or_off)(store) {
            self.on.get_cursor_position(store, area)
        } else {
            self.off.get_cursor_position(store, area)
        }
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        area: Rect,
        event: KeyEvent,
    ) -> Option<Message> {
        if (self.on_or_off)(store) {
            self.on.on_key(dispatcher, store, area, event)
        } else {
            self.off.on_key(dispatcher, store, area, event)
        }
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        self.on.activate(dispatcher, store);
        self.off.activate(dispatcher, store);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        self.on.deactivate(dispatcher, store);
        self.off.deactivate(dispatcher, store);
    }
}

impl<S, T, K, F> FormItem<S> for FormItemSwitch<S, T, K, F>
where
    T: FormItem<S>,
    K: FormItem<S>,
    F: Fn(&RefCell<S>) -> bool,
{
    fn item_title(&self, store: &RefCell<S>) -> &str {
        if (self.on_or_off)(store) {
            self.on.item_title(store)
        } else {
            self.off.item_title(store)
        }
    }

    fn item_state(&self, store: &RefCell<S>) -> FormItemState {
        if (self.on_or_off)(store) {
            self.on.item_state(store)
        } else {
            self.off.item_state(store)
        }
    }
}

/// An access that converts between `Option<T>` and `String`, where `T` is a positive integer type.
pub struct PositiveIntegerAccess<S, T, A>
where
    T: ToString + FromStr + ToOwned + Clone,
    A: Access<S, Option<T>>,
{
    _s: std::marker::PhantomData<S>,
    _a: std::marker::PhantomData<A>,
    _t: std::marker::PhantomData<T>,
}

impl<S, T, A> Access<S, String> for PositiveIntegerAccess<S, T, A>
where
    T: Eq + ToString + FromStr + ToOwned + Clone,
    A: Access<S, Option<T>>,
{
    fn get(s: &RefCell<S>) -> String {
        match A::get(s) {
            Some(a) => a.to_string(),
            None => String::new(),
        }
    }

    fn set(dispatcher: &mut Dispatcher, value: String) -> bool {
        let v = value.trim();
        if v.is_empty() {
            A::set(dispatcher, None)
        } else if let Ok(num) = v.parse::<T>() {
            A::set(dispatcher, Some(num))
        } else {
            tracing::debug!(value, "failed to parse as a positive integer");
            false
        }
    }
}
