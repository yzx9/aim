// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;

use crate::tui::component::{Component, Message};
use crate::tui::dispatcher::Dispatcher;

pub struct SinglePage<S, C: Component<S>> {
    title: String,
    inner: C,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, C: Component<S>> SinglePage<S, C> {
    pub fn new(title: String, inner: C) -> Self {
        Self {
            title,
            inner,
            _phantom: std::marker::PhantomData,
        }
    }

    fn block(&self) -> Block {
        Block::bordered().border_set(border::ROUNDED)
    }
}

impl<S, C: Component<S>> Component<S> for SinglePage<S, C> {
    fn render(&self, store: &Rc<RefCell<S>>, area: Rect, buf: &mut Buffer) {
        let title = Line::from(format!(" {} ", self.title).bold());
        let block = self
            .block()
            .title(title.centered())
            .title_bottom(instructions().centered())
            .white();

        let inner_area = block.inner(area);
        block.render(area, buf);
        self.inner.render(store, inner_area, buf);
    }

    fn get_cursor_position(&self, store: &Rc<RefCell<S>>, area: Rect) -> Option<(u16, u16)> {
        let inner_area = self.block().inner(area);
        self.inner.get_cursor_position(store, inner_area)
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &Rc<RefCell<S>>,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        if let Some(msg) = self.inner.on_key(dispatcher, store, area, key) {
            return Some(msg);
        }

        match key {
            KeyCode::Esc => Some(Message::Exit),
            _ => None,
        }
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher) {
        self.inner.activate(dispatcher);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher) {
        self.inner.deactivate(dispatcher);
    }
}

fn instructions() -> Line<'static> {
    Line::from(vec![
        " Prev ".into(),
        "<Up>".blue().bold(),
        " Next ".into(),
        "<Down>".blue().bold(),
        " Exit ".into(),
        "<Esc> ".blue().bold(),
    ])
}
