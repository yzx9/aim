// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::*;

use super::dispatcher::Dispatcher;

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Handled,
    CursorUpdated,
    Exit,
}

pub trait Component<S> {
    /// Renders the component into the given area.
    fn render(&self, store: &Rc<RefCell<S>>, area: Rect, buf: &mut Buffer);

    /// Handles key events for the component.
    fn on_key(
        &mut self,
        _dispatcher: &mut Dispatcher,
        _store: &Rc<RefCell<S>>,
        _area: Rect,
        _key: KeyCode,
    ) -> Option<Message> {
        None // Default implementation does nothing
    }

    /// Returns the cursor position (row, column) for the component, if applicable.
    fn get_cursor_position(&self, _store: &Rc<RefCell<S>>, _area: Rect) -> Option<(u16, u16)> {
        None // Default implementation returns no cursor position
    }
}
