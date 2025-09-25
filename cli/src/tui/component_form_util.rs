// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, str::FromStr};

use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect};

use crate::tui::{
    component::{Component, Message},
    component_form::{Access, FormItem, FormItemState},
    dispatcher::Dispatcher,
};

pub trait VisiblePredicate<S> {
    fn is_visible(store: &RefCell<S>) -> bool;
}

/// A form item that is only visible if the predicate function returns true.
pub struct VisibleIf<S, T, P>
where
    T: FormItem<S>,
    P: VisiblePredicate<S>,
{
    item: T,
    s: std::marker::PhantomData<S>,
    p: std::marker::PhantomData<P>,
}

impl<S, T, P> VisibleIf<S, T, P>
where
    T: FormItem<S>,
    P: VisiblePredicate<S>,
{
    pub fn new(item: T) -> Self {
        Self {
            item,
            s: std::marker::PhantomData,
            p: std::marker::PhantomData,
        }
    }
}

impl<S, T, P> Component<S> for VisibleIf<S, T, P>
where
    T: FormItem<S>,
    P: VisiblePredicate<S>,
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

impl<S, T, P> FormItem<S> for VisibleIf<S, T, P>
where
    T: FormItem<S>,
    P: VisiblePredicate<S>,
{
    fn item_title(&self, store: &RefCell<S>) -> &str {
        self.item.item_title(store)
    }

    fn item_state(&self, store: &RefCell<S>) -> FormItemState {
        match P::is_visible(store) {
            true => self.item.item_state(store), // Visible if percent_complete is set or status is InProcess
            false => FormItemState::Invisible,
        }
    }
}

pub trait SwitchPredicate<S> {
    fn is(store: &RefCell<S>) -> bool;
}

pub struct FormItemSwitch<S, T, K, P>
where
    T: FormItem<S>,
    K: FormItem<S>,
    P: SwitchPredicate<S>,
{
    on: T,
    off: K,
    s: std::marker::PhantomData<S>,
    p: std::marker::PhantomData<P>,
}

impl<S, T, K, P> FormItemSwitch<S, T, K, P>
where
    T: FormItem<S>,
    K: FormItem<S>,
    P: SwitchPredicate<S>,
{
    pub fn new(on: T, off: K) -> Self {
        Self {
            on,
            off,
            s: std::marker::PhantomData,
            p: std::marker::PhantomData,
        }
    }
}

impl<S, T, K, P> Component<S> for FormItemSwitch<S, T, K, P>
where
    T: FormItem<S>,
    K: FormItem<S>,
    P: SwitchPredicate<S>,
{
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        match P::is(store) {
            true => self.on.render(store, area, buf),
            false => self.off.render(store, area, buf),
        }
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        match P::is(store) {
            true => self.on.get_cursor_position(store, area),
            false => self.off.get_cursor_position(store, area),
        }
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        area: Rect,
        event: KeyEvent,
    ) -> Option<Message> {
        match P::is(store) {
            true => self.on.on_key(dispatcher, store, area, event),
            false => self.off.on_key(dispatcher, store, area, event),
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

impl<S, T, K, P> FormItem<S> for FormItemSwitch<S, T, K, P>
where
    T: FormItem<S>,
    K: FormItem<S>,
    P: SwitchPredicate<S>,
{
    fn item_title(&self, store: &RefCell<S>) -> &str {
        match P::is(store) {
            true => self.on.item_title(store),
            false => self.off.item_title(store),
        }
    }

    fn item_state(&self, store: &RefCell<S>) -> FormItemState {
        match P::is(store) {
            true => self.on.item_state(store),
            false => self.off.item_state(store),
        }
    }
}

/// An access that converts between `Option<T>` and `String`, where `T` is a positive integer type.
pub struct PositiveIntegerAccess<S, T, A>
where
    T: ToString + FromStr + ToOwned + Clone,
    A: Access<S, Option<T>>,
{
    _phantom_s: std::marker::PhantomData<S>,
    _phantom_a: std::marker::PhantomData<A>,
    _phantom_t: std::marker::PhantomData<T>,
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
