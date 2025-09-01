// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;

use ratatui::{crossterm::event::KeyCode, prelude::*};

use crate::tui::dispatcher::Dispatcher;

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Handled,
    CursorUpdated,
    Exit,
}

pub trait Component<S> {
    /// Renders the component into the given area.
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer);

    /// Returns the cursor position (row, column) for the component, if applicable.
    fn get_cursor_position(&self, _store: &RefCell<S>, _area: Rect) -> Option<(u16, u16)> {
        None // Default implementation returns no cursor position
    }

    /// Handles key events for the component.
    fn on_key(
        &mut self,
        _dispatcher: &mut Dispatcher,
        _store: &RefCell<S>,
        _area: Rect,
        _key: KeyCode,
    ) -> Option<Message> {
        None // Default implementation does nothing
    }

    /// Activates the component, allowing it to initialize resources or state.
    fn activate(&mut self, _dispatcher: &mut Dispatcher, _store: &RefCell<S>) {}

    /// Deactivates the component, allowing it to clean up resources or state.
    fn deactivate(&mut self, _dispatcher: &mut Dispatcher, _store: &RefCell<S>) {}

    /// Returns whether the component is currently visible.
    /// By default, all components are visible.
    fn is_visible(&self, _store: &RefCell<S>) -> bool {
        true
    }
}

impl<S, T> Component<S> for Box<T>
where
    T: Component<S> + ?Sized,
{
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        (**self).render(store, area, buf);
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        (**self).get_cursor_position(store, area)
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        (**self).on_key(dispatcher, store, area, key)
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        (**self).activate(dispatcher, store);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        (**self).deactivate(dispatcher, store);
    }

    fn is_visible(&self, store: &RefCell<S>) -> bool {
        (**self).is_visible(store)
    }
}
